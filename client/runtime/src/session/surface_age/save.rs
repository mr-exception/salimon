//! Domain-owned checkpoint mapping and historical schema admission.
use salimon_persistence::{SCHEMA_VERSION, SaveError, invalid};

pub(in crate::session) fn restore(
    source: Option<salimon_ship::surface_age::SurfaceAge>,
    version: u32,
) -> Result<salimon_ship::surface_age::SurfaceAge, SaveError> {
    let surface_age = match (version, source) {
        (23..=SCHEMA_VERSION, Some(age)) => age,
        (1..=22, None) => Default::default(),
        (1..=22, Some(age)) if age == Default::default() => age,
        _ => return Err(invalid("surface age missing or incompatible with schema")),
    };
    Ok(surface_age)
}
