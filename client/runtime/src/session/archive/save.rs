//! Domain-owned checkpoint mapping and historical schema admission.
use salimon_persistence::{SCHEMA_VERSION, SaveError, invalid};

pub(in crate::session) fn restore(
    source: Option<crate::session::archive::Archive>,
    version: u32,
) -> Result<crate::session::archive::Archive, SaveError> {
    let archive = match (version, source) {
        (6..=SCHEMA_VERSION, Some(archive)) => archive,
        (1..=5, _) => Default::default(),
        _ => return Err(invalid("archive source missing for save schema")),
    };
    Ok(archive)
}
