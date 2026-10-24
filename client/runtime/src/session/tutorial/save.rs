//! Domain-owned checkpoint mapping and historical schema admission.
use salimon_persistence::{SCHEMA_VERSION, SaveError, invalid};

pub(in crate::session) fn restore(
    source: Option<crate::session::tutorial::Tutorial>,
    version: u32,
) -> Result<crate::session::tutorial::Tutorial, SaveError> {
    let tutorial = match (version, source) {
        (4..=SCHEMA_VERSION, Some(tutorial)) => tutorial,
        (1..=3, _) => crate::session::tutorial::Tutorial::dismissed(),
        _ => return Err(invalid("tutorial source missing for save schema")),
    };
    Ok(tutorial)
}
