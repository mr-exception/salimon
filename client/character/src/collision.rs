//! Body-expanded hull, fixture and appendage collision.
use crate::layout::*;
use crate::spatial_contracts::{LANDING_COLLIDERS, THRUSTER_COLLIDERS, WING_COLLIDERS};
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

#[cfg(test)]
mod starter_layout_tests {
    use super::*;

    #[test]
    fn cargo_bulkhead_blocks_side_aisles_but_central_carrying_route_is_clear() {
        let y = SHIP_FLOOR_HEIGHT + PLAYER_EYE_HEIGHT_METERS;
        for z in [-2.2, 2.2] {
            let result = slide_around_fixtures([-3.8, y, z], [-7.0, y, z]);
            assert!(result[0] > -4.5, "bulkhead must block the entire body");
        }
        assert_eq!(
            slide_around_fixtures([-3.8, y, 0.0], [-7.0, y, 0.0]),
            [-7.0, y, 0.0]
        );
        assert_eq!(
            slide_around_fixtures([-7.0, y, 0.0], [-3.8, y, 0.0]),
            [-3.8, y, 0.0]
        );
    }

    #[test]
    fn starter_fixtures_reject_cargo_placement_but_preserve_service_aisles() {
        for bounds in crate::spatial_contracts::STARTER_COLLIDERS {
            let center = [
                (bounds[0] + bounds[1]) * 0.5,
                SHIP_FLOOR_HEIGHT,
                (bounds[4] + bounds[5]) * 0.5,
            ];
            assert!(crate::ship_floor_placement(center, 0.1).is_none());
        }
        for x in [-6.5, -5.2, -3.8] {
            assert!(crate::ship_floor_placement([x, SHIP_FLOOR_HEIGHT, 0.0], 0.4).is_some());
        }
        for z in [-2.2, 2.2] {
            assert!(crate::ship_floor_placement([0.5, SHIP_FLOOR_HEIGHT, z], 0.2).is_some());
        }
    }
}

pub(crate) fn slide_around_appendages(
    previous: [f64; 3],
    mut proposed: [f64; 3],
    tolerance: f64,
) -> [f64; 3] {
    for [x_min, x_max, y_min, y_max, z_min, z_max] in THRUSTER_COLLIDERS
        .into_iter()
        .chain(WING_COLLIDERS)
        .chain(LANDING_COLLIDERS)
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

// Reach the raised aft threshold through the existing gravity transition only
// within the narrow gate. Side hulls retain their finite authored height.
pub(crate) fn overlaps_boarding_gate(local: [f64; 3]) -> bool {
    local[1] + (PLAYER_BODY_HEIGHT_METERS - PLAYER_EYE_HEIGHT_METERS) > LANDING_COLLIDERS[0][2]
        && local[1] - PLAYER_EYE_HEIGHT_METERS < DOORWAY_CEILING_HEIGHT
}

#[cfg(test)]
mod landing_tests {
    use super::*;

    #[test]
    fn landing_supports_block_surface_bodies_and_leave_under_hull_and_gate_clear() {
        for [xmin, xmax, ymin, _ymax, zmin, zmax] in LANDING_COLLIDERS.into_iter().step_by(2) {
            let y = ymin + PLAYER_EYE_HEIGHT_METERS;
            let z = (zmin + zmax) * 0.5;
            let before = [xmin - 1.0, y, z];
            let after = slide_around_appendages(before, [xmax + 1.0, y, z], COLLISION_EPSILON);
            assert!((after[0] - (xmin - PLAYER_RADIUS_METERS)).abs() < 1e-9);
            let above = 100.0;
            let start = [xmin - 1.0, above, z];
            let end = [xmax + 1.0, above, z];
            assert_eq!(slide_around_appendages(start, end, COLLISION_EPSILON), end);
        }
        let start = [-10.0, -0.35, 0.0];
        let end = [-7.0, -0.35, 0.0];
        assert_eq!(slide_around_appendages(start, end, COLLISION_EPSILON), end);
        assert!(!overlaps_exterior_hull(start));
        assert!(overlaps_boarding_gate(start));
    }
}
