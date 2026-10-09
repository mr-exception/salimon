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
        let entries = materialize_nearby_deposits(
            *body,
            &default_resource_distribution(body.id),
            seed,
            None,
            player,
            ACTIVE_RADIUS_METERS,
        )?;
        for mut entry in entries {
            let up = entry
                .body_local_position_meters
                .map(|v| v / body.radius_meters);
            let visual = deposit_mesh(entry).expect("new deposits are untouched");
            let lift = visual.mesh.support_meters(up) * visual.side_meters + 0.005;
            let offset = up.map(|v| v * lift);
            entry.body_local_position_meters =
                std::array::from_fn(|i| entry.body_local_position_meters[i] + offset[i]);
            entry.deposit = salimon_world::resources::ResourceDeposit::new(
                entry.deposit.id(),
                entry.deposit.material(),
                entry.deposit.position().translated(offset),
                entry.deposit.remaining_mass_kg(),
            )
            .expect("finite terrain placement preserves deposit material and identity");
            deposits.push(entry);
        }
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
        ResourceId::IronOre => [
            ResourceMesh::IronDepositNodule,
            ResourceMesh::IronDepositVein,
            ResourceMesh::IronDepositLedge,
            ResourceMesh::IronDepositRubble,
        ],
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
        DepositId, FragmentId, RawMaterial, ResourceDeposit, ResourceFragment, ResourceTransform,
    };

    #[test]
    fn terrain_support_and_default_spacing_cover_enlarged_deposits() {
        for body in CELESTIAL_BODIES
            .iter()
            .filter(|b| b.role == BodyRole::Solid)
        {
            for direction in [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0 / 3.0_f64.sqrt(); 3]] {
                let position = body
                    .center
                    .translated(direction.map(|v| v * body.radius_meters))
                    .meters();
                let entries = nearby_deposits(position, 0).unwrap();
                for entry in &entries {
                    let mesh = deposit_mesh(*entry).unwrap();
                    let radial = entry.deposit.position().offset_from(body.center);
                    let length = radial[0].hypot(radial[1]).hypot(radial[2]);
                    let up = radial.map(|v| v / length);
                    let bottom = length
                        - body.radius_meters
                        - mesh.mesh.support_meters(up) * mesh.side_meters;
                    assert!((bottom - 0.005).abs() < 0.001);
                }
                let bounds: Vec<_> = entries
                    .iter()
                    .map(|entry| {
                        let mesh = deposit_mesh(*entry).unwrap();
                        let min: [f64; 3] = std::array::from_fn(|axis| {
                            let mut up = [0.0; 3];
                            up[axis] = 1.0;
                            mesh.center_meters[axis]
                                - mesh.mesh.support_meters(up) * mesh.side_meters
                        });
                        let max: [f64; 3] = std::array::from_fn(|axis| {
                            let mut down = [0.0; 3];
                            down[axis] = -1.0;
                            mesh.center_meters[axis]
                                + mesh.mesh.support_meters(down) * mesh.side_meters
                        });
                        (min, max)
                    })
                    .collect();
                let mut overlaps = 0;
                for (i, (min_a, max_a)) in bounds.iter().enumerate() {
                    for (min_b, max_b) in &bounds[i + 1..] {
                        if !(0..3)
                            .any(|axis| max_a[axis] < min_b[axis] || max_b[axis] < min_a[axis])
                        {
                            overlaps += 1;
                        }
                    }
                }
                eprintln!(
                    "spacing audit {} {:?}: {} deposits, {} possible mesh AABB overlaps",
                    body.name,
                    direction,
                    entries.len(),
                    overlaps
                );
                // Audit proximity without changing seeded identities or silently deleting mass.
                assert_eq!(entries, nearby_deposits(position, 0).unwrap());
            }
        }
    }

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
            (
                ResourceId::IronOre,
                [
                    ResourceMesh::IronDepositNodule,
                    ResourceMesh::IronDepositVein,
                    ResourceMesh::IronDepositLedge,
                    ResourceMesh::IronDepositRubble,
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
                    assert!(deposit_mesh(make(0.0)).is_none());
                    assert_eq!(original.deposit.remaining_mass_kg(), 30.0);
                    assert_eq!(original.deposit.material().resource(), resource);
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
