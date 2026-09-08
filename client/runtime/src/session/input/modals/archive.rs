use super::*;

pub(super) fn handle(session: &mut GameplaySession, key: Key, pressed: bool) -> bool {
    if session.archive_reader.open {
        if pressed {
            use crate::session::archive::ReaderAction as A;
            let action = match key {
                Key::Interact => Some(A::Close),
                Key::Recovery => Some(A::ToggleRecovery),
                Key::Up | Key::Previous => Some(A::Previous),
                Key::Down | Key::Next | Key::Tab => Some(A::Next),
                _ => None,
            };
            if let Some(action) = action {
                session.archive_action(action);
            }
        }
        return true;
    }
    false
}
