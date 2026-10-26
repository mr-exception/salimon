use super::*;

pub(super) fn handle(
    app: &mut GameplaySession,
    request: &CommandRequest<'_>,
) -> Option<CommandResult> {
    let command = request.fields;
    let result = match Some(request.operation) {
        Some("step") => match command.get("frames").and_then(Value::as_u64) {
            Some(frames @ 1..=600) => {
                let step = app.e2e_step().expect("automation only runs in E2E mode");
                let started = Instant::now();
                for _ in 0..frames {
                    app.advance_automation(step);
                }
                Ok(json!({"frames": frames,"elapsed_ms":started.elapsed().as_secs_f64()*1000.0}))
            }
            _ => Err((
                "invalid_argument",
                "frames must be an integer from 1 to 600",
            )),
        },
        _ => return None,
    };
    Some(result)
}
