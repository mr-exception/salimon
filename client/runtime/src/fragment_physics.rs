//! Fragment/session adapter for the portable physical-object simulation.
//! Selects geometry and frames; physical rules live in salimon-physics.
use std::time::{Duration, Instant};

use salimon_math::{dot, length, sub};

use crate::mining::ResourceGameplay;
#[cfg(test)]
use salimon_character::SHIP_FLOOR_HEIGHT_METERS;
use salimon_character::ShipFrame;
use salimon_world::resources::{FragmentId, ResourceTransform};
use salimon_world::{BodyRole, CELESTIAL_BODIES, WorldPosition};

use salimon_physics::{ObjectState, SphereSurface, Surface};

#[cfg(test)]
mod geometry_evaluation;

#[derive(Default)]
pub(crate) struct FragmentScratch {
    snapshots: Vec<(salimon_world::resources::ResourceFragment, ObjectState)>,
    states: Vec<ObjectState>,
    ids: Vec<FragmentId>,
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FragmentMotion {
    pub(crate) velocity: [f64; 3],
    pub(crate) angular_velocity: [f64; 3],
    pub(crate) ship_orientation: Option<[f64; 4]>,
}

pub(crate) fn release(
    tool: &mut ResourceGameplay,
    id: FragmentId,
    player_forward: [f64; 3],
    frame: ShipFrame,
    inside: bool,
) {
    tool.fragment_previous.remove(&id);
    let velocity = if inside {
        salimon_physics::floor_release_velocity([
            dot(player_forward, frame.axes[0]),
            0.0,
            dot(player_forward, frame.axes[2]),
        ])
    } else {
        let position = tool
            .session
            .fragment(id)
            .expect("released fragment exists")
            .transform()
            .position()
            .meters();
        salimon_physics::release_velocity(player_forward, position, nearest_body(position))
    };
    let world_orientation = tool
        .session
        .fragment(id)
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

/// Save presentation history before a native gameplay tick, in each object's
/// simulation frame. New/growing/transferred/released objects bypass history.
pub(crate) fn capture_previous(tool: &mut ResourceGameplay, frame: ShipFrame, player: [f64; 3]) {
    let mut scratch = std::mem::take(&mut tool.fragment_scratch);
    snapshots_into(tool, frame, player, &mut scratch.snapshots);
    tool.fragment_previous.clear();
    tool.fragment_previous.extend(
        scratch
            .snapshots
            .drain(..)
            .map(|(fragment, state)| (fragment.id(), state)),
    );
    tool.fragment_scratch = scratch;
}

pub(crate) fn interpolated_mesh(
    tool: &ResourceGameplay,
    fragment: salimon_world::resources::ResourceFragment,
    frame: ShipFrame,
    alpha: f64,
) -> salimon_renderer::ResourceMeshInstance {
    let mut mesh = crate::resource_presentation::fragment_mesh(fragment);
    let Some(previous) = tool.fragment_previous.get(&fragment.id()) else {
        return mesh;
    };
    let local = tool.ship_fragments.get(&fragment.id());
    if alpha >= 1.0
        || Some(fragment.id()) == tool.session.carried_id()
        || matches!(
            previous.surface,
            Surface::Floor { .. } | Surface::UnpoweredFloor { .. }
        ) != local.is_some()
        || previous.radius != sphere_radius(fragment)
        || previous.mass_kg != fragment.material().mass_kg()
    {
        return mesh;
    }
    let current = local
        .copied()
        .map(salimon_character::ShipLocalPoint::meters)
        .unwrap_or(mesh.center_meters);
    let position =
        std::array::from_fn(|i| previous.position[i] + (current[i] - previous.position[i]) * alpha);
    let current_q = if local.is_some() {
        tool.fragment_motion
            .get(&fragment.id())
            .and_then(|m| m.ship_orientation)
            .unwrap_or_else(|| to_local_orientation(frame, mesh.orientation_xyzw))
    } else {
        mesh.orientation_xyzw
    };
    let sign = if previous
        .orientation
        .iter()
        .zip(current_q)
        .map(|(a, b)| a * b)
        .sum::<f64>()
        < 0.0
    {
        -1.0
    } else {
        1.0
    };
    let q: [f64; 4] = std::array::from_fn(|i| {
        previous.orientation[i] + (sign * current_q[i] - previous.orientation[i]) * alpha
    });
    let norm = q.iter().map(|v| v * v).sum::<f64>().sqrt();
    let q = q.map(|v| v / norm);
    mesh.center_meters = if local.is_some() {
        frame
            .unproject_local(salimon_character::ShipLocalPoint::new(position))
            .meters()
    } else {
        position
    };
    mesh.orientation_xyzw = if local.is_some() {
        to_world_orientation(frame, q)
    } else {
        q
    };
    mesh
}

pub(crate) fn advance(
    tool: &mut ResourceGameplay,
    frame: ShipFrame,
    delta: Duration,
    player: [f64; 3],
) {
    advance_measured(tool, frame, delta, player, false);
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Measurements {
    pub(crate) solver: salimon_physics::StepStats,
    pub(crate) adapter_time: Duration,
    pub(crate) total_time: Duration,
    /// Velocity threshold observation, separate from persistent activation.
    pub(crate) moving_objects: usize,
}

impl Measurements {
    /// Keep population/activation observations on renders without a game tick,
    /// but never count the previous tick's work again.
    pub(crate) fn clear_work(&mut self) {
        let old = self.solver;
        *self = Self {
            solver: salimon_physics::StepStats {
                objects: old.objects,
                awake_objects: old.awake_objects,
                sleeping_objects: old.sleeping_objects,
                quiet_objects: old.quiet_objects,
                ..Default::default()
            },
            moving_objects: self.moving_objects,
            ..Default::default()
        };
    }

    /// Work totals cover every tick in the rendered frame; state counts observe
    /// the final tick. Native catch-up must not hide its first solver call.
    pub(crate) fn accumulate(&mut self, next: Self) {
        let old = self.solver;
        self.solver = next.solver;
        macro_rules! sum {
            ($($field:ident),*) => { $(self.solver.$field += old.$field;)* };
        }
        sum!(
            integrated_objects,
            substeps,
            solver_passes,
            pair_visits,
            radius_candidates,
            narrow_phase_tests,
            contacts,
            projection_builds,
            projection_storage_bytes,
            matrix_allocations,
            activation_time,
            integration_time,
            contact_time,
            grid_query_time,
            grid_queries
        );
        self.adapter_time += next.adapter_time;
        self.total_time += next.total_time;
        self.moving_objects = next.moving_objects;
    }
}

pub(crate) fn advance_measured(
    tool: &mut ResourceGameplay,
    frame: ShipFrame,
    delta: Duration,
    player: [f64; 3],
    profile: bool,
) -> Measurements {
    let started = profile.then(Instant::now);
    let mut measurements = Measurements::default();
    let elapsed = delta.as_secs_f64();
    if elapsed <= 0.0 {
        return measurements;
    }
    let mut scratch = std::mem::take(&mut tool.fragment_scratch);
    snapshots_into(tool, frame, player, &mut scratch.snapshots);
    if scratch.snapshots.is_empty() {
        tool.fragment_scratch = scratch;
        tool.fragment_sleep = salimon_physics::SleepTracker::default();
        if let Some(start) = started {
            measurements.total_time = start.elapsed();
            measurements.adapter_time = measurements.total_time;
        }
        return measurements;
    }
    scratch.states.clear();
    scratch.ids.clear();
    for (fragment, state) in scratch.snapshots.drain(..) {
        scratch.ids.push(fragment.id());
        scratch.states.push(state);
    }
    let states = &mut scratch.states;
    let ids = &scratch.ids;
    let solver_start = profile.then(Instant::now);
    let contains = |position, radius| {
        (salimon_character::cargo::cargo_position_allowed(
            position,
            radius,
            if tool.cargo_door_blocking { 0. } else { 1. },
        ) || tool.modular_interior.allows_sphere(position, radius))
            && tool.mount_geometry.mounts.iter().all(|m| {
                let p = m.position();
                !salimon_character::cargo::sphere_overlaps_box(
                    position,
                    (radius - 0.00001).max(0.),
                    [
                        p[0] - 0.5,
                        p[0] + 0.5,
                        p[1] - 0.03,
                        p[1] + 0.03,
                        p[2] - 0.175,
                        p[2] + 0.175,
                    ],
                )
            })
            && tool.interior_solids.iter().all(|b| {
                !salimon_character::cargo::sphere_overlaps_box(
                    position,
                    (radius - 0.00001).max(0.),
                    *b,
                )
            })
            && tool.secured.iter().all(|id| {
                tool.session.fragment(*id).is_none_or(|f| {
                    let p = tool
                        .ship_fragments
                        .get(id)
                        .map(|p| p.meters())
                        .unwrap_or_else(|| frame.world_to_local(f.transform().position().meters()));
                    length(sub(position, p)) >= radius + sphere_radius(*f)
                })
            })
    };
    measurements.solver = salimon_physics::advance_with_sleep(
        &mut tool.fragment_sleep,
        ids,
        states,
        delta,
        contains,
        profile,
    );
    if profile {
        measurements.moving_objects = states
            .iter()
            .filter(|p| length(p.velocity) >= 0.015 || length(p.angular_velocity) >= 0.015)
            .count();
    }
    let solver_time = solver_start
        .map(|start| start.elapsed())
        .unwrap_or_default();
    for (&id, piece) in ids.iter().zip(states.drain(..)) {
        tool.fragment_motion.insert(
            id,
            FragmentMotion {
                velocity: piece.velocity,
                angular_velocity: piece.angular_velocity,
                ship_orientation: matches!(
                    piece.surface,
                    Surface::Floor { .. } | Surface::UnpoweredFloor { .. }
                )
                .then_some(piece.orientation),
            },
        );
        let position = if matches!(
            piece.surface,
            Surface::Floor { .. } | Surface::UnpoweredFloor { .. }
        ) {
            let local = salimon_character::ShipLocalPoint::new(piece.position);
            tool.ship_fragments.insert(id, local);
            frame.unproject_local(local).meters()
        } else {
            piece.position
        };
        let pose = ResourceTransform::new(
            WorldPosition::new(position[0], position[1], position[2]),
            if matches!(
                piece.surface,
                Surface::Floor { .. } | Surface::UnpoweredFloor { .. }
            ) {
                to_world_orientation(frame, piece.orientation)
            } else {
                piece.orientation
            },
        )
        .expect("finite fragment physics pose");
        tool.session.move_loose(id, pose);
    }
    tool.fragment_scratch = scratch;
    if let Some(start) = started {
        measurements.total_time = start.elapsed();
        measurements.adapter_time = measurements.total_time.saturating_sub(solver_time);
    }
    measurements
}

/// Canonical production adapter input; also used for identical-snapshot solver replay.
pub(crate) fn snapshots(
    tool: &ResourceGameplay,
    frame: ShipFrame,
    player: [f64; 3],
) -> Vec<(salimon_world::resources::ResourceFragment, ObjectState)> {
    let mut output = Vec::new();
    snapshots_into(tool, frame, player, &mut output);
    output
}

fn snapshots_into(
    tool: &ResourceGameplay,
    frame: ShipFrame,
    player: [f64; 3],
    output: &mut Vec<(salimon_world::resources::ResourceFragment, ObjectState)>,
) {
    output.clear();
    let carried = tool.session.carried_id();
    output.extend(
        tool.session
            .fragments()
            .iter()
            .copied()
            .filter(|p| {
                Some(p.id()) != carried
                    && Some(p.id()) != tool.session.attached_sphere_id()
                    && !tool.secured.contains(&p.id())
            })
            .filter(|p| {
                // Ship advancement precedes final session-pose synchronization.
                // Select anchored contents in the current frame, not last frame's
                // world pose (which may be hundreds of metres behind in flight).
                let position = tool.ship_fragments.get(&p.id()).map_or_else(
                    || p.transform().position().meters(),
                    |local| frame.unproject_local(*local).meters(),
                );
                length(sub(position, player)) < 125.0
            })
            .map(|p| {
                let ship = tool.ship_fragments.contains_key(&p.id());
                let position = if ship {
                    tool.ship_fragments[&p.id()].meters()
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
                        radius: sphere_radius(p),
                        ground_support_meters: sphere_radius(p),
                        hull: None,
                        authored_mesh_scale: salimon_world::resource_fragments::diameter_meters(p),
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
                            if tool.ship_gravity_disabled {
                                Surface::UnpoweredFloor {
                                    height_meters: fixture_support_height(
                                        tool,
                                        position,
                                        sphere_radius(p),
                                    ),
                                    ceiling_meters: tool
                                        .modular_interior
                                        .volumes
                                        .iter()
                                        .filter(|b| {
                                            position[0] >= b[0]
                                                && position[0] <= b[1]
                                                && position[2] >= b[4]
                                                && position[2] <= b[5]
                                                && position[1] >= b[2]
                                                && position[1] <= b[3]
                                        })
                                        .map(|b| b[3] - 0.1)
                                        .max_by(f64::total_cmp)
                                        .unwrap_or(salimon_character::SHIP_CEILING_HEIGHT_METERS),
                                }
                            } else {
                                Surface::Floor {
                                    height_meters: fixture_support_height(
                                        tool,
                                        position,
                                        sphere_radius(p),
                                    ),
                                }
                            }
                        } else {
                            body.map(|body| {
                                if tool.terrain
                                    == salimon_world::terrain::TerrainMode::Representative
                                {
                                    Surface::Sampled {
                                        body,
                                        query: crate::terrain::physics_ground,
                                        point_query: crate::terrain::physics_ground_point,
                                    }
                                } else {
                                    Surface::Sphere(body)
                                }
                            })
                            .unwrap_or(Surface::Unsupported)
                        },
                    },
                )
            }),
    );
}

pub(crate) fn to_world_orientation(frame: ShipFrame, local: [f64; 4]) -> [f64; 4] {
    salimon_physics::compose(salimon_physics::orientation_from_axes(frame.axes), local)
}
pub(crate) fn to_local_orientation(frame: ShipFrame, world: [f64; 4]) -> [f64; 4] {
    let q = salimon_physics::orientation_from_axes(frame.axes);
    salimon_physics::compose([-q[0], -q[1], -q[2], q[3]], world)
}

pub(crate) fn sphere_radius(fragment: salimon_world::resources::ResourceFragment) -> f64 {
    salimon_world::spherical_mining::GrowingSphere {
        source: fragment.source(),
        material: fragment.material(),
    }
    .radius_meters()
}

pub(crate) fn support(fragment: salimon_world::resources::ResourceFragment, _up: [f64; 3]) -> f64 {
    sphere_radius(fragment)
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

fn fixture_support_height(tool: &ResourceGameplay, position: [f64; 3], radius: f64) -> f64 {
    let mut height = if tool.modular_interior.contains(position)
        && !salimon_character::ship_contains_object(position)
    {
        tool.modular_interior
            .support(position, radius, position[1] - radius + 0.001)
            .unwrap_or_else(|| {
                tool.modular_interior
                    .volumes
                    .iter()
                    .map(|b| b[2])
                    .min_by(f64::total_cmp)
                    .unwrap_or(salimon_character::SHIP_FLOOR_HEIGHT_METERS)
            })
    } else {
        salimon_character::cargo::cargo_support_height(position, radius)
    };
    for m in &tool.mount_geometry.mounts {
        let p = m.position();
        if (position[0] - p[0]).abs() + radius <= 0.5
            && (position[2] - p[2]).abs() + radius <= 0.175
            && position[1] >= p[1] + 0.03 + radius - 0.001
        {
            height = height.max(p[1] + 0.03);
        }
    }
    for b in &tool.interior_solids {
        if position[0] - radius >= b[0]
            && position[0] + radius <= b[1]
            && position[2] - radius >= b[4]
            && position[2] + radius <= b[5]
            && position[1] >= b[3] + radius - 0.001
        {
            height = height.max(b[3]);
        }
    }
    height = height.max(
        tool.modular_interior
            .support(position, radius, position[1] - radius + 0.001)
            .unwrap_or(height),
    );
    height
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
    // Preserve these historical adapter populations (4 kg means two 2 kg pieces),
    // but construct/validate each piece through current spherical extraction.
    fn mined(mass: f64) -> ResourceGameplay {
        let mut tool = ResourceGameplay::default();
        let mut remaining = mass;
        while remaining > 0.0 {
            let piece_mass = remaining.min(2.0);
            crate::sphere_fixtures::sphere(
                &mut tool.session,
                ResourceId::SilicateRock,
                piece_mass,
                ResourceTransform::new(WorldPosition::new(0.0, 0.0, 0.0), [0.0, 0.0, 0.0, 1.0])
                    .unwrap(),
            );
            remaining -= piece_mass;
        }
        assert_eq!(tool.session.fragments().len(), (mass / 2.0).ceil() as usize);
        tool
    }

    #[test]
    fn settled_tick_avoids_journal_and_mass_work_but_frame_changes_transfer() {
        let mut tool = ResourceGameplay::default();
        let frame = frame();
        let pose = ResourceTransform::new(
            WorldPosition::new(-5.0, SHIP_FLOOR_HEIGHT_METERS + 0.088027, 0.0),
            [0.0, 0.0, 0.0, 1.0],
        )
        .unwrap();
        let id = crate::sphere_fixtures::sphere(&mut tool.session, ResourceId::IronOre, 10.0, pose);
        let radius = sphere_radius(tool.session.fragments()[0]);
        tool.ship_fragments.insert(
            id,
            salimon_character::ShipLocalPoint::new([-5.0, SHIP_FLOOR_HEIGHT_METERS + radius, 0.0]),
        );
        let mut ship = salimon_ship::ShipController::flying(
            salimon_ship::ShipPose {
                position_meters: frame.origin_meters,
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
        );
        for _ in 0..90 {
            advance(
                &mut tool,
                frame,
                Duration::from_millis(16),
                [-5.0, 2.0, 0.0],
            );
            crate::carrying::sync_ship_mass(&mut tool, frame, &mut ship);
        }
        assert!(tool.fragment_sleep.is_sleeping(id));
        let capacities = (
            tool.fragment_scratch.snapshots.capacity(),
            tool.fragment_scratch.states.capacity(),
            tool.fragment_scratch.ids.capacity(),
        );
        capture_previous(&mut tool, frame, [-5.0, 2.0, 0.0]);
        advance(
            &mut tool,
            frame,
            Duration::from_millis(16),
            [-5.0, 2.0, 0.0],
        );
        crate::carrying::sync_ship_fragments(&mut tool, frame);
        assert_eq!(
            crate::carrying::sync_ship_mass(&mut tool, frame, &mut ship),
            0
        );
        assert_eq!(
            capacities,
            (
                tool.fragment_scratch.snapshots.capacity(),
                tool.fragment_scratch.states.capacity(),
                tool.fragment_scratch.ids.capacity()
            )
        );
        assert_eq!(ship.snapshot().total_mass_kg, 100_010.0);
        let moved = ShipFrame {
            origin_meters: [100.0, 0.0, 0.0],
            ..frame
        };
        crate::carrying::sync_ship_fragments(&mut tool, moved);
        assert!(crate::carrying::sync_ship_mass(&mut tool, moved, &mut ship) > 0);
        assert_eq!(ship.snapshot().total_mass_kg, 100_010.0);
        assert!(
            tool.session
                .fragment(id)
                .unwrap()
                .transform()
                .position()
                .meters()[0]
                > 90.0
        );
        assert!(tool.fragment_sleep.is_sleeping(id));
        // A stationary unanchored object crosses a MOVING containment boundary
        // without a pose event. Its cached mass must still be removed.
        tool.ship_fragments.remove(&id);
        let away = ShipFrame {
            origin_meters: [1000.0, 0.0, 0.0],
            ..frame
        };
        assert!(crate::carrying::sync_ship_mass(&mut tool, away, &mut ship) > 0);
        assert_eq!(ship.snapshot().total_mass_kg, 100_000.0);
        assert!(tool.session.remove_fragment(id));
        assert_eq!(
            crate::carrying::sync_ship_mass(&mut tool, away, &mut ship),
            1
        );
    }

    #[test]
    fn loose_snapshot_radius_matches_visual_and_attached_mass_has_no_body() {
        let mut tool = mined(2.0);
        let p = tool.session.fragments()[0];
        let states = snapshots(&tool, frame(), p.transform().position().meters());
        assert_eq!(states.len(), 1);
        let state = &states[0].1;
        let visual = crate::resource_presentation::fragment_mesh(p);
        assert!(state.hull.is_none());
        assert_eq!(
            state.radius,
            visual.authored_mesh_scale * salimon_renderer::SPHERE_AUTHORED_RADIUS_METERS
        );
        assert_eq!(state.ground_support_meters, state.radius);
        assert_eq!(state.mass_kg, p.material().mass_kg());
        let mut source = ResourceDeposit::new(
            DepositId {
                local: 999,
                ..p.source()
            },
            p.material(),
            p.transform().position(),
            2.0,
        )
        .unwrap();
        tool.session.tick_spherical(
            true,
            Some(&mut source),
            Duration::from_millis(16),
            Default::default(),
            p.transform(),
        );
        let attached = tool.session.attached_sphere_id().unwrap();
        assert!(
            snapshots(&tool, frame(), p.transform().position().meters())
                .iter()
                .all(|(p, _)| p.id() != attached)
        );
    }

    #[test]
    fn interpolation_is_local_to_ship_and_release_has_no_history_lag() {
        let mut tool = mined(2.0);
        let id = tool.session.fragments()[0].id();
        tool.ship_fragments
            .insert(id, salimon_character::ShipLocalPoint::new([-5.0, 1.3, 0.0]));
        capture_previous(&mut tool, frame(), [-5.0, 2.0, 0.0]);
        let moving = ShipFrame {
            origin_meters: [1e12, 2e12, -3e12],
            ..frame()
        };
        advance(
            &mut tool,
            moving,
            Duration::from_millis(16),
            moving.local_to_world([-5.0, 2.0, 0.0]),
        );
        let fragment = tool.session.fragments()[0];
        let start = interpolated_mesh(&tool, fragment, moving, 0.0);
        assert_eq!(start.center_meters, moving.local_to_world([-5.0, 1.3, 0.0]));
        let midpoint = interpolated_mesh(&tool, fragment, moving, 0.5);
        let local = tool.ship_fragments[&id];
        assert_eq!(
            midpoint.center_meters,
            moving.local_to_world([-5.0, (1.3 + local.meters()[1]) * 0.5, 0.0])
        );
        assert!((midpoint.orientation_xyzw.iter().map(|v| v * v).sum::<f64>() - 1.0).abs() < 1e-12);
        let authoritative = crate::resource_presentation::fragment_mesh(fragment);
        assert_eq!(
            interpolated_mesh(&tool, fragment, moving, 1.0).center_meters,
            authoritative.center_meters
        );
        release(&mut tool, id, moving.axes[0], moving, true);
        assert!(!tool.fragment_previous.contains_key(&id));
        assert_eq!(
            interpolated_mesh(&tool, fragment, moving, 0.0).center_meters,
            authoritative.center_meters
        );
    }

    #[test]
    fn changed_size_or_frame_and_carried_objects_bypass_interpolation() {
        let mut tool = mined(2.0);
        let fragment = tool.session.fragments()[0];
        let id = fragment.id();
        tool.ship_fragments
            .insert(id, salimon_character::ShipLocalPoint::new([-5.0, 1.3, 0.0]));
        capture_previous(&mut tool, frame(), [-5.0, 2.0, 0.0]);
        let expected = crate::resource_presentation::fragment_mesh(fragment).center_meters;
        tool.fragment_previous.get_mut(&id).unwrap().radius *= 0.5;
        assert_eq!(
            interpolated_mesh(&tool, fragment, frame(), 0.0).center_meters,
            expected
        );
        capture_previous(&mut tool, frame(), [-5.0, 2.0, 0.0]);
        tool.ship_fragments.remove(&id);
        assert_eq!(
            interpolated_mesh(&tool, fragment, frame(), 0.0).center_meters,
            expected
        );
        tool.ship_fragments
            .insert(id, salimon_character::ShipLocalPoint::new([-5.0, 1.3, 0.0]));
        assert!(tool.session.pick_up(id));
        assert_eq!(
            interpolated_mesh(&tool, fragment, frame(), 0.0).center_meters,
            expected
        );
    }

    #[test]
    fn measured_frame_sums_work_and_keeps_latest_activation() {
        let mut frame = Measurements::default();
        for sleeping in [10, 11] {
            frame.accumulate(Measurements {
                solver: salimon_physics::StepStats {
                    substeps: 2,
                    integrated_objects: 3,
                    sleeping_objects: sleeping,
                    ..Default::default()
                },
                total_time: Duration::from_millis(5),
                ..Default::default()
            });
        }
        assert_eq!(frame.solver.substeps, 4);
        assert_eq!(frame.solver.integrated_objects, 6);
        assert_eq!(frame.solver.sleeping_objects, 11);
        assert_eq!(frame.total_time, Duration::from_millis(10));
        frame.clear_work();
        assert_eq!(frame.solver.substeps, 0);
        assert_eq!(frame.total_time, Duration::ZERO);
        assert_eq!(frame.solver.sleeping_objects, 11);
    }

    #[test]
    fn all_resource_spheres_settle_rotated_at_current_radius_and_preserve_mass() {
        for resource in [
            ResourceId::IronOre,
            ResourceId::SilicateRock,
            ResourceId::WaterIce,
        ] {
            let mut tool = ResourceGameplay::default();
            for _ in 0..2 {
                crate::sphere_fixtures::sphere(
                    &mut tool.session,
                    resource,
                    10.0,
                    ResourceTransform::new(WorldPosition::new(0.0, 0.0, 0.0), [0.0, 0.0, 0.0, 1.0])
                        .unwrap(),
                );
            }
            assert_eq!(tool.session.fragments().len(), 2);
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
                tool.ship_fragments
                    .insert(p.id(), salimon_character::ShipLocalPoint::new(local));
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
            let stats = advance_measured(
                &mut tool,
                frame(),
                Duration::from_millis(16),
                [-5.0, 2.0, 0.0],
                true,
            );
            assert_eq!(stats.solver.sleeping_objects, 2, "{resource:?}");
            for (before, p) in initial.iter().zip(tool.session.fragments()) {
                assert_eq!(before.id(), p.id());
                assert_eq!(before.material(), p.material());
                let gap = tool.ship_fragments[&p.id()].meters()[1]
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
    fn sleeping_ship_fragment_follows_rigid_motion_and_streaming_return_wakes_safely() {
        let mut tool = mined(2.0);
        let original = tool.session.fragments()[0];
        let id = original.id();
        tool.ship_fragments
            .insert(id, salimon_character::ShipLocalPoint::new([-5.0, 1.3, 0.0]));
        tool.fragment_motion.insert(id, FragmentMotion::default());
        for _ in 0..500 {
            advance(
                &mut tool,
                frame(),
                Duration::from_millis(16),
                [-5.0, 2.0, 0.0],
            );
        }
        let stats = advance_measured(
            &mut tool,
            frame(),
            Duration::from_millis(16),
            [-5.0, 2.0, 0.0],
            true,
        );
        assert_eq!(
            stats.solver.sleeping_objects, 1,
            "{:?}",
            tool.fragment_sleep
        );
        let local = tool.ship_fragments[&id];
        let moving = ShipFrame {
            origin_meters: [1.0e12, 2.0e12, -3.0e12],
            axes: [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]],
        };
        let stats = advance_measured(
            &mut tool,
            moving,
            Duration::from_millis(16),
            moving.local_to_world([-5.0, 2.0, 0.0]),
            true,
        );
        assert_eq!(stats.solver.sleeping_objects, 1);
        assert_eq!(stats.solver.integrated_objects, 0);
        assert_eq!(tool.ship_fragments[&id], local);
        assert_eq!(
            tool.session.fragments()[0].transform().position().meters(),
            moving.unproject_local(local).meters()
        );
        // Leaving the selected set invalidates contacts; returning retains identity/mass.
        advance(&mut tool, moving, Duration::from_millis(16), [0.0; 3]);
        let stats = advance_measured(
            &mut tool,
            moving,
            Duration::from_millis(16),
            moving.local_to_world([-5.0, 2.0, 0.0]),
            true,
        );
        assert!(stats.solver.integrated_objects > 0);
        assert_eq!(tool.session.fragments()[0].id(), id);
        assert_eq!(tool.session.fragments()[0].material(), original.material());
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
        tool.ship_fragments
            .insert(id, salimon_character::ShipLocalPoint::new(local));
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
            frame.unproject_local(tool.ship_fragments[&id]).meters()
        );
        assert_ne!(after.transform().position(), before.transform().position());
    }

    #[test]
    fn released_fragment_falls_from_hand_and_settles_on_deck() {
        let mut tool = mined(2.0);
        let id = tool.session.fragments()[0].id();
        tool.ship_fragments
            .insert(id, salimon_character::ShipLocalPoint::new([-5.0, 1.3, 0.0]));
        tool.fragment_motion.insert(id, FragmentMotion::default());
        release(&mut tool, id, [1.0, 0.0, 0.0], frame(), true);
        advance(
            &mut tool,
            frame(),
            Duration::from_millis(100),
            [-5.0, 2.0, 0.0],
        );
        let falling = tool.ship_fragments[&id].meters();
        assert!(falling[0] > -5.0 && falling[1] < 1.3);
        for _ in 0..180 {
            advance(
                &mut tool,
                frame(),
                Duration::from_millis(16),
                [-5.0, 2.0, 0.0],
            );
        }
        let settled = tool.ship_fragments[&id].meters();
        let piece = tool.session.fragments()[0];
        let radius = support(piece, [0.0, 1.0, 0.0]);
        assert!((settled[1] - SHIP_FLOOR_HEIGHT_METERS - radius).abs() < 0.002);
    }

    #[test]
    fn two_fragments_contact_and_stack_on_the_deck() {
        let mut tool = mined(4.0);
        let pieces = tool.session.fragments().to_vec();
        for (i, piece) in pieces.iter().enumerate() {
            tool.ship_fragments.insert(
                piece.id(),
                salimon_character::ShipLocalPoint::new([-5.0, 0.5 + i as f64 * 0.4, 0.0]),
            );
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
        let a = tool.ship_fragments[&pieces[0].id()].meters();
        let b = tool.ship_fragments[&pieces[1].id()].meters();
        // Exact spherical contacts retain the sum of radii within solver tolerance.
        assert!(a[1].min(b[1]) >= SHIP_FLOOR_HEIGHT_METERS);
        assert!(length(sub(a, b)) > 0.05);
        assert!(length(sub(a, b)) + 0.002 >= pieces.iter().map(|p| sphere_radius(*p)).sum::<f64>());
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
        let mut tool = ResourceGameplay::default();
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
    fn legacy_tail_growth_and_restored_motion_never_receive_an_implicit_ejection() {
        // This regression deliberately exercises the retained legacy unsealed-tail
        // growth API. Released current spheres never reconnect to a later hold.
        let mut tool = ResourceGameplay::default();
        let mut source = ResourceDeposit::new(
            DepositId {
                body: CelestialBodyId::Earth,
                local: 1,
            },
            RawMaterial::new(ResourceId::SilicateRock, 1.0).unwrap(),
            WorldPosition::new(0.0, 0.0, 0.0),
            1.0,
        )
        .unwrap();
        tool.session
            .extract(&mut source, Duration::from_millis(250));
        assert_eq!(tool.session.fragments().len(), 1);
        crate::sphere_fixtures::assert_sphere(
            tool.session.fragments()[0],
            ResourceId::SilicateRock,
            0.5,
        );
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
