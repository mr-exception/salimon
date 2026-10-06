//! Ship-local walking, floor gravity and instant cockpit transitions.
use super::*;

impl CharacterController {
    pub(super) fn advance_interior(&mut self, mut local: [f64; 3], step: &Step) {
        let Step {
            seconds,
            input,
            ship,
            ship_velocity,
            door_open,
            ship_landed,
            doorway_passable,
            jump_started,
            ..
        } = *step;

        let previous = local;
        let movement = ship_planar_movement(input, self.yaw_radians);
        local[0] += movement[0] * WALK_SPEED_METERS_PER_SECOND * seconds;
        local[2] += movement[1] * WALK_SPEED_METERS_PER_SECOND * seconds;
        if jump_started && local[1] <= PLAYER_EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT + 0.001 {
            self.vertical_speed = JUMP_SPEED_METERS_PER_SECOND;
        }
        self.vertical_speed -= FIXED_GRAVITY_METERS_PER_SECOND_SQUARED * seconds;
        local[1] += self.vertical_speed * seconds;
        if local[1] <= PLAYER_EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT {
            local[1] = PLAYER_EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT;
            self.vertical_speed = 0.0;
        }
        local[2] = local[2].clamp(-INTERIOR_SIDE_LIMIT, INTERIOR_SIDE_LIMIT);
        local[0] = local[0].min(INTERIOR_FORWARD_MAX);
        local = slide_around_fixtures(previous, local);
        let ceiling = if local[0] < -6.88 && local[2].abs() < 1.84 {
            DOORWAY_CEILING_HEIGHT
        } else {
            SHIP_CEILING_HEIGHT
        };
        let maximum_eye = ceiling - (PLAYER_BODY_HEIGHT_METERS - PLAYER_EYE_HEIGHT_METERS);
        if local[1] > maximum_eye {
            local[1] = maximum_eye;
            self.vertical_speed = self.vertical_speed.min(0.0);
        }
        let within_doorway = local[2].abs() <= DOORWAY_SIDE_LIMIT;
        if local[0] < DOORWAY_FORWARD && within_doorway && door_open && !ship_landed {
            self.vertical_speed = 0.0;
            self.eva_inherited_velocity = ship_velocity;
            self.eva_control_velocity = [0.0; 3];
            self.eva_axes = ship.axes;
            self.nearby_surface = None;
            self.position = PositionState::Space {
                world: ship.local_to_world(local),
            };
        } else if local[0] < DOORWAY_FORWARD && within_doorway && doorway_passable {
            self.position = PositionState::Doorway {
                world: ship.local_to_world(local),
                elapsed: Duration::ZERO,
                leaving_ship: true,
            };
        } else {
            let aft_limit = if within_doorway {
                INTERIOR_FORWARD_MIN
            } else {
                AFT_WALL_FORWARD_MIN
            };
            local[0] = local[0].max(aft_limit);
            self.position = PositionState::Inside { local };
        }
    }
}

impl CharacterController {
    pub fn enter_cockpit(&mut self) {
        if matches!(self.position, PositionState::Inside { .. }) {
            self.position = PositionState::Cockpit;
            self.vertical_speed = 0.0;
        }
    }

    pub fn leave_cockpit(&mut self) {
        if matches!(self.position, PositionState::Cockpit) {
            self.position = PositionState::Inside {
                local: PLAYER_START,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use crate::ship_floor_placement;
    #[test]
    fn starts_inside_landed_ship_and_cockpit_transition_is_instant() {
        let mut controller = CharacterController::default();
        assert_eq!(controller.location(), CharacterLocation::InsideShip);
        controller.enter_cockpit();
        assert_eq!(controller.location(), CharacterLocation::Cockpit);
        controller.leave_cockpit();
        assert_eq!(controller.location(), CharacterLocation::InsideShip);
    }

    #[test]
    fn anchors_and_walk_bounds_match_the_shorter_ship() {
        assert_eq!(PLAYER_BODY_HEIGHT_METERS, 1.80);
        assert_eq!(PLAYER_EYE_HEIGHT_METERS, 1.75);
        for (actual, expected) in
            PLAYER_START
                .into_iter()
                .zip([0.50, 1.997_311_827_956_989_2, -2.20])
        {
            assert!((actual - expected).abs() < 1.0e-6);
        }
        assert!((COCKPIT_POSITION[0] - 2.76).abs() < 1.0e-6);
        assert!((COCKPIT_POSITION[1] - 1.799_032_258_064_516).abs() < 1.0e-6);
        assert!(COCKPIT_POSITION[2].abs() < 1.0e-12);
        assert_eq!(SHIP_FLOOR_HEIGHT, 0.247_311_827_956_989_25);
        assert_eq!(INTERIOR_FORWARD_MIN, -7.24);
        assert!((CABIN_FORWARD_MAX - 6.32).abs() < 1.0e-12);
        assert!((INTERIOR_FORWARD_MAX - 7.61).abs() < 1.0e-12);
        assert!((NOSE_SIDE_LIMIT - 0.86).abs() < 1.0e-12);
        assert!((INTERIOR_SIDE_LIMIT - 3.96).abs() < 1.0e-12);
        assert!((EXTERIOR_FORWARD - 8.77).abs() < 1.0e-12);
        assert!((COCKPIT_CONSOLE_OBSTACLE[2] + 1.59).abs() < 1.0e-12);
        assert_eq!(DOORWAY_FORWARD, -7.04);
    }

    #[test]
    fn cockpit_exit_returns_to_clear_aisle_without_a_position_snap() {
        let mut controller = CharacterController::default();
        controller.enter_cockpit();
        controller.leave_cockpit();
        assert_eq!(controller.local_ship_position(), Some(PLAYER_START));
        walk_steps(&mut controller, MovementInput::default(), 1);
        assert_eq!(controller.local_ship_position(), Some(PLAYER_START));
    }

    #[test]
    fn center_core_blocks_approaches_from_all_four_sides() {
        for (x, z, input, expected_x, expected_z) in [
            (
                -3.0,
                0.0,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                CORE_FORWARD_MIN,
                0.0,
            ),
            (
                1.2,
                0.0,
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
                CORE_FORWARD_MAX,
                0.0,
            ),
            (
                -1.0,
                -2.0,
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
                -1.0,
                -CORE_SIDE_LIMIT,
            ),
            (
                -1.0,
                2.0,
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
                -1.0,
                CORE_SIDE_LIMIT,
            ),
        ] {
            let mut controller = inside_at(x, z);
            walk_steps(&mut controller, input, 20);
            let local = controller.local_ship_position().unwrap();
            assert_eq!(local[0], expected_x);
            assert_eq!(local[2], expected_z);
        }
    }

    #[test]
    fn diagonal_movement_slides_along_core_and_can_round_its_corner() {
        let mut controller = inside_at(CORE_FORWARD_MIN, 0.0);
        let input = MovementInput {
            forward: true,
            right: true,
            ..MovementInput::default()
        };
        walk_steps(&mut controller, input, 3);
        let alongside = controller.local_ship_position().unwrap();
        assert_eq!(alongside[0], CORE_FORWARD_MIN);
        assert!(alongside[2] > 0.7);
        walk_steps(&mut controller, input, 8);
        let around_corner = controller.local_ship_position().unwrap();
        assert!(around_corner[0] > CORE_FORWARD_MIN + 1.0);
        assert!(around_corner[2] > CORE_SIDE_LIMIT);
    }

    #[test]
    fn both_side_aisles_reach_the_nose_shoulder() {
        for side in [-2.4, 2.4] {
            let mut controller = inside_at(-6.0, side);
            walk_steps(
                &mut controller,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                45,
            );
            let local = controller.local_ship_position().unwrap();
            assert_eq!(local[0], CABIN_FORWARD_MAX);
            assert!(local[0] > 1.30, "the old broad cockpit exclusion is gone");
            assert_eq!(local[2], side);
        }
    }

    #[test]
    fn cockpit_routes_pass_the_chair_then_stop_at_the_center_console() {
        for side in [-1.20, 1.20] {
            let mut controller = inside_at(0.50, side);
            walk_steps(
                &mut controller,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                20,
            );

            let local = controller.local_ship_position().unwrap();
            assert_eq!(local[0], COCKPIT_CONSOLE_OBSTACLE[0]);
            assert!(local[0] > COCKPIT_CHAIR_OBSTACLE[1]);
            assert_eq!(local[2], side);
        }
    }

    #[test]
    fn cockpit_chair_blocks_walkers_from_every_planar_direction() {
        let [forward_min, forward_max, side_min, side_max] = COCKPIT_CHAIR_OBSTACLE;
        for (x, z, input, expected_x, expected_z) in [
            (
                0.5,
                0.0,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                forward_min,
                0.0,
            ),
            (
                3.8,
                0.0,
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
                forward_max,
                0.0,
            ),
            (
                2.5,
                -1.5,
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
                2.5,
                side_min,
            ),
            (
                2.5,
                1.5,
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
                2.5,
                side_max,
            ),
        ] {
            let mut controller = inside_at(x, z);
            walk_steps(&mut controller, input, 20);
            let local = controller.local_ship_position().unwrap();
            assert_eq!(local[0], expected_x);
            assert_eq!(local[2], expected_z);
        }
    }

    #[test]
    fn unified_cockpit_console_blocks_all_three_monitor_positions() {
        for z in [-0.915, 0.075, 0.8025] {
            let mut controller = inside_at(3.56, z);
            walk_steps(
                &mut controller,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                20,
            );
            assert_eq!(
                controller.local_ship_position().unwrap()[0],
                COCKPIT_CONSOLE_OBSTACLE[0]
            );
        }
    }

    #[test]
    fn unified_console_blocks_walking_and_jumping_before_its_front_face() {
        for jump in [false, true] {
            let mut controller = inside_at(3.55, -0.5);
            walk_steps(
                &mut controller,
                MovementInput {
                    forward: true,
                    jump,
                    ..MovementInput::default()
                },
                20,
            );
            let eye = controller.local_ship_position().unwrap();
            assert!((eye[0] + PLAYER_RADIUS_METERS - 3.90).abs() < 1.0e-9);
        }
    }

    #[test]
    fn cockpit_collision_slides_around_the_chair_and_allows_retreat() {
        let mut controller = inside_at(COCKPIT_CHAIR_OBSTACLE[0], 0.0);
        walk_steps(
            &mut controller,
            MovementInput {
                forward: true,
                right: true,
                ..MovementInput::default()
            },
            8,
        );
        let around_corner = controller.local_ship_position().unwrap();
        assert!(around_corner[0] > COCKPIT_CHAIR_OBSTACLE[0]);
        assert!(around_corner[2] > COCKPIT_CHAIR_OBSTACLE[3]);

        walk_steps(
            &mut controller,
            MovementInput {
                backward: true,
                ..MovementInput::default()
            },
            3,
        );
        assert!(controller.local_ship_position().unwrap()[0] < around_corner[0]);
    }

    #[test]
    fn cockpit_side_hull_blocks_bypassing_the_consoles() {
        for (side, outward, hull, expected_side) in [
            (
                1.0,
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
                COCKPIT_PORT_HULL_OBSTACLE,
                COCKPIT_PORT_HULL_OBSTACLE[2],
            ),
            (
                -1.0,
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
                COCKPIT_STARBOARD_HULL_OBSTACLE,
                COCKPIT_STARBOARD_HULL_OBSTACLE[3],
            ),
        ] {
            let mut across_hull = inside_at(2.0, side * 3.2);
            walk_steps(
                &mut across_hull,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                10,
            );
            assert_eq!(across_hull.local_ship_position().unwrap()[0], hull[0]);

            let mut around_console = inside_at(3.0, side * 2.4);
            walk_steps(&mut around_console, outward, 10);
            assert_eq!(
                around_console.local_ship_position().unwrap()[2],
                expected_side
            );
        }
    }

    #[test]
    fn interior_walls_and_short_nose_keep_the_complete_player_inside() {
        for (start, input, axis, expected) in [
            (
                [-6.5, 0.0],
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
                2,
                INTERIOR_SIDE_LIMIT,
            ),
            (
                [1.0, 0.0],
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
                2,
                -INTERIOR_SIDE_LIMIT,
            ),
            (
                [5.0, 1.4],
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                0,
                CABIN_FORWARD_MAX,
            ),
            (
                [6.7, 0.0],
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                0,
                INTERIOR_FORWARD_MAX,
            ),
        ] {
            let mut controller = inside_at(start[0], start[1]);
            walk_steps(&mut controller, input, 20);
            let actual = controller.local_ship_position().unwrap()[axis];
            assert!((actual - expected).abs() < 1.0e-12);
        }
    }

    #[test]
    fn narrow_nose_shoulders_keep_the_player_over_the_floor() {
        for (input, expected_side) in [
            (
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
                -NOSE_SIDE_LIMIT,
            ),
            (
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
                NOSE_SIDE_LIMIT,
            ),
        ] {
            let mut controller = inside_at(7.3, 0.0);
            walk_steps(&mut controller, input, 10);
            let local = controller.local_ship_position().unwrap();
            assert_eq!(local[0], 7.3);
            assert_eq!(local[2], expected_side);
        }
    }

    #[test]
    fn both_console_sides_reach_the_level_nose_floor() {
        for (side, inward) in [
            (
                -2.2,
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
            ),
            (
                2.2,
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
            ),
        ] {
            let mut player = inside_at(0.5, side);
            walk_steps(
                &mut player,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                20,
            );
            assert_eq!(player.local_ship_position().unwrap()[0], CABIN_FORWARD_MAX);
            walk_steps(&mut player, inward, 4);
            walk_steps(
                &mut player,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                6,
            );
            let local = player.local_ship_position().unwrap();
            assert_eq!(player.location(), CharacterLocation::InsideShip);
            assert!(
                (local[0] - INTERIOR_FORWARD_MAX).abs() < 1e-12,
                "{side}: {local:?}"
            );
            assert!(local[2].abs() < NOSE_SIDE_LIMIT);
            assert_eq!(local[1], PLAYER_START[1]);
        }
    }

    #[test]
    fn removed_port_cargo_area_cannot_support_or_admit_a_walker() {
        for x in [-1.8, -1.0, 0.0, 1.3] {
            let mut cabin = inside_at(x, 3.0);
            walk_steps(
                &mut cabin,
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
                30,
            );
            assert!((cabin.local_ship_position().unwrap()[2] - INTERIOR_SIDE_LIMIT).abs() < 1e-12);
        }
        assert!(ship_floor_placement([-1.0, 0.0, 8.0], 0.1).is_none());
        assert!(ship_floor_placement([7.3, 0.0, 0.0], 0.1).is_some());
        assert!(ship_floor_placement([7.3, 0.0, 1.2], 0.1).is_none());
    }

    #[test]
    fn cockpit_chair_collision_preserves_instant_seating_and_safe_exit() {
        let mut controller = inside_at(COCKPIT_CHAIR_OBSTACLE[0], 0.0);
        controller.enter_cockpit();
        assert_eq!(controller.location(), CharacterLocation::Cockpit);
        assert_eq!(controller.local_ship_position(), Some(COCKPIT_POSITION));

        controller.leave_cockpit();
        assert_eq!(controller.local_ship_position(), Some(PLAYER_START));
        walk_steps(&mut controller, MovementInput::default(), 1);
        assert_eq!(controller.local_ship_position(), Some(PLAYER_START));
    }

    #[test]
    fn wider_cabin_allows_window_approaches_and_stops_before_window_sills() {
        for (side, input) in [
            (
                -1.0,
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
            ),
            (
                1.0,
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
            ),
        ] {
            let mut controller = inside_at(-6.5, 0.0);
            walk_steps(&mut controller, input, 20);
            let local = controller.local_ship_position().unwrap();
            assert!((local[2] - side * 3.96).abs() < 1.0e-12);
            assert!(local[2].abs() > 3.9);
        }
    }

    #[test]
    fn cleared_cabin_sides_allow_walking_to_the_wall() {
        for input in [
            MovementInput {
                right: true,
                ..MovementInput::default()
            },
            MovementInput {
                left: true,
                ..MovementInput::default()
            },
        ] {
            let mut controller = inside_at(-4.5, 0.0);
            walk_steps(&mut controller, input, 20);
            let local = controller.local_ship_position().unwrap();
            assert!((local[2].abs() - INTERIOR_SIDE_LIMIT).abs() < 1.0e-12);
        }
    }

    #[test]
    fn jump_keeps_player_head_below_ceiling_lamps_and_aft_lintel() {
        for (x, z, ceiling) in [
            (0.5, -2.2, SHIP_CEILING_HEIGHT),
            (-7.0, 0.0, DOORWAY_CEILING_HEIGHT),
        ] {
            let mut controller = inside_at(x, z);
            let mut highest_eye = PLAYER_START[1];
            let head_above_eye = PLAYER_BODY_HEIGHT_METERS - PLAYER_EYE_HEIGHT_METERS;
            for _ in 0..100 {
                controller.advance(
                    Duration::from_millis(16),
                    MovementInput {
                        jump: true,
                        ..MovementInput::default()
                    },
                    frame(),
                    surface(),
                    false,
                    true,
                );
                let eye = controller.local_ship_position().unwrap()[1];
                highest_eye = highest_eye.max(eye);
                assert!(eye + head_above_eye <= ceiling + 1.0e-12);
            }
            assert!((highest_eye + head_above_eye - ceiling).abs() < 1.0e-12);
            assert_eq!(
                controller.local_ship_position().unwrap()[1],
                PLAYER_START[1]
            );
        }
    }

    #[test]
    fn ship_floor_gravity_and_jump_use_the_shared_fixed_strength() {
        let mut controller = CharacterController::default();
        controller.advance(
            Duration::from_millis(16),
            MovementInput {
                jump: true,
                ..MovementInput::default()
            },
            frame(),
            surface(),
            false,
            true,
        );
        let local = controller.local_ship_position().unwrap();
        assert!(local[1] > PLAYER_START[1]);
        assert_eq!(FIXED_GRAVITY_METERS_PER_SECOND_SQUARED, 9.81);
    }

    #[test]
    fn closing_on_the_cabin_side_keeps_the_player_inside_with_body_clearance() {
        for leaving_ship in [false, true] {
            for eye_height in [PLAYER_START[1] - 0.3, PLAYER_START[1] + 0.2] {
                let mut controller = CharacterController {
                    position: PositionState::Doorway {
                        world: frame().local_to_world([-7.4, eye_height, 1.15]),
                        elapsed: Duration::from_millis(100),
                        leaving_ship,
                    },
                    ..CharacterController::default()
                };
                controller.advance(
                    Duration::from_millis(16),
                    MovementInput {
                        backward: true,
                        ..MovementInput::default()
                    },
                    frame(),
                    surface(),
                    false,
                    true,
                );
                let snapshot = controller.snapshot(frame(), surface());
                assert_eq!(snapshot.location, CharacterLocation::InsideShip);
                assert_eq!(snapshot.doorway_blend_fraction, None);
                let local = snapshot.local_ship_position_meters.unwrap();
                assert!(local[0] >= INTERIOR_FORWARD_MIN);
                assert_eq!(local[1], eye_height.max(PLAYER_START[1]));
                assert_eq!(local[2], 1.15);
            }
        }
    }
}
