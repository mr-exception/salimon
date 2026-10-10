//! Authored surface/clearance adapter. World owns allocation; physics owns impulses.
use salimon_character::CharacterSnapshot;
use salimon_math::{add, cross, dot, length, scale, sub};
use salimon_world::mining::MiningRay;
use salimon_world::resource_generation::SurfaceDeposit;
use salimon_world::resources::{FragmentId, RawMaterial, ResourceFragment, ResourceTransform};
use salimon_world::{CELESTIAL_BODIES, WorldPosition};

const CLEARANCE_METERS: f64 = 0.005;

pub(crate) struct EmissionSurface {
    entry: SurfaceDeposit,
    up: [f64; 3],
    player: CharacterSnapshot,
    // Points stay deposit-local until the final pose to preserve distant precision.
    candidates: Vec<([f64; 3], [f64; 3])>,
}

impl EmissionSurface {
    pub(crate) fn new(entry: SurfaceDeposit, ray: MiningRay, player: CharacterSnapshot) -> Self {
        let visual = crate::resource_presentation::deposit_mesh(entry)
            .expect("mining target is nondepleted");
        let body = CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == entry.deposit.id().body)
            .unwrap();
        let radial = entry.deposit.position().offset_from(body.center);
        let up = scale(radial, 1.0 / length(radial));
        let origin = ray.origin.offset_from(entry.deposit.position());
        let vertices: Vec<_> = visual
            .mesh
            .unit_vertices()
            .iter()
            .map(|v| scale(*v, visual.side_meters))
            .collect();
        let first_hit = vertices
            .as_chunks::<3>()
            .0
            .iter()
            .filter_map(|triangle| triangle_distance(origin, ray.direction, triangle))
            .min_by(f64::total_cmp);
        let aim = add(
            origin,
            scale(
                ray.direction,
                first_hit
                    .or_else(|| {
                        ray.sphere_distance(entry.deposit.position(), entry.bounds_radius_meters)
                    })
                    .unwrap_or(0.0),
            ),
        );
        let mut hits = Vec::new();
        let mut faces = Vec::new();
        for triangle in vertices.as_chunks::<3>().0 {
            let a = triangle[0];
            let edge = cross(sub(triangle[1], a), sub(triangle[2], a));
            if length(edge) < 1e-10 {
                continue;
            }
            let normal = scale(edge, 1.0 / length(edge));
            // Only exposed, eye-facing supporting facets. This avoids concave seams
            // placing a full growing envelope inside another part of the deposit.
            if dot(normal, sub(origin, a)) <= 0.0
                || vertices
                    .iter()
                    .any(|v| dot(normal, sub(*v, a)) > CLEARANCE_METERS)
            {
                continue;
            }
            if let Some(distance) = triangle_distance(origin, ray.direction, triangle)
                && first_hit.is_some_and(|first| distance <= first + CLEARANCE_METERS)
            {
                hits.push((
                    distance,
                    add(origin, scale(ray.direction, distance)),
                    normal,
                ));
            }
            let center = scale(add(add(a, triangle[1]), triangle[2]), 1.0 / 3.0);
            faces.push((length(sub(center, aim)), center, normal));
        }
        hits.sort_by(|a, b| a.0.total_cmp(&b.0));
        faces.sort_by(|a, b| a.0.total_cmp(&b.0));
        let candidates = hits
            .into_iter()
            .chain(faces)
            .map(|(_, p, n)| (p, n))
            .collect();
        Self {
            entry,
            up,
            player,
            candidates,
        }
    }

    pub(crate) fn spawn(
        &self,
        id: FragmentId,
        material: RawMaterial,
        existing: &[ResourceFragment],
    ) -> Option<(ResourceTransform, [f64; 3])> {
        let full = RawMaterial::new(
            material.resource(),
            salimon_world::resource_fragments::FRAGMENT_MAX_MASS_KG,
        )
        .unwrap();
        let template = ResourceFragment::new(
            id,
            self.entry.deposit.id(),
            full,
            ResourceTransform::new(self.entry.deposit.position(), [0.0, 0.0, 0.0, 1.0]).unwrap(),
        );
        let hull = crate::fragment_physics::fragment_hull(template);
        let side = salimon_world::resource_fragments::side_meters(template);
        let radius = salimon_world::resource_size::fragment_contact_radius_meters(full);
        let body = CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == self.entry.deposit.id().body)
            .unwrap();
        for &(contact, normal) in &self.candidates {
            let support = hull.support([0.0, 0.0, 0.0, 1.0], side, scale(normal, -1.0));
            let position = self
                .entry
                .deposit
                .position()
                .translated(add(contact, scale(normal, support + CLEARANCE_METERS)));
            let ground_support = hull.support([0.0, 0.0, 0.0, 1.0], side, scale(self.up, -1.0));
            if length(position.offset_from(body.center)) - body.radius_meters
                < ground_support + CLEARANCE_METERS
            {
                continue;
            }
            // Capsule clearance and existing envelopes are conservative. Try another
            // real facet, or pause extraction; never debit hidden/overlapping output.
            let eye_offset = position.offset_from(world(self.player.eye_position_meters));
            let along = dot(eye_offset, scale(self.up, -1.0)).clamp(0.0, 1.5);
            if length(add(eye_offset, scale(self.up, along))) < radius + 0.35 {
                continue;
            }
            if existing.iter().any(|p| {
                length(position.offset_from(p.transform().position()))
                    < radius
                        + salimon_world::resource_size::fragment_contact_radius_meters(p.material())
                        + CLEARANCE_METERS
            }) {
                continue;
            }
            let pose = ResourceTransform::new(position, [0.0, 0.0, 0.0, 1.0]).unwrap();
            let velocity =
                salimon_physics::surface_ejection_velocity(normal, self.up, (id.0 % 5) as u8);
            return Some((pose, velocity));
        }
        None
    }
}

fn world(p: [f64; 3]) -> WorldPosition {
    WorldPosition::new(p[0], p[1], p[2])
}

fn triangle_distance(origin: [f64; 3], direction: [f64; 3], t: &[[f64; 3]]) -> Option<f64> {
    let e1 = sub(t[1], t[0]);
    let e2 = sub(t[2], t[0]);
    let h = cross(direction, e2);
    let determinant = dot(e1, h);
    if determinant.abs() < 1e-10 {
        return None;
    }
    let s = sub(origin, t[0]);
    let u = dot(s, h) / determinant;
    let q = cross(s, e1);
    let v = dot(direction, q) / determinant;
    let distance = dot(e2, q) / determinant;
    (u >= 0.0 && v >= 0.0 && u + v <= 1.0 && distance >= 0.0).then_some(distance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_world::resources::{DepositId, RESOURCE_CATALOG, ResourceDeposit};

    fn surface(
        resource: salimon_world::resources::ResourceId,
        variant: u64,
        up: [f64; 3],
    ) -> EmissionSurface {
        let body = CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == salimon_world::CelestialBodyId::Earth)
            .unwrap();
        let center = body.center.translated(scale(up, body.radius_meters + 3.0));
        let deposit = ResourceDeposit::new(
            DepositId {
                body: body.id,
                local: variant,
            },
            RawMaterial::new(resource, 20.0).unwrap(),
            center,
            20.0,
        )
        .unwrap();
        let entry = SurfaceDeposit {
            deposit,
            body_local_position_meters: center.offset_from(body.center),
            bounds_radius_meters: salimon_world::resource_size::deposit_radius_meters(
                deposit.material(),
            ),
        };
        let eye = center.translated(scale(up, 4.0));
        let player = CharacterSnapshot {
            location: salimon_character::CharacterLocation::Surface,
            eye_position_meters: eye.meters(),
            look_target_meters: center.meters(),
            up: up.map(|v| v as f32),
            local_ship_position_meters: None,
            doorway_blend_fraction: None,
        };
        EmissionSurface::new(entry, MiningRay::new(eye, center).unwrap(), player)
    }

    #[test]
    fn all_deposit_variants_emit_from_real_facets_with_full_growth_clearance() {
        for material in RESOURCE_CATALOG {
            for variant in 0..4 {
                for up in [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]] {
                    let emission = surface(material.id, variant, up);
                    for mass in [0.032, 2.0] {
                        let (pose, velocity) = emission
                            .spawn(
                                FragmentId(1),
                                RawMaterial::new(material.id, mass).unwrap(),
                                &[],
                            )
                            .expect("exposed authored surface");
                        assert!(dot(velocity, up) > 0.0);
                        assert_eq!(
                            Some((pose, velocity)),
                            emission.spawn(
                                FragmentId(1),
                                RawMaterial::new(material.id, mass).unwrap(),
                                &[]
                            )
                        );
                        let full = RawMaterial::new(material.id, 2.0).unwrap();
                        let piece = ResourceFragment::new(
                            FragmentId(1),
                            emission.entry.deposit.id(),
                            full,
                            pose,
                        );
                        let hull = crate::fragment_physics::fragment_hull(piece);
                        let side = salimon_world::resource_fragments::side_meters(piece);
                        let local = pose
                            .position()
                            .offset_from(emission.entry.deposit.position());
                        let visual =
                            crate::resource_presentation::deposit_mesh(emission.entry).unwrap();
                        assert!(
                            emission.candidates.iter().any(|(point, normal)| {
                                let expected = add(
                                    *point,
                                    scale(
                                        *normal,
                                        hull.support(
                                            pose.orientation_xyzw(),
                                            side,
                                            scale(*normal, -1.0),
                                        ) + CLEARANCE_METERS,
                                    ),
                                );
                                length(sub(local, expected)) < 0.0003
                                    && visual.mesh.unit_vertices().iter().all(|v| {
                                        dot(*normal, sub(local, scale(*v, visual.side_meters)))
                                            >= hull.support(
                                                pose.orientation_xyzw(),
                                                side,
                                                scale(*normal, -1.0),
                                            ) - 0.0003
                                    })
                            }),
                            "{material:?} variant {variant}: fragment clears authored deposit"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn occupied_facets_wait_without_hidden_mass_and_retry_same_identity() {
        let mut emission = surface(
            salimon_world::resources::ResourceId::SilicateRock,
            0,
            [0.0, 1.0, 0.0],
        );
        let (point, normal) = emission.candidates[0];
        emission.candidates = vec![(point, normal)];
        let mut session = salimon_world::mining::MiningSession::default();
        let mut deposit = emission.entry.deposit;
        let mut emissions = 0;
        for _ in 0..100 {
            session.extract_with_spawn(
                &mut deposit,
                std::time::Duration::from_millis(16),
                |id, material, existing| {
                    let (pose, _) = emission.spawn(id, material, existing)?;
                    emissions += 1;
                    Some(pose)
                },
            );
        }
        assert_eq!(emissions, 1);
        assert_eq!(session.fragments().len(), 1);
        assert_eq!(session.extracted_mass_kg(), 2.0);
        assert_eq!(deposit.remaining_mass_kg(), 18.0);
        let first = session.fragments()[0];
        session.move_loose(
            first.id(),
            ResourceTransform::new(
                first.transform().position().translated([10.0, 0.0, 0.0]),
                first.transform().orientation_xyzw(),
            )
            .unwrap(),
        );
        session.extract_with_spawn(
            &mut deposit,
            std::time::Duration::from_millis(16),
            |id, material, existing| emission.spawn(id, material, existing).map(|v| v.0),
        );
        assert_eq!(session.fragments()[1].id(), FragmentId(2));
        assert_eq!(session.fragments()[0].material().mass_kg(), 2.0);
        let restored = session.fragments().to_vec();
        let mut requery = [emission.entry];
        session.apply_to(&mut requery);
        assert_eq!(session.fragments(), restored);
    }

    #[test]
    fn player_capsule_blocks_emission_on_crowded_surface() {
        let mut emission = surface(
            salimon_world::resources::ResourceId::WaterIce,
            0,
            [0.0, 1.0, 0.0],
        );
        let material =
            RawMaterial::new(salimon_world::resources::ResourceId::WaterIce, 2.0).unwrap();
        let (pose, _) = emission.spawn(FragmentId(1), material, &[]).unwrap();
        emission.candidates.truncate(1);
        emission.player.eye_position_meters = pose.position().meters();
        assert!(emission.spawn(FragmentId(1), material, &[]).is_none());
    }
}
