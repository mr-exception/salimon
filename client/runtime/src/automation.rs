//! Line-delimited JSON control channel, available only after an E2E renderer is ready.
//! The reader never touches game state; all commands run on the native event-loop thread.

use std::io::{self, BufRead, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::app::{ClientApplication, character_ship_frame, surface_frame_for_ship};
use salimon_character::CharacterLocation;
use salimon_world::CELESTIAL_BODIES;

const PROTOCOL: u8 = 1;
const TIMEOUT: Duration = Duration::from_secs(5);
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
            let request = Request {
                line,
                deadline: Instant::now() + TIMEOUT,
                reply,
            };
            if sender.send(request).is_err() {
                break;
            }
            match answer.recv_timeout(TIMEOUT) {
                Ok(value) => write_response(&value),
                Err(_) => write_response(&error(Value::Null, "timeout", "command timed out")),
            }
        }
    });
    receiver
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
    let result = match command.get("op").and_then(Value::as_str) {
        Some("inspect") => Ok(inspect(app)),
        Some("step") => match command.get("frames").and_then(Value::as_u64) {
            Some(frames @ 1..=600) => {
                let step = app.e2e_step().expect("automation only runs in E2E mode");
                for _ in 0..frames {
                    app.advance_game(step);
                }
                Ok(json!({"frames": frames}))
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
                    app.character.apply_mouse_delta(dx, dy);
                    app.sync_carried();
                    Ok(json!({"applied": true}))
                }
                _ => Err(("invalid_argument", "finite dx and dy numbers required")),
            }
        }
        Some("key") => {
            let key = command.get("key").and_then(Value::as_str);
            let pressed = command.get("pressed").and_then(Value::as_bool);
            match (key.and_then(key_code), pressed) {
                (Some(key), Some(pressed)) => {
                    app.automation_key(PhysicalKey::Code(key), pressed);
                    Ok(json!({"key": command["key"], "pressed": pressed}))
                }
                _ => Err(("invalid_argument", "known key and boolean pressed required")),
            }
        }
        Some("interact") => {
            app.interact();
            Ok(json!({"applied": true}))
        }
        Some("landing") => {
            app.ship.trigger_landing_action();
            Ok(json!({"applied": true}))
        }
        Some("thruster") => match command.get("direction").and_then(Value::as_i64) {
            Some(direction @ -1..=1) if direction != 0 => {
                if app.character.location() == CharacterLocation::Cockpit {
                    app.ship.adjust_thruster(direction as i8);
                }
                Ok(json!({"applied": app.character.location() == CharacterLocation::Cockpit}))
            }
            _ => Err(("invalid_argument", "direction must be -1 or 1")),
        },
        _ => Err((
            "unknown_op",
            "expected inspect, step, look, key, interact, landing, or thruster",
        )),
    };
    match result {
        Ok(result) => json!({"protocol": PROTOCOL, "id": id, "ok": true, "result": result}),
        Err((code, message)) => error(id, code, message),
    }
}

fn key_code(name: &str) -> Option<KeyCode> {
    match name {
        "forward" => Some(KeyCode::KeyW),
        "backward" => Some(KeyCode::KeyS),
        "left" => Some(KeyCode::KeyA),
        "right" => Some(KeyCode::KeyD),
        "jump" | "ascend" => Some(KeyCode::Space),
        "descend" => Some(KeyCode::ShiftLeft),
        "equip_mining_tool" => Some(KeyCode::KeyM),
        "pickup" => Some(KeyCode::KeyQ),
        "drop" => Some(KeyCode::KeyG),
        "mine" => Some(KeyCode::KeyF),
        "roll_left" => Some(KeyCode::ArrowLeft),
        "roll_right" => Some(KeyCode::ArrowRight),
        _ => None,
    }
}

fn inspect(app: &ClientApplication) -> Value {
    let ship = app.ship.snapshot();
    let frame = character_ship_frame(ship.pose);
    let character = app.character.snapshot(frame, surface_frame_for_ship(ship));
    let local_eye = frame.world_to_local(character.eye_position_meters);
    let local_look = frame.world_to_local(character.look_target_meters);
    let interaction = app.automation_interaction(local_eye, local_look);
    let bodies: Vec<_> = CELESTIAL_BODIES.iter().map(|body| {
        json!({"name": body.name, "center_meters": body.center.meters(),
            "radius_meters": body.radius_meters,
            "ship_surface_distance_meters": (body.center.meters().iter().zip(ship.pose.position_meters)
                .map(|(center, position)| (*center - position).powi(2)).sum::<f64>()).sqrt() - body.radius_meters})
    }).collect();
    let deposits = app.mining.nearby(
        character.eye_position_meters,
        app.e2e_config.map_or(0, |config| config.seed),
    );
    let (deposits, deposit_error) = match deposits {
        Ok(entries) => (entries.into_iter().map(|entry| json!({
            "id": format!("{:?}:{}", entry.deposit.id().body, entry.deposit.id().local),
            "resource": entry.deposit.material().resource().key(),
            "position_meters": entry.deposit.position().meters(),
            "ship_local_position_meters": frame.world_to_local(entry.deposit.position().meters()),
            "body_local_position_meters": entry.body_local_position_meters,
            "bounds_radius_meters": entry.bounds_radius_meters,
            "remaining_mass_kg": entry.deposit.remaining_mass_kg(),
            "distance_to_player_meters": entry.deposit.position().offset_from(salimon_world::WorldPosition::new(character.eye_position_meters[0], character.eye_position_meters[1], character.eye_position_meters[2])).iter().map(|v| v * v).sum::<f64>().sqrt(),
            "state": format!("{:?}", entry.deposit.state()),
            "visual": crate::resource_presentation::visual(entry).map(|mesh| json!({"color": mesh.color, "half_extents_meters": mesh.half_extents_meters}))
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
        character,
        frame,
        ship.door_state == salimon_ship::DoorState::Open,
        app.e2e_config.map_or(0, |config| config.seed),
    );
    let fragments: Vec<_> = app
        .mining
        .nearby_fragments(character.eye_position_meters)
        .map(|piece| {
            let mesh = crate::resource_presentation::fragment_visual(piece);
            json!({
                "id": piece.id().0,
                "in_cargo_room": crate::cargo::stored(&app.mining, piece),
                "reference_frame": if app.mining.ship_fragments.contains_key(&piece.id()) { "ship" } else { "world" },
                "carried": app.mining.session.carried_id() == Some(piece.id()),
                "distance_to_player_meters": piece.transform().position().offset_from(salimon_world::WorldPosition::new(character.eye_position_meters[0], character.eye_position_meters[1], character.eye_position_meters[2])).iter().map(|v| v * v).sum::<f64>().sqrt(),
                "ship_local_position_meters": frame.world_to_local(piece.transform().position().meters()),
                "source_deposit_id": format!("{:?}:{}", piece.source().body, piece.source().local),
                "resource": piece.material().resource().key(),
                "mass_kg": piece.material().mass_kg(), "volume_m3": piece.material().volume_m3(),
                "side_meters": salimon_world::resource_fragments::side_meters(piece),
                "position_meters": piece.transform().position().meters(),
                "orientation_xyzw": piece.transform().orientation_xyzw(),
                "visual": {"color": mesh.color, "half_extents_meters": mesh.half_extents_meters}
            })
        })
        .collect();
    let cargo_fragments: Vec<_> = app
        .mining
        .session
        .fragments()
        .iter()
        .copied()
        .filter(|piece| crate::cargo::stored(&app.mining, *piece))
        .map(|piece| {
            json!({"id": piece.id().0,
            "source_deposit_id": format!("{:?}:{}", piece.source().body, piece.source().local),
            "resource": piece.material().resource().key(), "mass_kg": piece.material().mass_kg(),
            "volume_m3": piece.material().volume_m3(),
            "ship_local_position_meters": app.mining.ship_fragments[&piece.id()]})
        })
        .collect();
    let fragment_target = crate::carrying::target(
        &app.mining,
        character,
        frame,
        ship.door_state == salimon_ship::DoorState::Open,
    );
    let resource_context = crate::resource_context::context(
        &app.mining,
        character,
        frame,
        ship.door_state == salimon_ship::DoorState::Open,
        app.e2e_config.map_or(0, |config| config.seed),
    );
    let player_velocity = app
        .character
        .eva_velocity()
        .unwrap_or(ship.velocity_meters_per_second);
    let relative_velocity =
        std::array::from_fn::<_, 3, _>(|i| player_velocity[i] - ship.velocity_meters_per_second[i]);
    json!({
        "resource_ui": {"context": resource_context},
        "carrying": {"object_id": app.mining.session.carried_id().map(|id| id.0),
            "target_id": fragment_target.map(|id| id.0),
            "context": crate::carrying::context(&app.mining, fragment_target, character),
            "last_action_feedback": app.mining.carry_feedback},
        "mining": { "equipped": app.mining.equipped, "held": app.mining.held,
            "active": app.mining.held && mining_target.is_some(),
            "extracted_mass_kg": app.mining.session.extracted_mass_kg(),
            "range_meters": salimon_world::mining::MINING_RANGE_METERS,
            "rate_kg_per_second": salimon_world::mining::MINING_RATE_KG_PER_SECOND,
            "target": mining_target.map(|target| json!({"id": format!("{:?}:{}", target.id.body, target.id.local), "distance_meters": target.distance_meters})) },
        "player": {"nearby_body": salimon_world::nearby_solid_body(salimon_world::WorldPosition::new(character.eye_position_meters[0], character.eye_position_meters[1], character.eye_position_meters[2])).map(|(body, distance)| json!({"name": body.name, "surface_distance_meters": distance})),
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
            "flight_state": format!("{:?}", ship.flight_state), "door_state": format!("{:?}", ship.door_state),
            "cockpit_control_active": ship.cockpit_control_active,
            "thruster_percentage": ship.thruster_percentage, "speed_meters_per_second": ship.speed_meters_per_second,
            "velocity_meters_per_second": ship.velocity_meters_per_second,
            "energy_core": {"stored_joules": ship.energy_core.stored_joules,
                "capacity_joules": ship.energy_core.capacity_joules},
            "nearby_body": ship.nearby_body.map(|body| json!({"name": body.name,
                "surface_distance_meters": body.surface_distance_meters,
                "radial_speed_meters_per_second": body.radial_speed_meters_per_second})),
            "cockpit_message": ship.cockpit_message.map(|message| message.text())},
        "cargo": {"bounds_min_meters": salimon_character::CARGO_ROOM_MIN_METERS,
            "bounds_max_meters": salimon_character::CARGO_ROOM_MAX_METERS,
            "fragment_count": cargo_fragments.len(), "fragments": cargo_fragments},
        "interaction": interaction,
        "world": {"fragments": fragments,
            "fragment_count": app.mining.session.fragments().len(),
            "fragment_mass_kg": app.mining.session.fragments().iter().map(|piece| piece.material().mass_kg()).sum::<f64>(),
            "nearest_deposit": nearest_deposit, "resource_types": resource_types, "deposits": deposits, "deposits_by_id": deposits_by_id, "deposit_query_error": deposit_error, "bodies": bodies, "camera_phase": format!("{:?}", app.camera_phase())}
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::e2e::{self, Config, Scenario};

    fn app(scenario: Scenario) -> ClientApplication {
        let mut app = ClientApplication::default();
        e2e::initialize(
            &mut app,
            Config {
                scenario,
                seed: 7,
                step: Duration::from_millis(20),
            },
        )
        .unwrap();
        app
    }

    #[test]
    fn deposit_walkthrough_uses_real_movement_and_generated_state() {
        let mut test = app(Scenario::LandedEarth);
        e2e::initialize(
            &mut test,
            Config {
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
        assert!(state["world"]["nearest_deposit"]["visual"].is_object());
    }

    #[test]
    fn mining_real_inputs_gate_surface_equipment_aim_and_release() {
        let mut test = app(Scenario::LandedEarth);
        test.mining.toggle();
        test.mining.held = true;
        test.advance_game(Duration::from_secs(1));
        assert_eq!(
            test.mining.session.extracted_mass_kg(),
            0.0,
            "inside cannot mine"
        );
        e2e::initialize(
            &mut test,
            Config {
                scenario: Scenario::LandedEarth,
                seed: 0,
                step: Duration::from_millis(16),
            },
        )
        .unwrap();
        // The fixture does not alter mining state; reset only test's initial input.
        test.mining = crate::mining::MiningTool::default();
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
                        "step {index}: {path}"
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
                        "step {index}: {path}"
                    );
                } else if let Some(expected) = check.get("lt") {
                    assert!(
                        value.as_f64().unwrap() < expected.as_f64().unwrap(),
                        "step {index}: {path}"
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
    fn moving_eva_walkthrough_uses_real_gameplay_actions() {
        assert_gameplay_scenario(
            Scenario::OpenSpace,
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
    fn physical_cargo_walkthrough_uses_real_gameplay_actions() {
        assert_gameplay_scenario(
            Scenario::LandedEarth,
            include_str!("../../../scenarios/physical-cargo.json"),
        );
    }

    #[test]
    fn resource_loop_uses_real_landing_mining_and_cargo_actions() {
        assert_gameplay_scenario(
            Scenario::ResourceApproach,
            include_str!("../../../scenarios/resource-loop.json"),
        );
    }

    fn assert_gameplay_scenario(initial: Scenario, source: &str) {
        let mut test = app(initial);
        let scenario: Value = serde_json::from_str(source).unwrap();
        e2e::initialize(
            &mut test,
            Config {
                scenario: initial,
                seed: scenario["setup"]["seed"].as_u64().unwrap(),
                step: Duration::from_millis(scenario["setup"]["step_ms"].as_u64().unwrap()),
            },
        )
        .unwrap();
        for (index, step) in scenario["steps"].as_array().unwrap().iter().enumerate() {
            if let Some(action) = step.get("action") {
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
                        piece["distance_to_player_meters"].as_f64().unwrap() < 1.0,
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
                        "step {index}: {path}"
                    );
                } else if let Some(expected) = check.get("lt") {
                    assert!(
                        value.as_f64().unwrap() < expected.as_f64().unwrap(),
                        "step {index}: {path}"
                    );
                } else {
                    panic!("unhandled check at step {index}");
                }
            }
        }
    }

    #[test]
    fn protocol_rejects_malformed_and_unavailable_commands() {
        let mut normal = ClientApplication::default();
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
