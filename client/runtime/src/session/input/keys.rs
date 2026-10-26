use super::*;

pub(crate) fn toolbar_slot(key: Key) -> Option<crate::equipment::ToolbarSlot> {
    use crate::equipment::ToolbarSlot;
    match key {
        Key::Slot1 => Some(ToolbarSlot::One),
        Key::Slot2 => Some(ToolbarSlot::Two),
        Key::Slot3 => Some(ToolbarSlot::Three),
        Key::Slot4 => Some(ToolbarSlot::Four),
        Key::Slot5 => Some(ToolbarSlot::Five),
        _ => None,
    }
}

pub(crate) fn throttle_direction(key: Key) -> Option<ThrottleDirection> {
    match key {
        Key::Up => Some(ThrottleDirection::Up),
        Key::Down => Some(ThrottleDirection::Down),
        _ => None,
    }
}

pub(crate) fn camera_command(key: Key, precision_tour: bool) -> Option<CameraCommand> {
    if !precision_tour {
        return None;
    }

    match key {
        Key::TourPause => Some(CameraCommand::TogglePause),
        Key::Recovery => Some(CameraCommand::Restart),
        Key::CoreRecover if precision_tour => Some(CameraCommand::JumpToNear),
        Key::Slot1 => Some(CameraCommand::InspectBody(CelestialBodyId::Sun)),
        Key::Slot2 => Some(CameraCommand::InspectBody(CelestialBodyId::Mercury)),
        Key::Slot3 => Some(CameraCommand::InspectBody(CelestialBodyId::Venus)),
        Key::Slot4 => Some(CameraCommand::InspectBody(CelestialBodyId::Earth)),
        Key::Slot5 => Some(CameraCommand::InspectBody(CelestialBodyId::Moon)),
        Key::Slot6 => Some(CameraCommand::InspectBody(CelestialBodyId::Mars)),
        _ => None,
    }
}

pub(crate) fn control_key(key: Key) -> Option<crate::session::ControlKey> {
    use crate::session::ControlKey::*;
    Some(match key {
        Key::Forward => Forward,
        Key::Backward => Backward,
        Key::Left => Left,
        Key::Right => Right,
        Key::Jump => Jump,
        Key::Descend => Descend,
        Key::Previous => RollLeft,
        Key::Next => RollRight,
        _ => return None,
    })
}
