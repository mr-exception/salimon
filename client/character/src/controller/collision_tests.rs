//! Controller integration regressions for the collision proxies.
use super::test_support::*;
use super::*;
use crate::spatial_contracts::{THRUSTER_COLLIDERS, WING_COLLIDERS};
#[test]
fn each_thruster_blocks_all_planar_approaches_without_bridging_clear_space() {
    for collider in [THRUSTER_COLLIDERS[0], THRUSTER_COLLIDERS[2]] {
        let [x_min, x_max, _, _, z_min, z_max] = collider;
        let mid_x = (x_min + x_max) / 2.0;
        let mid_z = (z_min + z_max) / 2.0;
        let eye = 1.75;
        for (start, end, axis, contact) in [
            (
                [x_min - 1.0, eye, mid_z],
                [x_min + 1.0, eye, mid_z],
                0,
                x_min - PLAYER_RADIUS_METERS,
            ),
            (
                [x_max + 1.0, eye, mid_z],
                [x_max - 1.0, eye, mid_z],
                0,
                x_max + PLAYER_RADIUS_METERS,
            ),
            (
                [mid_x, eye, z_min - 1.0],
                [mid_x, eye, z_min + 1.0],
                2,
                z_min - PLAYER_RADIUS_METERS,
            ),
            (
                [mid_x, eye, z_max + 1.0],
                [mid_x, eye, z_max - 1.0],
                2,
                z_max + PLAYER_RADIUS_METERS,
            ),
        ] {
            let stopped = slide_around_appendages(start, end, COLLISION_EPSILON);
            assert!(
                (stopped[axis] - contact).abs() < 1.0e-9,
                "{collider:?}: {stopped:?}"
            );
        }
        // Beyond the fitted engine and its body radius, travel stays open.
        let clear_z = if mid_z > 0.0 {
            z_max + 0.5
        } else {
            z_min - 0.5
        };
        let start = [x_min - 1.0, eye, clear_z];
        let end = [x_min + 1.0, eye, clear_z];
        assert_eq!(slide_around_appendages(start, end, COLLISION_EPSILON), end);
    }
    let gate = [-10.0, 1.75, 0.0];
    let inward = [-9.0, 1.75, 0.0];
    assert_eq!(
        slide_around_appendages(gate, inward, COLLISION_EPSILON),
        inward
    );
    // A body fully above the raised fin has no phantom horizontal wall.
    let high = [THRUSTER_COLLIDERS[0][0] - 1.0, 5.0, 7.1];
    let beyond = [THRUSTER_COLLIDERS[0][0] + 1.0, 5.0, 7.1];
    assert_eq!(
        slide_around_appendages(high, beyond, COLLISION_EPSILON),
        beyond
    );
}
#[test]
fn both_wings_block_body_sweeps_but_allow_clearance_above() {
    for [x_min, x_max, _, y_max, z_min, z_max] in WING_COLLIDERS {
        let x = (x_min + x_max) / 2.0;
        let (start_z, end_z, stop_z) = if z_min > 0.0 {
            (z_max + 1.0, z_max - 1.0, z_max + PLAYER_RADIUS_METERS)
        } else {
            (z_min - 1.0, z_min + 1.0, z_min - PLAYER_RADIUS_METERS)
        };
        let start = [x, 1.75, start_z];
        let end = [x, 1.75, end_z];
        let stopped = slide_around_appendages(start, end, COLLISION_EPSILON);
        assert!((stopped[2] - stop_z).abs() < 1.0e-9);

        let high = [x, y_max + PLAYER_EYE_HEIGHT_METERS + 0.1, start_z];
        let high_end = [x, high[1], end_z];
        assert_eq!(
            slide_around_appendages(high, high_end, COLLISION_EPSILON),
            high_end
        );
    }
}
#[test]
fn landed_thrusters_follow_ship_frame_on_both_sides() {
    for (ship, surface) in [(frame(), surface()), rotated_catalog_frames()] {
        for z in [7.1, -7.1] {
            let start = ship.local_to_world([-11.0, 1.75, z]);
            let mut controller = CharacterController {
                position: PositionState::Surface {
                    world: project_eye_to_surface(start, surface),
                },
                ..CharacterController::default()
            };
            for _ in 0..35 {
                controller.advance(
                    Duration::from_millis(100),
                    MovementInput {
                        forward: true,
                        ..MovementInput::default()
                    },
                    ship,
                    surface,
                    false,
                    true,
                );
            }
            let local = ship.world_to_local(controller.snapshot(ship, surface).eye_position_meters);
            assert!(local[0] <= -9.8 - PLAYER_RADIUS_METERS + 0.002, "{local:?}");
            assert_eq!(controller.location(), CharacterLocation::Surface);
        }
    }
}
#[test]
fn surface_walkers_cannot_enter_through_the_hull_with_either_door_state() {
    let cases = [
        ([0.0, PLAYER_START[1], -6.0], FRAC_PI_2, 2, -5.34),
        ([0.0, PLAYER_START[1], 6.0], -FRAC_PI_2, 2, 5.34),
        (
            [DOORWAY_FORWARD, PLAYER_START[1], -6.0],
            FRAC_PI_2,
            2,
            -5.34,
        ),
        ([DOORWAY_FORWARD, PLAYER_START[1], 6.0], -FRAC_PI_2, 2, 5.34),
        ([12.0, PLAYER_START[1], 0.0], std::f64::consts::PI, 0, 8.77),
        ([-9.0, PLAYER_START[1], -3.0], 0.0, 0, -8.16),
        ([-9.0, PLAYER_START[1], 3.0], 0.0, 0, -8.16),
        ([-9.0, PLAYER_START[1], -1.17], 0.0, 0, -8.16),
        ([-9.0, PLAYER_START[1], 1.17], 0.0, 0, -8.16),
    ];
    for axes in [
        frame().axes,
        [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]],
        [[0.6, 0.8, 0.0], [-0.8, 0.6, 0.0], [0.0, 0.0, 1.0]],
    ] {
        let ship = ShipFrame {
            axes,
            origin_meters: [60_000_000.0, 7_000_000.0, -3_000_000.0],
        };
        let surface = SurfaceFrame {
            body_center_meters: sub(ship.origin_meters, scale(ship.axes[1], 6_000_000.0)),
            radius_meters: 6_000_000.0,
        };
        for door_open in [false, true] {
            for (start, yaw_radians, axis, contact) in cases {
                let mut controller = CharacterController {
                    position: PositionState::Surface {
                        world: project_eye_to_surface(ship.local_to_world(start), surface),
                    },
                    yaw_radians,
                    ..CharacterController::default()
                };
                for _ in 0..100 {
                    controller.advance(
                        Duration::from_millis(100),
                        MovementInput {
                            forward: true,
                            ..MovementInput::default()
                        },
                        ship,
                        surface,
                        door_open,
                        true,
                    );
                    assert_eq!(controller.location(), CharacterLocation::Surface);
                }
                let snapshot = controller.snapshot(ship, surface);
                let local = ship.world_to_local(snapshot.eye_position_meters);
                assert!(
                    (local[axis] - contact).abs() < 1.0e-7,
                    "hull approach {start:?}, door open {door_open}: {local:?}"
                );
                let radius = sub(snapshot.eye_position_meters, surface.body_center_meters);
                assert!(
                    (dot(radius, radius).sqrt() - surface.radius_meters - PLAYER_EYE_HEIGHT_METERS)
                        .abs()
                        < 1.0e-7
                );
            }
        }
    }
}
