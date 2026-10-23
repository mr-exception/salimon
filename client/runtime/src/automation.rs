//! Line-delimited JSON control channel, available only after an E2E renderer is ready.
//! The reader never touches game state; all commands run on the native event-loop thread.

use std::io::{self, BufRead, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::app::ClientApplication;
use crate::session::GameplaySession;
use crate::session::frames::character_ship_frame;
use salimon_character::CharacterLocation;
use salimon_world::CELESTIAL_BODIES;

const PROTOCOL: u8 = 1;
const TIMEOUT: Duration = Duration::from_secs(5);
const STEP_TIMEOUT: Duration = Duration::from_secs(120);
const MAX_LINE: usize = 16 * 1024;

pub(crate) struct Request {
    pub(crate) line: String,
    pub(crate) deadline: Instant,
    pub(crate) reply: Sender<Value>,
}

pub(crate) fn start() -> Receiver<Request> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let stdin = io::stdin();
        let mut input = stdin.lock();
        loop {
            let mut line = String::new();
            match input.read_line(&mut line) {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
            if line.len() > MAX_LINE {
                write_response(&error(
                    Value::Null,
                    "line_too_long",
                    "maximum line size is 16 KiB",
                ));
                continue;
            }
            let (reply, answer) = mpsc::channel();
            let timeout = command_timeout(&line);
            let request = Request {
                line,
                deadline: Instant::now() + timeout,
                reply,
            };
            if sender.send(request).is_err() {
                break;
            }
            match answer.recv_timeout(timeout) {
                Ok(value) => write_response(&value),
                Err(_) => write_response(&error(Value::Null, "timeout", "command timed out")),
            }
        }
    });
    receiver
}

/// Even one loaded support-removal update can rebuild a large contact island.
/// Valid fixed steps have a bounded extended deadline; other commands and
/// malformed/out-of-range requests retain the default deadline. Client/scenario
/// budgets can still impose a shorter limit.
fn command_timeout(line: &str) -> Duration {
    let Ok(command) = serde_json::from_str::<Value>(line) else {
        return TIMEOUT;
    };
    if command.get("protocol").and_then(Value::as_u64) == Some(u64::from(PROTOCOL))
        && command
            .get("id")
            .is_some_and(|id| id.is_string() || id.is_number())
        && command.get("op").and_then(Value::as_str) == Some("step")
        && matches!(command.get("frames").and_then(Value::as_u64), Some(1..=600))
    {
        STEP_TIMEOUT
    } else {
        TIMEOUT
    }
}

pub(crate) fn ready(scenario: &str, seed: u64, step_ms: u128) {
    write_response(
        &json!({"protocol": PROTOCOL, "event": "ready", "scenario": scenario,
        "seed": seed, "step_ms": step_ms, "timeout_ms": TIMEOUT.as_millis()}),
    );
}

fn write_response(value: &Value) {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = writeln!(out, "{value}");
    let _ = out.flush();
}

fn error(id: Value, code: &str, message: &str) -> Value {
    json!({"protocol": PROTOCOL, "id": id, "ok": false,
        "error": {"code": code, "message": message}})
}

pub(crate) fn execute(app: &mut ClientApplication, line: &str) -> Value {
    let before = app.session.elapsed;
    let result = execute_native(app, line);
    app.advance_feedback(app.session.elapsed - before);
    app.show_session_feedback();
    result
}
fn execute_native(app: &mut ClientApplication, line: &str) -> Value {
    // Portable parser validates every envelope before any shell side effects.
    if app.e2e_step().is_none() {
        return error(Value::Null, "disabled", "automation requires --e2e");
    }
    let command: Value = match serde_json::from_str(line) {
        Ok(value) => value,
        Err(_) => return execute_session(&mut app.session, line),
    };
    let id = command.get("id").cloned().unwrap_or(Value::Null);
    if !command.is_object()
        || !(id.is_string() || id.is_number())
        || command.get("protocol").and_then(Value::as_u64) != Some(u64::from(PROTOCOL))
    {
        return execute_session(&mut app.session, line);
    }
    let result = match command.get("op").and_then(Value::as_str) {
        Some("scout_reference") => match command.get("view").and_then(Value::as_str) {
            Some("clear") => {
                app.scout_reference = None;
                Ok(json!({"view":"clear"}))
            }
            Some(name) => match crate::scout_reference::find(name) {
                Some(view) => {
                    app.scout_reference = Some(view);
                    Ok(json!({"view":name}))
                }
                None => Err(("invalid_view", "unknown Scout reference camera")),
            },
            None => Err(("invalid_view", "view is required")),
        },
        Some("restart") => Ok(json!({"applied":app.restart_terminal()})),
        Some("window") => {
            let action = command.get("action").and_then(Value::as_str).unwrap_or("");
            let size = command
                .get("width")
                .and_then(Value::as_u64)
                .zip(command.get("height").and_then(Value::as_u64))
                .and_then(|(w, h)| u32::try_from(w).ok().zip(u32::try_from(h).ok()))
                .map(|(w, h)| [w, h]);
            app.automation_window(action, size)
                .map(|()| json!({"requested":true}))
                .map_err(|_| {
                    (
                        "invalid_window",
                        "native window action or dimensions invalid",
                    )
                })
        }
        Some("capture") => match command.get("path").and_then(Value::as_str) {
            Some(path) => app
                .request_capture(path)
                .map(|()| json!({"scheduled":true}))
                .map_err(|_| {
                    (
                        "invalid_capture",
                        "new absolute .png path and capture-capable surface required",
                    )
                }),
            None => Err(("invalid_capture", "path is required")),
        },
        Some("performance_reset") => {
            app.observations.clear();
            Ok(json!({"reset":true}))
        }
        Some("inspect") => {
            let mut result = inspect_session(&app.session);
            let upload_times = app.terrain_upload_times();
            let (uploaded_bytes, peak_upload_bytes) = app.terrain_uploads();
            let native = json!({ "native_input_isolated":app.e2e_input_isolated, "terrain_upload_preparation_micros": upload_times,
            "visual_context": app.visual_context(),
                    "survival_details_open":app.survival_hud.details_open,
            "window": app.window_state(),
            "capture_status": app.capture_status,
            "performance": app.observations.snapshot(),
            "terrain": { "mode": format!("{:?}", app.session.mining.terrain), "active_patches": app.terrain_stream.active.len(),
                "cache_patches": app.terrain_stream.cached(), "pending_patches": app.terrain_stream.pending,
                "gpu_ready": app.terrain_gpu_ready(), "presentation_ready":app.terrain_presentation_ready(), "triangles": app.terrain_stream.triangles(), "cpu_cache_bytes": app.terrain_stream.cpu_bytes(),
                "gpu_cache_bytes": app.terrain_gpu_bytes(), "generated_last_frame": app.terrain_stream.generated,
                "selection_last_frame_micros": app.terrain_stream.selection_micros,
                "preparation_last_frame_micros": app.terrain_stream.preparation_micros,
                "preparation_peak_micros": app.terrain_stream.preparation_peak_micros,
                "preparation_overruns": app.terrain_stream.preparation_overruns,
                "generation_last_frame_micros": app.terrain_stream.generation_micros,
                "generation_peak_micros":app.terrain_stream.generation_peak_micros,
                "generation_total_micros":app.terrain_stream.generation_total_micros,
                "generated_peak":app.terrain_stream.generated_peak, "uploaded_last_frame_bytes":uploaded_bytes, "peak_upload_bytes":peak_upload_bytes, "total_generated": app.terrain_stream.total_generated },

                });
            for (key, value) in native.as_object().expect("native observation object") {
                result[key] = value.clone();
            }
            return json!({"protocol":PROTOCOL,"id":id,"ok":true,"result":result});
        }
        Some("key")
            if matches!(
                command.get("key").and_then(Value::as_str),
                Some(
                    "pause"
                        | "resume"
                        | "diagnostics"
                        | "status"
                        | "status_previous"
                        | "status_next"
                )
            ) =>
        {
            if let Some(pressed) = command.get("pressed").and_then(Value::as_bool) {
                let key = match command["key"].as_str() {
                    Some("pause") => KeyCode::Escape,
                    Some("resume") => KeyCode::Enter,
                    Some("status") => KeyCode::KeyH,
                    Some("status_previous") => KeyCode::BracketLeft,
                    Some("status_next") => KeyCode::BracketRight,
                    _ => KeyCode::F3,
                };
                app.automation_key(PhysicalKey::Code(key), pressed);
                Ok(json!({"applied":true}))
            } else {
                return execute_session(&mut app.session, line);
            }
        }
        _ => return execute_session(&mut app.session, line),
    };
    match result {
        Ok(result) => json!({"protocol":PROTOCOL,"id":id,"ok":true,"result":result}),
        Err((code, message)) => error(id, code, message),
    }
}

pub(crate) fn execute_session(app: &mut GameplaySession, line: &str) -> Value {
    if app.e2e_step().is_none() {
        return error(Value::Null, "disabled", "automation requires --e2e");
    }
    let command: Value = match serde_json::from_str(line) {
        Ok(value) => value,
        Err(_) => {
            return error(
                Value::Null,
                "invalid_json",
                "expected one JSON object per line",
            );
        }
    };
    let id = command.get("id").cloned().unwrap_or(Value::Null);
    if !command.is_object() || !(id.is_string() || id.is_number()) {
        return error(
            id,
            "invalid_request",
            "object with string or numeric id required",
        );
    }
    if command.get("protocol").and_then(Value::as_u64) != Some(u64::from(PROTOCOL)) {
        return error(id, "protocol_mismatch", "expected protocol 1");
    }
    if app.wheel.open
        && matches!(
            command.get("op").and_then(Value::as_str),
            Some("aim_fragment" | "restore_look" | "thruster")
        )
    {
        return json!({"protocol":PROTOCOL,"id":id,"ok":true,"result":{"applied":false}});
    }
    if app.is_paused()
        && !matches!(
            command.get("op").and_then(Value::as_str),
            Some("inspect" | "key" | "step")
        )
    {
        return error(id, "paused", "gameplay commands unavailable while paused");
    }
    let result = match command.get("op").and_then(Value::as_str) {
        Some("inspect") => Ok(inspect_session(app)),
        Some("consume") => {
            let slot = command
                .get("slot")
                .and_then(Value::as_u64)
                .filter(|n| (1..=5).contains(n))
                .map(|n| salimon_fabrication::ConsumableSlot::ALL[n as usize - 1]);
            let job = command
                .get("job")
                .and_then(Value::as_u64)
                .filter(|n| *n > 0);
            let unit = command
                .get("unit")
                .and_then(Value::as_u64)
                .and_then(|n| u32::try_from(n).ok())
                .filter(|n| *n > 0);
            match (slot, job, unit) {
                (Some(slot), Some(job), Some(unit)) => {
                    let result = app.consume_selection(crate::session::consumables::Selection {
                        slot,
                        job: salimon_fabrication::JobId(job),
                        unit,
                    });
                    match result {
                        Ok(item) => Ok(json!({"applied":true,"job":item.job.0,"unit":item.unit})),
                        Err(reason) => Ok(json!({"applied":false,"reason":format!("{reason:?}")})),
                    }
                }
                _ => Err((
                    "invalid_argument",
                    "slot 1..5, positive job and u32 unit required",
                )),
            }
        }
        Some("crafting") => {
            use crate::session::crafting_ui::CraftingAction as A;
            let action = match command.get("action").and_then(Value::as_str) {
                Some("close") => Some(A::Close),
                Some("previous_recipe") => Some(A::PreviousRecipe),
                Some("next_recipe") => Some(A::NextRecipe),
                Some("less") => Some(A::Less),
                Some("more") => Some(A::More),
                Some("next_job") => Some(A::NextJob),
                Some("cancel") => Some(A::Cancel),
                Some("add") => Some(A::Add),
                _ => None,
            };
            match action {
                Some(action) => {
                    let applied = app.crafting_ui.open;
                    app.crafting_action(action);
                    Ok(json!({"applied":applied}))
                }
                None => Err(("invalid_argument", "known crafting action required")),
            }
        }
        Some("step") => match command.get("frames").and_then(Value::as_u64) {
            Some(frames @ 1..=600) => {
                let step = app.e2e_step().expect("automation only runs in E2E mode");
                let started = Instant::now();
                for _ in 0..frames {
                    app.advance_automation(step);
                }
                Ok(json!({"frames": frames,"elapsed_ms":started.elapsed().as_secs_f64()*1000.0}))
            }
            _ => Err((
                "invalid_argument",
                "frames must be an integer from 1 to 600",
            )),
        },
        Some("look") => {
            let dx = command.get("dx").and_then(Value::as_f64);
            let dy = command.get("dy").and_then(Value::as_f64);
            match (dx, dy) {
                (Some(dx), Some(dy)) if dx.is_finite() && dy.is_finite() => {
                    app.command(crate::session::Command::Look(dx, dy));
                    app.sync_carried();
                    Ok(json!({"applied": true}))
                }
                _ => Err(("invalid_argument", "finite dx and dy numbers required")),
            }
        }
        Some("restore_look") => {
            if let Some([dx, dy]) = app.automation_aim_delta.take() {
                app.apply_look_delta(-dx, -dy);
                app.sync_carried();
                Ok(json!({"applied": true}))
            } else {
                Err(("invalid_argument", "aim_fragment must precede restore_look"))
            }
        }
        Some("aim_fragment") => {
            let fragment = command
                .get("fragment_id")
                .and_then(Value::as_u64)
                .and_then(|id| {
                    app.mining
                        .session
                        .fragments()
                        .iter()
                        .find(|piece| piece.id().0 == id)
                });
            if let Some(fragment) = fragment.copied() {
                use salimon_math::{cross, dot, length};
                let normalize = |v: [f64; 3]| {
                    let magnitude = length(v).max(1e-12);
                    v.map(|x| x / magnitude)
                };
                let ship = app.ship.snapshot();
                let frame = character_ship_frame(ship.pose);
                let side = salimon_world::resource_fragments::diameter_meters(fragment);
                let mut total = [0.0; 2];
                // Try the center first, then points inside the pickup
                // sphere. A pile can obscure its center while exposing an edge.
                for candidate in 0..9 {
                    let player = app.character.snapshot(frame, app.terrain_frame(ship));
                    let up = player.up.map(f64::from);
                    let current = normalize(std::array::from_fn(|i| {
                        player.look_target_meters[i] - player.eye_position_meters[i]
                    }));
                    let local = std::array::from_fn(|axis| {
                        if candidate == 0 {
                            0.0
                        } else if (candidate - 1) & (1 << axis) == 0 {
                            -0.25 * side
                        } else {
                            0.25 * side
                        }
                    });
                    let offset =
                        salimon_physics::rotate(fragment.transform().orientation_xyzw(), local);
                    let target = normalize(std::array::from_fn(|i| {
                        fragment.transform().position().meters()[i] - player.eye_position_meters[i]
                            + offset[i]
                    }));
                    let planar = |direction: [f64; 3]| {
                        normalize(std::array::from_fn(|i| {
                            direction[i] - up[i] * dot(direction, up)
                        }))
                    };
                    let a = planar(current);
                    let b = planar(target);
                    let yaw = (-dot(cross(a, b), up)).atan2(dot(a, b));
                    let pitch = dot(target, up).clamp(-1.0, 1.0).asin()
                        - dot(current, up).clamp(-1.0, 1.0).asin();
                    let delta = [yaw / 0.0022, -pitch / 0.0022];
                    app.apply_look_delta(delta[0], delta[1]);
                    total[0] += delta[0];
                    total[1] += delta[1];
                    app.sync_carried();
                    let player = app.character.snapshot(frame, app.terrain_frame(ship));
                    if crate::carrying::target(
                        &app.mining,
                        player,
                        frame,
                        ship.door_state == salimon_ship::DoorState::Open
                            && ship.door_open_fraction >= 0.95,
                    ) == Some(fragment.id())
                    {
                        break;
                    }
                }
                app.automation_aim_delta = Some(total);
                Ok(json!({"applied": true}))
            } else {
                Err(("invalid_argument", "existing fragment_id required"))
            }
        }
        Some("key") => {
            let key = command.get("key").and_then(Value::as_str);
            let pressed = command.get("pressed").and_then(Value::as_bool);
            match (key.and_then(key_code), pressed) {
                (Some(key), Some(pressed)) => {
                    crate::app::input::dispatch_session_key(app, PhysicalKey::Code(key), pressed);
                    Ok(json!({"key": command["key"], "pressed": pressed}))
                }
                _ => Err(("invalid_argument", "known key and boolean pressed required")),
            }
        }
        Some("wheel_hover") => {
            let slot = command.get("slot");
            let selection = slot
                .and_then(Value::as_u64)
                .and_then(|i| usize::try_from(i).ok())
                .and_then(|i| salimon_fabrication::ConsumableSlot::ALL.get(i).copied());
            if selection.is_some() || slot.is_some_and(Value::is_null) {
                app.command(crate::session::Command::Wheel(
                    crate::session::consumable_wheel::WheelAction::Hover(selection),
                ));
                Ok(json!({"applied":app.wheel.open}))
            } else {
                Err(("invalid_argument", "slot must be null or integer 0..4"))
            }
        }
        Some("mouse") => {
            match (
                command.get("button").and_then(Value::as_str),
                command.get("pressed").and_then(Value::as_bool),
            ) {
                (Some("left"), Some(pressed)) => {
                    if app.wheel.open {
                        app.command(crate::session::Command::Wheel(
                            crate::session::consumable_wheel::WheelAction::Primary(pressed),
                        ));
                    } else {
                        app.command(crate::session::Command::Mining(pressed));
                    }
                    Ok(json!({"button": "left", "pressed": pressed}))
                }
                (Some("right"), Some(true)) if app.building.active => {
                    app.command(crate::session::Command::Build(
                        crate::session::construction::BuildAction::Cancel,
                    ));
                    Ok(json!({"button":"right","pressed":true}))
                }
                (Some("right"), Some(false)) if app.building.active => {
                    Ok(json!({"button":"right","pressed":false}))
                }
                _ => Err((
                    "invalid_argument",
                    "left button and boolean pressed required",
                )),
            }
        }
        Some("interact") => {
            crate::app::input::dispatch_session_key(app, PhysicalKey::Code(KeyCode::KeyE), true);
            Ok(json!({"applied": app.view_name() == "Gameplay"}))
        }
        Some("landing") => {
            crate::app::input::dispatch_session_key(app, PhysicalKey::Code(KeyCode::KeyL), true);
            Ok(json!({"applied": app.view_name() == "Gameplay"}))
        }
        Some("thruster") => match command.get("direction").and_then(Value::as_i64) {
            Some(direction @ -1..=1) if direction != 0 => {
                let applied = app.view_name() == "Gameplay"
                    && app.character.location() == CharacterLocation::Cockpit
                    && app
                        .restrictions()
                        .permits(salimon_survival::restrictions::Action::Pilot)
                    && app.ship.adjust_thruster(direction as i8);
                Ok(json!({"applied": applied}))
            }
            _ => Err(("invalid_argument", "direction must be -1 or 1")),
        },
        _ => Err((
            "unknown_op",
            "expected inspect, step, look, aim_fragment, restore_look, key, mouse, interact, landing, thruster, crafting, or consume",
        )),
    };
    match result {
        Ok(result) => json!({"protocol": PROTOCOL, "id": id, "ok": true, "result": result}),
        Err((code, message)) => error(id, code, message),
    }
}

fn key_code(name: &str) -> Option<KeyCode> {
    match name {
        "consumable_wheel" => Some(KeyCode::KeyQ),
        "wheel_next" => Some(KeyCode::Tab),
        "wheel_confirm" => Some(KeyCode::Enter),
        "engines" => Some(KeyCode::KeyB),
        "build" => Some(KeyCode::KeyC),
        "rotate_blueprint" => Some(KeyCode::KeyR),
        "blueprint_6" => Some(KeyCode::Digit6),
        "blueprint_7" => Some(KeyCode::Digit7),
        "interact" => Some(KeyCode::KeyE),
        "recovery" => Some(KeyCode::KeyR),
        "pause" => Some(KeyCode::Escape),
        "status" => Some(KeyCode::KeyH),
        "skip_tutorial" => Some(KeyCode::KeyT),
        "resume" => Some(KeyCode::Enter),
        "view" => Some(KeyCode::F2),
        "diagnostics" => Some(KeyCode::F3),
        "near" => Some(KeyCode::KeyN),
        "thruster_up" => Some(KeyCode::ArrowUp),
        "thruster_down" => Some(KeyCode::ArrowDown),
        "forward" => Some(KeyCode::KeyW),
        "backward" => Some(KeyCode::KeyS),
        "left" => Some(KeyCode::KeyA),
        "right" => Some(KeyCode::KeyD),
        "jump" | "ascend" => Some(KeyCode::Space),
        "descend" => Some(KeyCode::ShiftLeft),
        "1" | "slot_1" => Some(KeyCode::Digit1),
        "2" | "slot_2" => Some(KeyCode::Digit2),
        "3" | "slot_3" => Some(KeyCode::Digit3),
        "4" | "slot_4" => Some(KeyCode::Digit4),
        "5" | "slot_5" => Some(KeyCode::Digit5),
        "6" => Some(KeyCode::Digit6),
        "grab_drop" | "pickup" | "drop" => Some(KeyCode::KeyF),
        "roll_left" => Some(KeyCode::ArrowLeft),
        "roll_right" => Some(KeyCode::ArrowRight),
        _ => None,
    }
}

pub(crate) fn inspect_session(app: &GameplaySession) -> Value {
    let snapshot = app.snapshot();
    let hud = crate::survival_hud::Observation::capture(app);
    let ship = snapshot.ship;
    let frame = character_ship_frame(ship.pose);
    let character = snapshot.character;
    let local_eye = frame.world_to_local(character.eye_position_meters);
    let local_look = frame.world_to_local(character.look_target_meters);
    let interaction = app.automation_interaction(local_eye, local_look);
    let bodies: Vec<_> = CELESTIAL_BODIES.iter().map(|body| {
        json!({"name": body.name, "center_meters": body.center.meters(),
            "radius_meters": body.radius_meters,
            "ship_surface_distance_meters": (body.center.meters().iter().zip(ship.pose.position_meters)
                .map(|(center, position)| (*center - position).powi(2)).sum::<f64>()).sqrt() - body.radius_meters})
    }).collect();
    let deposits = app.mining.nearby(character.eye_position_meters, app.seed);
    let (deposits, deposit_error) = match deposits {
        Ok(entries) => (entries.iter().copied().map(|entry| json!({
            "id": format!("{:?}:{}", entry.deposit.id().body, entry.deposit.id().local),
            "resource": entry.deposit.material().resource().key(),
            "position_meters": entry.deposit.position().meters(),
            "ship_local_position_meters": frame.world_to_local(entry.deposit.position().meters()),
            "body_local_position_meters": entry.body_local_position_meters,
            "bounds_radius_meters": entry.bounds_radius_meters,
            "remaining_mass_kg": entry.deposit.remaining_mass_kg(),
            "distance_to_player_meters": entry.deposit.position().offset_from(salimon_world::WorldPosition::new(character.eye_position_meters[0], character.eye_position_meters[1], character.eye_position_meters[2])).iter().map(|v| v * v).sum::<f64>().sqrt(),
            "state": format!("{:?}", entry.deposit.state()),
            "visual": Value::Null,
            "authored_visual": crate::resource_presentation::deposit_mesh_with_terrain(entry, app.mining.terrain).map(|mesh| json!({"mesh": format!("{:?}", mesh.mesh), "center_meters": mesh.center_meters, "side_meters": mesh.authored_mesh_scale}))
        })).collect::<Vec<_>>(), None),
        Err(error) => (Vec::new(), Some(format!("{error:?}"))),
    };
    let deposits_by_id: serde_json::Map<String, Value> = deposits
        .iter()
        .map(|entry| (entry["id"].as_str().unwrap().to_owned(), entry.clone()))
        .collect();
    let resource_types: std::collections::BTreeSet<_> = deposits
        .iter()
        .filter(|entry| entry["remaining_mass_kg"].as_f64().unwrap_or(0.0) > 0.0)
        .filter_map(|entry| entry["resource"].as_str())
        .collect();
    let nearest_deposit = deposits.iter().min_by(|a, b| {
        a["distance_to_player_meters"]
            .as_f64()
            .unwrap()
            .total_cmp(&b["distance_to_player_meters"].as_f64().unwrap())
    });
    let mining_target = app.mining.target(
        &app.equipment,
        character,
        frame,
        ship.door_state == salimon_ship::DoorState::Open && ship.door_open_fraction >= 0.95,
        app.seed,
    );
    let fragments: Vec<_> = app.mining
        .nearby_fragments(character.eye_position_meters)
        .map(|piece| {
            let mesh = crate::resource_presentation::fragment_visual(piece);
            let sphere = crate::resource_presentation::fragment_mesh(piece);
            json!({
                "id": piece.id().0,
                "reference_frame": if app.mining.ship_fragments.contains_key(&piece.id()) { "ship" } else { "world" },
                "carried": app.mining.session.carried_id() == Some(piece.id()),
                "attached_to_tool": app.mining.session.attached_sphere_id() == Some(piece.id()),
                "sleeping": app.mining.session.carried_id() != Some(piece.id()) && app.mining.fragment_sleep.is_sleeping(piece.id()),
                "distance_to_player_meters": piece.transform().position().offset_from(salimon_world::WorldPosition::new(character.eye_position_meters[0], character.eye_position_meters[1], character.eye_position_meters[2])).iter().map(|v| v * v).sum::<f64>().sqrt(),
                "ship_local_position_meters": app.mining.ship_fragments.get(&piece.id()).copied().map(salimon_character::ShipLocalPoint::meters).unwrap_or_else(|| frame.world_to_local(piece.transform().position().meters())),
                "contact_fragment_ids": if app.mining.session.carried_id() == Some(piece.id()) { Vec::new() } else { app.mining.fragment_sleep.contact_neighbors(piece.id()).iter().map(|id| id.0).collect::<Vec<_>>() },
                "environment_supported": app.mining.session.carried_id() != Some(piece.id()) && app.mining.fragment_sleep.environment_supported(piece.id(), |position, radius| salimon_character::ship_floor_placement(position, radius).is_some()),
                "source_deposit_id": format!("{:?}:{}", piece.source().body, piece.source().local),
                "resource": piece.material().resource().key(),
                "mass_kg": piece.material().mass_kg(), "volume_m3": piece.material().volume_m3(),
                "side_meters": salimon_world::resource_fragments::diameter_meters(piece),
                "position_meters": piece.transform().position().meters(),
                "orientation_xyzw": piece.transform().orientation_xyzw(),
                "sphere_visual": {"asset": "resource.spherical-fragment", "radius_meters": sphere.authored_mesh_scale * salimon_renderer::SPHERE_AUTHORED_RADIUS_METERS, "tint": sphere.tint},
                "visual": {"color": mesh.color, "half_extents_meters": mesh.half_extents_meters}
            })
        })
        .collect();
    let fragments_by_id: std::collections::BTreeMap<_, _> = fragments
        .iter()
        .map(|piece| {
            (
                piece["id"]
                    .as_u64()
                    .expect("physical fragment ID")
                    .to_string(),
                piece,
            )
        })
        .collect();
    let fragment_target = crate::carrying::target(
        &app.mining,
        character,
        frame,
        ship.door_state == salimon_ship::DoorState::Open && ship.door_open_fraction >= 0.95,
    );
    let resource_context = crate::resource_context::context(
        &app.mining,
        &app.equipment,
        character,
        frame,
        ship.door_state == salimon_ship::DoorState::Open && ship.door_open_fraction >= 0.95,
        app.seed,
    );
    let player_velocity = app
        .character
        .eva_velocity()
        .unwrap_or(ship.velocity_meters_per_second);
    let relative_velocity =
        std::array::from_fn::<_, 3, _>(|i| player_velocity[i] - ship.velocity_meters_per_second[i]);
    let (ground_entries, ground_hits, ground_misses) = crate::terrain::ground_cache_stats();
    let fabrication = app.fabrication_snapshot();
    let machinery = crate::scout_feedback::capture(app, ship);
    let item = |item: &salimon_fabrication::OutputItem| json!({"recipe":item.recipe.key(), "job":item.job.0, "unit":item.unit, "mass_kg":item.mass.kg(), "category":format!("{:?}",item.category), "effect":app.consumable_effects.get(item.recipe.key())});
    let mut observation = json!({
        "consumable_wheel": {"open":app.wheel.open,"held":app.wheel.held,"hovered":app.wheel.hovered.map(salimon_fabrication::ConsumableSlot::index)},
        "paused": app.is_paused(),
        "machinery": {
            "thrust":machinery.thrust, "rudder":machinery.rudder,
            "converting":machinery.converting, "core_charge":machinery.core_charge,
            "fabricator":format!("{:?}",machinery.fabricator),
            "fabrication_progress":machinery.fabrication_progress,
            "phase":machinery.phase, "cockpit_active":machinery.cockpit_active,
        },
        "fabricator": {
            "status":format!("{:?}",fabrication.status), "requested_watts":fabrication.requested_watts,
            "stored":fabrication.stored.iter().map(|bin| json!({"resource":bin.resource.key(),"milligrams":bin.milligrams})).collect::<Vec<_>>(),
            "jobs":fabrication.jobs.iter().map(|job| json!({"id":job.id.0,"recipe":job.recipe.key(),"remaining_quantity":job.remaining_quantity,"completed_quantity":job.completed_quantity,"progress_seconds":job.progress.as_secs_f64(),"unit_seconds":job.unit_duration.as_secs_f64()})).collect::<Vec<_>>(),
            "tray":fabrication.tray.as_ref().map(item), "queue_capacity":fabrication.queue_capacity,
            "ui_open":app.crafting_ui.open,
        },
        "consumables": {"food":app.consumables.food().iter().map(|slot|slot.as_ref().map(item)).collect::<Vec<_>>(), "medical":app.consumables.medical().iter().map(|slot|slot.as_ref().map(item)).collect::<Vec<_>>()},
        "simulation_backlog_millis": snapshot.backlog.as_secs_f64() * 1000.0,
        "resource_queries": app.mining.query_metrics(),
        "ground_cache": {"entries":ground_entries,"hits":ground_hits,"misses":ground_misses},
        "view_mode": app.view_name(),
        "terrain": { "mode": format!("{:?}", app.mining.terrain) },
        "resource_ui": {
            "context": resource_context.as_ref().map(|prompt| &prompt.text),
            "anchor_meters": resource_context.as_ref().and_then(|prompt| match prompt.placement {
                salimon_renderer::OverlayPlacement::World { anchor_meters, .. } => Some(anchor_meters),
                _ => None,
            }),
        },
        "journey_ended": app.survival.snapshot().terminal.then(||app.ended_run().map(|r|crate::journey_ended::Report::from(&r)).expect("validated terminal report")),
        "survival_hud": {"visible":hud.visible,"penalties":hud.penalties,"statuses":hud.statuses},
        "journey": {"active_duration":app.survival.snapshot().active_time, "distance_meters":app.run.journey.distance_meters, "location":crate::session::journey::NamedLocation::at(app.snapshot().character.eye_position_meters)},
        "survival": {"hunger":app.survival.snapshot().hunger.value(),
            "health":app.survival.snapshot().health.value(), "fatigue":app.survival.snapshot().fatigue.value(),
            "terminal":app.survival.snapshot().terminal,
            "effects":app.survival.active_effects().map(|(id,stacks,remaining)|
                json!({"id":id.0,"stacks":stacks,"remaining_seconds":remaining.map(|d|d.as_secs_f64())})).collect::<Vec<_>>()},
        "survival_restrictions": {
            "hunger_movement_bps": app.restrictions().hunger_movement_bps,
            "fatigue_movement_bps": app.restrictions().fatigue_movement_bps,
            "movement_bps": app.restrictions().movement_bps,
            "interaction_bps": app.restrictions().interaction_bps,
            "exhausted": app.restrictions().exhausted,
            "can_pilot": app.restrictions().permits(salimon_survival::restrictions::Action::Pilot),
            "can_mine": app.restrictions().permits(salimon_survival::restrictions::Action::Mine),
            "can_consume": app.restrictions().permits(salimon_survival::restrictions::Action::Consume)
        },
        "equipment": {"slots": app.equipment.slots().map(|tool| tool.map(|tool| match tool {
            crate::equipment::EquipmentTool::MiningTool => "mining_tool",
        })), "selected_slot": app.equipment.selected().map(|slot| slot.index() + 1)},
        "carrying": {"object_id": app.mining.session.carried_id().map(|id| id.0),
            "target_id": fragment_target.map(|id| id.0),
            "context": crate::carrying::context(&app.mining, fragment_target, character),
            "last_action_feedback": app.mining.carry_feedback},
        "mining": { "equipped": app.equipment.mining_equipped(), "held": app.mining.input.held,
            "active": app.mining.input.held && mining_target.is_some(),
            "held_item_visible": app.held_item(character, mining_target.is_some()).is_some(),
            "extracted_mass_kg": app.mining.session.extracted_mass_kg(),
            "range_meters": salimon_world::mining::MINING_RANGE_METERS,
            "rate_kg_per_second": app.mining.config.rate_kg_per_second(),
            "capacity_kg": app.mining.config.cap_kg(),
            "reserved_mass_kg": app.mining.session.attached_sphere_id().and_then(|id| app.mining.session.fragments().iter().find(|p| p.id() == id)).map_or(0.0, |p| p.material().mass_kg()),
            "attached_fragment_id": app.mining.session.attached_sphere_id().map(|id| id.0),
            "growth_preview": app.mining.growth_preview(&app.equipment, character).map(|mesh| json!({"center_meters": mesh.center_meters, "radius_meters": mesh.authored_mesh_scale * salimon_renderer::SPHERE_AUTHORED_RADIUS_METERS, "tint": mesh.tint})),
            "target": mining_target.map(|target| json!({"id": format!("{:?}:{}", target.id.body, target.id.local), "distance_meters": target.distance_meters})) },
        "player": {"nearby_body": app.mining.terrain.nearest_body(character.eye_position_meters, salimon_world::NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS, true).map(|(body, distance)| json!({"name": body.name, "surface_distance_meters": distance})),
            "velocity_meters_per_second": player_velocity,
            "speed_meters_per_second": player_velocity.iter().map(|v| v*v).sum::<f64>().sqrt(),
            "relative_velocity_meters_per_second": relative_velocity,
            "relative_speed_meters_per_second": relative_velocity.iter().map(|v| v*v).sum::<f64>().sqrt(),
            "location": format!("{:?}", character.location),
            "eye_position_meters": character.eye_position_meters,
            "ship_local_eye_position_meters": local_eye,
            "look_target_meters": character.look_target_meters, "up": character.up,
            "local_ship_position_meters": character.local_ship_position_meters},
        "ship": {"position_meters": ship.pose.position_meters, "orientation": ship.pose.orientation,
            "flight_state": format!("{:?}", ship.flight_state), "door_state": format!("{:?}", ship.door_state), "door_open_fraction": ship.door_open_fraction,
            "cockpit_control_active": ship.cockpit_control_active,
            "thruster_percentage": ship.thruster_percentage, "total_mass_kg": ship.total_mass_kg, "speed_meters_per_second": ship.speed_meters_per_second,
            "velocity_meters_per_second": ship.velocity_meters_per_second,
            "power_available": ship.power_available,
            "navigation": {"direction_local": ship.navigation.velocity_direction_local,
                "angle_degrees": ship.navigation.heading_velocity_angle_degrees,
                "acceleration": ship.navigation.acceleration_meters_per_second_squared,
                "braking": format!("{:?}", ship.navigation.braking)},
            "conversion_remaining_joules": ship.core_status.conversion_remaining_joules,
            "consumption_watts": ship.core_status.consumption_watts,
            "energy_core": {"stored_joules": ship.energy_core.stored_joules,
                "capacity_joules": ship.energy_core.capacity_joules},
            "nearby_body": ship.nearby_body.map(|body| json!({"name": body.name,
                "surface_distance_meters": body.surface_distance_meters,
                "radial_speed_meters_per_second": body.radial_speed_meters_per_second})),
            "cockpit_message": ship.cockpit_message.map(|message| message.text())},
        "interaction": interaction,
        "world": {"fragments": fragments, "fragments_by_id": fragments_by_id,
            "fragment_count": app.mining.session.fragments().len(),
            "sleeping_fragment_count": app.mining.session.fragments().iter().filter(|piece| app.mining.session.carried_id() != Some(piece.id()) && app.mining.fragment_sleep.is_sleeping(piece.id())).count(),
            "fragment_mass_kg": app.mining.session.fragments().iter().map(|piece| piece.material().mass_kg()).sum::<f64>(),
            "nearest_deposit": nearest_deposit, "resource_types": resource_types, "deposits": deposits, "deposits_by_id": deposits_by_id, "deposit_query_error": deposit_error, "bodies": bodies, "camera_phase": format!("{:?}", app.camera_phase())}
    });
    observation["cargo_door"] =
        json!({"open_fraction":app.cargo_door.open_fraction,"opening":app.cargo_door.opening});
    observation["construction"] = app.construction_inspect();
    observation["ship"]["engines_enabled"] = ship.engines_enabled.into();
    observation["archive"] =
        json!({"source":app.archive, "open":app.archive_reader.open, "text":app.archive_text()});
    observation["recovery"] = app.recovery_observation();
    observation["tutorial"] = json!({"state":app.tutorial,"prompt":app.tutorial.text()});
    observation
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{execute_session as execute, inspect_session as inspect};

    #[test]
    fn loaded_step_deadlines_are_bounded_and_ordinary_requests_stay_short() {
        for frames in [1, 299, 300, 600] {
            assert_eq!(
                command_timeout(
                    &json!({"protocol":1,"id":1,"op":"step","frames":frames}).to_string()
                ),
                Duration::from_secs(120)
            );
        }
        for request in [
            json!({"protocol":1,"id":1,"op":"inspect"}),
            json!({"protocol":1,"id":1,"op":"step","frames":0}),
            json!({"protocol":1,"id":1,"op":"step","frames":601}),
            json!({"protocol":2,"id":1,"op":"step","frames":600}),
            json!({"protocol":1,"id":1,"op":"step","frames":"600"}),
            json!({"protocol":1,"op":"step","frames":1}),
            json!({"protocol":1,"id":false,"op":"step","frames":1}),
        ] {
            assert_eq!(
                command_timeout(&request.to_string()),
                Duration::from_secs(5)
            );
        }
        assert_eq!(command_timeout("{"), Duration::from_secs(5));
    }

    use crate::e2e::{self, Config, Scenario};

    fn app(scenario: Scenario) -> GameplaySession {
        let mut app = GameplaySession::default();
        e2e::initialize(
            &mut app,
            Config {
                fragment_load: None,
                scenario,
                seed: 7,
                step: Duration::from_millis(20),
            },
        )
        .unwrap();
        app
    }

    #[test]
    fn scout_archive_fixture_and_observation_preserve_unread_source_until_real_e() {
        let mut s = app(Scenario::ScoutArchive);
        let before = s.save_bytes(100).unwrap();
        let initial = inspect(&s);
        assert_eq!(initial["interaction"], "archive");
        assert_eq!(initial["archive"]["source"]["read"], json!([]));
        assert_eq!(initial["archive"]["open"], false);
        assert_eq!(s.save_bytes(100).unwrap(), before);
        let result = execute(
            &mut s,
            r#"{"protocol":1,"id":1,"op":"key","key":"interact","pressed":true}"#,
        );
        assert_eq!(result["ok"], true);
        assert_eq!(inspect(&s)["archive"]["open"], true);
        assert_eq!(
            inspect(&s)["archive"]["source"]["read"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn deposit_walkthrough_uses_real_movement_and_generated_state() {
        let mut test = app(Scenario::LandedEarth);
        e2e::initialize(
            &mut test,
            Config {
                fragment_load: None,
                scenario: Scenario::LandedEarth,
                seed: 0,
                step: Duration::from_millis(16),
            },
        )
        .unwrap();
        let scenario: Value =
            serde_json::from_str(include_str!("../../../scenarios/resource-deposits.json"))
                .unwrap();
        for step in scenario["steps"].as_array().unwrap() {
            if let Some(action) = step.get("action") {
                let mut command = action.clone();
                command["protocol"] = json!(1);
                command["id"] = json!(1);
                assert_eq!(execute(&mut test, &command.to_string())["ok"], true);
            }
        }
        let state = inspect(&test);
        assert_eq!(state["player"]["location"], "Surface");
        assert_eq!(state["world"]["deposit_query_error"], Value::Null);
        assert_eq!(
            state["world"]["nearest_deposit"]["id"],
            "Earth:16443359315117775089"
        );
        assert!(
            state["world"]["nearest_deposit"]["distance_to_player_meters"]
                .as_f64()
                .unwrap()
                < 4.0
        );
        assert_eq!(
            state["world"]["nearest_deposit"]["resource"],
            "silicate-rock"
        );
        assert!(state["world"]["nearest_deposit"]["visual"].is_null());
        assert_eq!(
            state["world"]["nearest_deposit"]["authored_visual"]["mesh"],
            "SilicateDepositSlab"
        );
        let iron: Vec<_> = state["world"]["deposits"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entry| entry["resource"] == "iron-ore")
            .collect();
        assert!(!iron.is_empty());
        for entry in iron {
            assert!(entry["visual"].is_null());
            assert!(
                entry["authored_visual"]["mesh"]
                    .as_str()
                    .unwrap()
                    .starts_with("IronDeposit")
            );
            assert_eq!(
                entry["authored_visual"]["center_meters"],
                entry["position_meters"]
            );
        }
    }

    #[test]
    fn resource_prompt_anchors_match_deposit_and_carried_pose() {
        let mut test = app(Scenario::LandedEarth);
        e2e::initialize(
            &mut test,
            Config {
                fragment_load: None,
                scenario: Scenario::LandedEarth,
                seed: 0,
                step: Duration::from_millis(16),
            },
        )
        .unwrap();
        let scenario: Value =
            serde_json::from_str(include_str!("../../../scenarios/evidence/carrying.json"))
                .unwrap();
        let mut saw_deposit = false;
        let mut saw_carried = false;
        let mut saw_fragment = false;
        for (index, step) in scenario["steps"].as_array().unwrap().iter().enumerate() {
            if let Some(action) = step.get("action") {
                let mut command = action.clone();
                command["protocol"] = json!(1);
                command["id"] = json!(index);
                assert_eq!(execute(&mut test, &command.to_string())["ok"], true);
            }
            let state = inspect(&test);
            let anchor = &state["resource_ui"]["anchor_meters"];
            if anchor.is_null() {
                continue;
            }
            if let Some(id) = test.mining.session.carried_id() {
                let piece = test
                    .mining
                    .session
                    .fragments()
                    .iter()
                    .find(|piece| piece.id() == id)
                    .unwrap();
                assert_eq!(*anchor, json!(piece.transform().position().meters()));
                saw_carried = true;
            } else if state["carrying"]["target_id"].is_number() {
                let id = state["carrying"]["target_id"].as_u64().unwrap();
                let piece = test
                    .mining
                    .session
                    .fragments()
                    .iter()
                    .find(|piece| piece.id().0 == id)
                    .unwrap();
                assert_eq!(*anchor, json!(piece.transform().position().meters()));
                saw_fragment = true;
            } else {
                assert!(
                    state["world"]["deposits"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|entry| entry["position_meters"] == *anchor)
                );
                saw_deposit = true;
            }
        }
        assert!(saw_deposit && saw_fragment && saw_carried);
    }

    #[test]
    fn toolbar_stows_cancel_both_mining_inputs_and_never_resume_on_reselect() {
        let mut test = app(Scenario::LandedEarth);
        e2e::initialize(
            &mut test,
            Config {
                fragment_load: None,
                scenario: Scenario::LandedEarth,
                seed: 0,
                step: Duration::from_millis(16),
            },
        )
        .unwrap();
        let scenario: Value =
            serde_json::from_str(include_str!("../../../scenarios/mining.json")).unwrap();
        for step in scenario["steps"].as_array().unwrap() {
            if step.get("assert").and_then(|check| check["path"].as_str()) == Some("mining.active")
                && step["assert"]["equals"] == json!(true)
            {
                break;
            }
            if let Some(action) = step.get("action") {
                let mut command = action.clone();
                command["protocol"] = json!(1);
                command["id"] = json!(1);
                assert_eq!(execute(&mut test, &command.to_string())["ok"], true);
            }
        }
        assert_eq!(inspect(&test)["mining"]["active"], true);
        let ship = test.ship.snapshot();
        let frame = character_ship_frame(ship.pose);
        let player = test.character.snapshot(frame, test.terrain_frame(ship));
        for key in [
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
        ] {
            test.mining.set_mouse_held(true);
            crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(key), true);
            assert!(!test.mining.input.held);
            assert_eq!(inspect(&test)["mining"]["equipped"], false);
            assert_eq!(inspect(&test)["mining"]["target"], Value::Null);
            assert!(
                test.mining
                    .held_item(&test.equipment, player, true)
                    .is_none()
            );
            let dot = crate::reticle::image(true, &test.equipment).unwrap();
            let mass = test.mining.session.extracted_mass_kg();
            test.advance_game(Duration::from_millis(16));
            assert_eq!(test.mining.session.extracted_mass_kg(), mass);
            crate::app::input::dispatch_session_key(
                &mut test,
                PhysicalKey::Code(KeyCode::Digit1),
                true,
            );
            assert!(
                test.mining
                    .held_item(&test.equipment, player, true)
                    .is_some()
            );
            assert_ne!(
                crate::reticle::image(true, &test.equipment)
                    .unwrap()
                    .revision,
                dot.revision
            );
            assert!(
                !test.mining.input.held,
                "selection must not restart held mining"
            );
            assert!(test.mining.press_f());
            test.mining_mouse_input(true);
        }
        assert_eq!(key_code("equip_mining_tool"), None);
        crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(KeyCode::KeyM), true);
        assert!(
            test.equipment.mining_equipped(),
            "M cannot toggle selection"
        );
    }

    #[test]
    fn f_isolation_and_mouse_reset_use_shared_production_routes() {
        let mut test = app(Scenario::LandedEarth);
        e2e::initialize(
            &mut test,
            Config {
                fragment_load: None,
                scenario: Scenario::LandedEarth,
                seed: 0,
                step: Duration::from_millis(16),
            },
        )
        .unwrap();
        let scenario: Value =
            serde_json::from_str(include_str!("../../../scenarios/mining.json")).unwrap();
        for step in scenario["steps"].as_array().unwrap() {
            if step
                .get("assert")
                .and_then(|check| check["contains"].as_str())
                == Some("Hold left mouse to mine")
            {
                break;
            }
            if let Some(action) = step.get("action") {
                let mut command = action.clone();
                command["protocol"] = json!(1);
                command["id"] = json!(1);
                assert_eq!(execute(&mut test, &command.to_string())["ok"], true);
            }
        }
        assert!(inspect(&test)["mining"]["target"].is_object());
        assert_eq!(key_code("mine"), None);
        let old_mine = json!({"protocol": 1, "id": 1, "op": "key", "key": "mine", "pressed": true});
        assert_eq!(execute(&mut test, &old_mine.to_string())["ok"], false);
        let f = PhysicalKey::Code(KeyCode::KeyF);
        for _ in 0..3 {
            crate::app::input::dispatch_session_key(&mut test, f, true);
            test.advance_game(Duration::from_millis(16));
            assert!(!test.mining.input.held);
            assert_eq!(test.mining.session.extracted_mass_kg(), 0.0);
        }
        test.mining_mouse_input(true);
        crate::app::input::dispatch_session_key(&mut test, f, false);
        assert!(test.mining.input.held, "F release preserves mouse mining");
        test.advance_game(Duration::from_millis(16));
        assert!(test.mining.session.extracted_mass_kg() > 0.0);
        crate::app::input::dispatch_session_key(&mut test, f, true);
        test.mining_mouse_input(false);
        assert!(
            !test.mining.input.held,
            "held F cannot sustain mouse mining"
        );
        let mass = test.mining.session.extracted_mass_kg();
        test.advance_game(Duration::from_millis(16));
        assert_eq!(test.mining.session.extracted_mass_kg(), mass);
        test.mining_mouse_input(true);
        test.mining.clear_input(); // Native focus loss/cursor release use this reset.
        crate::app::input::dispatch_session_key(&mut test, f, true);
        assert!(!test.mining.input.held);
        test.mining_mouse_input(true);
        crate::app::input::dispatch_session_key(
            &mut test,
            PhysicalKey::Code(KeyCode::Digit2),
            true,
        );
        assert!(!test.mining.input.held);
        test.mining_mouse_input(true);
        assert!(
            !test.mining.input.held,
            "empty selection rejects mouse input"
        );
        crate::app::input::dispatch_session_key(
            &mut test,
            PhysicalKey::Code(KeyCode::Digit1),
            true,
        );
        assert!(!test.mining.input.held, "re-equip never resumes old input");
    }

    #[test]
    fn mining_real_inputs_gate_surface_equipment_aim_and_release() {
        let mut test = app(Scenario::LandedEarth);
        crate::app::input::dispatch_session_key(
            &mut test,
            PhysicalKey::Code(KeyCode::Digit1),
            true,
        );
        test.mining.input.held = true;
        test.advance_game(Duration::from_secs(1));
        assert_eq!(
            test.mining.session.extracted_mass_kg(),
            0.0,
            "inside cannot mine"
        );
        e2e::initialize(
            &mut test,
            Config {
                fragment_load: None,
                scenario: Scenario::LandedEarth,
                seed: 0,
                step: Duration::from_millis(16),
            },
        )
        .unwrap();
        // The fixture does not alter mining state; reset only test's initial input.
        test.mining = crate::mining::ResourceGameplay::default();
        test.equipment = crate::equipment::EquipmentToolbar::default();
        let scenario: Value =
            serde_json::from_str(include_str!("../../../scenarios/mining.json")).unwrap();
        for (index, step) in scenario["steps"].as_array().unwrap().iter().enumerate() {
            if let Some(action) = step.get("action") {
                let mut command = action.clone();
                command["protocol"] = json!(1);
                command["id"] = json!(index);
                assert_eq!(execute(&mut test, &command.to_string())["ok"], true);
            } else if let Some(check) = step.get("assert") {
                // Mining/deposit checks independently verify the real route and hold/release controls.
                let path = check["path"].as_str().unwrap();
                if !path.starts_with("mining.")
                    && !path.starts_with("world.")
                    && !path.starts_with("resource_ui.")
                {
                    continue;
                }
                let state = inspect(&test);
                let value = path.split('.').fold(&state, |value, key| {
                    if value.is_array() {
                        &value[key.parse::<usize>().unwrap()]
                    } else {
                        &value[key]
                    }
                });
                if let Some(expected) = check.get("equals") {
                    if value.is_number() && expected.is_number() {
                        assert_eq!(value.as_f64(), expected.as_f64(), "step {index}: {path}");
                    } else {
                        assert_eq!(value, expected, "step {index}: {path}");
                    }
                }
                if let Some(expected) = check.get("contains") {
                    assert!(
                        match value {
                            Value::Array(values) => values.contains(expected),
                            Value::String(text) => text.contains(expected.as_str().unwrap()),
                            _ => panic!("unsupported contains at step {index}: {path}"),
                        },
                        "step {index}: {path}: {value}"
                    );
                }
                if let Some(expected) = check.get("approx") {
                    assert!(
                        (value.as_f64().unwrap() - expected.as_f64().unwrap()).abs()
                            <= check["tolerance"].as_f64().unwrap(),
                        "step {index}: {path}: {value}"
                    );
                }
            }
        }
    }

    #[test]
    fn streaming_walkthrough_reloads_partial_and_depleted_state() {
        let mut test = app(Scenario::LandedEarth);
        e2e::initialize(
            &mut test,
            Config {
                fragment_load: None,
                scenario: Scenario::LandedEarth,
                seed: 0,
                step: Duration::from_millis(16),
            },
        )
        .unwrap();
        let scenario: Value =
            serde_json::from_str(include_str!("../../../scenarios/resource-streaming.json"))
                .unwrap();
        for (index, step) in scenario["steps"].as_array().unwrap().iter().enumerate() {
            if let Some(action) = step.get("action") {
                let mut command = action.clone();
                command["protocol"] = json!(1);
                command["id"] = json!(index);
                assert_eq!(execute(&mut test, &command.to_string())["ok"], true);
            } else if let Some(check) = step.get("assert") {
                let state = inspect(&test);
                let path = check["path"].as_str().unwrap();
                let value = path.split('.').try_fold(&state, |value, key| {
                    if value.is_array() {
                        value.get(key.parse::<usize>().unwrap())
                    } else {
                        value.get(key)
                    }
                });
                if let Some(expected) = check.get("exists") {
                    assert_eq!(
                        value.is_some(),
                        expected.as_bool().unwrap(),
                        "step {index}: {path}"
                    );
                    continue;
                }
                let value =
                    value.unwrap_or_else(|| panic!("missing step {index}: {path}: {state}"));
                if let Some(expected) = check.get("equals") {
                    if value.is_number() && expected.is_number() {
                        assert_eq!(value.as_f64(), expected.as_f64(), "step {index}: {path}");
                    } else {
                        assert_eq!(value, expected, "step {index}: {path}");
                    }
                } else if let Some(expected) = check.get("approx") {
                    assert!(
                        (value.as_f64().unwrap() - expected.as_f64().unwrap()).abs()
                            <= check["tolerance"].as_f64().unwrap(),
                        "step {index}: {path}: {value}"
                    );
                } else if let Some(expected) = check.get("gt") {
                    assert!(
                        value.as_f64().unwrap() > expected.as_f64().unwrap(),
                        "step {index}: {path}: {value}"
                    );
                } else if let Some(expected) = check.get("lt") {
                    assert!(
                        value.as_f64().unwrap() < expected.as_f64().unwrap(),
                        "step {index}: {path}: {value}"
                    );
                } else {
                    panic!("unhandled check at step {index}");
                }
            }
        }
    }

    #[test]
    fn physical_carrying_walkthrough_uses_real_gameplay_actions() {
        assert_gameplay_scenario(
            Scenario::LandedEarth,
            include_str!("../../../scenarios/carrying.json"),
        );
    }

    #[test]
    fn carrying_evidence_uses_the_same_f_contract() {
        assert_gameplay_scenario(
            Scenario::LandedEarth,
            include_str!("../../../scenarios/evidence/carrying.json"),
        );
    }

    #[test]
    fn equipped_f_pickup_consumes_the_press_and_e_does_not_release() {
        let mut test = app(Scenario::LandedEarth);
        let scenario: Value =
            serde_json::from_str(include_str!("../../../scenarios/carrying.json")).unwrap();
        e2e::initialize(
            &mut test,
            Config {
                fragment_load: None,
                scenario: Scenario::LandedEarth,
                seed: 0,
                step: Duration::from_millis(16),
            },
        )
        .unwrap();
        for step in scenario["steps"].as_array().unwrap() {
            if step.get("assert").and_then(|check| check["path"].as_str())
                == Some("carrying.target_id")
                && step["assert"]["equals"] == json!(4)
            {
                break;
            }
            if let Some(action) = step.get("action") {
                let mut command = action.clone();
                command["protocol"] = json!(1);
                command["id"] = json!(1);
                assert_eq!(execute(&mut test, &command.to_string())["ok"], true);
            }
        }
        crate::app::input::dispatch_session_key(
            &mut test,
            PhysicalKey::Code(KeyCode::Digit1),
            true,
        );
        assert!(test.equipment.mining_equipped());
        crate::app::input::dispatch_session_key(
            &mut test,
            PhysicalKey::Code(KeyCode::Digit1),
            true,
        );
        assert_eq!(inspect(&test)["equipment"]["selected_slot"], 1);
        let mass = test.mining.session.extracted_mass_kg();
        test.mining.set_mouse_held(true);
        test.interact();
        assert!(test.mining.session.carried_id().is_none());
        crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(KeyCode::KeyF), true);
        assert!(test.mining.session.carried_id().is_some());
        assert_eq!(inspect(&test)["equipment"]["selected_slot"], Value::Null);
        assert!(!test.equipment.mining_equipped());
        assert_eq!(inspect(&test)["mining"]["held_item_visible"], false);
        assert_eq!(test.equipment.presentation().selected, None);
        for key in ["slot_1", "slot_2", "slot_3", "slot_4", "slot_5"] {
            let request = json!({"protocol": 1, "id": 1, "op": "key", "key": key, "pressed": true});
            assert_eq!(execute(&mut test, &request.to_string())["ok"], true);
            assert_eq!(inspect(&test)["equipment"]["selected_slot"], Value::Null);
        }
        assert!(!test.mining.input.held);
        crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(KeyCode::KeyF), true);
        test.advance_game(Duration::from_secs(1));
        assert_eq!(test.mining.session.extracted_mass_kg(), mass);
        assert!(test.mining.session.carried_id().is_some());
        test.interact();
        assert!(test.mining.session.carried_id().is_some());
        crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(KeyCode::KeyF), false);
        crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(KeyCode::KeyF), true);
        assert!(test.mining.session.carried_id().is_none());
        assert!(
            !test.mining.input.held,
            "drop must not start F mining either"
        );
        assert_eq!(inspect(&test)["equipment"]["selected_slot"], Value::Null);
        crate::app::input::dispatch_session_key(
            &mut test,
            PhysicalKey::Code(KeyCode::Digit5),
            true,
        );
        assert_eq!(inspect(&test)["equipment"]["selected_slot"], 5);
        assert_eq!(inspect(&test)["equipment"]["slots"][4], Value::Null);
        crate::app::input::dispatch_session_key(
            &mut test,
            PhysicalKey::Code(KeyCode::Digit1),
            true,
        );
        assert_eq!(inspect(&test)["equipment"]["selected_slot"], 1);
        assert!(test.equipment.mining_equipped());
        assert_eq!(inspect(&test)["mining"]["held_item_visible"], true);
        assert!(
            !test.mining.input.held,
            "re-selection must not resume held mining"
        );
    }

    #[test]
    fn fragment_transfer_walkthrough_uses_real_gameplay_actions() {
        assert_gameplay_scenario(
            Scenario::LandedEarth,
            include_str!("../../../scenarios/fragment-transfer.json"),
        );
    }

    #[test]
    fn space_airlock_walkthrough_uses_real_gameplay_actions() {
        assert_gameplay_scenario(
            Scenario::OpenSpace,
            include_str!("../../../scenarios/space-airlock.json"),
        );
    }

    #[test]
    fn crash_states_map_to_powered_recovery_through_normal_landing_action() {
        let mut app = app(Scenario::OrbitEarth);
        let earth = salimon_world::CELESTIAL_BODIES
            .iter()
            .find(|body| body.id == salimon_world::CelestialBodyId::Earth)
            .unwrap();
        let mut pose = app.ship.snapshot().pose;
        pose.position_meters = salimon_math::add(
            earth.center.meters(),
            [
                0.0,
                earth.radius_meters + salimon_ship::COLLISION_RADIUS_METERS + 1.0,
                0.0,
            ],
        );
        app.ship =
            salimon_ship::ShipController::flying_with_velocity(pose, 50, [0.0, -1000.0, 0.0])
                .unwrap();
        execute(&mut app, r#"{"protocol":1,"id":1,"op":"step","frames":1}"#);
        let state = inspect(&app);
        assert_eq!(state["ship"]["flight_state"], "Bouncing { body: Earth }");
        assert_eq!(state["ship"]["thruster_percentage"], 0);
        execute(&mut app, r#"{"protocol":1,"id":2,"op":"landing"}"#);
        assert!(matches!(
            app.ship.snapshot().flight_state,
            salimon_ship::FlightState::Bouncing { .. }
        ));
        execute(
            &mut app,
            r#"{"protocol":1,"id":3,"op":"step","frames":200}"#,
        );
        assert_eq!(
            inspect(&app)["ship"]["flight_state"],
            "GroundedAfterCrash { body: Earth }"
        );
        assert_eq!(inspect(&app)["ship"]["speed_meters_per_second"], 0.0);
        let resting = app.ship.snapshot().pose;
        execute(&mut app, r#"{"protocol":1,"id":4,"op":"landing"}"#);
        app.advance_automation(Duration::from_secs(1));
        assert!(app.ship.snapshot().pose.position_meters[1] > resting.position_meters[1]);
        assert_eq!(
            inspect(&app)["ship"]["flight_state"],
            "AssistedLanding { body: Earth }"
        );
        app.advance_automation(Duration::from_secs(11));
        assert_eq!(
            inspect(&app)["ship"]["flight_state"],
            "Landed { body: Earth }"
        );
        assert_eq!(inspect(&app)["ship"]["speed_meters_per_second"], 0.0);
    }

    #[test]
    fn signed_held_throttle_uses_gameplay_keys_and_fixed_simulation_time() {
        let mut app = app(Scenario::OpenSpace);
        let key = |app: &mut GameplaySession, pressed| {
            execute(
                app,
                &json!({"protocol":1,"id":1,"op":"key","key":"thruster_down","pressed":pressed})
                    .to_string(),
            )
        };
        assert_eq!(key(&mut app, true)["ok"], true);
        key(&mut app, true);
        assert_eq!(app.ship.snapshot().thruster_percentage, -1);
        execute(
            &mut app,
            r#"{"protocol":1,"id":2,"op":"step","frames":100}"#,
        );
        assert_eq!(app.ship.snapshot().thruster_percentage, -21);
        assert!(app.ship.snapshot().velocity_meters_per_second[0] < 0.0);
        key(&mut app, false);
        execute(
            &mut app,
            r#"{"protocol":1,"id":3,"op":"step","frames":100}"#,
        );
        assert_eq!(app.ship.snapshot().thruster_percentage, -21);
        execute(&mut app, r#"{"protocol":1,"id":4,"op":"interact"}"#);
        let response = execute(
            &mut app,
            r#"{"protocol":1,"id":5,"op":"thruster","direction":1}"#,
        );
        assert_eq!(response["result"]["applied"], false);
        key(&mut app, true);
        app.advance_automation(Duration::from_secs(1));
        assert_eq!(app.ship.snapshot().thruster_percentage, -21);
    }

    #[test]
    fn moving_eva_walkthrough_uses_real_gameplay_actions() {
        assert_gameplay_scenario(
            Scenario::CoastingSpace,
            include_str!("../../../scenarios/moving-eva.json"),
        );
    }

    #[test]
    fn nearby_eva_walkthrough_uses_real_gameplay_actions() {
        assert_gameplay_scenario(
            Scenario::EvaApproach,
            include_str!("../../../scenarios/nearby-eva.json"),
        );
    }

    #[test]
    fn boarding_ramp_walkthrough_preserves_contact_and_rejects_occupied_closure() {
        assert_gameplay_scenario(
            Scenario::LandedEarth,
            include_str!("../../../scenarios/boarding-ramp.json"),
        );
    }

    #[test]
    fn cockpit_nose_walkthrough_uses_real_gameplay_actions() {
        assert_gameplay_scenario(
            Scenario::LandedEarth,
            include_str!("../../../scenarios/cockpit-nose.json"),
        );
    }

    #[test]
    fn resource_loop_uses_real_landing_mining_and_cabin_delivery_actions() {
        assert_gameplay_scenario(
            Scenario::ResourceApproach,
            include_str!("../../../scenarios/resource-loop.json"),
        );
    }

    #[test]
    fn mining_emission_walkthrough_preserves_rate_and_repeated_output() {
        assert_gameplay_scenario(
            Scenario::LandedEarth,
            include_str!("../../../scenarios/mining-emission.json"),
        );
    }

    #[test]
    fn assisted_baselines_preserve_landing_and_handoff_to_inertial_takeoff() {
        assert_gameplay_scenario(
            Scenario::OrbitEarth,
            include_str!("../../../scenarios/orbit-earth.json"),
        );
        assert_gameplay_scenario(
            Scenario::LandedEarth,
            include_str!("../../../scenarios/landed-earth.json"),
        );
    }

    fn assert_gameplay_scenario(initial: Scenario, source: &str) {
        let mut test = app(initial);
        let scenario: Value = serde_json::from_str(source).unwrap();
        e2e::initialize(
            &mut test,
            Config {
                fragment_load: None,
                scenario: initial,
                seed: scenario["setup"]["seed"].as_u64().unwrap(),
                step: Duration::from_millis(scenario["setup"]["step_ms"].as_u64().unwrap()),
            },
        )
        .unwrap();
        for (index, step) in scenario["steps"].as_array().unwrap().iter().enumerate() {
            if let Some(wait) = step.get("wait") {
                let path = wait["path"].as_str().unwrap();
                let interval_ms = scenario["setup"]["step_ms"].as_u64().unwrap()
                    * step["frames"].as_u64().unwrap();
                let timeout_ms = step["timeout_ms"].as_u64().unwrap();
                let mut elapsed = 0;
                loop {
                    let state = inspect(&test);
                    let value = path.split('.').fold(&state, |value, key| {
                        if value.is_array() {
                            &value[key.parse::<usize>().unwrap()]
                        } else {
                            &value[key]
                        }
                    });
                    if value == &wait["equals"] {
                        break;
                    }
                    assert!(
                        elapsed < timeout_ms,
                        "wait at step {index}: {path}: {value}"
                    );
                    test.advance_automation(Duration::from_millis(interval_ms));
                    elapsed += interval_ms;
                }
            } else if let Some(action) = step.get("action") {
                let mut command = action.clone();
                command["protocol"] = json!(1);
                command["id"] = json!(index);
                assert_eq!(execute(&mut test, &command.to_string())["ok"], true);
                if let Some(id) = test.mining.session.carried_id() {
                    let state = inspect(&test);
                    let piece = state["world"]["fragments"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|p| p["id"].as_u64() == Some(id.0))
                        .expect("carried identity stays visible");
                    assert!(
                        {
                            let radius =
                                piece["side_meters"].as_f64().unwrap() * 0.5 * 3.0_f64.sqrt();
                            let eye = &state["player"]["eye_position_meters"];
                            let up = &state["player"]["up"];
                            let delta: [f64; 3] = std::array::from_fn(|i| {
                                piece["position_meters"][i].as_f64().unwrap()
                                    - eye[i].as_f64().unwrap()
                            });
                            let up: [f64; 3] = std::array::from_fn(|i| up[i].as_f64().unwrap());
                            let height = salimon_math::dot(delta, up);
                            let planar = std::array::from_fn(|i| delta[i] - up[i] * height);
                            let distance = salimon_math::length(planar);
                            distance > radius + 0.6 && distance < radius + 0.7
                        },
                        "carried pose at step {index}"
                    );
                }
            } else if let Some(check) = step.get("assert") {
                let state = inspect(&test);
                let path = check["path"].as_str().unwrap();
                let value = path.split('.').try_fold(&state, |value, key| {
                    if value.is_array() {
                        value.get(key.parse::<usize>().unwrap())
                    } else {
                        value.get(key)
                    }
                });
                if let Some(expected) = check.get("exists") {
                    assert_eq!(
                        value.is_some(),
                        expected.as_bool().unwrap(),
                        "step {index}: {path}"
                    );
                    continue;
                }
                let value = value.unwrap_or_else(|| panic!("missing step {index}: {path}"));
                if let Some(expected) = check.get("equals") {
                    if value.is_number() && expected.is_number() {
                        assert_eq!(value.as_f64(), expected.as_f64(), "step {index}: {path}");
                    } else if let (Some(actual), Some(expected)) =
                        (value.as_array(), expected.as_array())
                    {
                        assert_eq!(actual.len(), expected.len(), "step {index}: {path}");
                        for (actual, expected) in actual.iter().zip(expected) {
                            if actual.is_number() && expected.is_number() {
                                assert_eq!(
                                    actual.as_f64(),
                                    expected.as_f64(),
                                    "step {index}: {path}"
                                );
                            } else {
                                assert_eq!(actual, expected, "step {index}: {path}");
                            }
                        }
                    } else {
                        assert_eq!(value, expected, "step {index}: {path}");
                    }
                } else if let Some(expected) = check.get("contains") {
                    assert!(
                        match value {
                            Value::Array(values) => values.contains(expected),
                            Value::String(text) => text.contains(expected.as_str().unwrap()),
                            _ => panic!("unsupported contains at step {index}: {path}"),
                        },
                        "step {index}: {path}: {value}"
                    );
                } else if let Some(expected) = check.get("approx") {
                    assert!(
                        (value.as_f64().unwrap() - expected.as_f64().unwrap()).abs()
                            <= check["tolerance"].as_f64().unwrap(),
                        "step {index}: {path}: {value}"
                    );
                } else if let Some(expected) = check.get("gt") {
                    assert!(
                        value.as_f64().unwrap() > expected.as_f64().unwrap(),
                        "step {index}: {path}: {value}"
                    );
                } else if let Some(expected) = check.get("lt") {
                    assert!(
                        value.as_f64().unwrap() < expected.as_f64().unwrap(),
                        "step {index}: {path}: {value}"
                    );
                } else {
                    panic!("unhandled check at step {index}");
                }
            }
        }
    }

    #[test]
    fn protocol_rejects_malformed_and_unavailable_commands() {
        let mut normal = GameplaySession::default();
        assert_eq!(
            execute(&mut normal, r#"{"protocol":1,"id":1,"op":"inspect"}"#)["error"]["code"],
            "disabled"
        );
        let mut test = app(Scenario::LandedEarth);
        assert_eq!(execute(&mut test, "{")["error"]["code"], "invalid_json");
        assert_eq!(
            execute(&mut test, r#"{"protocol":2,"id":"a","op":"inspect"}"#)["error"]["code"],
            "protocol_mismatch"
        );
        assert_eq!(
            execute(
                &mut test,
                r#"{"protocol":1,"id":2,"op":"step","frames":601}"#
            )["error"]["code"],
            "invalid_argument"
        );
        assert_eq!(
            execute(&mut test, r#"{"protocol":1,"id":3,"op":"warp"}"#)["error"]["code"],
            "unknown_op"
        );
    }

    #[test]
    fn commands_use_live_controller_paths_and_inspection() {
        let mut test = app(Scenario::LandedEarth);
        let before = execute(&mut test, r#"{"protocol":1,"id":1,"op":"inspect"}"#);
        assert_eq!(before["result"]["player"]["location"], "InsideShip");
        assert_eq!(
            before["result"]["world"]["bodies"]
                .as_array()
                .unwrap()
                .len(),
            6
        );
        assert_eq!(
            execute(
                &mut test,
                r#"{"protocol":1,"id":2,"op":"key","key":"forward","pressed":true}"#
            )["ok"],
            true
        );
        assert_eq!(
            execute(&mut test, r#"{"protocol":1,"id":3,"op":"step","frames":5}"#)["ok"],
            true
        );
        let after = execute(&mut test, r#"{"protocol":1,"id":4,"op":"inspect"}"#);
        assert_ne!(
            before["result"]["player"]["eye_position_meters"],
            after["result"]["player"]["eye_position_meters"]
        );
        execute(
            &mut test,
            r#"{"protocol":1,"id":5,"op":"key","key":"forward","pressed":false}"#,
        );
        let stable = execute(&mut test, r#"{"protocol":1,"id":6,"op":"inspect"}"#);
        execute(&mut test, r#"{"protocol":1,"id":7,"op":"step","frames":1}"#);
        assert_eq!(
            stable["result"]["player"]["eye_position_meters"],
            execute(&mut test, r#"{"protocol":1,"id":8,"op":"inspect"}"#)["result"]["player"]["eye_position_meters"]
        );
    }

    #[test]
    fn cockpit_commands_respect_authority() {
        let mut test = app(Scenario::OrbitEarth);
        let before = test.ship.snapshot().thruster_percentage;
        execute(
            &mut test,
            r#"{"protocol":1,"id":1,"op":"thruster","direction":1}"#,
        );
        assert_eq!(test.ship.snapshot().thruster_percentage, before + 1);
        execute(&mut test, r#"{"protocol":1,"id":2,"op":"interact"}"#);
        assert_eq!(test.character.location(), CharacterLocation::InsideShip);
        assert_eq!(
            execute(
                &mut test,
                r#"{"protocol":1,"id":3,"op":"thruster","direction":1}"#
            )["result"]["applied"],
            false
        );
        assert_eq!(test.ship.snapshot().thruster_percentage, before + 1);
    }
}
