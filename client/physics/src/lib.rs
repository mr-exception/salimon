//! Portable small-object motion/contact in caller-supplied metre reference frames.
use salimon_math::{add, cross, dot, length, scale, sub};
use std::time::Duration;

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
/// Identity, orientation, material and mass stay with the caller.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObjectState {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub radius: f64,
    pub surface: Surface,
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

/// New-piece ejection. Caller supplies a stable variant (fragment ID modulo 3).
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

/// Advance in stable slice order; preserve that order across updates.
/// `floor_contains` is a pure geometry query in ship-local metres, not a response rule.
/// Separate floor frames must be advanced in separate calls. Sphere centers distinguish
/// planetary contact groups. Substeps retain the legacy 1/90 s target and 48-step cap;
/// an unusually large delta can exceed that target. No time is discarded.
pub fn advance(
    pieces: &mut [ObjectState],
    delta: Duration,
    floor_contains: impl Fn([f64; 3], f64) -> bool,
) {
    let elapsed = delta.as_secs_f64();
    if elapsed <= 0.0 {
        return;
    }
    // A fixed maximum substep prevents fast ejected pieces from tunnelling through a pile.
    let steps = (elapsed / MAX_STEP).ceil().clamp(1.0, 48.0) as usize;
    let dt = elapsed / steps as f64;
    for _ in 0..steps {
        for piece in pieces.iter_mut() {
            let previous = piece.position;
            let up = piece
                .surface
                .sphere()
                .map(|body| normalize(sub(piece.position, body.center)))
                .unwrap_or([0.0, 1.0, 0.0]);
            piece.velocity = sub(piece.velocity, scale(up, GRAVITY * dt));
            piece.position = add(piece.position, scale(piece.velocity, dt));
            if piece.surface.is_floor() && !floor_contains(piece.position, piece.radius) {
                piece.position[0] = previous[0];
                piece.position[2] = previous[2];
                piece.velocity[0] *= -0.15;
                piece.velocity[2] *= -0.15;
            }
            ground_contact(piece);
        }
        for _ in 0..3 {
            for i in 0..pieces.len() {
                for j in i + 1..pieces.len() {
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
                    let normal = if distance > 1e-9 {
                        scale(separation, 1.0 / distance)
                    } else {
                        [0.0, 1.0, 0.0]
                    };
                    let correction = scale(normal, (minimum - distance + 0.0001) * 0.5);
                    let old_a = a.position;
                    let old_b = b.position;
                    a.position = sub(a.position, correction);
                    b.position = add(b.position, correction);
                    if a.surface.is_floor() && !floor_contains(a.position, a.radius) {
                        a.position[0] = old_a[0];
                        a.position[2] = old_a[2];
                    }
                    if b.surface.is_floor() && !floor_contains(b.position, b.radius) {
                        b.position[0] = old_b[0];
                        b.position[2] = old_b[2];
                    }
                    let approach = dot(sub(b.velocity, a.velocity), normal);
                    if approach < 0.0 {
                        let impulse = -(1.0 + RESTITUTION) * approach * 0.5;
                        a.velocity = sub(a.velocity, scale(normal, impulse));
                        b.velocity = add(b.velocity, scale(normal, impulse));
                    }
                    ground_contact(a);
                    ground_contact(b);
                }
            }
        }
        for piece in pieces.iter_mut() {
            piece.velocity = scale(piece.velocity, 0.995);
            if length(piece.velocity) < 0.015 {
                piece.velocity = [0.0; 3];
            }
        }
    }
}

fn ground_contact(piece: &mut ObjectState) {
    let (up, penetration) = if let Surface::Floor { height_meters } = piece.surface {
        (
            [0.0, 1.0, 0.0],
            height_meters + piece.radius - piece.position[1],
        )
    } else if let Some(body) = piece.surface.sphere() {
        let radial = sub(piece.position, body.center);
        let distance = length(radial);
        (normalize(radial), body.radius + piece.radius - distance)
    } else {
        return;
    };
    if penetration <= 0.0 {
        return;
    }
    piece.position = add(piece.position, scale(up, penetration));
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
            surface: Surface::Floor { height_meters: 0.0 },
        }
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
        let initial = [floor([0.0, 0.5, 0.0], [0.0; 3]); 2];
        let mut a = initial;
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
            ..sphere
        };
        let mut objects = [sphere, other, floor(sphere.position, sphere.velocity)];
        let mut isolated = objects;
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
        let initial = objects;
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
