//! Portable small-object motion/contact in caller-supplied metre reference frames.
use salimon_math::{add, cross, dot, length, scale, sub};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

mod broad_phase;
mod convex;
mod sleep;
pub use convex::{ConvexHull, ConvexHullComplexity, compose, orientation_from_axes, rotate};
pub use sleep::SleepTracker;

const GRAVITY: f64 = 9.81;
const RESTITUTION: f64 = 0.12;
const ROLLING_RESISTANCE: f64 = 0.1;
const MAX_STEP: f64 = 1.0 / 90.0;
const CONTACT_BIAS_METERS: f64 = 0.00001;

/// Spherical ground geometry in the same frame as its objects.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SphereSurface {
    pub center: [f64; 3],
    pub radius: f64,
}

/// Environmental support; floor objects share one ship-local frame per call.
#[derive(Clone, Copy, Debug)]
pub enum Surface {
    Floor {
        height_meters: f64,
    },
    /// Same physical deck/contact/frame, without artificial gravity.
    UnpoweredFloor {
        height_meters: f64,
        ceiling_meters: f64,
    },
    Sphere(SphereSurface),
    /// Pure caller-owned heightfield query; gravity remains radial to body.center.
    Sampled {
        body: SphereSurface,
        query: GroundQuery,
        /// Same authoritative point as query, without computing its normal stencil.
        point_query: GroundPointQuery,
    },
    /// Legacy unsupported objects still accelerate along -Y, without ground contact.
    Unsupported,
}
impl Surface {
    fn is_floor(self) -> bool {
        matches!(self, Self::Floor { .. } | Self::UnpoweredFloor { .. })
    }
    fn gravity(self) -> f64 {
        if matches!(self, Self::UnpoweredFloor { .. }) {
            0.0
        } else {
            GRAVITY
        }
    }

    fn sphere(self) -> Option<SphereSurface> {
        match self {
            Self::Sphere(body) | Self::Sampled { body, .. } => Some(body),
            _ => None,
        }
    }
}

/// Absolute environmental point and outward unit normal; query is allocation-free.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundSample {
    pub point: [f64; 3],
    pub normal: [f64; 3],
}
pub type GroundQuery = fn(SphereSurface, [f64; 3]) -> GroundSample;
pub type GroundPointQuery = fn(SphereSurface, [f64; 3]) -> [f64; 3];
impl PartialEq for Surface {
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            (Self::Floor { height_meters: a }, Self::Floor { height_meters: b }) => a == b,
            (
                Self::UnpoweredFloor {
                    height_meters: a,
                    ceiling_meters: ca,
                },
                Self::UnpoweredFloor {
                    height_meters: b,
                    ceiling_meters: cb,
                },
            ) => a == b && ca == cb,
            (Self::Sphere(a), Self::Sphere(b)) => a == b,
            (
                Self::Sampled {
                    body: a,
                    query: qa,
                    point_query: pa,
                },
                Self::Sampled {
                    body: b,
                    query: qb,
                    point_query: pb,
                },
            ) => a == b && std::ptr::fn_addr_eq(qa, qb) && std::ptr::fn_addr_eq(pa, pb),
            (Self::Unsupported, Self::Unsupported) => true,
            _ => false,
        }
    }
}
impl Surface {
    pub(crate) fn ground(self, position: [f64; 3]) -> Option<GroundSample> {
        match self {
            Self::Floor { height_meters } | Self::UnpoweredFloor { height_meters, .. } => {
                Some(GroundSample {
                    point: [position[0], height_meters, position[2]],
                    normal: [0.0, 1.0, 0.0],
                })
            }
            Self::Sphere(body) => {
                let normal = normalize(sub(position, body.center));
                Some(GroundSample {
                    point: add(body.center, scale(normal, body.radius)),
                    normal,
                })
            }
            Self::Sampled { body, query, .. } => Some(query(body, position)),
            Self::Unsupported => None,
        }
    }
}

/// Caller-validated finite pose, velocity (m/s) and positive spherical radius (m).
/// Identity/material stay with the caller; physics updates position/orientation.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectState {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub radius: f64,
    /// Authored support distance along the environmental normal, in metres.
    pub ground_support_meters: f64,
    pub surface: Surface,
    /// Immutable unit-scale authored convex envelope. None retains legacy sphere policy.
    pub hull: Option<Arc<ConvexHull>>,
    pub side_meters: f64,
    /// Unit local-to-simulation-frame quaternion; angular velocity is frame radians/s.
    pub orientation: [f64; 4],
    pub angular_velocity: [f64; 3],
    pub mass_kg: f64,
}

/// Surface release impulse. `forward` is a unit direction in the object's frame.
pub fn release_velocity(
    forward: [f64; 3],
    position: [f64; 3],
    body: Option<SphereSurface>,
) -> [f64; 3] {
    let up = body
        .map(|body| normalize(sub(position, body.center)))
        .unwrap_or([0.0, 1.0, 0.0]);
    add(scale(forward, 1.1), scale(up, 0.3))
}

/// Floor release impulse; caller projects forward onto local X and Z axes.
pub fn floor_release_velocity(local_forward: [f64; 3]) -> [f64; 3] {
    [local_forward[0] * 1.1, 0.35, local_forward[2] * 1.1]
}

/// One-shot emission impulse from an exposed facet, with bounded stable speed
/// variation and radial lift. Caller supplies unit world normal and gravity up.
pub fn surface_ejection_velocity(normal: [f64; 3], up: [f64; 3], variant: u8) -> [f64; 3] {
    let planar = sub(normal, scale(up, dot(normal, up)));
    let tangent = if length(planar) > 1e-6 {
        normalize(planar)
    } else {
        let axis = if up[1].abs() < 0.9 {
            [0.0, 1.0, 0.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        normalize(cross(up, axis))
    };
    // Top facets need lateral escape too, rather than falling straight back into
    // the source. Stable small fan variation changes motion, never spawn positions.
    let scatter = scale(cross(up, tangent), (f64::from(variant % 5) - 2.0) * 0.2);
    add(
        add(
            scale(normal, 2.0 + f64::from(variant % 5) * 0.15),
            scale(up, 1.8),
        ),
        add(scale(tangent, 1.8), scatter),
    )
}

/// Legacy geometry-free ejection with a stable variant (ID modulo 3).
pub fn ejection_velocity(position: [f64; 3], body: Option<SphereSurface>, variant: u8) -> [f64; 3] {
    if let Some(body) = body {
        let up = normalize(sub(position, body.center));
        let axis = if up[1].abs() < 0.9 {
            [0.0, 1.0, 0.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        let tangent = normalize(cross(up, axis));
        add(
            scale(up, 1.55),
            scale(tangent, 0.70 + f64::from(variant) * 0.12),
        )
    } else {
        [0.0; 3]
    }
}

/// Opt-in observations of the production solver, accumulated per call.
/// Counters count visits (including repeated solver passes), not unique pairs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepStats {
    pub objects: usize,
    pub awake_objects: usize,
    pub sleeping_objects: usize,
    /// Bodies satisfying dwell criteria; an unstable island member can keep them awake.
    pub quiet_objects: usize,
    pub integrated_objects: u64,
    pub substeps: u64,
    pub solver_passes: u64,
    pub pair_visits: u64,
    pub radius_candidates: u64,
    pub narrow_phase_tests: u64,
    pub contacts: u64,
    pub projection_builds: u64,
    /// Requested sparse pair-table/projection growth; excludes hash control bytes,
    /// allocator overhead, grid/transformed-hull/response scratch and temporaries.
    pub projection_storage_bytes: u64,
    /// Sparse pair-table growth events (legacy metric name for benchmark compatibility).
    /// Not a process allocator counter or a count of grid/scratch allocations.
    pub matrix_allocations: u64,
    /// Stable-ID validation and post-solve support graph/dwell processing.
    pub activation_time: Duration,
    pub integration_time: Duration,
    pub contact_time: Duration,
}

/// Profile with per-substep clocks, never a clock or allocator hook per pair.
pub fn advance_profiled(
    pieces: &mut [ObjectState],
    delta: Duration,
    floor_contains: impl Fn([f64; 3], f64) -> bool,
) -> StepStats {
    advance_impl::<true>(pieces, delta, floor_contains, None)
}

/// Advance in stable slice order; preserve that order across updates.
/// `floor_contains` is a pure geometry query in ship-local metres, not a response rule.
/// Separate floor frames must be advanced in separate calls. Sphere centers distinguish
/// planetary contact groups. Convex substeps adapt to scale/speed; sphere-only
/// calls retain the legacy 48-step cap. No elapsed time is discarded.
pub fn advance(
    pieces: &mut [ObjectState],
    delta: Duration,
    floor_contains: impl Fn([f64; 3], f64) -> bool,
) {
    advance_impl::<false>(pieces, delta, floor_contains, None);
}

/// Stateful activation path. IDs follow the snapshot order and must be unique.
/// `profile` controls wall clocks/counters; sleep behavior is identical either way.
pub fn advance_with_sleep<K: Copy + Eq + std::hash::Hash>(
    tracker: &mut SleepTracker<K>,
    ids: &[K],
    pieces: &mut [ObjectState],
    delta: Duration,
    floor_contains: impl Fn([f64; 3], f64) -> bool,
    profile: bool,
) -> StepStats {
    if delta.is_zero() {
        return StepStats::default();
    }
    let activation_start = profile.then(Instant::now);
    let mut activation = tracker.prepare(ids, pieces);
    for (i, piece) in pieces.iter().enumerate() {
        if piece.surface.is_floor() && !floor_contains(piece.position, piece.radius) {
            activation.wake(i);
            activation.invalidate_support(i);
        }
    }
    let prepare_time = activation_start.map(|s| s.elapsed()).unwrap_or_default();
    let mut stats = if profile {
        advance_impl::<true>(pieces, delta, &floor_contains, Some(&mut activation))
    } else {
        advance_impl::<false>(pieces, delta, &floor_contains, Some(&mut activation))
    };
    let finish_start = profile.then(Instant::now);
    tracker.finish(ids, pieces, delta, &mut activation, floor_contains);
    stats.activation_time = prepare_time + finish_start.map(|s| s.elapsed()).unwrap_or_default();
    stats.sleeping_objects = activation.sleeping.iter().filter(|s| **s).count();
    stats.awake_objects = pieces.len() - stats.sleeping_objects;
    if profile {
        stats.quiet_objects = tracker.quiet_objects();
    }
    stats
}

fn advance_impl<const PROFILE: bool>(
    pieces: &mut [ObjectState],
    delta: Duration,
    floor_contains: impl Fn([f64; 3], f64) -> bool,
    mut activation: Option<&mut sleep::Activation>,
) -> StepStats {
    let mut stats = StepStats {
        objects: pieces.len(),
        awake_objects: pieces.len(),
        ..StepStats::default()
    };
    let elapsed = delta.as_secs_f64();
    if elapsed <= 0.0
        || activation
            .as_ref()
            .is_some_and(|a| a.sleeping.iter().all(|s| *s))
    {
        return stats;
    }
    // A fixed maximum substep prevents fast ejected pieces from tunnelling through a pile.
    let has_hulls = pieces.iter().any(|p| p.hull.is_some());
    let base_steps = (elapsed / MAX_STEP)
        .ceil()
        .clamp(1.0, if has_hulls { 4096.0 } else { 48.0 }) as usize;
    let base_dt = elapsed / base_steps as f64;
    let mut remaining = elapsed;
    let mut completed = 0;
    let mut local_geometry = convex::PairGeometry::default();
    while remaining > 0.0 {
        // Reevaluate after each response: a fast ejection that lands must not
        // impose its initial timestep on the rest of this update. Sleeping
        // objects do not constrain motion until impact/support changes wake them.
        let motion_step = pieces
            .iter()
            .enumerate()
            .filter(|(i, p)| {
                p.hull.is_some() && !activation.as_ref().is_some_and(|a| a.sleeping[*i])
            })
            .map(|(_, p)| {
                0.15 * p.side_meters
                    / (length(p.velocity)
                        + length(p.angular_velocity) * p.radius
                        + 1.0
                        + GRAVITY * base_dt)
            })
            .fold(base_dt, f64::min)
            .max(1e-5);
        let available = if has_hulls { 4096 } else { base_steps } - completed;
        let dt = if has_hulls {
            let steps = (remaining / motion_step)
                .ceil()
                .clamp(1.0, available as f64);
            (remaining / steps)
                .min(base_dt)
                .max(remaining / available as f64)
        } else {
            remaining / available as f64
        };
        remaining = (remaining - dt).max(0.0);
        completed += 1;
        if let Some(a) = &mut activation {
            a.impacts(pieces, dt);
        }
        let mut ground_responses = vec![ContactResponse::default(); pieces.len()];
        let mut warm_pairs = activation
            .as_mut()
            .map(|a| std::mem::take(&mut a.warm_pairs))
            .unwrap_or_default();
        if let Some(a) = &mut activation {
            let ratio = if a.warm_dt > 0.0 {
                (dt / a.warm_dt).clamp(0.0, 2.0)
            } else {
                0.0
            };
            for response in warm_pairs.values_mut() {
                response.rearm(ratio);
            }
            for (index, mut response) in a.warm_ground.drain() {
                response.rearm(ratio);
                ground_responses[index] = response;
            }
        }
        let integration_start = PROFILE.then(Instant::now);
        let mut gravity_velocities = vec![[0.0; 3]; pieces.len()];
        let mut gravity_ups = vec![[0.0, 1.0, 0.0]; pieces.len()];
        for (i, piece) in pieces.iter_mut().enumerate() {
            if activation.as_ref().is_some_and(|a| a.sleeping[i]) {
                continue;
            }
            if PROFILE {
                stats.integrated_objects += 1;
            }
            let previous = piece.position;
            let up = piece
                .surface
                .sphere()
                .map(|body| normalize(sub(piece.position, body.center)))
                .unwrap_or([0.0, 1.0, 0.0]);
            piece.velocity = sub(piece.velocity, scale(up, piece.surface.gravity() * dt));
            gravity_velocities[i] = piece.velocity;
            gravity_ups[i] = up;
            if !has_hulls {
                piece.position = add(piece.position, scale(piece.velocity, dt));
                piece.orientation =
                    convex::integrate(piece.orientation, piece.angular_velocity, dt);
            }
            if piece.surface.is_floor() && !floor_contains(piece.position, piece.radius) {
                piece.position[0] = previous[0];
                piece.position[2] = previous[2];
                piece.velocity[0] *= -0.15;
                piece.velocity[2] *= -0.15;
            }
            ground_contact_cached(piece, &mut ground_responses[i]);
        }
        if let Some(start) = integration_start {
            stats.integration_time += start.elapsed();
            stats.substeps += 1;
        }
        if let Some(a) = &mut activation {
            a.impacts(pieces, 0.0);
        }
        let contact_start = PROFILE.then(Instant::now);
        let reused_geometry = activation.as_mut().and_then(|a| a.geometry.take());
        let mut pair_geometry =
            reused_geometry.unwrap_or_else(|| std::mem::take(&mut local_geometry));
        for geometry in pair_geometry.pairs.values_mut().flatten() {
            geometry.used_this_substep = false;
        }
        solve_contacts::<PROFILE>(
            pieces,
            &mut activation,
            &mut pair_geometry,
            &mut ground_responses,
            &mut warm_pairs,
            &floor_contains,
            &mut stats,
        );
        if let Some(start) = contact_start {
            stats.contact_time += start.elapsed();
        }
        let pose_start = PROFILE.then(Instant::now);
        let mut moved = false;
        let mut rotated = vec![false; pieces.len()];
        let mut support_reactions = vec![false; pieces.len()];
        let mut before_post_contact = vec![[0.0; 3]; pieces.len()];
        for (i, piece) in pieces.iter_mut().enumerate() {
            if activation.as_ref().is_some_and(|a| a.sleeping[i]) {
                continue;
            }
            let supported_reaction =
                dot(sub(piece.velocity, gravity_velocities[i]), gravity_ups[i])
                    > 0.5 * GRAVITY * dt;
            support_reactions[i] = supported_reaction;
            if !matches!(piece.surface, Surface::UnpoweredFloor { .. }) {
                piece.velocity = scale(piece.velocity, 0.995);
                piece.angular_velocity = scale(piece.angular_velocity, 0.98);
            }
            if !has_hulls && !matches!(piece.surface, Surface::UnpoweredFloor { .. }) {
                if length(piece.angular_velocity) < 0.015 {
                    piece.angular_velocity = [0.0; 3];
                }
                if length(piece.velocity) < 0.015 {
                    piece.velocity = [0.0; 3];
                }
            }
            if has_hulls {
                let previous = piece.position;
                let orientation = piece.orientation;
                piece.position = add(piece.position, scale(piece.velocity, dt));
                piece.orientation =
                    convex::integrate(piece.orientation, piece.angular_velocity, dt);
                if piece.surface.is_floor() && !floor_contains(piece.position, piece.radius) {
                    piece.position[0] = previous[0];
                    piece.position[2] = previous[2];
                    piece.velocity[0] *= -0.15;
                    piece.velocity[2] *= -0.15;
                }
                let before_ground = piece.velocity;
                if piece.orientation != orientation {
                    ground_responses[i].transport_response = true;
                }
                ground_contact_cached(piece, &mut ground_responses[i]);
                support_reactions[i] |=
                    dot(sub(piece.velocity, before_ground), gravity_ups[i]) > 0.5 * GRAVITY * dt;
                before_post_contact[i] = piece.velocity;
                moved |= piece.position != previous || piece.orientation != orientation;
                rotated[i] = piece.orientation != orientation;
            }
        }
        if let Some(start) = pose_start {
            stats.integration_time += start.elapsed();
        }
        if has_hulls && moved {
            pair_geometry.pairs.retain(|&(i, j), cache| {
                if rotated[i] || rotated[j] {
                    if let Some(geometry) = cache.as_mut()
                        && geometry.used_this_substep
                    {
                        let mut response = std::mem::take(&mut geometry.response);
                        response.transport_response = true;
                        warm_pairs.insert((i, j), response);
                    }
                    false
                } else {
                    true
                }
            });
            if let Some(a) = &mut activation {
                a.impacts(pieces, 0.0);
            }
            let started = PROFILE.then(Instant::now);
            solve_contacts::<PROFILE>(
                pieces,
                &mut activation,
                &mut pair_geometry,
                &mut ground_responses,
                &mut warm_pairs,
                &floor_contains,
                &mut stats,
            );
            if let Some(started) = started {
                stats.contact_time += started.elapsed();
            }
        }
        // Apply rest cutoff only after all constraint iterations. Clearing
        // velocities between solves invalidates their accumulated impulses.
        if has_hulls {
            for (i, piece) in pieces.iter_mut().enumerate() {
                if activation.as_ref().is_some_and(|a| a.sleeping[i]) {
                    continue;
                }
                let reaction = support_reactions[i]
                    || dot(sub(piece.velocity, before_post_contact[i]), gravity_ups[i])
                        > 0.5 * GRAVITY * dt;
                let surface_speed =
                    length(piece.velocity) + length(piece.angular_velocity) * piece.radius;
                if reaction && surface_speed < 0.015 {
                    piece.velocity = [0.0; 3];
                    piece.angular_velocity = [0.0; 3];
                } else if length(piece.angular_velocity) < 0.015 {
                    piece.angular_velocity = [0.0; 3];
                }
            }
        }
        if let Some(a) = &mut activation {
            a.warm_dt = dt;
            a.warm_pairs.clear();
            for (&pair, geometry) in &mut pair_geometry.pairs {
                if let Some(geometry) = geometry
                    && geometry.used_this_substep
                    && !geometry.response.arms.is_empty()
                {
                    a.warm_pairs
                        .insert(pair, std::mem::take(&mut geometry.response));
                }
            }
            a.warm_ground = ground_responses
                .into_iter()
                .enumerate()
                .filter(|(_, response)| !response.arms.is_empty())
                .collect();
            a.geometry = Some(pair_geometry);
        } else {
            local_geometry = pair_geometry;
        }
    }
    stats
}

fn solve_contacts<const PROFILE: bool>(
    pieces: &mut [ObjectState],
    activation: &mut Option<&mut sleep::Activation>,
    pair_geometry: &mut convex::PairGeometry,
    ground_responses: &mut [ContactResponse],
    warm_pairs: &mut std::collections::HashMap<(usize, usize), ContactResponse>,
    floor_contains: impl Fn([f64; 3], f64) -> bool,
    stats: &mut StepStats,
) {
    let has_hulls = pieces.iter().any(|piece| piece.hull.is_some());
    let old_capacity = pair_geometry.pairs.capacity();
    pair_geometry.grid.reset(pieces, 2.0 * CONTACT_BIAS_METERS);
    pair_geometry
        .pairs
        .retain(|&(i, j), g| g.is_some() && pair_geometry.grid.overlaps(i, j));
    pair_geometry
        .transformed
        .resize_with(pieces.len(), Default::default);
    for (cache, p) in pair_geometry.transformed.iter_mut().zip(pieces.iter()) {
        cache.refresh(p);
    }
    for _ in 0..if has_hulls { 16 } else { 3 } {
        if PROFILE {
            stats.solver_passes += 1;
        }
        for i in 0..pieces.len() {
            pair_geometry
                .grid
                .query(i, 0.0, &mut pair_geometry.candidates);
            let mut after = i;
            loop {
                let next_index = pair_geometry.candidates.partition_point(|&j| j <= after);
                let next = pair_geometry.candidates.get(next_index).copied();
                let Some(j) = next else {
                    break;
                };
                after = j;
                let before = (pieces[i].position, pieces[j].position);
                // Sparse lookup order never affects resolution order.
                resolve_candidate::<PROFILE>(
                    pieces,
                    i,
                    j,
                    activation,
                    &mut pair_geometry.pairs,
                    &pair_geometry.transformed,
                    ground_responses,
                    warm_pairs,
                    &floor_contains,
                    stats,
                );
                let changed_i = pieces[i].position != before.0;
                let changed_j = pieces[j].position != before.1;
                if changed_i {
                    pair_geometry
                        .grid
                        .update(i, &pieces[i], 2.0 * CONTACT_BIAS_METERS);
                }
                if changed_j {
                    pair_geometry
                        .grid
                        .update(j, &pieces[j], 2.0 * CONTACT_BIAS_METERS);
                }
                if changed_i || changed_j {
                    // Match brute-force semantics: newly reachable later IDs are
                    // visited this pass; earlier IDs wait until the next pass.
                    pair_geometry
                        .grid
                        .query(i, 0.0, &mut pair_geometry.candidates);
                }
            }
        }
    }
    pair_geometry
        .pairs
        .retain(|&(i, j), g| g.is_some() && pair_geometry.grid.overlaps(i, j));
    if PROFILE && pair_geometry.pairs.capacity() > old_capacity {
        stats.matrix_allocations += 1;
        stats.projection_storage_bytes += ((pair_geometry.pairs.capacity() - old_capacity)
            * std::mem::size_of::<((usize, usize), Option<Box<convex::ContactGeometry>>)>())
            as u64;
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_candidate<const PROFILE: bool>(
    pieces: &mut [ObjectState],
    i: usize,
    j: usize,
    activation: &mut Option<&mut sleep::Activation>,
    pairs: &mut std::collections::HashMap<(usize, usize), Option<Box<convex::ContactGeometry>>>,
    transformed: &[convex::TransformedHull],
    ground_responses: &mut [ContactResponse],
    warm_pairs: &mut std::collections::HashMap<(usize, usize), ContactResponse>,
    floor_contains: impl Fn([f64; 3], f64) -> bool,
    stats: &mut StepStats,
) {
    if activation
        .as_ref()
        .is_some_and(|a| a.sleeping[i] && a.sleeping[j])
    {
        return;
    }
    if PROFILE {
        stats.pair_visits += 1;
    }
    let (left, right) = pieces.split_at_mut(j);
    let a = &mut left[i];
    let b = &mut right[0];
    if a.surface.is_floor() != b.surface.is_floor()
        || (a.surface.sphere().is_some()
            && b.surface.sphere().is_some()
            && a.surface.sphere().unwrap().center != b.surface.sphere().unwrap().center)
    {
        return;
    }
    let separation = sub(b.position, a.position);
    let distance = length(separation);
    let minimum = a.radius + b.radius;
    let convex_pair = a.hull.is_some() && b.hull.is_some();
    if if convex_pair {
        distance > minimum + 2.0 * CONTACT_BIAS_METERS
    } else {
        distance >= minimum
    } {
        return;
    }
    if PROFILE {
        stats.radius_candidates += 1;
    }
    if PROFILE && convex_pair {
        stats.narrow_phase_tests += 1;
    }
    // Do not allocate a sparse slot for rejected AABB/radius/face candidates.
    if convex_pair
        && !pairs
            .get(&(i, j))
            .and_then(Option::as_ref)
            .is_some_and(|g| g.used_this_substep)
        && convex::separated_by_faces(a, b)
    {
        return;
    }
    let mut sphere_cache = None;
    let cached = if convex_pair {
        pairs.entry((i, j)).or_default()
    } else {
        &mut sphere_cache
    };
    let (normal, depth) = if a.hull.is_some() && b.hull.is_some() {
        let geometry = cached.get_or_insert_with(|| {
            let projections = convex::transformed_projections(&transformed[i], &transformed[j]);
            if PROFILE {
                stats.projection_builds += 1;
                stats.projection_storage_bytes +=
                    (projections.capacity() * std::mem::size_of::<convex::Projection>()) as u64;
            }
            if PROFILE {
                stats.projection_storage_bytes +=
                    std::mem::size_of::<convex::ContactGeometry>() as u64;
            }
            Box::new(convex::ContactGeometry::new(projections))
        });
        geometry.used_this_substep = true;
        if geometry.response.arms.is_empty()
            && let Some(response) = warm_pairs.remove(&(i, j))
        {
            geometry.response = response;
        }
        let (contact, requested_bytes) = geometry.contact(separation);
        if PROFILE {
            stats.projection_storage_bytes += requested_bytes as u64;
        }
        let Some(contact) = contact else {
            return;
        };
        contact
    } else {
        (
            if distance > 1e-9 {
                scale(separation, 1.0 / distance)
            } else {
                [0.0, 1.0, 0.0]
            },
            minimum - distance,
        )
    };
    if PROFILE {
        stats.contacts += 1;
    }
    if let Some(activation) = activation.as_mut() {
        activation.wake(i);
        activation.wake(j);
    }
    let inverse_a = 1.0 / a.mass_kg;
    let inverse_b = 1.0 / b.mass_kg;
    let direction_a = correction_direction(a, scale(normal, -1.0));
    let direction_b = correction_direction(b, normal);
    let available = -dot(direction_a, normal) * inverse_a + dot(direction_b, normal) * inverse_b;
    let correction = (depth + CONTACT_BIAS_METERS).max(0.0) / available;
    let old_a = a.position;
    let old_b = b.position;
    a.position = add(a.position, scale(direction_a, correction * inverse_a));
    b.position = add(b.position, scale(direction_b, correction * inverse_b));
    if a.surface.is_floor() && !floor_contains(a.position, a.radius) {
        a.position[0] = old_a[0];
        a.position[2] = old_a[2];
    }
    if b.surface.is_floor() && !floor_contains(b.position, b.radius) {
        b.position[0] = old_b[0];
        b.position[2] = old_b[2];
    }
    // The constrained positional correction need not move along
    // the original SAT axis. Re-evaluate the actual contact before
    // applying velocity impulses at a potentially separated feature.
    let normal = if a.hull.is_some() && b.hull.is_some() {
        let geometry = cached.as_mut().expect("convex geometry");
        let (contact, requested_bytes) = geometry.contact(sub(b.position, a.position));
        if PROFILE {
            stats.projection_storage_bytes += requested_bytes as u64;
        }
        let Some((normal, _)) = contact else {
            ground_contact_cached(a, &mut ground_responses[i]);
            ground_contact_cached(b, &mut ground_responses[j]);
            return;
        };
        normal
    } else {
        normal
    };
    if let Some(geometry) = cached {
        pair_impulse_cached(a, b, normal, &mut geometry.response);
    } else {
        pair_impulse(a, b, normal);
    }
    ground_contact_cached(a, &mut ground_responses[i]);
    ground_contact_cached(b, &mut ground_responses[j]);
}

#[derive(Clone, Debug, Default)]
struct ContactResponse {
    pending_warm_start: bool,
    transport_response: bool,
    normal: [f64; 3],
    arms: Vec<([f64; 3], [f64; 3])>,
    impulses: Vec<f64>,
    friction: Vec<[f64; 3]>,
    rolling: [f64; 3],
    targets: Vec<f64>,
}
impl ContactResponse {
    fn rearm(&mut self, ratio: f64) {
        for impulse in &mut self.impulses {
            *impulse *= ratio;
        }
        for impulse in &mut self.friction {
            *impulse = scale(*impulse, ratio);
        }
        self.rolling = scale(self.rolling, ratio);
        self.targets.fill(0.0);
        self.pending_warm_start = true;
    }
    fn prepare(
        &mut self,
        normal: [f64; 3],
        arms: &[([f64; 3], [f64; 3])],
        targets: impl Fn(usize) -> f64,
    ) {
        let transport = self.pending_warm_start || self.transport_response;
        let max_arm_change = if transport { 0.001 } else { 0.0001 };
        let normal_matches = if transport {
            dot(self.normal, normal) > 0.999
        } else {
            length(sub(self.normal, normal)) < 1e-8
        };
        let unchanged = normal_matches
            && self.arms.len() == arms.len()
            && self.arms.iter().zip(arms).all(|(old, new)| {
                length(sub(old.0, new.0)) < max_arm_change
                    && length(sub(old.1, new.1)) < max_arm_change
            });
        if unchanged && transport {
            let normal_factor = dot(self.normal, normal).max(0.0);
            for i in 0..self.impulses.len() {
                self.impulses[i] *= normal_factor;
                self.friction[i] = sub(
                    self.friction[i],
                    scale(normal, dot(self.friction[i], normal)),
                );
                let magnitude = length(self.friction[i]);
                let limit = self.impulses[i] * 0.65;
                if magnitude > limit {
                    self.friction[i] = scale(self.friction[i], limit / magnitude);
                }
            }
            self.normal = normal;
            self.arms.clear();
            self.arms.extend_from_slice(arms);
        }
        self.transport_response = false;
        if !unchanged {
            self.pending_warm_start = false;
            self.normal = normal;
            self.arms = arms.to_vec();
            self.impulses = vec![0.0; arms.len()];
            self.friction = vec![[0.0; 3]; arms.len()];
            self.rolling = [0.0; 3];
            self.targets = (0..arms.len()).map(targets).collect();
        }
    }
}

// A bounded contact torque models energy lost to rough-contact rolling/spin.
// Its limit is 10% of normal impulse times bounding radius; no load means no drag.
fn rolling_delta(
    relative: [f64; 3],
    previous: &mut [f64; 3],
    limit: f64,
    inverse_angular_mass: f64,
) -> [f64; 3] {
    let mut candidate = sub(*previous, scale(relative, 1.0 / inverse_angular_mass));
    let magnitude = length(candidate);
    if magnitude > limit {
        candidate = scale(candidate, limit / magnitude);
    }
    let delta = sub(candidate, *previous);
    *previous = candidate;
    delta
}

fn weightless_ceiling_contact(piece: &mut ObjectState) {
    if let Surface::UnpoweredFloor { ceiling_meters, .. } = piece.surface {
        // Conservative bounding support contains rotating cargo at the ceiling.
        let maximum_center = ceiling_meters - piece.radius;
        if piece.position[1] > maximum_center {
            piece.position[1] = maximum_center;
            if piece.velocity[1] > 0.0 {
                piece.velocity[1] *= -RESTITUTION;
            }
        }
    }
}

#[cfg(test)]
fn ground_contact(piece: &mut ObjectState) {
    weightless_ceiling_contact(piece);
    ground_contact_cached(piece, &mut ContactResponse::default());
}
struct SampledContact {
    normal: [f64; 3],
    penetration: f64,
    arms: Vec<[f64; 3]>,
}
fn sampled_contact(piece: &ObjectState) -> Option<SampledContact> {
    let Surface::Sampled {
        body,
        query,
        point_query,
    } = piece.surface
    else {
        return None;
    };
    let hull = piece.hull.as_ref()?;
    let mut points = Vec::with_capacity(hull.vertices.len());
    let mut deepest = f64::INFINITY;
    let center_normal = query(body, piece.position).normal;
    let mut witness = piece.position;
    for vertex in &hull.vertices {
        let arm = convex::rotate(piece.orientation, scale(*vertex, piece.side_meters));
        let point = add(piece.position, arm);
        let ground = point_query(body, point);
        let gap = dot(sub(point, ground), center_normal);
        if gap < deepest {
            deepest = gap;
            witness = point;
        }
        points.push((arm, gap));
    }
    let arms = points
        .into_iter()
        .filter(|(_, gap)| *gap <= deepest + 0.001 * piece.side_meters)
        .map(|(arm, _)| arm)
        .collect();
    let normal = query(body, witness).normal;
    Some(SampledContact {
        normal,
        penetration: -deepest,
        arms,
    })
}
fn ground_contact_cached(piece: &mut ObjectState, response: &mut ContactResponse) {
    weightless_ceiling_contact(piece);
    let Some(ground) = piece.surface.ground(piece.position) else {
        *response = ContactResponse::default();
        return;
    };
    let sampled = sampled_contact(piece);
    let up = sampled.as_ref().map_or(ground.normal, |c| c.normal);
    if let Some(hull) = &piece.hull {
        piece.ground_support_meters =
            hull.support(piece.orientation, piece.side_meters, scale(up, -1.0));
    }
    let penetration = sampled.as_ref().map_or_else(
        || {
            let gap = if let Surface::Sphere(body) = piece.surface {
                length(sub(piece.position, body.center)) - body.radius
            } else {
                dot(sub(piece.position, ground.point), up)
            };
            if let Surface::Sphere(body) = piece.surface {
                body.radius + piece.ground_support_meters - length(sub(piece.position, body.center))
            } else {
                piece.ground_support_meters - gap
            }
        },
        |c| c.penetration,
    );
    if penetration < -CONTACT_BIAS_METERS {
        *response = ContactResponse::default();
        return;
    }
    piece.position = add(piece.position, scale(up, penetration.max(0.0)));
    if let Some(hull) = &piece.hull {
        let arms = sampled.map_or_else(
            || hull.ground_arms(piece.orientation, piece.side_meters, scale(up, -1.0)),
            |c| c.arms,
        );
        let paired_arms: Vec<_> = arms.iter().map(|&arm| (arm, [0.0; 3])).collect();
        response.prepare(up, &paired_arms, |_| 0.0);
        if std::mem::take(&mut response.pending_warm_start) {
            piece.angular_velocity = add(
                piece.angular_velocity,
                scale(response.rolling, inverse_inertia(piece)),
            );
            for (i, &arm) in arms.iter().enumerate() {
                apply_impulse(
                    piece,
                    arm,
                    add(scale(up, response.impulses[i]), response.friction[i]),
                );
            }
        }
        let normal_impulses = &mut response.impulses;
        // Accumulate unilateral impulses within this face solve. Later corners
        // can undo excess impulses while every corner's total stays nonnegative.
        // Positive-only corner impulses otherwise eject and rock resting faces.
        let friction_impulses = &mut response.friction;
        for _ in 0..16 {
            for (index, &arm) in arms.iter().enumerate() {
                let approach = dot(add(piece.velocity, cross(piece.angular_velocity, arm)), up);
                let denominator = 1.0 / piece.mass_kg
                    + dot(cross(arm, up), cross(arm, up)) * inverse_inertia(piece);
                let previous = normal_impulses[index];
                normal_impulses[index] = (previous - approach / denominator).max(0.0);
                apply_impulse(piece, arm, scale(up, normal_impulses[index] - previous));
            }
            for (i, &arm) in arms.iter().enumerate() {
                let velocity = add(piece.velocity, cross(piece.angular_velocity, arm));
                let denominator = |direction| {
                    1.0 / piece.mass_kg
                        + dot(cross(arm, direction), cross(arm, direction)) * inverse_inertia(piece)
                };
                let impulse = friction_delta(
                    velocity,
                    up,
                    &mut friction_impulses[i],
                    normal_impulses[i] * 0.65,
                    denominator,
                );
                apply_impulse(piece, arm, impulse);
            }
            let torque = rolling_delta(
                piece.angular_velocity,
                &mut response.rolling,
                ROLLING_RESISTANCE * normal_impulses.iter().sum::<f64>() * piece.radius,
                inverse_inertia(piece),
            );
            piece.angular_velocity = add(
                piece.angular_velocity,
                scale(torque, inverse_inertia(piece)),
            );
        }
        return;
    }
    let downward = dot(piece.velocity, up);
    if downward < 0.0 {
        piece.velocity = sub(piece.velocity, scale(up, downward * (1.0 + RESTITUTION)));
    }
    let normal_speed = dot(piece.velocity, up);
    piece.velocity = add(
        scale(sub(piece.velocity, scale(up, normal_speed)), 0.88),
        scale(up, normal_speed),
    );
}

// A convex body already supported by a rigid deck cannot resolve another
// contact by moving down through that deck. Project the positional response
// onto its available directions and give the remaining separation to its
// neighbor. Previously ground projection undid that share of every correction,
// leaving loaded stacks to compact and chatter over repeated passes. Physical
// mass/inertia and velocity impulses are unchanged; this is a support constraint.
fn correction_direction(piece: &ObjectState, direction: [f64; 3]) -> [f64; 3] {
    if piece.hull.is_some()
        && let Surface::Floor { height_meters } | Surface::UnpoweredFloor { height_meters, .. } =
            piece.surface
        && piece.position[1] - height_meters - piece.ground_support_meters <= CONTACT_BIAS_METERS
        && direction[1] < 0.0
    {
        [direction[0], 0.0, direction[2]]
    } else {
        direction
    }
}

fn inverse_inertia(piece: &ObjectState) -> f64 {
    if piece.hull.is_some() {
        1.0 / (0.4 * piece.mass_kg * piece.radius * piece.radius)
    } else {
        0.0
    }
}
fn apply_impulse(piece: &mut ObjectState, arm: [f64; 3], impulse: [f64; 3]) {
    piece.velocity = add(piece.velocity, scale(impulse, 1.0 / piece.mass_kg));
    piece.angular_velocity = add(
        piece.angular_velocity,
        scale(cross(arm, impulse), inverse_inertia(piece)),
    );
}
// Project each accumulated tangential impulse onto its Coulomb disk. Distributed
// contacts resist twist as well as translation; later iterations can undo an
// earlier corner's excess impulse without exceeding its normal-load budget.
fn friction_delta(
    relative: [f64; 3],
    normal: [f64; 3],
    previous: &mut [f64; 3],
    limit: f64,
    denominator: impl Fn([f64; 3]) -> f64,
) -> [f64; 3] {
    let tangent = sub(relative, scale(normal, dot(relative, normal)));
    let speed = length(tangent);
    let mut candidate = *previous;
    if speed > 1e-9 {
        candidate = sub(
            candidate,
            scale(tangent, 1.0 / denominator(scale(tangent, 1.0 / speed))),
        );
    }
    let magnitude = length(candidate);
    if magnitude > limit {
        candidate = scale(candidate, limit / magnitude);
    }
    let delta = sub(candidate, *previous);
    *previous = candidate;
    delta
}

fn pair_impulse(a: &mut ObjectState, b: &mut ObjectState, normal: [f64; 3]) {
    pair_impulse_cached(a, b, normal, &mut ContactResponse::default());
}

fn pair_impulse_cached(
    a: &mut ObjectState,
    b: &mut ObjectState,
    normal: [f64; 3],
    response: &mut ContactResponse,
) {
    let inverse_mass_a = 1.0 / a.mass_kg;
    let inverse_mass_b = 1.0 / b.mass_kg;
    let inertia_a = inverse_inertia(a);
    let inertia_b = inverse_inertia(b);
    if a.hull.is_some() && b.hull.is_some() {
        let arms = convex::contact_manifold(a, b, normal);
        if !arms.is_empty() {
            let relative = |a: &ObjectState, b: &ObjectState, aa, ba| {
                sub(
                    add(b.velocity, cross(b.angular_velocity, ba)),
                    add(a.velocity, cross(a.angular_velocity, aa)),
                )
            };
            response.prepare(normal, &arms, |i| {
                let approach = dot(relative(a, b, arms[i].0, arms[i].1), normal);
                if approach < -0.5 {
                    -RESTITUTION * approach
                } else {
                    0.0
                }
            });
            if std::mem::take(&mut response.pending_warm_start) {
                a.angular_velocity = sub(a.angular_velocity, scale(response.rolling, inertia_a));
                b.angular_velocity = add(b.angular_velocity, scale(response.rolling, inertia_b));
                for (i, &(aa, ba)) in arms.iter().enumerate() {
                    let impulse = add(scale(normal, response.impulses[i]), response.friction[i]);
                    apply_impulse(a, aa, scale(impulse, -1.0));
                    apply_impulse(b, ba, impulse);
                }
            }
            let targets = &response.targets;
            let impulses = &mut response.impulses;
            let friction_impulses = &mut response.friction;
            for _ in 0..16 {
                for (i, &(aa, ba)) in arms.iter().enumerate() {
                    let denominator = inverse_mass_a
                        + inverse_mass_b
                        + dot(cross(aa, normal), cross(aa, normal)) * inertia_a
                        + dot(cross(ba, normal), cross(ba, normal)) * inertia_b;
                    let previous = impulses[i];
                    impulses[i] = (previous
                        + (targets[i] - dot(relative(a, b, aa, ba), normal)) / denominator)
                        .max(0.0);
                    let impulse = scale(normal, impulses[i] - previous);
                    apply_impulse(a, aa, scale(impulse, -1.0));
                    apply_impulse(b, ba, impulse);
                }
                for (i, &(aa, ba)) in arms.iter().enumerate() {
                    let velocity = relative(a, b, aa, ba);
                    let denominator = |direction| {
                        inverse_mass_a
                            + inverse_mass_b
                            + dot(cross(aa, direction), cross(aa, direction)) * inertia_a
                            + dot(cross(ba, direction), cross(ba, direction)) * inertia_b
                    };
                    let impulse = friction_delta(
                        velocity,
                        normal,
                        &mut friction_impulses[i],
                        impulses[i] * 0.65,
                        denominator,
                    );
                    apply_impulse(a, aa, scale(impulse, -1.0));
                    apply_impulse(b, ba, impulse);
                }
                let torque = rolling_delta(
                    sub(b.angular_velocity, a.angular_velocity),
                    &mut response.rolling,
                    ROLLING_RESISTANCE * impulses.iter().sum::<f64>() * a.radius.min(b.radius),
                    inertia_a + inertia_b,
                );
                a.angular_velocity = sub(a.angular_velocity, scale(torque, inertia_a));
                b.angular_velocity = add(b.angular_velocity, scale(torque, inertia_b));
            }
            return;
        }
    }
    let (aa, ba) = if a.hull.is_some() && b.hull.is_some() {
        convex::contact_arms(a, b, normal)
    } else {
        (scale(normal, a.radius), scale(normal, -b.radius))
    };
    let relative = sub(
        add(b.velocity, cross(b.angular_velocity, ba)),
        add(a.velocity, cross(a.angular_velocity, aa)),
    );
    let approach = dot(relative, normal);
    if approach >= 0.0 {
        return;
    }
    let denom = |direction| {
        inverse_mass_a
            + inverse_mass_b
            + dot(cross(aa, direction), cross(aa, direction)) * inertia_a
            + dot(cross(ba, direction), cross(ba, direction)) * inertia_b
    };
    let restitution = if a.hull.is_none() || approach < -0.5 {
        RESTITUTION
    } else {
        0.0
    };
    let magnitude = -(1.0 + restitution) * approach / denom(normal);
    let tangent = sub(relative, scale(normal, approach));
    let speed = length(tangent);
    let friction = if speed > 1e-9 {
        let direction = scale(tangent, 1.0 / speed);
        scale(direction, -(speed / denom(direction)).min(magnitude * 0.65))
    } else {
        [0.0; 3]
    };
    let impulse = add(scale(normal, magnitude), friction);
    apply_impulse(a, aa, scale(impulse, -1.0));
    apply_impulse(b, ba, impulse);
}

fn normalize(a: [f64; 3]) -> [f64; 3] {
    let len = length(a);
    if len > 1e-12 {
        scale(a, 1.0 / len)
    } else {
        [0.0, 1.0, 0.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weightless_cargo_retains_drift_and_bounces_below_ceiling() {
        let mut piece = ObjectState {
            position: [0.0, 1.0, 0.0],
            velocity: [1.0, 0.0, 0.0],
            radius: 0.1,
            ground_support_meters: 0.0,
            hull: None,
            side_meters: 0.2,
            orientation: [0.0, 0.0, 0.0, 1.0],
            angular_velocity: [0.0; 3],
            mass_kg: 1.0,
            surface: Surface::UnpoweredFloor {
                height_meters: 0.0,
                ceiling_meters: 3.0,
            },
        };
        advance(
            std::slice::from_mut(&mut piece),
            Duration::from_secs(1),
            |_, _| true,
        );
        assert!((piece.position[0] - 1.0).abs() < 1e-10);
        assert_eq!(piece.velocity, [1.0, 0.0, 0.0]);
        piece.velocity = [0.0, 10.0, 0.0];
        advance(
            std::slice::from_mut(&mut piece),
            Duration::from_millis(250),
            |_, _| true,
        );
        assert!(piece.position[1] <= 2.9);
        assert!(piece.velocity[1] < 0.0);
        piece.surface = Surface::Floor { height_meters: 0.0 };
        advance(
            std::slice::from_mut(&mut piece),
            Duration::from_secs(10),
            |_, _| true,
        );
        assert!(piece.position[1] <= 0.101);
    }

    #[test]
    fn unpowered_deck_keeps_contacts_but_disables_gravity() {
        let mut piece = floor([0.0, 2.0, 0.0], [0.0; 3]);
        piece.surface = Surface::UnpoweredFloor {
            height_meters: 0.0,
            ceiling_meters: 3.0,
        };
        advance(
            std::slice::from_mut(&mut piece),
            Duration::from_millis(100),
            |_, _| true,
        );
        assert_eq!(piece.position, [0.0, 2.0, 0.0]);
        assert_eq!(piece.velocity, [0.0; 3]);
        piece.surface = Surface::Floor { height_meters: 0.0 };
        advance(
            std::slice::from_mut(&mut piece),
            Duration::from_millis(100),
            |_, _| true,
        );
        assert!(piece.position[1] < 2.0);
    }

    fn floor(position: [f64; 3], velocity: [f64; 3]) -> ObjectState {
        ObjectState {
            position,
            velocity,
            radius: 0.2,
            ground_support_meters: 0.2,
            surface: Surface::Floor { height_meters: 0.0 },
            hull: None,
            side_meters: 0.4,
            orientation: [0.0, 0.0, 0.0, 1.0],
            angular_velocity: [0.0; 3],
            mass_kg: 1.0,
        }
    }

    #[test]
    fn authored_support_controls_ground_while_pair_contact_keeps_conservative_radius() {
        let mut objects = [
            floor([0.0, 0.1, 0.0], [0.0; 3]),
            floor([0.0, 0.1, 0.0], [0.0; 3]),
        ];
        for object in &mut objects {
            object.radius = 0.8;
            object.ground_support_meters = 0.3;
        }
        for _ in 0..180 {
            advance(&mut objects, Duration::from_millis(16), |_, _| true);
        }
        assert!(objects.iter().all(|p| p.position[1] >= 0.3 - 1e-9));
        assert!(objects.iter().any(|p| (p.position[1] - 0.3).abs() < 0.002));
        assert!(length(sub(objects[0].position, objects[1].position)) >= 1.6 - 0.002);
    }

    #[test]
    fn normalization_keeps_radial_fallback_and_threshold() {
        for vector in [[0.0; 3], [1.0e-12, 0.0, 0.0], [f64::NAN, 0.0, 0.0]] {
            assert_eq!(normalize(vector), [0.0, 1.0, 0.0]);
        }
        assert_eq!(normalize([0.0, 0.0, 2.0e-12]), [0.0, 0.0, 1.0]);
        assert!((length(normalize([3.0, 4.0, 0.0])) - 1.0).abs() < 1.0e-15);
    }

    #[test]
    fn impulses_preserve_surface_direction_and_stable_variants() {
        let body = SphereSurface {
            center: [10.0, 0.0, 0.0],
            radius: 2.0,
        };
        assert_eq!(
            release_velocity([0.0, 1.0, 0.0], [12.0, 0.0, 0.0], Some(body)),
            [0.3, 1.1, 0.0]
        );
        assert_eq!(floor_release_velocity([1.0, 0.9, -1.0]), [1.1, 0.35, -1.1]);
        for variant in 0..3 {
            assert_eq!(
                ejection_velocity([12.0, 0.0, 0.0], Some(body), variant),
                [1.55, 0.0, 0.70 + f64::from(variant) * 0.12]
            );
        }
        assert_eq!(ejection_velocity([0.0; 3], None, 0), [0.0; 3]);
        assert_eq!(
            release_velocity([1.0, 0.0, 0.0], [0.0; 3], None),
            [1.1, 0.3, 0.0]
        );
    }

    #[test]
    fn ground_contact_preserves_restitution_and_tangential_friction() {
        let mut object = floor([0.0, 0.1, 0.0], [1.0, -2.0, 0.0]);
        ground_contact(&mut object);
        assert_eq!(object.position, [0.0, 0.2, 0.0]);
        assert!((object.velocity[1] - 0.24).abs() < 1e-15);
        assert_eq!(object.velocity[0], 0.88);
    }

    #[test]
    fn floor_geometry_rejects_horizontal_motion_with_legacy_response() {
        let mut objects = [floor([0.0, 1.0, 0.0], [1.0, 0.0, 2.0])];
        advance(&mut objects, Duration::from_secs_f64(1.0 / 90.0), |_, _| {
            false
        });
        assert_eq!(objects[0].position[0], 0.0);
        assert_eq!(objects[0].position[2], 0.0);
        assert_eq!(objects[0].velocity[0], -0.15 * 0.995);
        assert_eq!(objects[0].velocity[2], -0.30 * 0.995);
    }

    #[test]
    fn sphere_gravity_is_radial_and_settles_at_radius() {
        let mut objects = [ObjectState {
            position: [12.5, 0.0, 0.0],
            velocity: [0.0; 3],
            radius: 0.2,
            ground_support_meters: 0.2,
            hull: None,
            side_meters: 0.4,
            orientation: [0.0, 0.0, 0.0, 1.0],
            angular_velocity: [0.0; 3],
            mass_kg: 1.0,
            surface: Surface::Sphere(SphereSurface {
                center: [10.0, 0.0, 0.0],
                radius: 2.0,
            }),
        }];
        for _ in 0..180 {
            advance(&mut objects, Duration::from_millis(16), |_, _| {
                panic!("sphere must not query floor")
            });
        }
        assert!((objects[0].position[0] - 12.2).abs() < 0.002);
        assert_eq!(objects[0].position[1..], [0.0, 0.0]);
    }

    #[test]
    fn coincident_objects_separate_and_stack_deterministically() {
        let initial = std::array::from_fn::<_, 2, _>(|_| floor([0.0, 0.5, 0.0], [0.0; 3]));
        let mut a = initial.clone();
        let mut b = initial;
        for _ in 0..150 {
            advance(&mut a, Duration::from_millis(16), |_, _| true);
            advance(&mut b, Duration::from_millis(16), |_, _| true);
        }
        assert_eq!(a, b);
        assert!(length(sub(a[0].position, a[1].position)) >= 0.4 - 0.002);
        assert!(a.iter().all(|p| p.position[1] >= 0.2 - 0.002));
        assert!(a.iter().any(|p| p.position[1] > 0.4));
    }

    #[test]
    fn distinct_support_frames_do_not_contact() {
        let sphere = ObjectState {
            position: [0.0, 1.0, 0.0],
            velocity: [0.0; 3],
            radius: 0.2,
            ground_support_meters: 0.2,
            hull: None,
            side_meters: 0.4,
            orientation: [0.0, 0.0, 0.0, 1.0],
            angular_velocity: [0.0; 3],
            mass_kg: 1.0,
            surface: Surface::Sphere(SphereSurface {
                center: [0.0; 3],
                radius: 0.5,
            }),
        };
        let other = ObjectState {
            surface: Surface::Sphere(SphereSurface {
                center: [0.0, -1.0, 0.0],
                radius: 0.5,
            }),
            ..sphere.clone()
        };
        let mut objects = [
            sphere.clone(),
            other,
            floor(sphere.position, sphere.velocity),
        ];
        let mut isolated = objects.clone();
        advance(&mut objects, Duration::from_millis(16), |_, _| true);
        for p in &mut isolated {
            advance(
                std::slice::from_mut(p),
                Duration::from_millis(16),
                |_, _| true,
            );
        }
        assert_eq!(objects, isolated);
    }

    #[test]
    fn zero_delta_is_noop_and_large_delta_retains_48_substep_cap() {
        let mut objects = [ObjectState {
            surface: Surface::Unsupported,
            ..floor([0.0, 100.0, 0.0], [1.0, 0.0, 0.0])
        }];
        let initial = objects.clone();
        advance(&mut objects, Duration::ZERO, |_, _| panic!("zero delta"));
        assert_eq!(objects, initial);
        advance(&mut objects, Duration::from_secs(1), |_, _| {
            panic!("unsupported")
        });
        let expected_x_speed = 0.995_f64.powi(48);
        assert!((objects[0].velocity[0] - expected_x_speed).abs() < 1e-14);
        assert!(objects[0].position[1] < 96.0);
    }
}

#[cfg(test)]
mod terrain_tests {
    use super::*;
    fn ridge(_: SphereSurface, p: [f64; 3]) -> GroundSample {
        let height = 0.2 * (1.0 - p[0].abs()).max(0.0);
        let slope = if p[0].abs() < 1.0 {
            -0.2 * p[0].signum()
        } else {
            0.0
        };
        GroundSample {
            point: [p[0], height, p[2]],
            normal: normalize([-slope, 1.0, 0.0]),
        }
    }
    #[test]
    fn convex_terrain_contact_samples_vertices_and_sleep_uses_the_same_support() {
        let hull = Arc::new(ConvexHull::new([-0.5, 0.5].into_iter().flat_map(|x| {
            [-0.5, 0.5]
                .into_iter()
                .flat_map(move |y| [-0.5, 0.5].map(move |z| [x, y, z]))
        })));
        let mut p = ObjectState {
            position: [0.3, 0.5, 0.0],
            velocity: [0.0; 3],
            radius: 1.0,
            ground_support_meters: 0.5,
            surface: Surface::Sampled {
                body: SphereSurface {
                    center: [0.0, -1e9, 0.0],
                    radius: 1e9,
                },
                query: ridge,
                point_query: |body, p| ridge(body, p).point,
            },
            hull: Some(hull.clone()),
            side_meters: 1.0,
            orientation: [0.0, 0.0, 0.0, 1.0],
            angular_velocity: [0.0; 3],
            mass_kg: 1.0,
        };
        // Center tangent clearance misses the higher ridge under an actual corner.
        for _ in 0..8 {
            ground_contact(&mut p);
        }
        for vertex in &hull.vertices {
            let point = add(p.position, *vertex);
            let ground = ridge(
                SphereSurface {
                    center: [0.0; 3],
                    radius: 1.0,
                },
                point,
            );
            assert!(dot(sub(point, ground.point), ground.normal) >= -0.000001);
        }
        assert!(sampled_contact(&p).unwrap().penetration.abs() < 0.001);
    }
}
