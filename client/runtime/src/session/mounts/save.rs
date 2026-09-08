//! Domain-owned checkpoint mapping and historical schema admission.
use crate::session::GameplaySession;
use salimon_persistence::{SCHEMA_VERSION, SaveError, invalid};

pub(in crate::session) fn restore(
    source: Option<salimon_ship::attachments::Attachments>,
    version: u32,
) -> Result<salimon_ship::attachments::Attachments, SaveError> {
    let mounts = match (version, source) {
        (11..=SCHEMA_VERSION, Some(m)) => m,
        (1..=10, None) => Default::default(),
        (1..=10, Some(m)) if m == Default::default() => Default::default(),
        _ => return Err(invalid("mount source missing/incompatible with schema")),
    };
    Ok(mounts)
}

pub(in crate::session) fn reconcile_carried_mass(
    restored: &mut GameplaySession,
) -> Result<(), SaveError> {
    if let Some(c) = &restored.mounts.carried {
        let frame = crate::session::frames::character_ship_frame(restored.ship.snapshot().pose);
        let absent =
            !(salimon_character::ship_contains_object(frame.world_to_local(c.world_position))
                || restored.structural_volumes().iter().any(|b| {
                    let p = frame.world_to_local(c.world_position);
                    (0..3).all(|i| p[i] >= b[2 * i] && p[i] <= b[2 * i + 1])
                }));
        restored
            .mining
            .ship_mass
            .set_dry_equipment_absent(
                c.id,
                absent.then_some(salimon_ship::attachments::PORTABLE_SHELF_MASS_KG),
            )
            .map_err(|_| invalid("accessory mass"))?;
    }
    Ok(())
}
