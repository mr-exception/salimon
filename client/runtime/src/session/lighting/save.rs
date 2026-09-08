//! Domain-owned checkpoint mapping and historical schema admission.
use salimon_persistence::{SaveError, Validate, invalid};

pub(in crate::session) fn validate_schema(
    source: &salimon_ship::lighting::Lighting,
    version: u32,
) -> Result<(), SaveError> {
    if version < 16 && *source != Default::default() {
        return Err(invalid("lighting requires schema 16"));
    }
    Ok(())
}

pub(in crate::session) fn validate_context(
    source: &salimon_ship::lighting::Lighting,
) -> Result<(), SaveError> {
    source.validate()?;
    if !source.rooms.contains_key(&1) || !source.rooms.contains_key(&2) {
        return Err(invalid("installed light room missing"));
    }
    Ok(())
}
