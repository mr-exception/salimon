use super::*;

pub(super) fn handle(session: &mut GameplaySession, key: Key, pressed: bool) -> bool {
    if session.wheel.open {
        use crate::session::consumable_wheel::WheelAction as W;
        let action = match (key, pressed) {
            (Key::Tab | Key::Next | Key::Down, true) => Some(W::Next),
            (Key::Previous | Key::Up, true) => Some(W::Previous),
            (Key::Confirm | Key::ConfirmAlternate, true) => Some(W::Confirm),
            (Key::Cancel, true) => Some(W::Cancel),
            (Key::View, true) => {
                session.command(Command::ToggleView);
                return true;
            }
            _ => None,
        };
        if let Some(action) = action {
            session.command(Command::Wheel(action));
        }
        return true;
    }
    false
}
