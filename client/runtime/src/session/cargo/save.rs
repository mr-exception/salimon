//! Domain-owned checkpoint mapping and historical schema admission.
use salimon_persistence::{SCHEMA_VERSION, SaveError, invalid};

pub(in crate::session) fn restore(
    source: Option<crate::session::cargo::CargoDoor>,
    version: u32,
) -> Result<crate::session::cargo::CargoDoor, SaveError> {
    let cargo_door = match (version, source) {
        (9..=SCHEMA_VERSION, Some(door)) => door,
        (1..=8, None) => Default::default(),
        (1..=8, Some(door)) if door.open_fraction == 1.0 && door.opening => Default::default(),
        _ => {
            return Err(invalid(
                "cargo door source missing or incompatible with schema",
            ));
        }
    };
    Ok(cargo_door)
}
