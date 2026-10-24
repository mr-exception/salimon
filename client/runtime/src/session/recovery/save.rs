//! Domain-owned checkpoint mapping and historical schema admission.
use salimon_persistence::{SCHEMA_VERSION, SaveError, Validate, invalid};

pub(in crate::session) fn restore(
    source: Option<crate::session::recovery::Recovery>,
    version: u32,
) -> Result<crate::session::recovery::Recovery, SaveError> {
    let recovery = match (version, source) {
        (7..=SCHEMA_VERSION, Some(recovery)) => recovery,
        (1..=6, _) => Default::default(),
        _ => return Err(invalid("recovery source missing for schema 7")),
    };
    Ok(recovery)
}

pub(in crate::session) fn validate_with_fabrication(
    source: &Option<crate::session::recovery::Recovery>,
    fabrication: &salimon_fabrication::FabricationSaveState,
) -> Result<(), SaveError> {
    if let Some(recovery) = source {
        recovery.validate()?;
        let machine = fabrication.machine.snapshot(true);
        let committed = machine
            .jobs
            .iter()
            .filter(|j| j.recipe.key() == "data-recovery-module")
            .map(|j| u64::from(j.remaining_quantity))
            .sum::<u64>()
            + u64::from(
                machine
                    .tray
                    .as_ref()
                    .is_some_and(|i| i.recipe.key() == "data-recovery-module"),
            )
            + u64::from(recovery.module.is_some());
        if committed > 1 {
            return Err(invalid("multiple committed Recovery Modules"));
        }
        if let Some(module) = &recovery.module {
            fabrication.validate_installed_component(module)?;
        }
    }
    Ok(())
}
