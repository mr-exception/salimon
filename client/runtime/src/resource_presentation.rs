//! Composition-only resource mapping; world retains physical state.
use salimon_renderer::{ResourceMesh, ResourceMeshInstance, SceneInstance};
use salimon_world::resource_distribution::default_resource_distribution;
use salimon_world::resource_generation::{
    GenerationError, SurfaceDeposit, materialize_nearby_deposits,
};
use salimon_world::resources::{DepositState, ResourceFragment, ResourceId};
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
    if entry.deposit.state() == DepositState::Depleted
        || entry.deposit.material().resource() != ResourceId::IronOre
    {
        return None;
    }
    let color = [0.75, 0.22, 0.06, 1.0];
    let proportions: [f64; 3] = [1.0, 0.8, 0.65];
    // Inscribe the greybox in the authoritative spherical bound. Its center is
    // the surface anchor, so only the outward half protrudes on every body/latitude.
    let norm = proportions[0].hypot(proportions[1]).hypot(proportions[2]);
    Some(SceneInstance {
        center_meters: entry.deposit.position().meters(),
        half_extents_meters: proportions.map(|p| (p * entry.bounds_radius_meters / norm) as f32),
        color,
    })
}

/// Stable local identity selects presentation only. World bounds and mass stay authoritative.
pub(crate) fn deposit_mesh(entry: SurfaceDeposit) -> Option<ResourceMeshInstance> {
    if entry.deposit.state() == DepositState::Depleted {
        return None;
    }
    let variants = match entry.deposit.material().resource() {
        ResourceId::WaterIce => [
            ResourceMesh::IceDepositSpire,
            ResourceMesh::IceDepositCrown,
            ResourceMesh::IceDepositRidge,
            ResourceMesh::IceDepositShelf,
        ],
        ResourceId::SilicateRock => [
            ResourceMesh::SilicateDepositBoulder,
            ResourceMesh::SilicateDepositSlab,
            ResourceMesh::SilicateDepositRidge,
            ResourceMesh::SilicateDepositScree,
        ],
        ResourceId::IronOre => return None,
    };
    let mesh = variants[(entry.deposit.id().local % 4) as usize];
    Some(ResourceMeshInstance {
        mesh,
        center_meters: entry.deposit.position().meters(),
        // Every baked vertex fits +/-0.48 m. Inscribe its cube in the
        // authoritative sphere without relying on orientation or geometry shape.
        side_meters: entry.bounds_radius_meters / (0.48 * 3.0_f64.sqrt()),
    })
}

pub(crate) fn fragment_visual(fragment: ResourceFragment) -> SceneInstance {
    let color = match fragment.material().resource() {
        ResourceId::IronOre => [0.75, 0.22, 0.06, 1.0],
        ResourceId::SilicateRock => [0.45, 0.50, 0.36, 1.0],
        ResourceId::WaterIce => [0.25, 0.85, 1.0, 1.0],
    };
    SceneInstance {
        center_meters: fragment.transform().position().meters(),
        half_extents_meters: [(salimon_world::resource_fragments::side_meters(fragment) * 0.5)
            as f32; 3],
        color,
    }
}

/// Identity alone chooses the authored variant; movement and mass cannot change it.
pub(crate) fn fragment_mesh(fragment: ResourceFragment) -> ResourceMeshInstance {
    let even = fragment.id().0.is_multiple_of(2);
    let mesh = match fragment.material().resource() {
        ResourceId::WaterIce => {
            if even {
                ResourceMesh::IceShard
            } else {
                ResourceMesh::IceCluster
            }
        }
        ResourceId::SilicateRock => {
            if even {
                ResourceMesh::SilicateSlab
            } else {
                ResourceMesh::SilicateRidge
            }
        }
        ResourceId::IronOre => {
            if even {
                ResourceMesh::IronChunk
            } else {
                ResourceMesh::IronShard
            }
        }
    };
    ResourceMeshInstance {
        mesh,
        center_meters: fragment.transform().position().meters(),
        side_meters: salimon_world::resource_fragments::side_meters(fragment),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_world::CelestialBodyId;
    use salimon_world::resources::{
        DepositId, FragmentId, RESOURCE_CATALOG, RawMaterial, ResourceDeposit, ResourceFragment,
        ResourceTransform,
    };

    #[test]
    fn authored_fragment_variants_follow_identity_and_authoritative_size() {
        for resource in [
            ResourceId::WaterIce,
            ResourceId::SilicateRock,
            ResourceId::IronOre,
        ] {
            for id in [1, 2, 3, 4] {
                for mass in [0.01, 2.0, 100.0] {
                    let mut fragment = ResourceFragment::new(
                        FragmentId(id),
                        DepositId {
                            body: CelestialBodyId::Earth,
                            local: 1,
                        },
                        RawMaterial::new(resource, mass).unwrap(),
                        ResourceTransform::new(
                            WorldPosition::new(4.0, 5.0, 6.0),
                            [0.0, 0.0, 0.0, 1.0],
                        )
                        .unwrap(),
                    );
                    let visual = fragment_mesh(fragment);
                    let expected = match (resource, id % 2 == 0) {
                        (ResourceId::WaterIce, true) => ResourceMesh::IceShard,
                        (ResourceId::WaterIce, false) => ResourceMesh::IceCluster,
                        (ResourceId::SilicateRock, true) => ResourceMesh::SilicateSlab,
                        (ResourceId::SilicateRock, false) => ResourceMesh::SilicateRidge,
                        (ResourceId::IronOre, true) => ResourceMesh::IronChunk,
                        (ResourceId::IronOre, false) => ResourceMesh::IronShard,
                    };
                    assert_eq!(visual.mesh, expected);
                    assert_eq!(
                        visual.side_meters,
                        salimon_world::resource_fragments::side_meters(fragment)
                    );
                    assert_eq!(
                        visual.center_meters,
                        fragment.transform().position().meters()
                    );
                    fragment.set_transform(
                        ResourceTransform::new(
                            WorldPosition::new(1e12, 2e12, 3e12),
                            [0.0, 0.0, 0.0, 1.0],
                        )
                        .unwrap(),
                    );
                    let moved = fragment_mesh(fragment);
                    assert_eq!(moved.mesh, visual.mesh);
                    assert_eq!(moved.side_meters, visual.side_meters);
                    assert_eq!(fragment.material().mass_kg(), mass);
                }
            }
        }
    }

    #[test]
    fn iron_greybox_is_bounded_at_its_exact_anchor() {
        let mut appearances = Vec::new();
        for material in RESOURCE_CATALOG
            .iter()
            .filter(|m| m.id == ResourceId::IronOre)
        {
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
    fn authored_deposit_identity_bounds_and_depletion_survive_rematerialization() {
        for (resource, variants) in [
            (
                ResourceId::WaterIce,
                [
                    ResourceMesh::IceDepositSpire,
                    ResourceMesh::IceDepositCrown,
                    ResourceMesh::IceDepositRidge,
                    ResourceMesh::IceDepositShelf,
                ],
            ),
            (
                ResourceId::SilicateRock,
                [
                    ResourceMesh::SilicateDepositBoulder,
                    ResourceMesh::SilicateDepositSlab,
                    ResourceMesh::SilicateDepositRidge,
                    ResourceMesh::SilicateDepositScree,
                ],
            ),
        ] {
            for local in 0..8 {
                for radius in [0.01, 0.2, 3.0] {
                    let make = |mass| SurfaceDeposit {
                        deposit: ResourceDeposit::new(
                            DepositId {
                                body: CelestialBodyId::Earth,
                                local,
                            },
                            RawMaterial::new(resource, 30.0).unwrap(),
                            WorldPosition::new(1e12, -7.5e11, 2.5e11),
                            mass,
                        )
                        .unwrap(),
                        body_local_position_meters: [0.0, 0.0, 6e6],
                        bounds_radius_meters: radius,
                    };
                    let original = make(30.0);
                    let mesh = deposit_mesh(original).unwrap();
                    assert_eq!(mesh.mesh, variants[(local % 4) as usize]);
                    assert_eq!(mesh.center_meters, original.deposit.position().meters());
                    assert!((mesh.side_meters * 0.48 * 3.0_f64.sqrt() - radius).abs() < 1e-12);
                    assert_eq!(deposit_mesh(make(1.0)), Some(mesh));
                    assert_eq!(deposit_mesh(make(30.0)), Some(mesh));
                    assert!(visual(original).is_none());
                    assert!(deposit_mesh(make(0.0)).is_none());
                    let mut rock = original;
                    rock.deposit = ResourceDeposit::new(
                        original.deposit.id(),
                        RawMaterial::new(ResourceId::IronOre, 30.0).unwrap(),
                        original.deposit.position(),
                        30.0,
                    )
                    .unwrap();
                    assert!(deposit_mesh(rock).is_none());
                    assert!(visual(rock).is_some());
                }
            }
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
