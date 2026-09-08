use super::*;

pub(super) fn handle(
    app: &mut GameplaySession,
    request: &CommandRequest<'_>,
) -> Option<CommandResult> {
    let command = request.fields;
    let result = match Some(request.operation) {
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
        _ => return None,
    };
    Some(result)
}
