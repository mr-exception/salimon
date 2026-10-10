//! Fragment/session adapter for the portable physical-object simulation.
//! Selects geometry and frames; physical rules live in salimon-physics.
use std::time::Duration;

use salimon_math::{dot, length, sub};

use crate::mining::MiningTool;
use salimon_character::{SHIP_FLOOR_HEIGHT_METERS, ShipFrame, ship_floor_placement};
use salimon_world::resources::{FragmentId, ResourceTransform};
use salimon_world::{BodyRole, CELESTIAL_BODIES, WorldPosition};

use salimon_physics::{ObjectState, SphereSurface, Surface};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FragmentMotion {
    pub(crate) velocity: [f64; 3],
    pub(crate) angular_velocity: [f64; 3],
    pub(crate) ship_orientation: Option<[f64; 4]>,
}

pub(crate) fn release(
    tool: &mut MiningTool,
    id: FragmentId,
    player_forward: [f64; 3],
    frame: ShipFrame,
    inside: bool,
) {
    let velocity = if inside {
        salimon_physics::floor_release_velocity([
            dot(player_forward, frame.axes[0]),
            0.0,
            dot(player_forward, frame.axes[2]),
        ])
    } else {
        let position = tool
            .session
            .fragments()
            .iter()
            .find(|p| p.id() == id)
            .expect("released fragment exists")
            .transform()
            .position()
            .meters();
        salimon_physics::release_velocity(player_forward, position, nearest_body(position))
    };
    let world_orientation = tool
        .session
        .fragments()
        .iter()
        .find(|p| p.id() == id)
        .expect("released fragment")
        .transform()
        .orientation_xyzw();
    tool.fragment_motion.insert(
        id,
        FragmentMotion {
            velocity,
            angular_velocity: [0.0; 3],
            ship_orientation: inside.then(|| to_local_orientation(frame, world_orientation)),
        },
    );
}

pub(crate) fn advance(tool: &mut MiningTool, frame: ShipFrame, delta: Duration, player: [f64; 3]) {
    let elapsed = delta.as_secs_f64();
    if elapsed <= 0.0 {
        return;
    }
    let carried = tool.session.carried_id();
    let pieces: Vec<_> = tool
        .session
        .fragments()
        .iter()
        .copied()
        .filter(|p| Some(p.id()) != carried)
        .filter(|p| length(sub(p.transform().position().meters(), player)) < 125.0)
        .map(|p| {
            let ship = tool.ship_fragments.contains_key(&p.id());
            let position = if ship {
                tool.ship_fragments[&p.id()]
            } else {
                p.transform().position().meters()
            };
            let body = if ship { None } else { nearest_body(position) };
            // Only creation installs an ejection. Missing/restored motion is at rest.
            let initial = tool
                .fragment_motion
                .get(&p.id())
                .map_or([0.0; 3], |m| m.velocity);
            (
                p,
                ObjectState {
                    position,
                    velocity: initial,
                    radius: salimon_world::resource_size::fragment_contact_radius_meters(
                        p.material(),
                    ),
                    ground_support_meters: 0.0,
                    hull: Some(fragment_hull(p)),
                    side_meters: salimon_world::resource_fragments::side_meters(p),
                    orientation: if ship {
                        tool.fragment_motion
                            .get(&p.id())
                            .and_then(|m| m.ship_orientation)
                            .unwrap_or_else(|| {
                                to_local_orientation(frame, p.transform().orientation_xyzw())
                            })
                    } else {
                        p.transform().orientation_xyzw()
                    },
                    angular_velocity: tool
                        .fragment_motion
                        .get(&p.id())
                        .map_or([0.0; 3], |m| m.angular_velocity),
                    mass_kg: p.material().mass_kg(),
                    surface: if ship {
                        Surface::Floor {
                            height_meters: SHIP_FLOOR_HEIGHT_METERS,
                        }
                    } else {
                        body.map(Surface::Sphere).unwrap_or(Surface::Unsupported)
                    },
                },
            )
        })
        .collect();
    if pieces.is_empty() {
        return;
    }
    let mut states: Vec<_> = pieces.iter().map(|(_, state)| state.clone()).collect();
    salimon_physics::advance(&mut states, delta, |position, radius| {
        ship_floor_placement(position, radius).is_some()
    });
    for ((fragment, _), piece) in pieces.into_iter().zip(states) {
        tool.fragment_motion.insert(
            fragment.id(),
            FragmentMotion {
                velocity: piece.velocity,
                angular_velocity: piece.angular_velocity,
                ship_orientation: matches!(piece.surface, Surface::Floor { .. })
                    .then_some(piece.orientation),
            },
        );
        let position = if matches!(piece.surface, Surface::Floor { .. }) {
            tool.ship_fragments.insert(fragment.id(), piece.position);
            frame.local_to_world(piece.position)
        } else {
            piece.position
        };
        let pose = ResourceTransform::new(
            WorldPosition::new(position[0], position[1], position[2]),
            if matches!(piece.surface, Surface::Floor { .. }) {
                to_world_orientation(frame, piece.orientation)
            } else {
                piece.orientation
            },
        )
        .expect("finite fragment physics pose");
        tool.session.move_loose(fragment.id(), pose);
    }
}

pub(crate) fn fragment_hull(
    fragment: salimon_world::resources::ResourceFragment,
) -> std::sync::Arc<salimon_physics::ConvexHull> {
    use salimon_renderer::ResourceMesh;
    static HULLS: std::sync::LazyLock<[std::sync::Arc<salimon_physics::ConvexHull>; 6]> =
        std::sync::LazyLock::new(|| {
            [
                ResourceMesh::IceShard,
                ResourceMesh::IceCluster,
                ResourceMesh::SilicateSlab,
                ResourceMesh::SilicateRidge,
                ResourceMesh::IronChunk,
                ResourceMesh::IronShard,
            ]
            .map(|mesh| {
                std::sync::Arc::new(salimon_physics::ConvexHull::new(
                    mesh.unit_vertices().iter().copied(),
                ))
            })
        });
    HULLS[crate::resource_presentation::fragment_mesh(fragment).mesh as usize].clone()
}

pub(crate) fn to_world_orientation(frame: ShipFrame, local: [f64; 4]) -> [f64; 4] {
    salimon_physics::compose(salimon_physics::orientation_from_axes(frame.axes), local)
}
fn to_local_orientation(frame: ShipFrame, world: [f64; 4]) -> [f64; 4] {
    let q = salimon_physics::orientation_from_axes(frame.axes);
    salimon_physics::compose([-q[0], -q[1], -q[2], q[3]], world)
}

pub(crate) fn support(fragment: salimon_world::resources::ResourceFragment, up: [f64; 3]) -> f64 {
    fragment_hull(fragment).support(
        fragment.transform().orientation_xyzw(),
        salimon_world::resource_fragments::side_meters(fragment),
        up.map(|v| -v),
    )
}

fn nearest_body(position: [f64; 3]) -> Option<SphereSurface> {
    CELESTIAL_BODIES
        .iter()
        .filter(|body| body.role == BodyRole::Solid)
        .min_by(|a, b| {
            (length(sub(position, a.center.meters())) - a.radius_meters)
                .abs()
                .total_cmp(&(length(sub(position, b.center.meters())) - b.radius_meters).abs())
        })
        .map(|body| SphereSurface {
            center: body.center.meters(),
            radius: body.radius_meters,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_world::CelestialBodyId;
    use salimon_world::resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId};

    fn frame() -> ShipFrame {
        ShipFrame {
            origin_meters: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }
    fn mined(mass: f64) -> MiningTool {
        let mut tool = MiningTool::default();
        let mut deposit = ResourceDeposit::new(
            DepositId {
                body: CelestialBodyId::Earth,
                local: 1,
            },
            RawMaterial::new(ResourceId::SilicateRock, mass).unwrap(),
            WorldPosition::new(0.0, 0.0, 0.0),
            mass,
        )
        .unwrap();
        tool.session
            .extract(&mut deposit, Duration::from_secs_f64(mass / 2.0));
        tool
    }

    #[test]
    fn all_authored_variants_settle_rotated_at_current_scale_and_preserve_mass() {
        for resource in [
            ResourceId::IronOre,
            ResourceId::SilicateRock,
            ResourceId::WaterIce,
        ] {
            let mut tool = MiningTool::default();
            let mut deposit = ResourceDeposit::new(
                DepositId {
                    body: CelestialBodyId::Earth,
                    local: 99,
                },
                RawMaterial::new(resource, 4.0).unwrap(),
                WorldPosition::new(0.0, 0.0, 0.0),
                4.0,
            )
            .unwrap();
            tool.session.extract(&mut deposit, Duration::from_secs(2));
            let initial = tool.session.fragments().to_vec();
            for (index, p) in initial.iter().enumerate() {
                let local = [-5.0, 1.3, index as f64 * 1.5 - 0.75];
                tool.session.move_loose(
                    p.id(),
                    ResourceTransform::new(
                        WorldPosition::new(local[0], local[1], local[2]),
                        [0.0, 0.0, 0.35_f64.sin(), 0.35_f64.cos()],
                    )
                    .unwrap(),
                );
                tool.ship_fragments.insert(p.id(), local);
                tool.fragment_motion
                    .insert(p.id(), FragmentMotion::default());
            }
            for _ in 0..500 {
                advance(
                    &mut tool,
                    frame(),
                    Duration::from_millis(16),
                    [-5.0, 2.0, 0.0],
                );
            }
            for (before, p) in initial.iter().zip(tool.session.fragments()) {
                assert_eq!(before.id(), p.id());
                assert_eq!(before.material(), p.material());
                let gap = tool.ship_fragments[&p.id()][1]
                    - SHIP_FLOOR_HEIGHT_METERS
                    - support(*p, [0.0, 1.0, 0.0]);
                assert!(gap.abs() < 0.003, "{resource:?} {:?}: gap {gap}", p.id());
                assert!(
                    length(tool.fragment_motion[&p.id()].velocity) < 0.05,
                    "{resource:?}: {:?}",
                    tool.fragment_motion[&p.id()]
                );
            }
        }
    }

    #[test]
    fn adapter_excludes_carried_and_distant_fragments() {
        let mut tool = mined(4.0);
        let ids: Vec<_> = tool.session.fragments().iter().map(|p| p.id()).collect();
        assert_eq!(ids.len(), 2);
        assert!(tool.session.pick_up(ids[0]));
        assert!(
            tool.session.move_loose(
                ids[1],
                ResourceTransform::new(WorldPosition::new(125.0, 0.0, 0.0), [0.0, 0.0, 0.0, 1.0])
                    .unwrap()
            )
        );
        let before = tool.session.fragments().to_vec();
        advance(&mut tool, frame(), Duration::from_millis(16), [0.0; 3]);
        assert_eq!(tool.session.fragments(), before);
        assert!(tool.fragment_motion.is_empty());
    }

    #[test]
    fn adapter_preserves_entity_contract_and_maps_rotated_ship_frame() {
        let mut tool = mined(2.0);
        let id = tool.session.fragments()[0].id();
        let frame = ShipFrame {
            origin_meters: [1000.0, 2000.0, 3000.0],
            axes: [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]],
        };
        let local = [-5.0, 1.3, 0.0];
        let world = frame.local_to_world(local);
        let pose = ResourceTransform::new(
            WorldPosition::new(world[0], world[1], world[2]),
            [0.0, 1.0, 0.0, 0.0],
        )
        .unwrap();
        assert!(tool.session.move_loose(id, pose));
        tool.ship_fragments.insert(id, local);
        tool.fragment_motion.insert(id, FragmentMotion::default());
        let before = tool.session.fragments()[0];
        release(&mut tool, id, frame.axes[0], frame, true);
        assert_eq!(tool.fragment_motion[&id].velocity, [1.1, 0.35, 0.0]);
        advance(&mut tool, frame, Duration::from_millis(100), world);
        let after = tool.session.fragments()[0];
        assert_eq!(after.id(), before.id());
        assert_eq!(after.source(), before.source());
        assert_eq!(after.material(), before.material());
        assert!(
            (after
                .transform()
                .orientation_xyzw()
                .iter()
                .map(|v| v * v)
                .sum::<f64>()
                - 1.0)
                .abs()
                < 1e-9
        );
        assert_eq!(
            after.transform().position().meters(),
            frame.local_to_world(tool.ship_fragments[&id])
        );
        assert_ne!(after.transform().position(), before.transform().position());
    }

    #[test]
    fn released_fragment_falls_from_hand_and_settles_on_deck() {
        let mut tool = mined(2.0);
        let id = tool.session.fragments()[0].id();
        tool.ship_fragments.insert(id, [-5.0, 1.3, 0.0]);
        tool.fragment_motion.insert(id, FragmentMotion::default());
        release(&mut tool, id, [1.0, 0.0, 0.0], frame(), true);
        advance(
            &mut tool,
            frame(),
            Duration::from_millis(100),
            [-5.0, 2.0, 0.0],
        );
        let falling = tool.ship_fragments[&id];
        assert!(falling[0] > -5.0 && falling[1] < 1.3);
        for _ in 0..180 {
            advance(
                &mut tool,
                frame(),
                Duration::from_millis(16),
                [-5.0, 2.0, 0.0],
            );
        }
        let settled = tool.ship_fragments[&id];
        let piece = tool.session.fragments()[0];
        let radius = support(piece, [0.0, 1.0, 0.0]);
        assert!((settled[1] - SHIP_FLOOR_HEIGHT_METERS - radius).abs() < 0.002);
    }

    #[test]
    fn two_fragments_contact_and_stack_on_the_deck() {
        let mut tool = mined(4.0);
        let pieces = tool.session.fragments().to_vec();
        for (i, piece) in pieces.iter().enumerate() {
            tool.ship_fragments
                .insert(piece.id(), [-5.0, 0.5 + i as f64 * 0.4, 0.0]);
            tool.fragment_motion
                .insert(piece.id(), FragmentMotion::default());
        }
        for _ in 0..150 {
            advance(
                &mut tool,
                frame(),
                Duration::from_millis(16),
                [-5.0, 2.0, 0.0],
            );
        }
        let a = tool.ship_fragments[&pieces[0].id()];
        let b = tool.ship_fragments[&pieces[1].id()];
        // Broad-phase spheres may overlap; visible authored hulls define the pile.
        assert!(a[1].min(b[1]) >= SHIP_FLOOR_HEIGHT_METERS);
        assert!(length(sub(a, b)) > 0.05);
        assert!(
            length(sub(a, b))
                < pieces
                    .iter()
                    .map(
                        |p| salimon_world::resource_size::fragment_contact_radius_meters(
                            p.material()
                        )
                    )
                    .sum::<f64>()
        );
    }

    #[test]
    fn freshly_mined_fragment_ejects_then_returns_to_surface() {
        let body = CELESTIAL_BODIES
            .iter()
            .find(|body| body.id == CelestialBodyId::Earth)
            .unwrap();
        let mut deposit = ResourceDeposit::new(
            DepositId {
                body: body.id,
                local: 9,
            },
            RawMaterial::new(ResourceId::IronOre, 2.0).unwrap(),
            body.center.translated([0.0, body.radius_meters, 0.0]),
            2.0,
        )
        .unwrap();
        let mut tool = MiningTool::default();
        tool.session.extract(&mut deposit, Duration::from_secs(1));
        let id = tool.session.fragments()[0].id();
        tool.fragment_motion.insert(
            id,
            FragmentMotion {
                velocity: salimon_physics::surface_ejection_velocity(
                    [0.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                    1,
                ),
                ..Default::default()
            },
        );
        let before = tool.session.fragments()[0].transform().position().meters();
        let player = deposit.position().translated([0.0, 1.7, 0.0]).meters();
        advance(&mut tool, frame(), Duration::from_millis(50), player);
        let popped = tool.session.fragments()[0].transform().position().meters();
        assert!(length(sub(popped, before)) > 0.03);
        assert!(popped[1] > before[1]);
        for _ in 0..120 {
            advance(&mut tool, frame(), Duration::from_millis(16), player);
        }
        let settled = tool.session.fragments()[0].transform().position().meters();
        let piece = tool.session.fragments()[0];
        let radius = support(piece, [0.0, 1.0, 0.0]);
        assert!(
            (length(sub(settled, body.center.meters())) - body.radius_meters - radius).abs()
                < 0.002
        );
    }

    #[test]
    fn missing_or_restored_motion_never_receives_an_implicit_ejection() {
        let mut tool = mined(0.5);
        let piece = tool.session.fragments()[0];
        let body = CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == CelestialBodyId::Earth)
            .unwrap();
        let position = body.center.translated([0.0, body.radius_meters + 3.0, 0.0]);
        tool.session.move_loose(
            piece.id(),
            ResourceTransform::new(position, [0.0, 0.0, 0.0, 1.0]).unwrap(),
        );
        advance(
            &mut tool,
            frame(),
            Duration::from_millis(16),
            position.meters(),
        );
        let velocity = tool.fragment_motion[&piece.id()].velocity;
        assert!(velocity[1] < 0.0);
        assert_eq!(velocity[0], 0.0);
        assert_eq!(velocity[2], 0.0);
        let before = tool.session.fragments()[0].transform();
        // Growing the same piece preserves its current pose and motion.
        let mut deposit = ResourceDeposit::new(
            DepositId {
                body: body.id,
                local: 1,
            },
            RawMaterial::new(ResourceId::SilicateRock, 1.0).unwrap(),
            WorldPosition::new(0.0, 0.0, 0.0),
            1.0,
        )
        .unwrap();
        tool.session
            .extract_with_spawn(&mut deposit, Duration::from_millis(16), |_, _, _| {
                panic!("growth cannot spawn")
            });
        assert_eq!(tool.session.fragments()[0].transform(), before);
        assert_eq!(tool.fragment_motion[&piece.id()].velocity, velocity);
    }
}
