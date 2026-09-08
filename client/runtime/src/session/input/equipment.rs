use super::*;

pub(super) fn command(key: Key) -> Option<Command> {
    toolbar_slot(key).map(Command::Select)
}
