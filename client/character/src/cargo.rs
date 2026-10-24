//! Geometry of the authored cargo doorway and open bin shells.
use crate::{
    SHIP_FLOOR_HEIGHT_METERS,
    spatial_contracts::{CARGO_HEADER_BOUNDS, STARTER_COLLIDERS},
};

pub const CARGO_DOOR_TARGET: [f64; 3] = [
    (CARGO_HEADER_BOUNDS[0] + CARGO_HEADER_BOUNDS[1]) * 0.5,
    1.35,
    1.35,
];
/// Swept envelope of the opposed leaves on the cargo side of the bulkhead.
pub fn cargo_door_bounds(open: f64) -> [f64; 6] {
    let h = CARGO_HEADER_BOUNDS;
    let edge = 1.49 + open * 1.54;
    [
        h[0] - 0.075,
        h[0] - 0.015,
        SHIP_FLOOR_HEIGHT_METERS,
        h[2] - 0.01,
        -edge,
        edge,
    ]
}

pub fn sphere_overlaps_box(p: [f64; 3], radius: f64, b: [f64; 6]) -> bool {
    (0..3)
        .map(|i| (p[i] - p[i].clamp(b[2 * i], b[2 * i + 1])).powi(2))
        .sum::<f64>()
        <= radius * radius
}

/// Original Blender shells: 4 cm walls, floor 12 cm above the deck, open top.
pub fn cargo_bin_solids() -> [[f64; 6]; 10] {
    let mut solids = [[0.0; 6]; 10];
    for (index, b) in STARTER_COLLIDERS[2..4].iter().enumerate() {
        let [x0, x1, y0, y1, z0, z1] = *b;
        solids[index * 5..index * 5 + 5].copy_from_slice(&[
            [x0, x1, y0, y0 + 0.12, z0, z1],
            [x0, x0 + 0.04, y0, y1, z0, z1],
            [x1 - 0.04, x1, y0, y1, z0, z1],
            [x0 + 0.04, x1 - 0.04, y0, y1, z0, z0 + 0.04],
            [x0 + 0.04, x1 - 0.04, y0, y1, z1 - 0.04, z1],
        ]);
    }
    solids
}

/// Pure ship-local geometry query. No inventory slots or storage authority.
pub fn cargo_position_allowed(p: [f64; 3], radius: f64, door_open: f64) -> bool {
    if !radius.is_finite() || radius <= 0.0 || !p.iter().all(|v| v.is_finite()) {
        return false;
    }
    if door_open < 1.0 && sphere_overlaps_box(p, radius, cargo_door_bounds(0.0)) {
        return false;
    }
    // Preserve the ordinary hull/furniture admission away from the bins. The
    // old envelope intentionally blocks walking, but is hollow for loose cargo.
    let near_bin = STARTER_COLLIDERS[2..4].iter().any(|b| {
        p[0] + radius > b[0] && p[0] - radius < b[1] && p[2] + radius > b[4] && p[2] - radius < b[5]
    });
    if !near_bin {
        return crate::ship_floor_placement(p, radius).is_some();
    }
    let cabin = crate::spatial_contracts::CABIN_TRAVERSAL_BOUNDS;
    if p[0] - radius < cabin[0]
        || p[0] + radius > cabin[1]
        || p[2] - radius < cabin[4]
        || p[2] + radius > cabin[5]
    {
        return false;
    }
    // Bottom contact belongs to the solver's floor response. Side walls remain
    // solid below their rim, while an object above the open top can cross it.
    cargo_bin_solids()
        .into_iter()
        .enumerate()
        .all(|(i, b)| i % 5 == 0 || !sphere_overlaps_box(p, radius, b))
}

pub fn cargo_support_height(p: [f64; 3], radius: f64) -> f64 {
    for b in &STARTER_COLLIDERS[2..4] {
        if p[0] - radius >= b[0] + 0.04
            && p[0] + radius <= b[1] - 0.04
            && p[2] - radius >= b[4] + 0.04
            && p[2] + radius <= b[5] - 0.04
        {
            return b[2] + 0.12;
        }
    }
    SHIP_FLOOR_HEIGHT_METERS
}

/// Ray distance to the closed internal leaf; ordinary hull sight is queried separately.
pub fn cargo_sight_obstruction(eye: [f64; 3], target: [f64; 3]) -> Option<f64> {
    box_sight_obstruction(eye, target, cargo_door_bounds(0.0))
}

/// Shared exact slab query for installed physical fixture geometry.
pub fn box_sight_obstruction(eye: [f64; 3], target: [f64; 3], b: [f64; 6]) -> Option<f64> {
    let delta = salimon_math::sub(target, eye);
    let length = salimon_math::length(delta);
    if !length.is_finite() || length <= 0.0 {
        return None;
    }
    let direction = delta.map(|v| v / length);
    let mut near: f64 = 0.0;
    let mut far = f64::INFINITY;
    for i in 0..3 {
        if direction[i].abs() < 1e-12 {
            if eye[i] < b[2 * i] || eye[i] > b[2 * i + 1] {
                return None;
            }
        } else {
            let a = (b[2 * i] - eye[i]) / direction[i];
            let z = (b[2 * i + 1] - eye[i]) / direction[i];
            near = near.max(a.min(z));
            far = far.min(a.max(z));
        }
    }
    (far >= near).then_some(near)
}

/// Opposed leaves retract into the existing side bulkheads without exiting the hull.
pub fn cargo_door_leaves(open: f64) -> [[f64; 6]; 2] {
    let mut port = cargo_door_bounds(0.0);
    port[4] = open * 1.54;
    port[5] = 1.49 + open * 1.54;
    let mut starboard = port;
    starboard[4] = -port[5];
    starboard[5] = -port[4];
    [port, starboard]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn open_bins_admit_real_objects_only_through_the_top() {
        let p = [-6.1, 0.6, 3.45];
        assert!(cargo_position_allowed(p, 0.1, 1.0));
        assert!(!cargo_position_allowed([-6.1, 0.6, 2.86], 0.1, 1.0));
        assert!(cargo_position_allowed([-6.1, 1.4, 2.86], 0.1, 1.0));
        assert!((cargo_support_height(p, 0.1) - (STARTER_COLLIDERS[2][2] + 0.12)).abs() < 1e-12);
        assert!(!cargo_position_allowed(p, 0.7, 1.0));
    }
    #[test]
    fn cargo_leaf_is_independent_of_the_ramp_and_rejects_partial_open_crossings() {
        let p = [-4.79, 1.4, 0.0];
        assert!(cargo_position_allowed(p, 0.1, 1.0));
        for f in [0.0, 0.2, 0.99] {
            assert!(!cargo_position_allowed(p, 0.1, f));
        }
    }
}
