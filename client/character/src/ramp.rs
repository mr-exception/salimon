//! Authored ramp support and conservative moving-leaf clearance in ship axes.
use crate::layout::PLAYER_RADIUS_METERS;
use crate::ramp_geometry::{
    RAMP_ANGLE_RADIANS, RAMP_HALF_WIDTH_METERS, RAMP_HINGE, RAMP_LENGTH_METERS,
};
use crate::{PLAYER_BODY_HEIGHT_METERS, PLAYER_EYE_HEIGHT_METERS};

pub(crate) fn tip_x() -> f64 {
    RAMP_HINGE[0] - RAMP_LENGTH_METERS * RAMP_ANGLE_RADIANS.sin()
}
pub(crate) fn side_limit() -> f64 {
    RAMP_HALF_WIDTH_METERS - PLAYER_RADIUS_METERS
}
pub(crate) fn floor_at(x: f64) -> f64 {
    let distance = (RAMP_HINGE[0] - x).max(0.0);
    RAMP_HINGE[1] + distance * RAMP_ANGLE_RADIANS.cos() / RAMP_ANGLE_RADIANS.sin()
}

/// True when a complete player body intersects the deployment envelope. Runtime
/// uses this for both toggle interlocks and traversal during the animation.
#[must_use]
pub fn boarding_ramp_occupied(local_eye: [f64; 3]) -> bool {
    local_eye[0] >= tip_x() - PLAYER_RADIUS_METERS
        && local_eye[0] <= RAMP_HINGE[0] + PLAYER_RADIUS_METERS
        && local_eye[2].abs() <= RAMP_HALF_WIDTH_METERS + PLAYER_RADIUS_METERS
        && local_eye[1] - PLAYER_EYE_HEIGHT_METERS <= RAMP_HINGE[1] + RAMP_LENGTH_METERS
        && local_eye[1] + PLAYER_BODY_HEIGHT_METERS - PLAYER_EYE_HEIGHT_METERS
            >= floor_at(tip_x()) - 0.2
}

#[must_use]
pub fn boarding_ramp_tip_meters() -> [f64; 3] {
    [tip_x(), floor_at(tip_x()), 0.0]
}

/// Reserve the player's expanded deployment AABB while the leaf/tray move.
/// Slab intersection prevents a fast 3D EVA step tunneling into the sweep.
pub(crate) fn resolve_moving_sweep(previous: [f64; 3], proposed: [f64; 3]) -> [f64; 3] {
    let lower = [
        tip_x() - PLAYER_RADIUS_METERS,
        floor_at(tip_x()) - (PLAYER_BODY_HEIGHT_METERS - PLAYER_EYE_HEIGHT_METERS) - 0.2,
        -RAMP_HALF_WIDTH_METERS - PLAYER_RADIUS_METERS,
    ];
    let upper = [
        RAMP_HINGE[0] + PLAYER_RADIUS_METERS,
        RAMP_HINGE[1] + RAMP_LENGTH_METERS - crate::ramp_geometry::RAMP_EXTENSION_METERS
            + PLAYER_EYE_HEIGHT_METERS,
        RAMP_HALF_WIDTH_METERS + PLAYER_RADIUS_METERS,
    ];
    if (0..3).all(|i| previous[i] > lower[i] && previous[i] < upper[i]) {
        // Environmental frame changes may start a sample in the reservation;
        // permit escape instead of trapping that player inside the volume.
        return proposed;
    }
    let mut entry = 0.0_f64;
    let mut exit = 1.0_f64;
    let mut blocked_axis = None;
    for i in 0..3 {
        let delta = proposed[i] - previous[i];
        if delta.abs() < 1e-12 {
            if previous[i] <= lower[i] || previous[i] >= upper[i] {
                return proposed;
            }
        } else {
            let a = (lower[i] - previous[i]) / delta;
            let b = (upper[i] - previous[i]) / delta;
            let near = a.min(b);
            if near >= entry {
                entry = near;
                blocked_axis = Some(i);
            }
            exit = exit.min(a.max(b));
        }
    }
    if entry > exit || !(0.0..=1.0).contains(&entry) {
        return proposed;
    }
    let Some(axis) = blocked_axis else {
        return proposed;
    };
    let mut resolved = proposed;
    resolved[axis] = previous[axis] + (proposed[axis] - previous[axis]) * entry;
    resolved
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn moving_sweep_blocks_fast_eva_and_vertical_entry_but_allows_escape() {
        let x = (tip_x() + RAMP_HINGE[0]) / 2.0;
        let from = [x, 8.0, 0.0];
        let to = [x, -5.0, 0.0];
        let resolved = resolve_moving_sweep(from, to);
        assert!(resolved[1] > RAMP_HINGE[1]);
        assert_eq!(
            resolve_moving_sweep([tip_x() - 10.0, 1.0, 0.0], [0.0, 1.0, 0.0])[0],
            tip_x() - PLAYER_RADIUS_METERS
        );
        assert_eq!(resolve_moving_sweep([x, 1.0, 0.0], from), from);
        assert_eq!(
            resolve_moving_sweep([x, 8.0, 3.0], [x, -5.0, 3.0]),
            [x, -5.0, 3.0]
        );
    }
}
