//! Body-expanded hull, fixture and appendage collision.
use crate::layout::*;
use crate::thruster_collision::{THRUSTER_COLLIDERS, WING_COLLIDERS};
use crate::{PLAYER_BODY_HEIGHT_METERS, PLAYER_EYE_HEIGHT_METERS};
pub(crate) fn overlaps_exterior_hull(local: [f64; 3]) -> bool {
    local[1] + (PLAYER_BODY_HEIGHT_METERS - PLAYER_EYE_HEIGHT_METERS) > EXTERIOR_BOTTOM
        && local[1] - PLAYER_EYE_HEIGHT_METERS < EXTERIOR_TOP
}

pub(crate) fn world_round_trip_tolerance(world: [f64; 3]) -> f64 {
    // Conversion to/from a translated, rotated frame incurs world-coordinate
    // rounding. Allow a few ULPs when recognizing an existing jamb contact,
    // without widening the passable aperture or changing the stopping plane.
    let magnitude = world.into_iter().map(f64::abs).fold(0.0, f64::max);
    COLLISION_EPSILON.max(4.0 * f64::EPSILON * magnitude)
}

pub(crate) fn slide_around_fixtures(previous: [f64; 3], proposed: [f64; 3]) -> [f64; 3] {
    slide_around_obstacles(previous, proposed, &INTERIOR_OBSTACLES)
}

pub(crate) fn slide_around_appendages(
    previous: [f64; 3],
    mut proposed: [f64; 3],
    tolerance: f64,
) -> [f64; 3] {
    for [x_min, x_max, y_min, y_max, z_min, z_max] in
        THRUSTER_COLLIDERS.into_iter().chain(WING_COLLIDERS)
    {
        // Test the entire body, so a raised foot or a low surface approach
        // cannot cross a thruster merely because the eye is outside its box.
        let body_bottom = proposed[1].min(previous[1]) - PLAYER_EYE_HEIGHT_METERS;
        let body_top =
            proposed[1].max(previous[1]) + (PLAYER_BODY_HEIGHT_METERS - PLAYER_EYE_HEIGHT_METERS);
        if body_bottom < y_max && body_top > y_min {
            let obstacle = [[
                x_min - PLAYER_RADIUS_METERS,
                x_max + PLAYER_RADIUS_METERS,
                z_min - PLAYER_RADIUS_METERS,
                z_max + PLAYER_RADIUS_METERS,
            ]];
            proposed =
                slide_around_obstacles_with_tolerance(previous, proposed, &obstacle, tolerance);
        }
    }
    proposed
}

pub(crate) fn slide_around_obstacles(
    previous: [f64; 3],
    proposed: [f64; 3],
    obstacles: &[[f64; 4]],
) -> [f64; 3] {
    slide_around_obstacles_with_tolerance(previous, proposed, obstacles, COLLISION_EPSILON)
}

pub(crate) fn slide_around_obstacles_with_tolerance(
    previous: [f64; 3],
    mut proposed: [f64; 3],
    obstacles: &[[f64; 4]],
    tolerance: f64,
) -> [f64; 3] {
    // Resolve one planar axis at a time so diagonal input slides along the
    // fixtures instead of stopping the player or tunneling through a corner.
    for &[forward_min, forward_max, side_min, side_max] in obstacles {
        if previous[2] > side_min && previous[2] < side_max {
            proposed[0] = stop_at_obstacle(
                previous[0],
                proposed[0],
                forward_min,
                forward_max,
                tolerance,
            );
        }
    }
    for &[forward_min, forward_max, side_min, side_max] in obstacles {
        if proposed[0] > forward_min && proposed[0] < forward_max {
            proposed[2] = stop_at_obstacle(previous[2], proposed[2], side_min, side_max, tolerance);
        }
    }
    proposed
}

pub(crate) fn stop_at_obstacle(
    previous: f64,
    proposed: f64,
    minimum: f64,
    maximum: f64,
    tolerance: f64,
) -> f64 {
    if previous <= minimum + tolerance && proposed > minimum {
        minimum
    } else if previous >= maximum - tolerance && proposed < maximum {
        maximum
    } else {
        proposed
    }
}
