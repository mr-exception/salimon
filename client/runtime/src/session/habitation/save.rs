//! Domain-owned checkpoint mapping and historical schema admission.
use salimon_persistence::{SCHEMA_VERSION, SaveError, invalid};

pub(in crate::session) fn restore(
    source: Option<crate::session::habitation::Habitation>,
    version: u32,
) -> Result<crate::session::habitation::Habitation, SaveError> {
    let habitation = match (version, source) {
        (10..=SCHEMA_VERSION, Some(h)) => h,
        (1..=9, None) => Default::default(),
        (1..=9, Some(h)) if h.bunk_open_fraction == 0.0 && !h.bunk_deploying => Default::default(),
        _ => {
            return Err(invalid(
                "habitation source missing or incompatible with schema",
            ));
        }
    };
    if version < 12 && habitation.cabin.is_some() {
        return Err(invalid(
            "expanded cabin source incompatible with historical schema",
        ));
    }
    Ok(habitation)
}
