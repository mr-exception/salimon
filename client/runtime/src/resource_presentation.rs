//! Composition-only mapping: world owns generated deposits; GPU receives generic cuboids.
use salimon_renderer::SceneInstance;
use salimon_world::resource_distribution::default_resource_distribution;
use salimon_world::resource_generation::{
    GenerationError, SurfaceDeposit, materialize_nearby_deposits,
};
use salimon_world::resources::{DepositState, ResourceId};
use salimon_world::{BodyRole, CELESTIAL_BODIES, WorldPosition};

pub(crate) const ACTIVE_RADIUS_METERS: f64 = 120.0;

pub(crate) fn nearby_deposits(
    position: [f64; 3],
    seed: u64,
) -> Result<Vec<SurfaceDeposit>, GenerationError> {
    let player = WorldPosition::new(position[0], position[1], position[2]);
    let mut deposits = Vec::new();
    for body in CELESTIAL_BODIES
        .iter()
        .filter(|body| body.role == BodyRole::Solid)
    {
        let local = player.offset_from(body.center);
        let altitude = local[0].hypot(local[1]).hypot(local[2]) - body.radius_meters;
        if altitude.abs() > ACTIVE_RADIUS_METERS {
            continue;
        }
        deposits.extend(materialize_nearby_deposits(
            *body,
            &default_resource_distribution(body.id),
            seed,
            None,
            player,
            ACTIVE_RADIUS_METERS,
        )?);
    }
    // Stable inspection order, independent of camera movement/query enumeration.
    deposits.sort_by_key(|entry| {
        (
            entry.deposit.material().resource().key(),
            entry.deposit.id().local,
        )
    });
    Ok(deposits)
}

pub(crate) fn visual(entry: SurfaceDeposit) -> Option<SceneInstance> {
    if entry.deposit.state() == DepositState::Depleted {
        return None;
    }
    let (color, proportions): ([f32; 4], [f64; 3]) = match entry.deposit.material().resource() {
        ResourceId::IronOre => ([0.75, 0.22, 0.06, 1.0], [1.0, 0.8, 0.65]),
        ResourceId::SilicateRock => ([0.45, 0.50, 0.36, 1.0], [0.8, 0.6, 1.0]),
        ResourceId::WaterIce => ([0.25, 0.85, 1.0, 1.0], [0.55, 1.0, 0.55]),
    };
    // Inscribe the greybox in the authoritative spherical bound. Its center is
    // the surface anchor, so only the outward half protrudes on every body/latitude.
    let norm = proportions[0].hypot(proportions[1]).hypot(proportions[2]);
    Some(SceneInstance {
        center_meters: entry.deposit.position().meters(),
        half_extents_meters: proportions.map(|p| (p * entry.bounds_radius_meters / norm) as f32),
        color,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_world::CelestialBodyId;
    use salimon_world::resources::{DepositId, RESOURCE_CATALOG, RawMaterial, ResourceDeposit};

    #[test]
    fn every_material_has_distinct_bounded_geometry_at_its_exact_anchor() {
        let mut appearances = Vec::new();
        for material in RESOURCE_CATALOG {
            let position = WorldPosition::new(1e12, -7.5e11, 2.5e11);
            let deposit = ResourceDeposit::new(
                DepositId {
                    body: CelestialBodyId::Earth,
                    local: 1,
                },
                RawMaterial::new(material.id, 30.0).unwrap(),
                position,
                30.0,
            )
            .unwrap();
            let entry = SurfaceDeposit {
                deposit,
                body_local_position_meters: [0.0, 0.0, 6e6],
                bounds_radius_meters: 0.2,
            };
            let mesh = visual(entry).unwrap();
            assert_eq!(mesh.center_meters, position.meters());
            let radius = mesh
                .half_extents_meters
                .map(f64::from)
                .iter()
                .map(|v| v * v)
                .sum::<f64>()
                .sqrt();
            assert!((radius - entry.bounds_radius_meters).abs() < 1e-8);
            assert!(!appearances.contains(&(mesh.color, mesh.half_extents_meters)));
            appearances.push((mesh.color, mesh.half_extents_meters));
            let mut depleted = entry;
            depleted.deposit.set_remaining_mass_kg(0.0).unwrap();
            assert!(visual(depleted).is_none());
        }
    }

    #[test]
    fn bounded_generation_is_deterministic_and_absent_in_space() {
        for body in CELESTIAL_BODIES
            .iter()
            .filter(|body| body.role == BodyRole::Solid)
        {
            let position = body
                .center
                .translated([0.0, body.radius_meters + 1.7, 0.0])
                .meters();
            let a = nearby_deposits(position, 0).unwrap();
            assert!(!a.is_empty());
            assert_eq!(a, nearby_deposits(position, 0).unwrap());
            assert!(a.iter().all(|entry| entry.deposit.id().body == body.id));
            assert!(
                nearby_deposits(
                    body.center
                        .translated([0.0, body.radius_meters + 1000.0, 0.0])
                        .meters(),
                    0
                )
                .unwrap()
                .is_empty()
            );
        }
    }
}
