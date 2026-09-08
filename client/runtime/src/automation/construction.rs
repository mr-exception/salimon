use super::*;

pub(super) fn handle(
    app: &mut GameplaySession,
    request: &CommandRequest<'_>,
) -> Option<CommandResult> {
    let command = request.fields;
    let result = match Some(request.operation) {
        Some("construction_impact") => {
            match command
                .get("impact")
                .cloned()
                .and_then(|v| serde_json::from_value(v).ok())
            {
                Some(impact) => app
                    .construction_impact(impact)
                    .map(|()| json!({"applied":true}))
                    .map_err(|e| ("invalid_argument", e)),
                None => Err(("invalid_argument", "typed impact required")),
            }
        }
        _ => return None,
    };
    Some(result)
}
