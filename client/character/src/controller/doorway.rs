//! Body-clear gate transitions and the timed gravity blend.
use super::*;

impl CharacterController {
    pub(super) fn advance_doorway(
        &mut self,
        mut world: [f64; 3],
        mut elapsed: Duration,
        leaving_ship: bool,
        step: &Step,
    ) {
        let Step {
            delta,
            seconds,
            input,
            ship,
            surface,
            doorway_passable,
            ..
        } = *step;

        if !doorway_passable {
            // Recheck before moving: an in-progress gravity blend does
            // not grant passage through a door that has since closed.
            let mut local = ship.world_to_local(world);
            // Preserve doorway body clearance after the world round-trip;
            // a rounded jamb contact must not become an aft-wall overlap.
            local[2] = local[2].clamp(-DOORWAY_SIDE_LIMIT, DOORWAY_SIDE_LIMIT);
            self.vertical_speed = 0.0;
            self.position = if local[0] < CLOSED_GATE_MIDPOINT {
                local[0] = local[0].min(EXTERIOR_AFT);
                PositionState::Surface {
                    world: surface_eye_at_ship_planar_position(local, ship, surface),
                }
            } else {
                local[0] = local[0].max(INTERIOR_FORWARD_MIN);
                local[1] = local[1].max(PLAYER_EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT);
                PositionState::Inside { local }
            };
            return;
        }
        let movement = ship_planar_movement(input, self.yaw_radians);
        world = add(
            world,
            scale(
                add(
                    scale(ship.axes[0], movement[0]),
                    scale(ship.axes[2], movement[1]),
                ),
                WALK_SPEED_METERS_PER_SECOND * seconds,
            ),
        );
        // Keep body clearance while oblique movement slides along the jamb.
        let mut local = ship.world_to_local(world);
        local[2] = local[2].clamp(-DOORWAY_SIDE_LIMIT, DOORWAY_SIDE_LIMIT);
        world = ship.local_to_world(local);
        elapsed = elapsed.saturating_add(delta);
        // A slow/diagonal entry may finish its gravity blend while still
        // in the gate. Keep walking through it before selecting the cabin
        // state; otherwise the next interior tick treats it as a new exit.
        let reached_destination =
            leaving_ship || local[0] >= DOORWAY_FORWARD || local[0] < EXTERIOR_AFT;
        if elapsed >= DOORWAY_GRAVITY_BLEND_DURATION && reached_destination {
            if local[0] < DOORWAY_FORWARD {
                self.position = PositionState::Surface {
                    world: project_eye_to_surface(world, surface),
                };
            } else {
                local[1] = PLAYER_EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT;
                self.position = PositionState::Inside { local };
            }
        } else {
            self.position = PositionState::Doorway {
                world,
                elapsed,
                leaving_ship,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    #[test]
    fn closed_door_keeps_player_inside_on_surface_and_in_space() {
        let mut controller = CharacterController {
            position: PositionState::Inside {
                local: [-7.0, PLAYER_START[1], 0.0],
            },
            ..CharacterController::default()
        };
        let leave = MovementInput {
            backward: true,
            ..MovementInput::default()
        };
        controller.advance(
            Duration::from_secs(1),
            leave,
            frame(),
            surface(),
            false,
            true,
        );
        assert_eq!(controller.location(), CharacterLocation::InsideShip);
        controller.advance(
            Duration::from_secs(1),
            leave,
            frame(),
            surface(),
            false,
            false,
        );
        assert_eq!(controller.location(), CharacterLocation::InsideShip);
    }

    #[test]
    fn open_door_does_not_allow_walking_through_rear_windows_or_door_jambs() {
        for side in [-4.0, -1.20, 1.20, 4.0] {
            let mut controller = inside_at(-7.0, side);
            for _ in 0..10 {
                controller.advance(
                    Duration::from_millis(100),
                    MovementInput {
                        backward: true,
                        ..MovementInput::default()
                    },
                    frame(),
                    surface(),
                    true,
                    true,
                );
            }
            assert_eq!(controller.location(), CharacterLocation::InsideShip);
            assert_eq!(
                controller.local_ship_position().unwrap()[0],
                AFT_WALL_FORWARD_MIN
            );
        }
    }

    #[test]
    fn doorway_aperture_accepts_player_body_clearance_on_both_sides() {
        for side in [-1.15, 0.0, 1.15] {
            let mut controller = inside_at(-7.0, side);
            controller.advance(
                Duration::from_millis(50),
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
                frame(),
                surface(),
                true,
                true,
            );
            assert_eq!(controller.location(), CharacterLocation::DoorwayBlend);
        }
    }

    #[test]
    fn surface_reentry_uses_same_clear_doorway_and_reaches_cabin_floor() {
        for side in [-1.30_f64, -1.15, 1.15, 1.30] {
            let mut controller = CharacterController {
                position: PositionState::Surface {
                    world: project_eye_to_surface(
                        frame().local_to_world([-8.0, PLAYER_START[1], side]),
                        surface(),
                    ),
                },
                ..CharacterController::default()
            };
            let input = MovementInput {
                forward: true,
                ..MovementInput::default()
            };
            controller.advance(
                Duration::from_millis(100),
                input,
                frame(),
                surface(),
                true,
                true,
            );
            if side.abs() > DOORWAY_SIDE_LIMIT {
                assert_eq!(controller.location(), CharacterLocation::Surface);
            } else {
                assert_eq!(controller.location(), CharacterLocation::DoorwayBlend);
                for _ in 0..16 {
                    controller.advance(
                        Duration::from_millis(16),
                        input,
                        frame(),
                        surface(),
                        true,
                        true,
                    );
                }
                assert_eq!(controller.location(), CharacterLocation::InsideShip);
                let local = controller.local_ship_position().unwrap();
                assert!(local[0] > DOORWAY_FORWARD);
                assert!(local[2].abs() < DOORWAY_SIDE_LIMIT);
                assert_eq!(local[1], PLAYER_START[1]);
            }
        }
    }

    #[test]
    fn closed_or_unlanded_gate_blocks_surface_entry() {
        for (door_open, landed) in [(false, true), (true, false)] {
            let mut controller = CharacterController {
                position: PositionState::Surface {
                    world: project_eye_to_surface(
                        frame().local_to_world([-9.0, PLAYER_START[1], 0.0]),
                        surface(),
                    ),
                },
                ..CharacterController::default()
            };
            for _ in 0..100 {
                controller.advance(
                    Duration::from_millis(100),
                    MovementInput {
                        forward: true,
                        ..MovementInput::default()
                    },
                    frame(),
                    surface(),
                    door_open,
                    landed,
                );
                assert_eq!(controller.location(), CharacterLocation::Surface);
            }
            let local =
                frame().world_to_local(controller.snapshot(frame(), surface()).eye_position_meters);
            assert!((local[0] + 8.16).abs() < 1.0e-9);
        }
    }

    #[test]
    fn closing_after_a_short_exit_clears_the_gate_before_surface_movement() {
        let mut controller = inside_at(-7.0, 0.0);
        controller.advance(
            Duration::from_millis(50),
            MovementInput {
                backward: true,
                ..MovementInput::default()
            },
            frame(),
            surface(),
            true,
            true,
        );
        controller.advance(
            DOORWAY_GRAVITY_BLEND_DURATION,
            MovementInput::default(),
            frame(),
            surface(),
            true,
            true,
        );
        assert_eq!(controller.location(), CharacterLocation::Surface);
        let outside = controller.snapshot(frame(), surface());
        assert!(frame().world_to_local(outside.eye_position_meters)[0] > EXTERIOR_AFT);

        for step in 0..30 {
            controller.advance(
                Duration::from_millis(16),
                MovementInput {
                    forward: true,
                    jump: step % 2 == 0,
                    ..MovementInput::default()
                },
                frame(),
                surface(),
                false,
                true,
            );
            let snapshot = controller.snapshot(frame(), surface());
            let local = frame().world_to_local(snapshot.eye_position_meters);
            assert_eq!(snapshot.location, CharacterLocation::Surface);
            assert_eq!(snapshot.doorway_blend_fraction, None);
            assert!(local[0] <= EXTERIOR_AFT + COLLISION_EPSILON, "{local:?}");
            let radial = sub(snapshot.eye_position_meters, surface().body_center_meters);
            assert!(
                (dot(radial, radial).sqrt() - surface().radius_meters - PLAYER_EYE_HEIGHT_METERS)
                    .abs()
                    < 1.0e-9
            );
            assert_eq!(snapshot.up, normalize(radial).map(|axis| axis as f32));
        }
    }

    #[test]
    fn closing_during_an_outside_crossing_cancels_the_gravity_blend() {
        for leaving_ship in [false, true] {
            for (door_open, landed) in [(false, true), (true, false)] {
                let mut controller = CharacterController {
                    position: PositionState::Doorway {
                        world: frame().local_to_world([-8.0, PLAYER_START[1], 0.0]),
                        elapsed: Duration::from_millis(240),
                        leaving_ship,
                    },
                    ..CharacterController::default()
                };
                controller.advance(
                    Duration::from_millis(16),
                    MovementInput {
                        forward: true,
                        jump: true,
                        ..MovementInput::default()
                    },
                    frame(),
                    surface(),
                    door_open,
                    landed,
                );
                let snapshot = controller.snapshot(frame(), surface());
                assert_eq!(snapshot.location, CharacterLocation::Surface);
                assert_eq!(snapshot.doorway_blend_fraction, None);
                assert!(
                    frame().world_to_local(snapshot.eye_position_meters)[0]
                        <= EXTERIOR_AFT + COLLISION_EPSILON
                );
                controller.enter_cockpit();
                assert_eq!(controller.location(), CharacterLocation::Surface);
            }
        }
    }

    #[test]
    fn closed_gate_remains_solid_in_a_rotated_frame_at_catalog_coordinates() {
        let (ship, surface) = rotated_catalog_frames();
        let mut controller = CharacterController::default();
        for (steps, milliseconds, input) in [
            (
                18,
                100,
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
            ),
            (
                6,
                100,
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
            ),
            (
                2,
                100,
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
            ),
            (
                10,
                16,
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
            ),
            (20, 16, MovementInput::default()),
        ] {
            for _ in 0..steps {
                controller.advance(
                    Duration::from_millis(milliseconds),
                    input,
                    ship,
                    surface,
                    true,
                    true,
                );
            }
        }
        assert_eq!(controller.location(), CharacterLocation::Surface);
        let outside = controller.clone();
        for side in [-DOORWAY_SIDE_LIMIT, 0.08, DOORWAY_SIDE_LIMIT] {
            let mut controller = outside.clone();
            let mut local =
                ship.world_to_local(controller.snapshot(ship, surface).eye_position_meters);
            local[2] = side;
            controller.position = PositionState::Surface {
                world: surface_eye_at_ship_planar_position(local, ship, surface),
            };
            for step in 0..100 {
                controller.advance(
                    Duration::from_millis(16),
                    MovementInput {
                        forward: step > 0,
                        jump: step % 2 == 0,
                        ..MovementInput::default()
                    },
                    ship,
                    surface,
                    false,
                    true,
                );
                let snapshot = controller.snapshot(ship, surface);
                let local = ship.world_to_local(snapshot.eye_position_meters);
                assert_eq!(snapshot.location, CharacterLocation::Surface);
                assert_eq!(snapshot.doorway_blend_fraction, None);
                // At 1e12 m, a world-coordinate ULP is about 0.12 mm. The gate
                // must remain stable within that precision, never admit a walk step.
                assert!(
                    (local[0] - EXTERIOR_AFT).abs() < 0.001,
                    "step {step}: {local:?}"
                );
            }
        }
    }

    #[test]
    fn closing_at_a_rotated_cabin_jamb_preserves_the_interior_stopping_plane() {
        let (ship, surface) = rotated_catalog_frames();
        for side in [-DOORWAY_SIDE_LIMIT, DOORWAY_SIDE_LIMIT] {
            for leaving_ship in [false, true] {
                let mut controller = CharacterController {
                    position: PositionState::Doorway {
                        world: ship.local_to_world([-7.4, PLAYER_START[1], side]),
                        elapsed: Duration::from_millis(100),
                        leaving_ship,
                    },
                    ..CharacterController::default()
                };
                for _ in 0..30 {
                    controller.advance(
                        Duration::from_millis(16),
                        MovementInput {
                            backward: true,
                            ..MovementInput::default()
                        },
                        ship,
                        surface,
                        false,
                        true,
                    );
                    assert_eq!(controller.location(), CharacterLocation::InsideShip);
                    let local = controller.local_ship_position().unwrap();
                    assert_eq!(local[0], INTERIOR_FORWARD_MIN);
                }
            }
        }
    }

    #[test]
    fn closed_gate_blocks_diagonal_and_jump_input_at_both_edges() {
        for side in [-1.0, 1.0] {
            let mut controller = CharacterController {
                position: PositionState::Surface {
                    world: project_eye_to_surface(
                        frame().local_to_world([-9.0, PLAYER_START[1], side * 1.15]),
                        surface(),
                    ),
                },
                ..CharacterController::default()
            };
            for step in 0..100 {
                controller.advance(
                    Duration::from_millis(16),
                    MovementInput {
                        forward: true,
                        right: side > 0.0,
                        left: side < 0.0,
                        jump: step % 2 == 0,
                        ..MovementInput::default()
                    },
                    frame(),
                    surface(),
                    false,
                    true,
                );
                let snapshot = controller.snapshot(frame(), surface());
                let local = frame().world_to_local(snapshot.eye_position_meters);
                assert_eq!(snapshot.location, CharacterLocation::Surface);
                assert_eq!(snapshot.doorway_blend_fraction, None);
                assert!(
                    local[0] <= EXTERIOR_AFT + COLLISION_EPSILON
                        || local[2].abs() >= EXTERIOR_SIDE - COLLISION_EPSILON,
                    "{local:?}"
                );
            }
        }
    }

    #[test]
    fn repeated_door_cycles_restore_entry_and_exit_at_center_and_edges() {
        for side in [-1.15, 0.0, 1.15] {
            let mut controller = inside_at(-7.0, side);
            for _ in 0..3 {
                for _ in 0..6 {
                    controller.advance(
                        Duration::from_millis(100),
                        MovementInput {
                            backward: true,
                            ..MovementInput::default()
                        },
                        frame(),
                        surface(),
                        true,
                        true,
                    );
                }
                assert_eq!(controller.location(), CharacterLocation::Surface);

                for step in 0..40 {
                    controller.advance(
                        Duration::from_millis(16),
                        MovementInput {
                            forward: true,
                            jump: step % 2 == 0,
                            ..MovementInput::default()
                        },
                        frame(),
                        surface(),
                        false,
                        true,
                    );
                    let snapshot = controller.snapshot(frame(), surface());
                    assert_eq!(snapshot.location, CharacterLocation::Surface);
                    assert_eq!(snapshot.doorway_blend_fraction, None);
                    assert!(
                        frame().world_to_local(snapshot.eye_position_meters)[0]
                            <= EXTERIOR_AFT + COLLISION_EPSILON
                    );
                }
                let mut entered_blend = false;
                for _ in 0..40 {
                    controller.advance(
                        Duration::from_millis(16),
                        MovementInput {
                            forward: true,
                            ..MovementInput::default()
                        },
                        frame(),
                        surface(),
                        true,
                        true,
                    );
                    let snapshot = controller.snapshot(frame(), surface());
                    if !entered_blend && snapshot.location == CharacterLocation::DoorwayBlend {
                        assert_eq!(snapshot.doorway_blend_fraction, Some(0.0));
                        entered_blend = true;
                    }
                    if snapshot.location == CharacterLocation::InsideShip {
                        break;
                    }
                }
                assert!(entered_blend);
                assert_eq!(controller.location(), CharacterLocation::InsideShip);
                for _ in 0..10 {
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
                    assert_eq!(controller.location(), CharacterLocation::InsideShip);
                }
            }
        }
    }

    #[test]
    fn surface_walkers_can_round_the_hull_and_enter_only_at_the_open_gate() {
        for side in [-1.0, 1.0] {
            let ship = frame();
            let surface = SurfaceFrame {
                body_center_meters: sub(ship.origin_meters, [0.0, 6_000_000.0, 0.0]),
                radius_meters: 6_000_000.0,
            };
            let mut controller = CharacterController {
                position: PositionState::Surface {
                    world: project_eye_to_surface(
                        ship.local_to_world([0.0, PLAYER_START[1], side * 6.0]),
                        surface,
                    ),
                },
                yaw_radians: -side * 3.0 * std::f64::consts::FRAC_PI_4,
                ..CharacterController::default()
            };
            let input = MovementInput {
                forward: true,
                ..MovementInput::default()
            };
            // Push diagonally against the side, slide aft, and round the corner.
            for _ in 0..40 {
                controller.advance(Duration::from_millis(100), input, ship, surface, true, true);
                assert_eq!(controller.location(), CharacterLocation::Surface);
                let local =
                    ship.world_to_local(controller.snapshot(ship, surface).eye_position_meters);
                assert!(local[0] <= -8.16 + 1.0e-7 || local[2].abs() >= 5.34 - 1.0e-7);
            }
            controller.yaw_radians = -side * FRAC_PI_2;
            for _ in 0..100 {
                let local =
                    ship.world_to_local(controller.snapshot(ship, surface).eye_position_meters);
                if local[2].abs() < 0.05 {
                    break;
                }
                controller.advance(Duration::from_millis(16), input, ship, surface, true, true);
                assert_eq!(controller.location(), CharacterLocation::Surface);
            }
            controller.yaw_radians = 0.0;
            let mut entered = false;
            for _ in 0..100 {
                controller.advance(Duration::from_millis(16), input, ship, surface, true, true);
                let local =
                    ship.world_to_local(controller.snapshot(ship, surface).eye_position_meters);
                if controller.location() == CharacterLocation::InsideShip {
                    entered = true;
                    assert!(local[0] >= DOORWAY_FORWARD);
                } else {
                    assert!(
                        !entered,
                        "held entry movement must not start another exit blend"
                    );
                }
            }
            assert!(entered);
        }
    }

    #[test]
    fn open_gate_cannot_capture_surface_walkers_below_the_ship() {
        let ship = ShipFrame {
            origin_meters: add(frame().origin_meters, [0.0, 50.0, 0.0]),
            ..frame()
        };
        let mut controller = CharacterController {
            position: PositionState::Surface {
                world: project_eye_to_surface(
                    frame().local_to_world([-7.5, PLAYER_START[1], 0.0]),
                    surface(),
                ),
            },
            ..CharacterController::default()
        };
        for _ in 0..50 {
            controller.advance(
                Duration::from_millis(100),
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                ship,
                surface(),
                true,
                true,
            );
            assert_eq!(controller.location(), CharacterLocation::Surface);
        }
        let local = ship.world_to_local(controller.snapshot(ship, surface()).eye_position_meters);
        assert!(local[0] > 10.0);
    }

    #[test]
    fn doorway_exit_follows_held_wasd_across_ship_orientations() {
        let inputs = [
            (
                std::f64::consts::PI,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
            ),
            (
                0.0,
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
            ),
            (
                FRAC_PI_2,
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
            ),
            (
                -FRAC_PI_2,
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
            ),
        ];
        for axes in [
            frame().axes,
            [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]],
        ] {
            let ship = ShipFrame { axes, ..frame() };
            let surface = SurfaceFrame {
                body_center_meters: sub(ship.origin_meters, scale(ship.axes[1], 6_000_000.0)),
                radius_meters: 6_000_000.0,
            };
            for (yaw_radians, input) in inputs {
                let mut controller = CharacterController {
                    yaw_radians,
                    ..inside_at(-7.0, 0.0)
                };
                let mut previous_forward = -7.0;
                let mut crossed_doorway = false;
                for _ in 0..60 {
                    controller.advance(Duration::from_millis(16), input, ship, surface, true, true);
                    let local =
                        ship.world_to_local(controller.snapshot(ship, surface).eye_position_meters);
                    assert!(
                        local[0] < previous_forward,
                        "held exit input must keep moving aft, including during the blend"
                    );
                    previous_forward = local[0];
                    crossed_doorway |= controller.location() == CharacterLocation::DoorwayBlend;
                }
                assert!(crossed_doorway);
                assert_eq!(controller.location(), CharacterLocation::Surface);
                assert!(previous_forward < -10.0);
            }
        }
    }

    #[test]
    fn doorway_blend_preserves_camera_relative_diagonal_movement() {
        let mut controller = CharacterController {
            position: PositionState::Doorway {
                world: frame().local_to_world([-7.2, PLAYER_START[1], 0.0]),
                elapsed: Duration::ZERO,
                leaving_ship: true,
            },
            yaw_radians: std::f64::consts::PI,
            ..CharacterController::default()
        };
        let before = controller.snapshot(frame(), surface());
        let forward = normalize(sub(before.look_target_meters, before.eye_position_meters));
        let right = cross(forward, before.up.map(f64::from));
        let expected = scale(
            normalize(add(forward, right)),
            WALK_SPEED_METERS_PER_SECOND * 0.016,
        );
        controller.advance(
            Duration::from_millis(16),
            MovementInput {
                forward: true,
                right: true,
                ..MovementInput::default()
            },
            frame(),
            surface(),
            true,
            true,
        );
        let actual = sub(
            controller.snapshot(frame(), surface()).eye_position_meters,
            before.eye_position_meters,
        );
        assert!(dot(sub(actual, expected), sub(actual, expected)).sqrt() < 1.0e-9);
    }

    #[test]
    fn oblique_doorway_crossings_slide_along_both_jambs() {
        let ship = ShipFrame {
            axes: [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]],
            ..frame()
        };
        let surface = SurfaceFrame {
            body_center_meters: sub(ship.origin_meters, scale(ship.axes[1], 6_000_000.0)),
            radius_meters: 6_000_000.0,
        };
        let input = MovementInput {
            forward: true,
            ..MovementInput::default()
        };
        for side in [-1.0, 1.0] {
            for leaving_ship in [false, true] {
                let position = if leaving_ship {
                    PositionState::Inside {
                        local: [-7.0, PLAYER_START[1], side * 1.10],
                    }
                } else {
                    PositionState::Surface {
                        world: project_eye_to_surface(
                            ship.local_to_world([-7.5, PLAYER_START[1], side * 1.10]),
                            surface,
                        ),
                    }
                };
                let mut controller = CharacterController {
                    position,
                    yaw_radians: side * FRAC_PI_2 * if leaving_ship { 1.5 } else { 0.5 },
                    ..CharacterController::default()
                };
                controller.advance(Duration::from_millis(16), input, ship, surface, true, true);
                assert_eq!(controller.location(), CharacterLocation::DoorwayBlend);

                let mut touched_jamb = false;
                for _ in 0..16 {
                    controller.advance(Duration::from_millis(16), input, ship, surface, true, true);
                    let local =
                        ship.world_to_local(controller.snapshot(ship, surface).eye_position_meters);
                    assert!(
                        local[2].abs() <= DOORWAY_SIDE_LIMIT + 1.0e-9,
                        "the complete player body must remain clear of the jamb during a blend"
                    );
                    touched_jamb |= (local[2].abs() - DOORWAY_SIDE_LIMIT).abs() < 1.0e-9;
                }
                assert!(touched_jamb, "oblique movement must slide along the jamb");
                if leaving_ship {
                    assert_eq!(controller.location(), CharacterLocation::Surface);
                } else {
                    assert_eq!(controller.location(), CharacterLocation::InsideShip);
                    let local = controller.local_ship_position().unwrap();
                    assert!(local[0] > DOORWAY_FORWARD);
                    assert_eq!(local[1], PLAYER_START[1]);
                }
            }
        }
    }

    #[test]
    fn surface_reentry_depends_on_ship_relative_direction_instead_of_key() {
        let cases = [
            (
                0.0,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
            ),
            (
                std::f64::consts::PI,
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
            ),
            (
                -FRAC_PI_2,
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
            ),
            (
                FRAC_PI_2,
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
            ),
        ];
        for (inward_yaw, input) in cases {
            for entering in [false, true] {
                let mut controller = CharacterController {
                    position: PositionState::Surface {
                        world: project_eye_to_surface(
                            frame().local_to_world([-7.5, PLAYER_START[1], 0.0]),
                            surface(),
                        ),
                    },
                    yaw_radians: inward_yaw + if entering { 0.0 } else { std::f64::consts::PI },
                    ..CharacterController::default()
                };
                controller.advance(
                    Duration::from_millis(16),
                    input,
                    frame(),
                    surface(),
                    true,
                    true,
                );
                assert_eq!(
                    controller.location(),
                    if entering {
                        CharacterLocation::DoorwayBlend
                    } else {
                        CharacterLocation::Surface
                    }
                );
                for _ in 0..16 {
                    controller.advance(
                        Duration::from_millis(16),
                        input,
                        frame(),
                        surface(),
                        true,
                        true,
                    );
                }
                assert_eq!(
                    controller.location(),
                    if entering {
                        CharacterLocation::InsideShip
                    } else {
                        CharacterLocation::Surface
                    }
                );
                if entering {
                    let local = controller.local_ship_position().unwrap();
                    assert!(local[0] > DOORWAY_FORWARD);
                    assert_eq!(local[1], PLAYER_START[1]);
                }
            }
        }
    }

    #[test]
    fn surface_parallel_or_idle_movement_does_not_trigger_reentry() {
        for input in [
            MovementInput::default(),
            MovementInput {
                forward: true,
                ..MovementInput::default()
            },
        ] {
            let mut controller = CharacterController {
                position: PositionState::Surface {
                    world: project_eye_to_surface(
                        frame().local_to_world([-7.5, PLAYER_START[1], 0.0]),
                        surface(),
                    ),
                },
                yaw_radians: FRAC_PI_2,
                ..CharacterController::default()
            };
            controller.advance(
                Duration::from_millis(16),
                input,
                frame(),
                surface(),
                true,
                true,
            );
            assert_eq!(controller.location(), CharacterLocation::Surface);
        }
    }

    #[test]
    fn landed_open_door_starts_exact_quarter_second_gravity_blend() {
        let mut controller = CharacterController {
            position: PositionState::Inside {
                local: [-7.0, PLAYER_START[1], 0.0],
            },
            ..CharacterController::default()
        };
        let leave = MovementInput {
            backward: true,
            ..MovementInput::default()
        };
        controller.advance(
            Duration::from_millis(50),
            leave,
            frame(),
            surface(),
            true,
            true,
        );
        assert_eq!(controller.location(), CharacterLocation::DoorwayBlend);
        controller.advance(
            Duration::from_millis(249),
            leave,
            frame(),
            surface(),
            true,
            true,
        );
        assert_eq!(controller.location(), CharacterLocation::DoorwayBlend);
        controller.advance(
            Duration::from_millis(1),
            leave,
            frame(),
            surface(),
            true,
            true,
        );
        assert_eq!(controller.location(), CharacterLocation::Surface);
    }
}
