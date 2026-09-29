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
                write_response(&error(Value::Null, "line_too_long", "maximum line size is 16 KiB"));
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
    write_response(&json!({"protocol": PROTOCOL, "event": "ready", "scenario": scenario,
        "seed": seed, "step_ms": step_ms, "timeout_ms": TIMEOUT.as_millis()}));
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
        Err(_) => return error(Value::Null, "invalid_json", "expected one JSON object per line"),
    };
    let id = command.get("id").cloned().unwrap_or(Value::Null);
    if !command.is_object() || !(id.is_string() || id.is_number()) {
        return error(id, "invalid_request", "object with string or numeric id required");
    }
    if command.get("protocol").and_then(Value::as_u64) != Some(u64::from(PROTOCOL)) {
        return error(id, "protocol_mismatch", "expected protocol 1");
    }
    let result = match command.get("op").and_then(Value::as_str) {
        Some("inspect") => Ok(inspect(app)),
        Some("step") => {
            match command.get("frames").and_then(Value::as_u64) {
                Some(frames @ 1..=600) => {
                    let step = app.e2e_step().expect("automation only runs in E2E mode");
                    for _ in 0..frames {
                        app.advance_game(step);
                    }
                    Ok(json!({"frames": frames}))
                }
                _ => Err(("invalid_argument", "frames must be an integer from 1 to 600")),
            }
        }
        Some("look") => {
            let dx = command.get("dx").and_then(Value::as_f64);
            let dy = command.get("dy").and_then(Value::as_f64);
            match (dx, dy) {
                (Some(dx), Some(dy)) if dx.is_finite() && dy.is_finite() => {
                    app.character.apply_mouse_delta(dx, dy);
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
        Some("thruster") => {
            match command.get("direction").and_then(Value::as_i64) {
                Some(direction @ -1..=1) if direction != 0 => {
                    if app.character.location() == CharacterLocation::Cockpit {
                        app.ship.adjust_thruster(direction as i8);
                    }
                    Ok(json!({"applied": app.character.location() == CharacterLocation::Cockpit}))
                }
                _ => Err(("invalid_argument", "direction must be -1 or 1")),
            }
        }
        _ => Err(("unknown_op", "expected inspect, step, look, key, interact, landing, or thruster")),
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
        "jump" => Some(KeyCode::Space),
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
    json!({
        "player": {"location": format!("{:?}", character.location),
            "eye_position_meters": character.eye_position_meters,
            "look_target_meters": character.look_target_meters,
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
        "interaction": interaction,
        "world": {"bodies": bodies, "camera_phase": format!("{:?}", app.camera_phase())}
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::e2e::{self, Config, Scenario};

    fn app(scenario: Scenario) -> ClientApplication {
        let mut app = ClientApplication::default();
        e2e::initialize(&mut app, Config {
            scenario,
            seed: 7,
            step: Duration::from_millis(20),
        }).unwrap();
        app
    }

    #[test]
    fn protocol_rejects_malformed_and_unavailable_commands() {
        let mut normal = ClientApplication::default();
        assert_eq!(execute(&mut normal, r#"{"protocol":1,"id":1,"op":"inspect"}"#)["error"]["code"], "disabled");
        let mut test = app(Scenario::LandedEarth);
        assert_eq!(execute(&mut test, "{")["error"]["code"], "invalid_json");
        assert_eq!(execute(&mut test, r#"{"protocol":2,"id":"a","op":"inspect"}"#)["error"]["code"], "protocol_mismatch");
        assert_eq!(execute(&mut test, r#"{"protocol":1,"id":2,"op":"step","frames":601}"#)["error"]["code"], "invalid_argument");
        assert_eq!(execute(&mut test, r#"{"protocol":1,"id":3,"op":"warp"}"#)["error"]["code"], "unknown_op");
    }

    #[test]
    fn commands_use_live_controller_paths_and_inspection() {
        let mut test = app(Scenario::LandedEarth);
        let before = execute(&mut test, r#"{"protocol":1,"id":1,"op":"inspect"}"#);
        assert_eq!(before["result"]["player"]["location"], "InsideShip");
        assert_eq!(before["result"]["world"]["bodies"].as_array().unwrap().len(), 6);
        assert_eq!(execute(&mut test, r#"{"protocol":1,"id":2,"op":"key","key":"forward","pressed":true}"#)["ok"], true);
        assert_eq!(execute(&mut test, r#"{"protocol":1,"id":3,"op":"step","frames":5}"#)["ok"], true);
        let after = execute(&mut test, r#"{"protocol":1,"id":4,"op":"inspect"}"#);
        assert_ne!(before["result"]["player"]["eye_position_meters"], after["result"]["player"]["eye_position_meters"]);
        execute(&mut test, r#"{"protocol":1,"id":5,"op":"key","key":"forward","pressed":false}"#);
        let stable = execute(&mut test, r#"{"protocol":1,"id":6,"op":"inspect"}"#);
        execute(&mut test, r#"{"protocol":1,"id":7,"op":"step","frames":1}"#);
        assert_eq!(stable["result"]["player"]["eye_position_meters"], execute(&mut test, r#"{"protocol":1,"id":8,"op":"inspect"}"#)["result"]["player"]["eye_position_meters"]);
    }

    #[test]
    fn cockpit_commands_respect_authority() {
        let mut test = app(Scenario::CockpitEarth);
        let before = test.ship.snapshot().thruster_percentage;
        execute(&mut test, r#"{"protocol":1,"id":1,"op":"thruster","direction":1}"#);
        assert_eq!(test.ship.snapshot().thruster_percentage, before + 1);
        execute(&mut test, r#"{"protocol":1,"id":2,"op":"interact"}"#);
        assert_eq!(test.character.location(), CharacterLocation::InsideShip);
        assert_eq!(execute(&mut test, r#"{"protocol":1,"id":3,"op":"thruster","direction":1}"#)["result"]["applied"], false);
        assert_eq!(test.ship.snapshot().thruster_percentage, before + 1);
    }
}
