use super::*;

pub(super) fn handle(
    app: &mut GameplaySession,
    request: &CommandRequest<'_>,
) -> Option<CommandResult> {
    let command = request.fields;
    let result = match Some(request.operation) {
        Some("crafting") => {
            use crate::session::crafting_ui::CraftingAction as A;
            let action = match command.get("action").and_then(Value::as_str) {
                Some("close") => Some(A::Close),
                Some("previous_recipe") => Some(A::PreviousRecipe),
                Some("next_recipe") => Some(A::NextRecipe),
                Some("less") => Some(A::Less),
                Some("more") => Some(A::More),
                Some("next_job") => Some(A::NextJob),
                Some("cancel") => Some(A::Cancel),
                Some("add") => Some(A::Add),
                _ => None,
            };
            match action {
                Some(action) => {
                    let applied = app.crafting_ui.open;
                    app.crafting_action(action);
                    Ok(json!({"applied":applied}))
                }
                None => Err(("invalid_argument", "known crafting action required")),
            }
        }
        _ => return None,
    };
    Some(result)
}
