use super::*;

pub(super) fn command(key: Key) -> Option<Command> {
    if key == Key::CoreInput {
        Some(Command::CoreUpgrade(
            crate::session::core_upgrades::CoreUpgradeAction::ToggleInput,
        ))
    } else if key == Key::CoreApply {
        Some(Command::CoreUpgrade(
            crate::session::core_upgrades::CoreUpgradeAction::Apply,
        ))
    } else if key == Key::CoreRecover {
        Some(Command::CoreUpgrade(
            crate::session::core_upgrades::CoreUpgradeAction::Recover,
        ))
    } else if key == Key::FinishGroup {
        Some(Command::Finish(
            crate::session::finishes::FinishAction::SelectGroup,
        ))
    } else if key == Key::FinishNext {
        Some(Command::Finish(
            crate::session::finishes::FinishAction::Next,
        ))
    } else if key == Key::FinishReset {
        Some(Command::Finish(
            crate::session::finishes::FinishAction::Reset,
        ))
    } else if key == Key::Place {
        Some(Command::Mount(crate::session::mounts::MountAction::Place))
    } else if key == Key::Rest {
        Some(Command::Cabin(crate::session::cabin::CabinAction::Rest))
    } else if key == Key::Restraint {
        Some(Command::Cabin(
            crate::session::cabin::CabinAction::Restraint,
        ))
    } else if key == Key::MoveAccessory {
        Some(Command::Mount(
            crate::session::mounts::MountAction::MoveAccessory,
        ))
    } else if key == Key::Secure {
        Some(Command::Mount(
            crate::session::mounts::MountAction::ToggleLock,
        ))
    } else {
        None
    }
}
