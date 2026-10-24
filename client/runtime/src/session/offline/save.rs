//! Domain-owned checkpoint mapping and historical schema admission.
use salimon_persistence::{SCHEMA_VERSION, SaveError, invalid};

pub(in crate::session) fn restore(
    source: Option<crate::session::offline::OfflineClock>,
    version: u32,
    saved_at: u64,
) -> Result<crate::session::offline::OfflineClock, SaveError> {
    let offline = match (version, source) {
        (22..=SCHEMA_VERSION, Some(clock)) => clock,
        (1..=21, None) => crate::session::offline::OfflineClock::at(saved_at),
        (1..=21, Some(clock)) if clock.pending_nanos == 0 => {
            crate::session::offline::OfflineClock::at(saved_at)
        }
        _ => {
            return Err(invalid(
                "offline construction clock missing or incompatible with schema",
            ));
        }
    };
    if offline.watermark < saved_at {
        return Err(invalid("offline watermark precedes checkpoint"));
    }
    Ok(offline)
}
