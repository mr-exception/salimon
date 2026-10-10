//! Portable small-object motion/contact in caller-supplied metre reference frames.
use salimon_math::{add, cross, dot, length, scale, sub};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

mod convex;
mod sleep;
pub use convex::{ConvexHull, compose, orientation_from_axes, rotate};
pub use sleep::SleepTracker;

const GRAVITY: f64 = 9.81;
const RESTITUTION: f64 = 0.12;
const MAX_STEP: f64 = 1.0 / 90.0;

/// Spherical ground geometry in the same frame as its objects.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SphereSurface {
    pub center: [f64; 3],
    pub radius: f64,
}

/// Environmental support; floor objects share one ship-local frame per call.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Surface {
    Floor {
        height_meters: f64,
    },
    Sphere(SphereSurface),
    /// Legacy unsupported objects still accelerate along -Y, without ground contact.
    Unsupported,
}
impl Surface {
    fn is_floor(self) -> bool {
        matches!(self, Self::Floor { .. })
    }
    fn sphere(self) -> Option<SphereSurface> {
        if let Self::Sphere(body) = self {
            Some(body)
        } else {
            None
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

/// Opt-in observations of the unchanged production solver, accumulated per call.
/// Counters count visits (including repeated solver passes), not unique pairs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepStats {
    pub objects: usize,
    pub awake_objects: usize,
    pub sleeping_objects: usize,
    pub integrated_objects: u64,
    pub substeps: u64,
    pub solver_passes: u64,
    pub pair_visits: u64,
    pub radius_candidates: u64,
    pub narrow_phase_tests: u64,
    pub contacts: u64,
    pub projection_builds: u64,
    /// Requested matrix/projection storage; excludes allocator overhead and temporaries.
    pub projection_storage_bytes: u64,
    /// Number of matrix Vec allocations for the current vec![vec![..]; n] implementation;
    /// One outer Vec plus one row Vec per object. Not a process allocator counter.
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
    let motion_step = pieces
        .iter()
        .filter(|p| p.hull.is_some())
        .map(|p| {
            (0.15 * p.side_meters
                / (length(p.velocity) + length(p.angular_velocity) * p.radius + 1.0))
                .min(MAX_STEP)
        })
        .fold(MAX_STEP, f64::min);
    let steps = if has_hulls {
        (elapsed / motion_step.max(1e-5)).ceil().clamp(1.0, 4096.0) as usize
    } else {
        (elapsed / MAX_STEP).ceil().clamp(1.0, 48.0) as usize
    };
    let dt = elapsed / steps as f64;
    for _ in 0..steps {
        if let Some(a) = &mut activation {
            a.impacts(pieces, dt);
        }
        let integration_start = PROFILE.then(Instant::now);
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
            piece.velocity = sub(piece.velocity, scale(up, GRAVITY * dt));
            piece.position = add(piece.position, scale(piece.velocity, dt));
            piece.orientation = convex::integrate(piece.orientation, piece.angular_velocity, dt);
            if piece.surface.is_floor() && !floor_contains(piece.position, piece.radius) {
                piece.position[0] = previous[0];
                piece.position[2] = previous[2];
                piece.velocity[0] *= -0.15;
                piece.velocity[2] *= -0.15;
            }
            ground_contact(piece);
        }
        if let Some(start) = integration_start {
            stats.integration_time += start.elapsed();
            stats.substeps += 1;
            if !pieces.is_empty() {
                stats.matrix_allocations += pieces.len() as u64 + 1;
            }
            stats.projection_storage_bytes += ((pieces.len() * pieces.len())
                * std::mem::size_of::<Option<Vec<convex::Projection>>>()
                + pieces.len() * std::mem::size_of::<Vec<Option<Vec<convex::Projection>>>>())
                as u64;
        }
        if let Some(a) = &mut activation {
            a.impacts(pieces, 0.0);
        }
        let contact_start = PROFILE.then(Instant::now);
        let mut pair_geometry: Vec<Vec<Option<Vec<convex::Projection>>>> =
            vec![vec![None; pieces.len()]; pieces.len()];
        for _ in 0..if has_hulls { 16 } else { 3 } {
            if PROFILE {
                stats.solver_passes += 1;
            }
            for (i, row) in pair_geometry.iter_mut().enumerate() {
                for (j, cached) in row.iter_mut().enumerate().skip(i + 1) {
                    if activation
                        .as_ref()
                        .is_some_and(|a| a.sleeping[i] && a.sleeping[j])
                    {
                        continue;
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
                            && a.surface.sphere().unwrap().center
                                != b.surface.sphere().unwrap().center)
                    {
                        continue;
                    }
                    let separation = sub(b.position, a.position);
                    let distance = length(separation);
                    let minimum = a.radius + b.radius;
                    if distance >= minimum {
                        continue;
                    }
                    if PROFILE {
                        stats.radius_candidates += 1;
                    }
                    let (normal, depth) = if a.hull.is_some() && b.hull.is_some() {
                        if PROFILE {
                            stats.narrow_phase_tests += 1;
                        }
                        if cached.is_none() && convex::separated_by_faces(a, b) {
                            continue;
                        }
                        let geometry = cached.get_or_insert_with(|| {
                            let projections = convex::projections(a, b);
                            if PROFILE {
                                stats.projection_builds += 1;
                                stats.projection_storage_bytes += (projections.capacity()
                                    * std::mem::size_of::<convex::Projection>())
                                    as u64;
                            }
                            projections
                        });
                        let Some(contact) = convex::projected_contact(geometry, separation) else {
                            continue;
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
                    if let Some(activation) = &mut activation {
                        activation.wake(i);
                        activation.wake(j);
                    }
                    let inverse_a = 1.0 / a.mass_kg;
                    let inverse_b = 1.0 / b.mass_kg;
                    let correction = scale(normal, (depth + 0.00001) / (inverse_a + inverse_b));
                    let old_a = a.position;
                    let old_b = b.position;
                    a.position = sub(a.position, scale(correction, inverse_a));
                    b.position = add(b.position, scale(correction, inverse_b));
                    if a.surface.is_floor() && !floor_contains(a.position, a.radius) {
                        a.position[0] = old_a[0];
                        a.position[2] = old_a[2];
                    }
                    if b.surface.is_floor() && !floor_contains(b.position, b.radius) {
                        b.position[0] = old_b[0];
                        b.position[2] = old_b[2];
                    }
                    pair_impulse(a, b, normal);
                    ground_contact(a);
                    ground_contact(b);
                }
            }
        }
        if let Some(start) = contact_start {
            stats.contact_time += start.elapsed();
        }
        for (i, piece) in pieces.iter_mut().enumerate() {
            if activation.as_ref().is_some_and(|a| a.sleeping[i]) {
                continue;
            }
            piece.velocity = scale(piece.velocity, 0.995);
            piece.angular_velocity = scale(piece.angular_velocity, 0.98);
            if length(piece.angular_velocity) < 0.015 {
                piece.angular_velocity = [0.0; 3];
            }
            if length(piece.velocity) < 0.015 {
                piece.velocity = [0.0; 3];
            }
        }
    }
    stats
}

fn ground_contact(piece: &mut ObjectState) {
    let normal = piece
        .surface
        .sphere()
        .map(|b| normalize(sub(piece.position, b.center)))
        .unwrap_or([0.0, 1.0, 0.0]);
    if let Some(hull) = &piece.hull {
        piece.ground_support_meters =
            hull.support(piece.orientation, piece.side_meters, scale(normal, -1.0));
    }
    let (up, penetration) = if let Surface::Floor { height_meters } = piece.surface {
        (
            [0.0, 1.0, 0.0],
            height_meters + piece.ground_support_meters - piece.position[1],
        )
    } else if let Some(body) = piece.surface.sphere() {
        let radial = sub(piece.position, body.center);
        let distance = length(radial);
        (
            normalize(radial),
            body.radius + piece.ground_support_meters - distance,
        )
    } else {
        return;
    };
    if penetration < -0.00001 {
        return;
    }
    piece.position = add(piece.position, scale(up, penetration.max(0.0)));
    if let Some(hull) = &piece.hull {
        let arm = hull.ground_arm(piece.orientation, piece.side_meters, scale(up, -1.0));
        let contact_velocity = add(piece.velocity, cross(piece.angular_velocity, arm));
        let approach = dot(contact_velocity, up);
        if approach < 0.0 {
            let denominator =
                1.0 / piece.mass_kg + dot(cross(arm, up), cross(arm, up)) * inverse_inertia(piece);
            apply_impulse(piece, arm, scale(up, -approach / denominator));
        }
        let tangent = sub(contact_velocity, scale(up, dot(contact_velocity, up)));
        let speed = length(tangent);
        if speed > 1e-9 {
            let direction = scale(tangent, 1.0 / speed);
            let denominator = 1.0 / piece.mass_kg
                + dot(cross(arm, direction), cross(arm, direction)) * inverse_inertia(piece);
            let friction = (speed / denominator).min(piece.mass_kg * GRAVITY * MAX_STEP * 0.65);
            apply_impulse(piece, arm, scale(direction, -friction));
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
fn pair_impulse(a: &mut ObjectState, b: &mut ObjectState, normal: [f64; 3]) {
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
        1.0 / a.mass_kg
            + 1.0 / b.mass_kg
            + dot(cross(aa, direction), cross(aa, direction)) * inverse_inertia(a)
            + dot(cross(ba, direction), cross(ba, direction)) * inverse_inertia(b)
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
