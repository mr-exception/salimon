//! Composition-only resource mapping; world retains physical state.
use salimon_renderer::{ResourceMesh, ResourceMeshInstance, SceneInstance};
use salimon_world::resource_distribution::default_resource_distribution;
use salimon_world::resource_generation::{
    GenerationError, SurfaceDeposit, materialize_nearby_deposits,
};
use salimon_world::resources::{DepositState, ResourceFragment, ResourceId};
use salimon_world::{BodyRole, CELESTIAL_BODIES, WorldPosition};

pub(crate) const ACTIVE_RADIUS_METERS: f64 = 120.0;
type PlacementKey = (u8, u64, u64, Vec<u8>);
#[derive(Default)]
pub(crate) struct PlacementCache {
    entries: std::collections::BTreeMap<PlacementKey, (SurfaceDeposit, u64)>,
    clock: u64,
    pub(crate) misses: u64,
    pub(crate) micros: u128,
}
impl PlacementCache {
    fn get(&mut self, key: &PlacementKey) -> Option<SurfaceDeposit> {
        self.clock += 1;
        self.entries.get_mut(key).map(|(entry, age)| {
            *age = self.clock;
            *entry
        })
    }
    fn insert(&mut self, key: PlacementKey, entry: SurfaceDeposit) {
        if self.entries.len() >= 1024 {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, (_, age))| *age)
                .map(|(k, _)| k.clone());
            if let Some(key) = oldest {
                self.entries.remove(&key);
            }
        }
        self.clock += 1;
        self.entries.insert(key, (entry, self.clock));
    }
}

#[cfg(test)]
pub(crate) fn nearby_deposits(
    position: [f64; 3],
    seed: u64,
) -> Result<Vec<SurfaceDeposit>, GenerationError> {
    nearby_deposits_with_terrain(position, seed, salimon_world::terrain::TerrainMode::Flat)
}
#[cfg(test)]
pub(crate) fn nearby_deposits_with_terrain(
    position: [f64; 3],
    seed: u64,
    terrain: salimon_world::terrain::TerrainMode,
) -> Result<Vec<SurfaceDeposit>, GenerationError> {
    nearby_deposits_impl(position, seed, terrain, None)
}
pub(crate) fn nearby_deposits_cached(
    position: [f64; 3],
    seed: u64,
    terrain: salimon_world::terrain::TerrainMode,
    cache: &std::cell::RefCell<PlacementCache>,
) -> Result<Vec<SurfaceDeposit>, GenerationError> {
    nearby_deposits_impl(position, seed, terrain, Some(cache))
}
fn nearby_deposits_impl(
    position: [f64; 3],
    seed: u64,
    terrain: salimon_world::terrain::TerrainMode,
    cache: Option<&std::cell::RefCell<PlacementCache>>,
) -> Result<Vec<SurfaceDeposit>, GenerationError> {
    let player = WorldPosition::new(position[0], position[1], position[2]);
    let mut deposits = Vec::new();
    for body in CELESTIAL_BODIES
        .iter()
        .filter(|body| body.role == BodyRole::Solid)
    {
        let local = player.offset_from(body.center);
        let distance = local[0].hypot(local[1]).hypot(local[2]);
        let direction = local.map(|v| v / distance);
        let altitude = terrain.clearance(*body, position);
        if altitude.abs() > ACTIVE_RADIUS_METERS {
            continue;
        }
        let entries = materialize_nearby_deposits(
            *body,
            &default_resource_distribution(body.id),
            seed,
            None,
            if terrain == salimon_world::terrain::TerrainMode::Flat {
                player
            } else {
                body.center
                    .translated(direction.map(|v| v * body.radius_meters))
            },
            ACTIVE_RADIUS_METERS,
        )?;
        let definition = terrain
            .definition(body.id)
            .map_or_else(Vec::new, |d| d.to_bytes());
        for mut entry in entries {
            let placement_key = (
                body.id as u8,
                seed,
                entry.deposit.id().local,
                definition.clone(),
            );
            if terrain != salimon_world::terrain::TerrainMode::Flat
                && let Some(found) = cache.and_then(|c| c.borrow_mut().get(&placement_key))
            {
                deposits.push(found);
                continue;
            }
            let placement_started = std::time::Instant::now();
            let up = entry
                .body_local_position_meters
                .map(|v| v / body.radius_meters);
            let ground = terrain.sample(*body, up);
            let visual =
                deposit_mesh_with_terrain(entry, terrain).expect("new deposits are untouched");
            let lift = visual.mesh.support_meters(
                if terrain == salimon_world::terrain::TerrainMode::Flat {
                    up
                } else {
                    [0.0, 1.0, 0.0]
                },
            ) * visual.authored_mesh_scale
                + 0.005;
            let offset = if terrain == salimon_world::terrain::TerrainMode::Flat {
                up.map(|v| v * lift)
            } else {
                std::array::from_fn(|i| {
                    ground.body_local_position_meters[i] - entry.body_local_position_meters[i]
                        + ground.outward_normal[i] * lift
                })
            };
            entry.body_local_position_meters =
                std::array::from_fn(|i| entry.body_local_position_meters[i] + offset[i]);
            entry.deposit = salimon_world::resources::ResourceDeposit::new(
                entry.deposit.id(),
                entry.deposit.material(),
                entry.deposit.position().translated(offset),
                entry.deposit.remaining_mass_kg(),
            )
            .expect("finite terrain placement preserves deposit material and identity");
            if terrain != salimon_world::terrain::TerrainMode::Flat {
                // Preserve the sampled pose and test every authored vertex, rather than
                // treating its tangent support plane as the uneven surface itself.
                for _ in 0..5 {
                    let mesh = deposit_mesh_with_terrain(entry, terrain).expect("new deposit");
                    let mut min_gap = f64::INFINITY;
                    let mut witness = up;
                    for vertex in mesh.mesh.unit_vertices() {
                        let arm = salimon_physics::rotate(
                            mesh.orientation_xyzw,
                            salimon_math::scale(*vertex, mesh.authored_mesh_scale),
                        );
                        let p = salimon_math::add(entry.body_local_position_meters, arm);
                        let radial = salimon_math::scale(p, 1.0 / salimon_math::length(p));
                        let gap = salimon_math::length(p)
                            - body.radius_meters
                            - terrain.height(*body, radial);
                        if gap < min_gap {
                            min_gap = gap;
                            witness = radial;
                        }
                    }
                    if (min_gap - 0.005).abs() <= 0.00001 {
                        break;
                    }
                    let witness_normal = terrain.sample(*body, witness).outward_normal;
                    let derivative = salimon_math::dot(ground.outward_normal, witness_normal)
                        / salimon_math::dot(witness, witness_normal).max(0.1);
                    let correction = (0.005 - min_gap) / derivative.max(0.1);
                    let offset = salimon_math::scale(ground.outward_normal, correction);
                    entry.body_local_position_meters =
                        salimon_math::add(entry.body_local_position_meters, offset);
                    entry.deposit = salimon_world::resources::ResourceDeposit::new(
                        entry.deposit.id(),
                        entry.deposit.material(),
                        entry.deposit.position().translated(offset),
                        entry.deposit.remaining_mass_kg(),
                    )
                    .expect("finite sampled support");
                }
            }
            if terrain != salimon_world::terrain::TerrainMode::Flat
                && let Some(cache) = cache
            {
                cache.borrow_mut().insert(placement_key, entry);
            }
            if let Some(cache) = cache {
                let mut cache = cache.borrow_mut();
                cache.misses += 1;
                cache.micros += placement_started.elapsed().as_micros();
            }
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
#[cfg(test)]
pub(crate) fn deposit_mesh(entry: SurfaceDeposit) -> Option<ResourceMeshInstance> {
    deposit_mesh_with_terrain(entry, salimon_world::terrain::TerrainMode::Flat)
}
pub(crate) fn deposit_mesh_with_terrain(
    entry: SurfaceDeposit,
    terrain: salimon_world::terrain::TerrainMode,
) -> Option<ResourceMeshInstance> {
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
        tint: [1.0; 4],
        mesh,
        center_meters: entry.deposit.position().meters(),
        // Every baked vertex fits +/-salimon_renderer::SPHERE_AUTHORED_RADIUS_METERS m. Inscribe its cube in the
        // authoritative sphere without relying on orientation or geometry shape.
        authored_mesh_scale: entry.bounds_radius_meters
            / (salimon_renderer::SPHERE_AUTHORED_RADIUS_METERS * 3.0_f64.sqrt()),
        orientation_xyzw: if terrain == salimon_world::terrain::TerrainMode::Flat {
            [0.0, 0.0, 0.0, 1.0]
        } else {
            let body = *CELESTIAL_BODIES
                .iter()
                .find(|b| b.id == entry.deposit.id().body)
                .expect("resource body");
            let local = entry.body_local_position_meters;
            let normal = terrain
                .sample(body, local.map(|v| v / salimon_math::length(local)))
                .outward_normal;
            let axis = if normal[0].abs() < 0.8 {
                [1.0, 0.0, 0.0]
            } else {
                [0.0, 0.0, 1.0]
            };
            let tangent = salimon_math::sub(
                axis,
                salimon_math::scale(normal, salimon_math::dot(axis, normal)),
            );
            let forward = salimon_math::scale(tangent, 1.0 / salimon_math::length(tangent));
            salimon_physics::orientation_from_axes([
                forward,
                normal,
                salimon_math::cross(forward, normal),
            ])
        },
    })
}

fn fragment_tint(resource: ResourceId) -> [f32; 4] {
    match resource {
        ResourceId::IronOre => [0.75, 0.22, 0.06, 1.0],
        ResourceId::SilicateRock => [0.45, 0.50, 0.36, 1.0],
        ResourceId::WaterIce => [0.25, 0.85, 1.0, 1.0],
    }
}

pub(crate) fn fragment_visual(fragment: ResourceFragment) -> SceneInstance {
    let color = fragment_tint(fragment.material().resource());
    SceneInstance {
        center_meters: fragment.transform().position().meters(),
        half_extents_meters: [(salimon_world::resource_fragments::diameter_meters(fragment) * 0.5)
            as f32; 3],
        color,
    }
}

/// One shared authored mesh; mass/density determine radius, never fragment ID.
pub(crate) fn fragment_mesh(fragment: ResourceFragment) -> ResourceMeshInstance {
    let color = fragment_tint(fragment.material().resource());
    let radius = salimon_world::spherical_mining::GrowingSphere {
        source: fragment.source(),
        material: fragment.material(),
    }
    .radius_meters();
    ResourceMeshInstance {
        mesh: ResourceMesh::Sphere,
        tint: color,
        center_meters: fragment.transform().position().meters(),
        authored_mesh_scale: radius / salimon_renderer::SPHERE_AUTHORED_RADIUS_METERS,
        orientation_xyzw: fragment.transform().orientation_xyzw(),
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
                for entry in entries.iter() {
                    let mesh = deposit_mesh(*entry).unwrap();
                    let radial = entry.deposit.position().offset_from(body.center);
                    let length = radial[0].hypot(radial[1]).hypot(radial[2]);
                    let up = radial.map(|v| v / length);
                    let bottom = length
                        - body.radius_meters
                        - mesh.mesh.support_meters(up) * mesh.authored_mesh_scale;
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
                                - mesh.mesh.support_meters(up) * mesh.authored_mesh_scale
                        });
                        let max: [f64; 3] = std::array::from_fn(|axis| {
                            let mut down = [0.0; 3];
                            down[axis] = -1.0;
                            mesh.center_meters[axis]
                                + mesh.mesh.support_meters(down) * mesh.authored_mesh_scale
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
    fn shared_sphere_preserves_identity_pose_and_mass_derived_radius() {
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
                    assert_eq!(visual.mesh, ResourceMesh::Sphere);
                    let radius = salimon_world::spherical_mining::GrowingSphere {
                        source: fragment.source(),
                        material: fragment.material(),
                    }
                    .radius_meters();
                    assert_eq!(
                        visual.authored_mesh_scale,
                        radius / salimon_renderer::SPHERE_AUTHORED_RADIUS_METERS
                    );
                    assert_eq!(
                        visual.orientation_xyzw,
                        fragment.transform().orientation_xyzw()
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
                    assert_eq!(moved.authored_mesh_scale, visual.authored_mesh_scale);
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
                    assert!(
                        (mesh.authored_mesh_scale
                            * salimon_renderer::SPHERE_AUTHORED_RADIUS_METERS
                            * 3.0_f64.sqrt()
                            - radius)
                            .abs()
                            < 1e-12
                    );
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

#[cfg(test)]
mod terrain_tests {
    use super::*;
    use salimon_world::terrain::TerrainMode;
    #[test]
    fn sampled_deposit_vertices_clear_ground_and_cached_streaming_preserves_mining() {
        let mode = TerrainMode::Representative;
        for id in [
            salimon_world::CelestialBodyId::Earth,
            salimon_world::CelestialBodyId::Moon,
            salimon_world::CelestialBodyId::Mars,
        ] {
            let body = *CELESTIAL_BODIES.iter().find(|b| b.id == id).unwrap();
            for direction in [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] {
                let ground = mode.sample(body, direction);
                let player = body
                    .center
                    .translated(ground.body_local_position_meters)
                    .meters();
                let mut tool = crate::mining::ResourceGameplay::default();
                tool.terrain = mode;
                let entries = tool.nearby(player, 181).unwrap();
                assert!(!entries.is_empty());
                let oracle = nearby_deposits_with_terrain(player, 181, mode).unwrap();
                assert_eq!(entries.as_ref(), &oracle);
                let mut lowest = f64::INFINITY;
                for entry in entries.iter() {
                    let mesh = deposit_mesh_with_terrain(*entry, mode).unwrap();
                    let mut min = f64::INFINITY;
                    for vertex in mesh.mesh.unit_vertices() {
                        let p = salimon_math::add(
                            entry.body_local_position_meters,
                            salimon_physics::rotate(
                                mesh.orientation_xyzw,
                                salimon_math::scale(*vertex, mesh.authored_mesh_scale),
                            ),
                        );
                        let radius = salimon_math::length(p);
                        let d = salimon_math::scale(p, 1.0 / radius);
                        let gap = radius - body.radius_meters - mode.height(body, d);
                        min = min.min(gap);
                    }
                    assert!(
                        (0.0045..=0.006).contains(&min),
                        "{} deposit {:?} support gap {min}",
                        body.name,
                        entry.deposit.id()
                    );
                    lowest = lowest.min(min);
                }
                let mut source = entries[0].deposit;
                tool.session
                    .extract(&mut source, std::time::Duration::from_secs(100));
                assert_eq!(source.remaining_mass_kg(), 0.0);
                tool.nearby(salimon_math::add(player, [1000.0, 0.0, 0.0]), 181)
                    .unwrap();
                let restored = tool.nearby(player, 181).unwrap();
                let entry = restored
                    .iter()
                    .find(|e| e.deposit.id() == source.id())
                    .unwrap();
                assert_eq!(entry.deposit.remaining_mass_kg(), 0.0);
                assert_eq!(entry.deposit.position(), source.position());
                eprintln!(
                    "{} {:?}: {} deposits; lowest vertex gap {lowest} m",
                    body.name,
                    direction,
                    entries.len()
                );
            }
        }
    }
}
