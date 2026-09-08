//! Domain-owned checkpoint mapping and historical schema admission.
use salimon_persistence::{SaveError, invalid};

pub(in crate::session) fn validate_schema(
    source: &salimon_ship::finishes::Finishes,
    version: u32,
) -> Result<(), SaveError> {
    if version < 13 && *source != Default::default() {
        return Err(invalid(
            "custom finishes incompatible with historical schema",
        ));
    }
    Ok(())
}
