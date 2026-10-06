//! Radial surface walking, camera tangents and collision-resolved eye height.
use super::*;

impl CharacterController {
    pub(super) fn advance_surface(&mut self, mut world: [f64; 3], step: &Step) {
        let Step {
            seconds,
            input,
            ship,
            surface,
            doorway_passable,
            ..
        } = *step;

        let mut previous = ship.world_to_local(world);
        if !doorway_passable
            && overlaps_exterior_hull(previous)
            && previous[0] > EXTERIOR_AFT
            && previous[0] < DOORWAY_FORWARD + PLAYER_RADIUS_METERS
            && previous[2].abs() <= DOORWAY_SIDE_LIMIT + world_round_trip_tolerance(world)
        {
            // A stopped exit can finish its blend within the gate.
            // Closing around that surface walker must first clear the
            // overlap; swept collision alone only handles new contact.
            previous[0] = EXTERIOR_AFT;
            world = surface_eye_at_ship_planar_position(previous, ship, surface);
            // Keep the exact resolved plane for the sweep. At catalog
            // coordinates, a world round-trip can decode just inside
            // it and incorrectly admit the next inward walk step.
        }
        let up = normalize(sub(world, surface.body_center_meters));
        let (forward, right) = surface_view_basis(ship.axes, up, self.yaw_radians);
        let movement = movement_axes(input);
        let tangent = add(scale(forward, movement[0]), scale(right, movement[1]));
        world = add(
            world,
            scale(tangent, WALK_SPEED_METERS_PER_SECOND * seconds),
        );
        world = project_eye_to_surface(world, surface);

        let proposed = ship.world_to_local(world);
        let overlaps_hull = overlaps_exterior_hull(proposed);
        let mut local = proposed;
        if overlaps_hull {
            local = slide_around_obstacles(previous, local, &EXTERIOR_OBSTACLES);
            if !doorway_passable {
                local = slide_around_obstacles(previous, local, &CLOSED_GATE_OBSTACLE);
            }
            if local[0] != proposed[0] || local[2] != proposed[2] {
                // Preserve the collision-resolved X/Z while putting the eye
                // back on the sphere. Radial projection here would shrink
                // those coordinates and push the player through a wall.
                world = surface_eye_at_ship_planar_position(local, ship, surface);
                local = ship.world_to_local(world);
            }
        }
        let before_appendages = local;
        local = slide_around_appendages(previous, local, world_round_trip_tolerance(world));
        if local[0] != before_appendages[0] || local[2] != before_appendages[2] {
            world = surface_eye_at_ship_planar_position(local, ship, surface);
        }
        let through_gate = overlaps_hull
            && previous[0] < DOORWAY_FORWARD
            && local[0] > EXTERIOR_AFT
            && local[0] <= DOORWAY_FORWARD + COLLISION_EPSILON
            && previous[2].abs() <= DOORWAY_SIDE_LIMIT + COLLISION_EPSILON
            && local[2].abs() <= DOORWAY_SIDE_LIMIT + COLLISION_EPSILON
            && local[1]
                <= DOORWAY_CEILING_HEIGHT - (PLAYER_BODY_HEIGHT_METERS - PLAYER_EYE_HEIGHT_METERS);
        // Re-entry follows actual inward movement, regardless of which
        // key produces it. Ignore rounding noise from parallel strafing.
        let moving_into_ship = dot(tangent, ship.axes[0]) > 1.0e-6;
        if doorway_passable && through_gate && moving_into_ship {
            self.position = PositionState::Doorway {
                world,
                elapsed: Duration::ZERO,
                leaving_ship: false,
            };
        } else {
            self.position = PositionState::Surface { world };
        }
    }
}

pub(super) fn surface_view_basis(
    ship_axes: [[f64; 3]; 3],
    up: [f64; 3],
    yaw_radians: f64,
) -> ([f64; 3], [f64; 3]) {
    let (planar_forward, _) = planar_look(yaw_radians);
    let raw_forward = add(
        scale(ship_axes[0], planar_forward[0]),
        scale(ship_axes[2], planar_forward[1]),
    );
    let forward = normalize_or(
        reject(raw_forward, up),
        normalize_or(reject(ship_axes[2], up), orthogonal_tangent(up)),
    );
    let right = normalize(cross(forward, up));
    (forward, right)
}
pub(super) fn project_eye_to_surface(world: [f64; 3], surface: SurfaceFrame) -> [f64; 3] {
    add(
        surface.body_center_meters,
        scale(
            normalize(sub(world, surface.body_center_meters)),
            surface.radius_meters + PLAYER_EYE_HEIGHT_METERS,
        ),
    )
}

pub(super) fn surface_eye_at_ship_planar_position(
    mut local: [f64; 3],
    ship: ShipFrame,
    surface: SurfaceFrame,
) -> [f64; 3] {
    let center = ship.world_to_local(surface.body_center_meters);
    let radius = surface.radius_meters + PLAYER_EYE_HEIGHT_METERS;
    let forward = local[0] - center[0];
    let side = local[2] - center[2];
    let height_squared = radius * radius - forward * forward - side * side;
    if height_squared < 0.0 {
        return project_eye_to_surface(ship.local_to_world(local), surface);
    }
    local[1] = center[1] + height_squared.sqrt().copysign(local[1] - center[1]);
    ship.local_to_world(local)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    #[test]
    fn surface_motion_stays_on_full_sphere_with_fixed_eye_height() {
        let mut controller = CharacterController {
            position: PositionState::Surface {
                world: [0.0, 11.75, 0.0],
            },
            ..CharacterController::default()
        };
        let small_surface = SurfaceFrame {
            body_center_meters: [0.0, 0.0, 0.0],
            radius_meters: 10.0,
        };
        controller.advance(
            Duration::from_secs(1),
            MovementInput {
                forward: true,
                ..MovementInput::default()
            },
            frame(),
            small_surface,
            true,
            true,
        );
        let snapshot = controller.snapshot(frame(), small_surface);
        let radius = dot(snapshot.eye_position_meters, snapshot.eye_position_meters).sqrt();
        assert!((radius - 11.75).abs() < 1.0e-9);
    }

    #[test]
    fn surface_wasd_follows_camera_tangent_basis_across_bodies_and_orientations() {
        let cases = [
            ("Mercury", 2_400_000.0, [0.0, 1.0, 0.0], 0.0),
            ("Venus", 5_500_000.0, [1.0, 0.0, 0.0], 0.65),
            ("Earth", 6_000_000.0, [0.0, 0.0, 1.0], 1.4),
            ("Mars", 3_500_000.0, normalize([1.0, 2.0, -3.0]), 2.2),
            ("Moon", 1_600_000.0, normalize([-2.0, 1.0, 1.0]), 5.1),
        ];
        let inputs = [
            (
                "forward",
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                [1.0, 0.0],
            ),
            (
                "backward",
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
                [-1.0, 0.0],
            ),
            (
                "right",
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
                [0.0, 1.0],
            ),
            (
                "left",
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
                [0.0, -1.0],
            ),
        ];

        for (body, radius_meters, up, yaw_radians) in cases {
            let surface = SurfaceFrame {
                body_center_meters: [125.0, -80.0, 45.0],
                radius_meters,
            };
            let start = add(
                surface.body_center_meters,
                scale(up, radius_meters + PLAYER_EYE_HEIGHT_METERS),
            );

            for (direction, input, expected_axes) in inputs {
                let mut controller = CharacterController {
                    position: PositionState::Surface { world: start },
                    yaw_radians,
                    ..CharacterController::default()
                };
                let before = controller.snapshot(frame(), surface);
                let camera_forward = normalize(reject(
                    sub(before.look_target_meters, before.eye_position_meters),
                    up,
                ));
                let camera_right = normalize(cross(camera_forward, up));
                let expected = add(
                    scale(camera_forward, expected_axes[0]),
                    scale(camera_right, expected_axes[1]),
                );

                controller.advance(
                    Duration::from_millis(10),
                    input,
                    frame(),
                    surface,
                    false,
                    true,
                );

                let after = controller.snapshot(frame(), surface);
                let traveled = normalize(reject(
                    sub(after.eye_position_meters, before.eye_position_meters),
                    up,
                ));
                assert!(
                    dot(traveled, expected) > 0.999_999,
                    "{body} {direction} movement must follow the camera tangent basis"
                );
                let altitude = dot(
                    sub(after.eye_position_meters, surface.body_center_meters),
                    sub(after.eye_position_meters, surface.body_center_meters),
                )
                .sqrt();
                assert!(
                    (altitude - radius_meters - PLAYER_EYE_HEIGHT_METERS).abs() < 1.0e-8,
                    "{body} movement must preserve surface eye height"
                );
            }
        }
    }

    #[test]
    fn surface_forward_direction_tracks_camera_yaw_changes() {
        let surface = SurfaceFrame {
            body_center_meters: [0.0, 0.0, 0.0],
            radius_meters: 100.0,
        };
        let start = [0.0, 101.75, 0.0];
        let mut before_turn = CharacterController {
            position: PositionState::Surface { world: start },
            ..CharacterController::default()
        };
        let mut after_turn = before_turn.clone();
        after_turn.apply_mouse_delta(400.0, 0.0);

        for controller in [&mut before_turn, &mut after_turn] {
            let before = controller.snapshot(frame(), surface);
            let expected = normalize(reject(
                sub(before.look_target_meters, before.eye_position_meters),
                [0.0, 1.0, 0.0],
            ));
            controller.advance(
                Duration::from_millis(10),
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                frame(),
                surface,
                false,
                true,
            );
            let after = controller.snapshot(frame(), surface);
            let traveled = normalize(reject(
                sub(after.eye_position_meters, before.eye_position_meters),
                [0.0, 1.0, 0.0],
            ));
            assert!(dot(traveled, expected) > 0.999_999);
        }

        let first = before_turn.snapshot(frame(), surface).eye_position_meters;
        let turned = after_turn.snapshot(frame(), surface).eye_position_meters;
        assert!(
            sub(first, start)[2].abs() < 1.0e-8,
            "zero yaw should move along ship forward"
        );
        assert!(
            sub(turned, start)[2] > 0.0,
            "positive yaw should rotate surface-forward movement toward screen right"
        );
    }
}
