//! Domain-owned checkpoint mapping and historical schema admission.
use salimon_persistence::{SCHEMA_VERSION, SaveError, invalid};

pub(in crate::session) fn restore(
    source: Option<crate::session::construction::ConstructionSave>,
    version: u32,
) -> Result<crate::session::construction::ConstructionSave, SaveError> {
    let construction = match (version, source) {
        (8..=SCHEMA_VERSION, Some(source)) => source,
        (1..=7, None) => Default::default(),
        (1..=7, Some(source)) if source.projects.jobs().next().is_none() => Default::default(),
        _ => {
            return Err(invalid(
                "construction source missing or incompatible with save schema",
            ));
        }
    };
    if version < 21
        && (construction.damage != Default::default()
            || construction.projects.jobs().any(|p| p.work.destroyed))
    {
        return Err(invalid("construction destruction requires schema 21"));
    }
    if version < 20 && !construction.modules.is_empty() {
        return Err(invalid("module commissioning requires schema 20"));
    }
    if version < 19 && construction.projects.jobs().any(|p| p.work.stopped) {
        return Err(invalid("construction stop latch requires schema 19"));
    }
    if version < 18
        && (!construction.removed_partitions.is_empty()
            || !construction.open_doors.is_empty()
            || construction.projects.jobs().any(|p| {
                p.work.completed
                    && (crate::session::construction::is_structural(p)
                        || p.blueprint.id == "internal-door")
            }))
    {
        return Err(invalid("installed structure requires schema 18"));
    }
    if version < 15
        && construction
            .projects
            .jobs()
            .any(|p| !p.funding.receipts.is_empty() || p.funding.progress_ppm != 0)
    {
        return Err(invalid("funded construction requires schema 15"));
    }
    if version < 17
        && construction
            .projects
            .jobs()
            .any(|p| p.work != Default::default())
    {
        return Err(invalid("construction work requires schema 17"));
    }
    Ok(construction)
}
