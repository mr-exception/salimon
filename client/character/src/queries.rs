//! Ship-local floor placement and sight queries over traversal proxies.
use crate::layout::*;
use crate::thruster_collision::{THRUSTER_COLLIDERS, WING_COLLIDERS};
use salimon_math::{dot, scale, sub};
/// A conservative physical cube placement on the cabin and narrow nose deck.
/// Uses the traversal proxies; furniture and hull edges stay solid.
#[must_use]
pub fn ship_floor_placement(local: [f64; 3], half: f64) -> Option<[f64; 3]> {
    if !half.is_finite() || half <= 0.0 || !local.iter().all(|v| v.is_finite()) {
        return None;
    }
    let cabin = local[0] - half >= INTERIOR_FORWARD_MIN
        && local[0] + half <= CABIN_FORWARD_MAX
        && local[2].abs() + half <= INTERIOR_SIDE_LIMIT;
    let nose = local[0] - half >= INTERIOR_FORWARD_MIN
        && local[0] + half <= INTERIOR_FORWARD_MAX
        && local[2].abs() + half <= NOSE_SIDE_LIMIT;
    if !(cabin || nose) || SHIP_FLOOR_HEIGHT + half * 2.0 >= SHIP_CEILING_HEIGHT {
        return None;
    }
    if INTERIOR_OBSTACLES.iter().any(|b| {
        local[0] + half > b[0]
            && local[0] - half < b[1]
            && local[2] + half > b[2]
            && local[2] - half < b[3]
    }) {
        return None;
    }
    Some([local[0], SHIP_FLOOR_HEIGHT + half + 0.005, local[2]])
}

/// First ray hit on the same solid cabin, gate, and engine proxies used for traversal.
/// Inputs are ship-local; the returned distance is in meters along a unit ray.
#[must_use]
pub fn ship_sight_obstruction(eye: [f64; 3], target: [f64; 3], door_open: bool) -> Option<f64> {
    let direction = sub(target, eye);
    let length = dot(direction, direction).sqrt();
    if !length.is_finite() || length <= 0.0 || !eye.iter().all(|v| v.is_finite()) {
        return None;
    }
    let direction = scale(direction, 1.0 / length);
    let mut boxes: Vec<[f64; 6]> = EXTERIOR_OBSTACLES
        .iter()
        .map(|b| {
            [
                b[0] + PLAYER_RADIUS_METERS,
                b[1] - PLAYER_RADIUS_METERS,
                EXTERIOR_BOTTOM,
                EXTERIOR_TOP,
                b[2] + PLAYER_RADIUS_METERS,
                b[3] - PLAYER_RADIUS_METERS,
            ]
        })
        .collect();
    if !door_open {
        let b = CLOSED_GATE_OBSTACLE[0];
        boxes.push([
            b[0] + PLAYER_RADIUS_METERS,
            b[1] - PLAYER_RADIUS_METERS,
            EXTERIOR_BOTTOM,
            EXTERIOR_TOP,
            b[2] + PLAYER_RADIUS_METERS,
            b[3] - PLAYER_RADIUS_METERS,
        ]);
    }
    let inside_cabin = eye[0] >= INTERIOR_FORWARD_MIN
        && eye[0] <= CABIN_FORWARD_MAX
        && eye[2].abs() <= INTERIOR_SIDE_LIMIT;
    let inside_nose = eye[0] >= INTERIOR_FORWARD_MIN
        && eye[0] <= INTERIOR_FORWARD_MAX
        && eye[2].abs() <= NOSE_SIDE_LIMIT;
    if (inside_cabin || inside_nose) && eye[1] >= SHIP_FLOOR_HEIGHT && eye[1] <= SHIP_CEILING_HEIGHT
    {
        boxes.clear();
        boxes.extend(INTERIOR_OBSTACLES.iter().map(|b| {
            [
                b[0],
                b[1],
                SHIP_FLOOR_HEIGHT,
                SHIP_CEILING_HEIGHT,
                b[2],
                b[3],
            ]
        }));
        // The shared deck and ceiling occlude objects on the other side.
        for (x0, x1, z0, z1) in [
            (
                INTERIOR_FORWARD_MIN,
                CABIN_FORWARD_MAX,
                -INTERIOR_SIDE_LIMIT,
                INTERIOR_SIDE_LIMIT,
            ),
            (
                CABIN_FORWARD_MAX,
                INTERIOR_FORWARD_MAX,
                -NOSE_SIDE_LIMIT,
                NOSE_SIDE_LIMIT,
            ),
        ] {
            boxes.push([x0, x1, EXTERIOR_BOTTOM, SHIP_FLOOR_HEIGHT, z0, z1]);
            boxes.push([x0, x1, SHIP_CEILING_HEIGHT, EXTERIOR_TOP, z0, z1]);
        }
        boxes.extend([
            [
                INTERIOR_FORWARD_MAX,
                EXTERIOR_FORWARD,
                EXTERIOR_BOTTOM,
                EXTERIOR_TOP,
                -EXTERIOR_SIDE,
                EXTERIOR_SIDE,
            ],
            [
                INTERIOR_FORWARD_MIN,
                2.5,
                EXTERIOR_BOTTOM,
                EXTERIOR_TOP,
                -EXTERIOR_SIDE,
                -INTERIOR_SIDE_LIMIT,
            ],
            [
                INTERIOR_FORWARD_MIN,
                INTERIOR_FORWARD_MAX,
                EXTERIOR_BOTTOM,
                EXTERIOR_TOP,
                INTERIOR_SIDE_LIMIT,
                5.1,
            ],
            [
                EXTERIOR_AFT,
                INTERIOR_FORWARD_MIN,
                EXTERIOR_BOTTOM,
                EXTERIOR_TOP,
                -EXTERIOR_SIDE,
                -DOORWAY_SIDE_LIMIT,
            ],
            [
                EXTERIOR_AFT,
                INTERIOR_FORWARD_MIN,
                EXTERIOR_BOTTOM,
                EXTERIOR_TOP,
                DOORWAY_SIDE_LIMIT,
                EXTERIOR_SIDE,
            ],
        ]);
        if !door_open {
            boxes.push([
                EXTERIOR_AFT,
                INTERIOR_FORWARD_MIN,
                EXTERIOR_BOTTOM,
                EXTERIOR_TOP,
                -DOORWAY_SIDE_LIMIT,
                DOORWAY_SIDE_LIMIT,
            ]);
        }
    }
    boxes.extend(THRUSTER_COLLIDERS);
    boxes.extend(WING_COLLIDERS);
    boxes
        .into_iter()
        .filter_map(|b| {
            let mut near: f64 = 0.0;
            let mut far = f64::INFINITY;
            for axis in 0..3 {
                let low = b[axis * 2];
                let high = b[axis * 2 + 1];
                if direction[axis].abs() < 1e-12 {
                    if eye[axis] < low || eye[axis] > high {
                        return None;
                    }
                } else {
                    let a = (low - eye[axis]) / direction[axis];
                    let z = (high - eye[axis]) / direction[axis];
                    near = near.max(a.min(z));
                    far = far.min(a.max(z));
                }
            }
            (far >= near).then_some(near)
        })
        .min_by(f64::total_cmp)
}
#[cfg(test)]
mod sight_tests {
    use super::ship_sight_obstruction;

    #[test]
    fn sight_respects_solid_ship_gate_and_engines() {
        assert!(ship_sight_obstruction([-10.0, 1.0, 0.0], [-9.0, 1.0, 0.0], false).unwrap() < 4.0);
        assert!(ship_sight_obstruction([-10.0, 1.0, 0.0], [-9.0, 1.0, 0.0], true).unwrap() < 4.0);
        // Open aperture itself is clear until the solid cabin farther ahead.
        assert!(ship_sight_obstruction([-10.0, 1.0, 0.0], [-9.0, 1.0, 0.0], true).unwrap() > 2.0);
        assert!(ship_sight_obstruction([-10.0, 1.0, 7.0], [-9.0, 1.0, 7.0], true).unwrap() < 1.0);
        assert!(ship_sight_obstruction([-10.0, 4.0, 0.0], [-9.0, 4.0, 0.0], false).is_none());
        assert!(ship_sight_obstruction([-10.0, 1.0, 0.0], [-11.0, 1.0, 0.0], false).is_none());
    }
}
