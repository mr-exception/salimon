use super::*;

pub(super) fn handle(session: &mut GameplaySession, key: Key, pressed: bool) -> bool {
    if session.crafting_ui.open {
        if pressed {
            use crate::session::crafting_ui::CraftingAction as A;
            let action = match key {
                Key::Cancel | Key::Interact => Some(A::Close),
                Key::Up => Some(A::PreviousRecipe),
                Key::Down => Some(A::NextRecipe),
                Key::Previous => Some(A::Less),
                Key::Next => Some(A::More),
                Key::Tab => Some(A::NextJob),
                Key::Delete => Some(A::Cancel),
                Key::Confirm => Some(A::Add),
                _ => None,
            };
            if let Some(action) = action {
                session.crafting_action(action);
            }
        }
        return true;
    }
    false
}
