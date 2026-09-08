use super::*;

pub(super) fn command(key: Key) -> Option<Command> {
    if key == Key::Engines {
        Some(Command::ToggleEngines)
    } else if key == Key::Landing {
        Some(Command::Landing)
    } else if let Some(camera) = camera_command(key, false) {
        Some(Command::Camera(camera))
    } else {
        throttle_direction(key).map(|direction| Command::Throttle(direction, true))
    }
}
