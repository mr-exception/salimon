use super::*;
mod archive;
mod crafting;
mod wheel;

pub(super) fn handle(session: &mut GameplaySession, key: Key, pressed: bool) -> bool {
    if pressed && key == Key::Cancel && session.view_mode == crate::session::ViewMode::Gameplay {
        session.command(Command::TogglePause);
        return true;
    }
    if session.is_paused() {
        if pressed && matches!(key, Key::Confirm | Key::ConfirmAlternate) {
            session.command(Command::TogglePause);
        }
        return true;
    }
    if archive::handle(session, key, pressed) {
        return true;
    }
    if key == Key::Wheel {
        session.command(Command::Wheel(
            crate::session::consumable_wheel::WheelAction::Hold(pressed),
        ));
        return true;
    }
    if pressed && key == Key::Tutorial {
        session.command(Command::SkipTutorial);
        return true;
    }
    if wheel::handle(session, key, pressed) {
        return true;
    }
    if crafting::handle(session, key, pressed) {
        return true;
    }
    false
}
