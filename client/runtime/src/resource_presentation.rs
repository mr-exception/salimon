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
pub(crate) fn fragment_mesh(fragment: ResourceFragment) -> Option<ResourceMeshInstance> {
    (fragment.material().resource() == ResourceId::WaterIce).then(|| ResourceMeshInstance {
        mesh: if fragment.id().0.is_multiple_of(2) {
            ResourceMesh::IceShard
        } else {
            ResourceMesh::IceCluster
        },
        center_meters: fragment.transform().position().meters(),
        side_meters: salimon_world::resource_fragments::side_meters(fragment),
    })
}

/// Three low-cost cuboids give unmigrated materials a recognizable silhouette while
/// keeping all pieces inside the physical fragment's bounding cube.
pub(crate) fn fragment_visuals(fragment: ResourceFragment) -> Vec<SceneInstance> {
    type ShapeParts = [([f64; 3], [f64; 3]); 3];
    type ShapeColors = [[f32; 4]; 3];
    let side = salimon_world::resource_fragments::side_meters(fragment);
    let center = fragment.transform().position().meters();
    let flip = if fragment.id().0.is_multiple_of(2) {
        -1.0
    } else {
        1.0
    };
    let (parts, colors): (ShapeParts, ShapeColors) = match fragment.material().resource() {
        ResourceId::IronOre => (
            [
                ([0.0, -0.09, 0.0], [0.43, 0.31, 0.37]),
                ([0.22 * flip, 0.22, -0.13], [0.19, 0.23, 0.18]),
                ([-0.25 * flip, 0.04, 0.22], [0.18, 0.18, 0.16]),
            ],
            [
                [0.55, 0.16, 0.10, 1.0],
                [0.96, 0.40, 0.12, 1.0],
                [0.28, 0.22, 0.20, 1.0],
            ],
        ),
        ResourceId::SilicateRock => (
            [
                ([0.0, -0.20, 0.0], [0.48, 0.25, 0.45]),
                ([0.15 * flip, 0.04, 0.06], [0.30, 0.18, 0.32]),
                ([-0.24 * flip, -0.02, -0.17], [0.20, 0.12, 0.22]),
            ],
            [
                [0.42, 0.47, 0.35, 1.0],
                [0.62, 0.65, 0.52, 1.0],
                [0.27, 0.31, 0.26, 1.0],
            ],
        ),
        ResourceId::WaterIce => return Vec::new(),
    };
    parts
        .into_iter()
        .zip(colors)
        .map(|((offset, extent), color)| SceneInstance {
            center_meters: std::array::from_fn(|i| center[i] + offset[i] * side),
            half_extents_meters: extent.map(|v| (v * side) as f32),
            color,
        })
        .collect()
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
    fn ice_variant_follows_identity_and_authoritative_size() {
        for id in [1, 2, 3, 4] {
            for mass in [0.01, 2.0, 100.0] {
                let mut fragment = ResourceFragment::new(
                    FragmentId(id),
                    DepositId {
                        body: CelestialBodyId::Earth,
                        local: 1,
                    },
                    RawMaterial::new(ResourceId::WaterIce, mass).unwrap(),
                    ResourceTransform::new(WorldPosition::new(4.0, 5.0, 6.0), [0.0, 0.0, 0.0, 1.0])
                        .unwrap(),
                );
                let visual = fragment_mesh(fragment).unwrap();
                assert_eq!(
                    visual.mesh,
                    if id % 2 == 0 {
                        ResourceMesh::IceShard
                    } else {
                        ResourceMesh::IceCluster
                    }
                );
                assert_eq!(
                    visual.side_meters,
                    salimon_world::resource_fragments::side_meters(fragment)
                );
                assert_eq!(
                    visual.center_meters,
                    fragment.transform().position().meters()
                );
                assert!(fragment_visuals(fragment).is_empty());
                fragment.set_transform(
                    ResourceTransform::new(
                        WorldPosition::new(1e12, 2e12, 3e12),
                        [0.0, 0.0, 0.0, 1.0],
                    )
                    .unwrap(),
                );
                let moved = fragment_mesh(fragment).unwrap();
                assert_eq!(moved.mesh, visual.mesh);
                assert_eq!(moved.side_meters, visual.side_meters);
                assert_eq!(fragment.material().mass_kg(), mass);
            }
        }
    }

    #[test]
    fn fragment_materials_have_distinct_silhouettes_and_accents() {
        let mut shapes = Vec::new();
        for material in RESOURCE_CATALOG {
            let center = WorldPosition::new(4.0, 5.0, 6.0);
            let piece = ResourceFragment::new(
                FragmentId(1),
                DepositId {
                    body: CelestialBodyId::Earth,
                    local: 1,
                },
                RawMaterial::new(material.id, 2.0).unwrap(),
                ResourceTransform::new(center, [0.0, 0.0, 0.0, 1.0]).unwrap(),
            );
            if material.id == ResourceId::WaterIce {
                assert!(fragment_visuals(piece).is_empty());
                continue;
            }
            let visuals = fragment_visuals(piece);
            assert_eq!(visuals.len(), 3);
            let side = salimon_world::resource_fragments::side_meters(piece);
            for visual in &visuals {
                for axis in 0..3 {
                    assert!(
                        (visual.center_meters[axis] - center.meters()[axis]).abs()
                            + f64::from(visual.half_extents_meters[axis])
                            <= side * 0.51
                    );
                }
            }
            let signature: Vec<_> = visuals
                .iter()
                .map(|part| (part.half_extents_meters, part.color))
                .collect();
            assert!(!shapes.contains(&signature));
            shapes.push(signature);
            let alternate = ResourceFragment::new(
                FragmentId(2),
                piece.source(),
                piece.material(),
                piece.transform(),
            );
            assert_ne!(visuals, fragment_visuals(alternate));
        }
    }

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
