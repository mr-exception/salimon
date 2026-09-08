//! Domain-owned checkpoint mapping and historical schema admission.
use salimon_persistence::{SaveError, invalid};

pub(in crate::session) fn restore(
    source: salimon_fabrication::FabricationSaveState,
    version: u32,
) -> Result<
    (
        salimon_fabrication::Fabricator,
        salimon_fabrication::ConsumableInventory,
    ),
    SaveError,
> {
    let (mut fabricator, consumables) = source.restore()?;
    if version <= 6 {
        fabricator
            .migrate_recovery_recipe()
            .map_err(|e| invalid(e.to_string()))?;
    }
    Ok((fabricator, consumables))
}
