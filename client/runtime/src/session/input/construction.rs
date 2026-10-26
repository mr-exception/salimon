use super::*;

pub(super) fn handle(session: &mut GameplaySession, key: Key, pressed: bool) -> bool {
    if pressed
        && session.view_mode == crate::session::ViewMode::Gameplay
        && !session.is_paused()
        && !session.archive_reader.open
        && !session.crafting_ui.open
        && !session.wheel.open
    {
        use crate::session::construction::BuildAction as B;
        let action = match key {
            Key::Build => Some(B::Toggle),
            Key::Cancel if session.building.active => Some(B::Toggle),
            Key::Recovery if session.building.active => Some(B::Rotate),
            Key::Status if session.building.active => Some(B::Repair),
            Key::Rest if session.building.active => Some(B::Dismantle),
            Key::Place if session.building.active => Some(B::Module),
            Key::Tutorial if session.building.active => Some(B::StopResume),
            Key::Delete if session.building.active => Some(B::RemovePartition),
            code if session.building.active => match code {
                Key::Slot1 => Some(B::Select(0)),
                Key::Slot2 => Some(B::Select(1)),
                Key::Slot3 => Some(B::Select(2)),
                Key::Slot4 => Some(B::Select(3)),
                Key::Slot5 => Some(B::Select(4)),
                Key::Slot6 => Some(B::Select(5)),
                Key::Slot7 => Some(B::Select(6)),
                _ => None,
            },
            _ => None,
        };
        if let Some(action) = action {
            session.command(Command::Build(action));
            return true;
        }
    }
    false
}
