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

mod clock;
mod construction;
mod crafting;
mod input;
mod inspection;
mod keys;
mod lifecycle;
mod movement;
mod native;
mod survival;
#[cfg(test)]
mod tests;
pub(crate) use inspection::inspect_session;
use keys::key_code;
use native::execute_native;

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
        && ((command.get("op").and_then(Value::as_str) == Some("step")
            && matches!(command.get("frames").and_then(Value::as_u64), Some(1..=600)))
            || (command.get("op").and_then(Value::as_str) == Some("offline_resume")
                && matches!(
                    command.get("seconds").and_then(Value::as_u64),
                    Some(0..=86400)
                )))
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
pub(crate) fn execute_session(app: &mut GameplaySession, line: &str) -> Value {
    if app.e2e_step().is_none() {
        return error(Value::Null, "disabled", "automation requires --e2e");
    }
    let command = match parse_envelope(line) {
        Ok(command) => command,
        Err(response) => return response,
    };
    let id = command["id"].clone();
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
    let request = CommandRequest {
        operation: command.get("op").and_then(Value::as_str).unwrap_or(""),
        fields: &command,
    };
    let result = dispatch_session(app, &request);
    match result {
        Ok(result) => json!({"protocol": PROTOCOL, "id": id, "ok": true, "result": result}),
        Err((code, message)) => error(id, code, message),
    }
}

fn parse_envelope(line: &str) -> Result<Value, Value> {
    let command: Value = serde_json::from_str(line).map_err(|_| {
        error(
            Value::Null,
            "invalid_json",
            "expected one JSON object per line",
        )
    })?;
    let id = command.get("id").cloned().unwrap_or(Value::Null);
    if !command.is_object() || !(id.is_string() || id.is_number()) {
        return Err(error(
            id,
            "invalid_request",
            "object with string or numeric id required",
        ));
    }
    if command.get("protocol").and_then(Value::as_u64) != Some(u64::from(PROTOCOL)) {
        return Err(error(id, "protocol_mismatch", "expected protocol 1"));
    }
    Ok(command)
}

/// Validated envelope fields stay borrowed on the native execution thread.
struct CommandRequest<'a> {
    operation: &'a str,
    fields: &'a Value,
}
type CommandResult = Result<Value, (&'static str, &'static str)>;
type Handler = fn(&mut GameplaySession, &CommandRequest<'_>) -> Option<CommandResult>;

fn dispatch_session(app: &mut GameplaySession, request: &CommandRequest<'_>) -> CommandResult {
    if request.operation == "inspect" {
        return Ok(inspect_session(app));
    }
    // Registration changes only for a new owner, not for new operations.
    const HANDLERS: &[Handler] = &[
        survival::handle,
        crafting::handle,
        construction::handle,
        lifecycle::handle,
        clock::handle,
        movement::handle,
        input::handle,
    ];
    for handle in HANDLERS {
        if let Some(result) = handle(app, request) {
            return result;
        }
    }
    Err((
        "unknown_op",
        "expected inspect, step, look, aim_fragment, restore_look, key, mouse, interact, landing, thruster, crafting, or consume",
    ))
}
