use super::*;

pub(super) fn handle(
    app: &mut GameplaySession,
    request: &CommandRequest<'_>,
) -> Option<CommandResult> {
    let command = request.fields;
    let result = match Some(request.operation) {
        Some("consume") => {
            let slot = command
                .get("slot")
                .and_then(Value::as_u64)
                .filter(|n| (1..=5).contains(n))
                .map(|n| salimon_fabrication::ConsumableSlot::ALL[n as usize - 1]);
            let job = command
                .get("job")
                .and_then(Value::as_u64)
                .filter(|n| *n > 0);
            let unit = command
                .get("unit")
                .and_then(Value::as_u64)
                .and_then(|n| u32::try_from(n).ok())
                .filter(|n| *n > 0);
            match (slot, job, unit) {
                (Some(slot), Some(job), Some(unit)) => {
                    let result = app.consume_selection(crate::session::consumables::Selection {
                        slot,
                        job: salimon_fabrication::JobId(job),
                        unit,
                    });
                    match result {
                        Ok(item) => Ok(json!({"applied":true,"job":item.job.0,"unit":item.unit})),
                        Err(reason) => Ok(json!({"applied":false,"reason":format!("{reason:?}")})),
                    }
                }
                _ => Err((
                    "invalid_argument",
                    "slot 1..5, positive job and u32 unit required",
                )),
            }
        }
        Some("wheel_hover") => {
            let slot = command.get("slot");
            let selection = slot
                .and_then(Value::as_u64)
                .and_then(|i| usize::try_from(i).ok())
                .and_then(|i| salimon_fabrication::ConsumableSlot::ALL.get(i).copied());
            if selection.is_some() || slot.is_some_and(Value::is_null) {
                app.command(crate::session::Command::Wheel(
                    crate::session::consumable_wheel::WheelAction::Hover(selection),
                ));
                Ok(json!({"applied":app.wheel.open}))
            } else {
                Err(("invalid_argument", "slot must be null or integer 0..4"))
            }
        }
        _ => return None,
    };
    Some(result)
}
