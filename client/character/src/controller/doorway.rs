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
        let previous = ship.world_to_local(world);
        let movement = ship_planar_movement(input, self.yaw_radians);
        let mut local = previous;
        // Normalize travel along the sloping support, not its horizontal shadow.
        let slope = crate::ramp_geometry::RAMP_ANGLE_RADIANS.cos()
            / crate::ramp_geometry::RAMP_ANGLE_RADIANS.sin();
        let speed_scale = if previous[0] < crate::ramp_geometry::RAMP_HINGE[0] {
            (1.0 + movement[0] * movement[0] * slope * slope).sqrt()
        } else {
            1.0
        };
        local[0] += movement[0] * WALK_SPEED_METERS_PER_SECOND * seconds / speed_scale;
        local[2] = (local[2] + movement[1] * WALK_SPEED_METERS_PER_SECOND * seconds / speed_scale)
            .clamp(-crate::ramp::side_limit(), crate::ramp::side_limit());
        // Contact changes with distance along the deployed mesh. A stopped
        // walker never completes a timer and drops from the elevated threshold.
        local[1] = crate::ramp::floor_at(local[0]) + PLAYER_EYE_HEIGHT_METERS;
        self.vertical_speed = 0.0;
        world = ship.local_to_world(local);
        elapsed = elapsed.saturating_add(delta);
        if local[0] <= crate::ramp::tip_x() {
            self.position = PositionState::Surface {
                world: surface_eye_at_ship_planar_position(local, ship, surface),
            };
        } else if local[0] >= DOORWAY_FORWARD {
            self.position = PositionState::Inside { local };
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

    fn raised_frames(rotated: bool) -> (ShipFrame, SurfaceFrame) {
        let ship = if rotated {
            rotated_catalog_frames().0
        } else {
            frame()
        };
        let body = SurfaceFrame {
            body_center_meters: sub(
                ship.origin_meters,
                scale(ship.axes[1], 6_000_000.0 + 2.10752688172043),
            ),
            radius_meters: 6_000_000.0,
        };
        (ship, body)
    }

    #[test]
    fn ramp_support_is_continuous_both_directions_at_all_edges_and_rotated_anchor() {
        for rotated in [false, true] {
            let (ship, body) = raised_frames(rotated);
            for z in [-1.15, 0.0, 1.15] {
                for (yaw, input) in [
                    (
                        0.0,
                        MovementInput {
                            backward: true,
                            ..MovementInput::default()
                        },
                    ),
                    (
                        std::f64::consts::PI,
                        MovementInput {
                            forward: true,
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
                ] {
                    let mut player = inside_at(-7.0, z);
                    player.yaw_radians = yaw;
                    let mut previous = player.snapshot(ship, body).eye_position_meters;
                    let mut saw_ramp = false;
                    for _ in 0..100 {
                        player.advance(Duration::from_millis(16), input, ship, body, true, true);
                        let snap = player.snapshot(ship, body);
                        let distance =
                            salimon_math::length(sub(snap.eye_position_meters, previous));
                        assert!(distance < 0.072, "no doorway drop: {distance}");
                        previous = snap.eye_position_meters;
                        if snap.location == CharacterLocation::DoorwayBlend {
                            saw_ramp = true;
                            let local = ship.world_to_local(previous);
                            assert!(
                                (local[1]
                                    - PLAYER_EYE_HEIGHT_METERS
                                    - crate::ramp::floor_at(local[0]))
                                .abs()
                                    < 0.001
                            );
                            assert!(local[2].abs() < 1.161);
                        }
                    }
                    assert!(saw_ramp);
                    assert_eq!(player.location(), CharacterLocation::Surface);
                    // Reverse the same camera-relative direction and return.
                    player.yaw_radians = (yaw + std::f64::consts::PI).rem_euclid(TAU);
                    for _ in 0..105 {
                        player.advance(Duration::from_millis(16), input, ship, body, true, true);
                        let snap = player.snapshot(ship, body);
                        assert!(
                            salimon_math::length(sub(snap.eye_position_meters, previous)) < 0.072
                        );
                        previous = snap.eye_position_meters;
                    }
                    assert_eq!(player.location(), CharacterLocation::InsideShip);
                    assert!(
                        (player.local_ship_position().unwrap()[1] - PLAYER_START[1]).abs() < 1e-9
                    );
                }
            }
        }
    }

    #[test]
    fn idle_ramp_contact_never_completes_a_timer_or_falls_to_ground() {
        let (ship, body) = raised_frames(false);
        let x = (crate::ramp::tip_x() + crate::ramp_geometry::RAMP_HINGE[0]) / 2.0;
        let world =
            ship.local_to_world([x, crate::ramp::floor_at(x) + PLAYER_EYE_HEIGHT_METERS, 0.0]);
        let mut player = CharacterController {
            position: PositionState::Doorway {
                world,
                elapsed: Duration::ZERO,
                leaving_ship: true,
            },
            ..CharacterController::default()
        };
        for _ in 0..100 {
            player.advance(
                Duration::from_millis(100),
                MovementInput::default(),
                ship,
                body,
                true,
                true,
            );
        }
        assert_eq!(player.location(), CharacterLocation::DoorwayBlend);
        assert_eq!(player.snapshot(ship, body).eye_position_meters, world);
        assert!(crate::boarding_ramp_occupied(ship.world_to_local(world)));
    }

    #[test]
    fn ramp_side_edges_slide_and_surface_side_approach_cannot_teleport_to_deck() {
        let (ship, body) = raised_frames(false);
        let x = -9.5;
        let world =
            ship.local_to_world([x, crate::ramp::floor_at(x) + PLAYER_EYE_HEIGHT_METERS, 0.0]);
        let mut player = CharacterController {
            position: PositionState::Doorway {
                world,
                elapsed: Duration::ZERO,
                leaving_ship: true,
            },
            ..CharacterController::default()
        };
        for _ in 0..50 {
            player.advance(
                Duration::from_millis(16),
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
                ship,
                body,
                true,
                true,
            );
        }
        let local = ship.world_to_local(player.snapshot(ship, body).eye_position_meters);
        assert!((local[2] - crate::ramp::side_limit()).abs() < 1e-9);
        assert_eq!(local[0], x);
        let mut ground = CharacterController {
            position: PositionState::Surface {
                world: surface_eye_at_ship_planar_position([x, 0.0, 3.0], ship, body),
            },
            ..CharacterController::default()
        };
        for _ in 0..100 {
            ground.advance(
                Duration::from_millis(16),
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
                ship,
                body,
                true,
                true,
            );
        }
        assert_eq!(ground.location(), CharacterLocation::Surface);
        assert!(
            ship.world_to_local(ground.snapshot(ship, body).eye_position_meters)[2] >= 1.64 - 1e-6
        );
    }

    #[test]
    fn closed_gate_is_solid_after_continuous_ramp_exit_at_distant_rotated_anchor() {
        let (ship, body) = raised_frames(true);
        let mut player = inside_at(-7.0, 0.0);
        for _ in 0..100 {
            player.advance(
                Duration::from_millis(16),
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
                ship,
                body,
                true,
                true,
            );
        }
        assert_eq!(player.location(), CharacterLocation::Surface);
        for _ in 0..200 {
            player.advance(
                Duration::from_millis(16),
                MovementInput {
                    forward: true,
                    jump: true,
                    ..MovementInput::default()
                },
                ship,
                body,
                false,
                true,
            );
        }
        assert_eq!(player.location(), CharacterLocation::Surface);
        let local = ship.world_to_local(player.snapshot(ship, body).eye_position_meters);
        assert!((local[0] - EXTERIOR_AFT).abs() < 0.001);
    }

    #[test]
    fn animated_ramp_envelope_blocks_surface_entry_before_contact_is_ready() {
        let (ship, body) = raised_frames(false);
        let mut player = CharacterController {
            position: PositionState::Surface {
                world: surface_eye_at_ship_planar_position(
                    [crate::ramp::tip_x() - 1.0, 0.0, 0.0],
                    ship,
                    body,
                ),
            },
            ..CharacterController::default()
        };
        player.set_ramp_moving(true);
        for _ in 0..100 {
            player.advance(
                Duration::from_millis(16),
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                ship,
                body,
                false,
                true,
            );
        }
        assert_eq!(player.location(), CharacterLocation::Surface);
        assert!(
            ship.world_to_local(player.snapshot(ship, body).eye_position_meters)[0]
                <= crate::ramp::tip_x() + 1e-6
        );
    }
}
