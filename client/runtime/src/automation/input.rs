use super::*;

pub(super) fn handle(
    app: &mut GameplaySession,
    request: &CommandRequest<'_>,
) -> Option<CommandResult> {
    let command = request.fields;
    let result = match Some(request.operation) {
        Some("chart_text") => match command.get("text").and_then(Value::as_str) {
            Some(text) => app
                .surface_text(text)
                .map(|()| json!({"accepted":true}))
                .map_err(|_| {
                    (
                        "invalid_argument",
                        "note editor unavailable or invalid text",
                    )
                }),
            None => Err(("invalid_argument", "note text required")),
        },
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
        _ => return None,
    };
    Some(result)
}
