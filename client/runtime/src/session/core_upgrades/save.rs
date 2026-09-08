//! Domain-owned checkpoint mapping and historical schema admission.
use salimon_persistence::{SaveError, invalid};

pub(in crate::session) fn validate_schema(
    source: &salimon_ship::core_upgrades::CoreUpgrades,
    version: u32,
) -> Result<(), SaveError> {
    if version < 14
        && (source.mode != salimon_ship::core_upgrades::InputMode::Fuel
            || !source.applied.is_empty()
            || !source.pending.is_empty())
    {
        return Err(invalid(
            "Core upgrade source incompatible with historical schema",
        ));
    }
    Ok(())
}
