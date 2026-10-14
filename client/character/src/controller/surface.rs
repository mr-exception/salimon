//! Radial surface walking, camera tangents and collision-resolved eye height.
use super::*;

/// 40 degrees, measured by the shared ground query.
pub const MAX_TRAVERSABLE_SLOPE: f64 = 0.839_099_631_177_28;
pub const MAX_STEP_HEIGHT_METERS: f64 = 0.3;

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
            && overlaps_boarding_gate(previous)
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
        let old_world = world;
        let up = normalize(sub(world, surface.body_center_meters));
        let (forward, right) = surface_view_basis(ship.axes, up, self.yaw_radians);
        let movement = movement_axes(input);
        let tangent = add(scale(forward, movement[0]), scale(right, movement[1]));
        world = add(
            world,
            scale(tangent, WALK_SPEED_METERS_PER_SECOND * seconds),
        );
        let ground = surface.sample(world);
        let previous_ground = surface.sample(old_world);
        let rise = dot(
            sub(ground.position_meters, previous_ground.position_meters),
            up,
        );
        if surface.query.is_some()
            && ((ground.slope > MAX_TRAVERSABLE_SLOPE && rise >= -0.001)
                || rise > MAX_STEP_HEIGHT_METERS)
        {
            world = old_world;
        }
        if surface.query.is_none() {
            // The old deterministic sphere fixtures retain their pre-terrain walking policy.
            world = project_eye_to_surface(world, surface);
        } else {
            let old_height = dot(sub(old_world, surface.eye_on_ground(old_world)), up).max(0.0);
            if step.jump_started && self.vertical_speed == 0.0 && old_height < 0.001 {
                self.vertical_speed = JUMP_SPEED_METERS_PER_SECOND;
            }
            let on_ground =
                old_height < 0.001 && self.vertical_speed == 0.0 && rise >= -MAX_STEP_HEIGHT_METERS;
            let height = if on_ground {
                0.0
            } else {
                (dot(sub(world, surface.eye_on_ground(world)), up) + self.vertical_speed * seconds
                    - 0.5 * FIXED_GRAVITY_METERS_PER_SECOND_SQUARED * seconds * seconds)
                    .max(0.0)
            };
            self.vertical_speed -= FIXED_GRAVITY_METERS_PER_SECOND_SQUARED * seconds;
            if height == 0.0 {
                self.vertical_speed = 0.0;
            }
            world = add(project_eye_to_surface(world, surface), scale(up, height));
        }

        let proposed = ship.world_to_local(world);
        let overlaps_hull = overlaps_exterior_hull(proposed);
        let overlaps_gate = overlaps_boarding_gate(proposed);
        let mut local = proposed;
        if overlaps_hull {
            local = slide_around_obstacles(previous, local, &EXTERIOR_OBSTACLES);
            if !doorway_passable && !overlaps_gate {
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
        if !doorway_passable && overlaps_gate {
            let before_gate = local;
            local = slide_around_obstacles(previous, local, &CLOSED_GATE_OBSTACLE);
            if local[0] != before_gate[0] || local[2] != before_gate[2] {
                world = surface_eye_at_ship_planar_position(local, ship, surface);
            }
        }
        if (self.ramp_moving && step.ship_landed)
            || (doorway_passable && !approaching_ramp_tip(previous, local))
        {
            let obstacles = [[
                crate::ramp::tip_x(),
                crate::ramp_geometry::RAMP_HINGE[0],
                -crate::ramp_geometry::RAMP_HALF_WIDTH_METERS - PLAYER_RADIUS_METERS,
                crate::ramp_geometry::RAMP_HALF_WIDTH_METERS + PLAYER_RADIUS_METERS,
            ]];
            let before = local;
            local = slide_around_obstacles(previous, local, &obstacles);
            if local[0] != before[0] || local[2] != before[2] {
                world = surface_eye_at_ship_planar_position(local, ship, surface);
            }
        }
        let before_appendages = local;
        local = slide_around_appendages(previous, local, world_round_trip_tolerance(world));
        if local[0] != before_appendages[0] || local[2] != before_appendages[2] {
            world = surface_eye_at_ship_planar_position(local, ship, surface);
        }
        let tip = crate::ramp::tip_x();
        let moving_into_ship = dot(tangent, ship.axes[0]) > 1.0e-6;
        let approaching_tip = previous[0] <= tip + world_round_trip_tolerance(world)
            && local[0] > tip
            && previous[2].abs() <= crate::ramp::side_limit()
            && local[2].abs() <= crate::ramp::side_limit()
            && (local[1] - PLAYER_EYE_HEIGHT_METERS - crate::ramp::floor_at(tip)).abs() < 0.3;
        if doorway_passable && approaching_tip && moving_into_ship {
            local[1] = crate::ramp::floor_at(local[0]) + PLAYER_EYE_HEIGHT_METERS;
            self.position = PositionState::Doorway {
                world: ship.local_to_world(local),
                elapsed: Duration::ZERO,
                leaving_ship: false,
            };
        } else {
            self.position = PositionState::Surface { world };
        }
    }
}

fn approaching_ramp_tip(previous: [f64; 3], proposed: [f64; 3]) -> bool {
    previous[0] <= crate::ramp::tip_x() + 0.001
        && previous[2].abs() <= crate::ramp::side_limit()
        && proposed[2].abs() <= crate::ramp::side_limit()
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
    surface.eye_on_ground(world)
}

pub(super) fn surface_eye_at_ship_planar_position(
    mut local: [f64; 3],
    ship: ShipFrame,
    surface: SurfaceFrame,
) -> [f64; 3] {
    if surface.query.is_some() {
        for _ in 0..6 {
            let world = ship.local_to_world(local);
            let eye = surface.eye_on_ground(world);
            let normal = surface.sample(world).normal;
            let denominator = dot(ship.axes[1], normal);
            if denominator.abs() < 0.1 {
                return eye;
            }
            local[1] -= dot(sub(world, eye), normal) / denominator;
        }
        return ship.local_to_world(local);
    }
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
            query: None,
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
                query: None,
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
            query: None,
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

#[cfg(test)]
mod terrain_tests {
    use super::*;
    use crate::GroundSample;
    fn plane(_: SurfaceFrame, p: [f64; 3]) -> GroundSample {
        GroundSample {
            position_meters: [p[0], 0.4 * p[0], p[2]],
            normal: normalize([-0.4, 1.0, 0.0]),
            slope: 0.4,
        }
    }
    fn steep(_: SurfaceFrame, p: [f64; 3]) -> GroundSample {
        GroundSample {
            position_meters: [p[0], 2.0 * p[0], p[2]],
            normal: normalize([-2.0, 1.0, 0.0]),
            slope: 2.0,
        }
    }
    fn ledge(_: SurfaceFrame, p: [f64; 3]) -> GroundSample {
        GroundSample {
            position_meters: [p[0], if p[0] > 50.04 { 1.0 } else { 0.0 }, p[2]],
            normal: [0.0, 1.0, 0.0],
            slope: 0.0,
        }
    }
    fn setup(query: crate::GroundQuery) -> (CharacterController, ShipFrame, SurfaceFrame) {
        let surface = SurfaceFrame {
            body_center_meters: [0.0, -1e12, 0.0],
            radius_meters: 1e12,
            query: Some(query),
        };
        let ship = ShipFrame {
            origin_meters: [10000.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        };
        let world = surface.eye_on_ground([50.0, 0.0, 0.0]);
        (
            CharacterController {
                position: PositionState::Surface { world },
                ..CharacterController::default()
            },
            ship,
            surface,
        )
    }
    #[test]
    fn walking_tracks_actual_ground_and_blocks_steep_uphill_and_large_steps() {
        let (mut c, ship, s) = setup(plane);
        for _ in 0..100 {
            c.advance(
                Duration::from_millis(16),
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                ship,
                s,
                false,
                true,
            );
        }
        let p = c.snapshot(ship, s).eye_position_meters;
        assert!(p[0] > 55.0);
        assert!((p[1] - 0.4 * p[0] - PLAYER_EYE_HEIGHT_METERS).abs() < 1e-6);
        for query in [steep as crate::GroundQuery, ledge] {
            let (mut c, ship, s) = setup(query);
            let before = c.snapshot(ship, s).eye_position_meters;
            for _ in 0..100 {
                c.advance(
                    Duration::from_millis(16),
                    MovementInput {
                        forward: true,
                        ..MovementInput::default()
                    },
                    ship,
                    s,
                    false,
                    true,
                );
            }
            assert!(
                salimon_math::length(sub(c.snapshot(ship, s).eye_position_meters, before)) < 1e-6
            );
        }
        let (mut c, ship, s) = setup(steep);
        for _ in 0..100 {
            c.advance(
                Duration::from_millis(16),
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
                ship,
                s,
                false,
                true,
            );
        }
        assert!(
            c.snapshot(ship, s).eye_position_meters[0] < 45.0,
            "downhill escape remains possible"
        );
    }
    #[test]
    fn terrain_jump_is_ballistic_and_held_input_cannot_relaunch() {
        let (mut c, ship, s) = setup(plane);
        for _ in 0..20 {
            c.advance(
                Duration::from_millis(16),
                MovementInput {
                    jump: true,
                    ..MovementInput::default()
                },
                ship,
                s,
                false,
                true,
            );
        }
        let p = c.snapshot(ship, s).eye_position_meters;
        assert!(p[1] - 0.4 * p[0] - PLAYER_EYE_HEIGHT_METERS > 0.8);
        for _ in 0..100 {
            c.advance(
                Duration::from_millis(16),
                MovementInput {
                    jump: true,
                    ..MovementInput::default()
                },
                ship,
                s,
                false,
                true,
            );
        }
        let p = c.snapshot(ship, s).eye_position_meters;
        assert!((p[1] - 0.4 * p[0] - PLAYER_EYE_HEIGHT_METERS).abs() < 1e-6);
    }
}
