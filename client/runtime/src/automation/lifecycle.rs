use super::*;

pub(super) fn handle(
    app: &mut GameplaySession,
    request: &CommandRequest<'_>,
) -> Option<CommandResult> {
    let command = request.fields;
    let result = match Some(request.operation) {
        Some("offline_resume") => {
            match command
                .get("seconds")
                .and_then(Value::as_u64)
                .filter(|s| *s <= 86400)
            {
                Some(seconds) => {
                    let saved = app.offline.watermark.max(100);
                    match saved
                        .checked_add(seconds)
                        .and_then(|launch| app.save_bytes(saved).ok().map(|bytes| (launch, bytes)))
                    {
                        Some((launch, bytes)) => app
                            .restore_for_launch(&bytes, launch)
                            .map(|()| json!({"applied":true,"closed_seconds":seconds}))
                            .map_err(|_| ("invalid_argument", "checkpoint reconciliation failed")),
                        None => Err(("invalid_argument", "checkpoint time or encoding failed")),
                    }
                }
                None => Err((
                    "invalid_argument",
                    "fixture seconds must be an integer from 0 to 86400",
                )),
            }
        }
        _ => return None,
    };
    Some(result)
}
