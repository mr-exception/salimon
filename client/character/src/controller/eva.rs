//! Independent world drift, view-relative flight assist and nearby-body influence.
use super::*;

impl CharacterController {
    pub(super) fn advance_eva(&mut self, world: [f64; 3], step: &Step) {
        let Step {
            seconds,
            input,
            ship,
            surface,
            door_open,
            ..
        } = *step;

        // Integrate the inherited world motion independently of the ship.
        // The runtime has already advanced the ship by this same interval.
        let gravity = self.nearby_surface.map_or([0.0; 3], |body| {
            scale(
                normalize(sub(body.body_center_meters, world)),
                FIXED_GRAVITY_METERS_PER_SECOND_SQUARED,
            )
        });
        let drifted = add(
            world,
            add(
                scale(self.eva_inherited_velocity, seconds),
                scale(gravity, 0.5 * seconds * seconds),
            ),
        );
        self.eva_inherited_velocity = add(self.eva_inherited_velocity, scale(gravity, seconds));
        let mut previous = ship.world_to_local(drifted);
        if !door_open
            && previous[0] > EXTERIOR_AFT
            && previous[0] < DOORWAY_FORWARD + PLAYER_RADIUS_METERS
            && previous[2].abs() <= DOORWAY_SIDE_LIMIT
        {
            previous[0] = EXTERIOR_AFT;
        }
        let view = self.snapshot(ship, surface);
        let forward = normalize(sub(view.look_target_meters, view.eye_position_meters));
        let up = self.nearby_surface.map_or(self.eva_axes[1], |body| {
            normalize(sub(world, body.body_center_meters))
        });
        let right = normalize_or(cross(forward, up), normalize(cross(self.eva_axes[0], up)));
        let movement = add(
            add(
                scale(forward, axis(input.forward, input.backward)),
                scale(right, axis(input.right, input.left)),
            ),
            scale(up, axis(input.jump, input.descend)),
        );
        let length = dot(movement, movement).sqrt().max(1.0);
        self.eva_control_velocity = scale(movement, WALK_SPEED_METERS_PER_SECOND / length);
        let proposed = ship.world_to_local(add(
            ship.local_to_world(previous),
            scale(self.eva_control_velocity, seconds),
        ));
        let mut local = proposed;
        if overlaps_exterior_hull(local) {
            local = slide_around_obstacles(previous, local, &EXTERIOR_OBSTACLES);
            if !door_open {
                local = slide_around_obstacles(previous, local, &CLOSED_GATE_OBSTACLE);
            }
        }
        local = slide_around_appendages(previous, local, world_round_trip_tolerance(world));
        let through_gate = previous[0] < DOORWAY_FORWARD
            && local[0] >= DOORWAY_FORWARD - COLLISION_EPSILON
            && previous[2].abs() <= DOORWAY_SIDE_LIMIT + COLLISION_EPSILON
            && local[2].abs() <= DOORWAY_SIDE_LIMIT + COLLISION_EPSILON
            && local[1] >= PLAYER_EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT - COLLISION_EPSILON
            && local[1]
                <= DOORWAY_CEILING_HEIGHT - (PLAYER_BODY_HEIGHT_METERS - PLAYER_EYE_HEIGHT_METERS);
        if door_open && through_gate && local[0] > previous[0] {
            local[0] = local[0].max(DOORWAY_FORWARD);
            self.vertical_speed = 0.0;
            self.nearby_surface = None;
            self.eva_control_velocity = [0.0; 3];
            // Preserve the world look direction when adopting ship gravity.
            self.yaw_radians = dot(forward, ship.axes[2])
                .atan2(dot(forward, ship.axes[0]))
                .rem_euclid(TAU);
            self.pitch_radians = dot(forward, ship.axes[1]).clamp(-1.0, 1.0).asin();
            self.position = PositionState::Inside { local };
        } else {
            // Remove blocked velocity components from inspection state.
            let actual = scale(
                sub(ship.local_to_world(local), ship.local_to_world(previous)),
                1.0 / seconds,
            );
            self.eva_control_velocity = actual;
            let next = ship.local_to_world(local);
            if self.nearby_surface.is_some()
                && dot(
                    sub(next, surface.body_center_meters),
                    sub(next, surface.body_center_meters),
                )
                .sqrt()
                    <= surface.radius_meters + PLAYER_EYE_HEIGHT_METERS
            {
                self.yaw_radians = dot(forward, ship.axes[2])
                    .atan2(dot(forward, ship.axes[0]))
                    .rem_euclid(TAU);
                self.position = PositionState::Surface {
                    world: project_eye_to_surface(next, surface),
                };
                self.eva_inherited_velocity = [0.0; 3];
                self.eva_control_velocity = [0.0; 3];
            } else {
                self.position = PositionState::Space { world: next };
            }
        }
    }
}

impl CharacterController {
    /// World velocity is retained at exit; control velocity uses flight assist.
    #[must_use]
    pub fn eva_velocity(&self) -> Option<[f64; 3]> {
        matches!(self.position, PositionState::Space { .. })
            .then(|| add(self.eva_inherited_velocity, self.eva_control_velocity))
    }

    /// Runtime selects the nearby solid body using the shared world-distance rule.
    /// Changing movement mode never projects position or resets drift velocity.
    pub fn set_nearby_surface(&mut self, surface: Option<SurfaceFrame>) {
        let PositionState::Space { world } = self.position else {
            if matches!(
                self.position,
                PositionState::Inside { .. } | PositionState::Cockpit
            ) {
                self.nearby_surface = None;
            }
            return;
        };
        if self.nearby_surface == surface {
            return;
        }
        if let Some(body) = surface {
            let (planar, _) = planar_look(self.yaw_radians);
            let old_forward = normalize(add(
                scale(self.eva_axes[0], planar[0]),
                scale(self.eva_axes[2], planar[1]),
            ));
            let look = add(
                scale(old_forward, self.pitch_radians.cos()),
                scale(self.eva_axes[1], self.pitch_radians.sin()),
            );
            let up = normalize(sub(world, body.body_center_meters));
            let forward = normalize_or(reject(old_forward, up), orthogonal_tangent(up));
            self.eva_axes = [forward, up, normalize(cross(forward, up))];
            self.yaw_radians = dot(look, self.eva_axes[2])
                .atan2(dot(look, forward))
                .rem_euclid(TAU);
            self.pitch_radians = dot(look, up).clamp(-1.0, 1.0).asin();
        }
        self.nearby_surface = surface;
    }
}
#[cfg(test)]
mod nearby_eva_tests {
    use super::*;

    fn body() -> SurfaceFrame {
        SurfaceFrame {
            body_center_meters: [0.0; 3],
            radius_meters: 6_000_000.0,
        }
    }
    fn ship() -> ShipFrame {
        ShipFrame {
            origin_meters: [1_000_000.0, 0.0, 0.0],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }
    fn player(altitude: f64) -> CharacterController {
        CharacterController {
            position: PositionState::Space {
                world: [0.0, body().radius_meters + altitude, 0.0],
            },
            eva_inherited_velocity: [0.0, -25_000.0, 0.0],
            ..CharacterController::default()
        }
    }

    #[test]
    fn influence_changes_preserve_position_velocity_and_look() {
        for yaw in [0.0, 0.8, 2.4] {
            let mut p = player(3_000_000.0);
            p.yaw_radians = yaw;
            p.pitch_radians = -0.6;
            let before = p.snapshot(ship(), body());
            let velocity = p.eva_velocity();
            p.set_nearby_surface(Some(body()));
            assert_eq!(p.location(), CharacterLocation::NearbyBody);
            assert_eq!(
                p.snapshot(ship(), body()),
                CharacterSnapshot {
                    location: CharacterLocation::NearbyBody,
                    ..before
                }
            );
            assert_eq!(p.eva_velocity(), velocity);
            p.set_nearby_surface(None);
            assert_eq!(p.location(), CharacterLocation::Space);
            assert_eq!(p.snapshot(ship(), body()), before);
            assert_eq!(p.eva_velocity(), velocity);
        }
        // View parallel to radial gravity must still produce a finite tangent frame.
        let mut p = player(3_000_000.0);
        p.eva_axes = [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]];
        p.set_nearby_surface(Some(body()));
        assert!(
            p.snapshot(ship(), body())
                .look_target_meters
                .iter()
                .all(|v| v.is_finite())
        );
        assert!(dot(p.eva_axes[0], p.eva_axes[1]).abs() < 1e-12);
    }

    #[test]
    fn radial_gravity_is_applied_once_and_is_frame_rate_independent() {
        let mut results = Vec::new();
        for ms in [10, 20, 100] {
            let mut p = player(3_000_000.0);
            p.set_nearby_surface(Some(body()));
            for _ in 0..1000 / ms {
                p.set_nearby_surface(Some(body()));
                p.advance(
                    Duration::from_millis(ms),
                    MovementInput::default(),
                    ship(),
                    body(),
                    false,
                    false,
                );
            }
            let eye = p.snapshot(ship(), body()).eye_position_meters;
            assert!((eye[1] - (9_000_000.0 - 25_000.0 - 4.905)).abs() < 1e-6);
            assert!((p.eva_velocity().unwrap()[1] + 25_009.81).abs() < 1e-7);
            results.push(eye);
            let velocity = p.eva_velocity();
            p.set_nearby_surface(None);
            p.advance(
                Duration::from_millis(ms),
                MovementInput::default(),
                ship(),
                body(),
                false,
                false,
            );
            assert_eq!(p.eva_velocity(), velocity);
        }
        for eye in results {
            assert!((eye[1] - (9_000_000.0 - 25_004.905)).abs() < 1e-6);
        }
    }

    #[test]
    fn contact_lands_on_selected_body_and_interior_clears_influence() {
        let mut p = player(3.0);
        p.eva_inherited_velocity = [0.0, -10.0, 0.0];
        p.set_nearby_surface(Some(body()));
        for _ in 0..4 {
            p.advance(
                Duration::from_millis(100),
                MovementInput::default(),
                ship(),
                body(),
                false,
                false,
            );
        }
        assert_eq!(p.location(), CharacterLocation::Surface);
        let other = SurfaceFrame {
            radius_meters: 1.0,
            ..body()
        };
        let eye = p.snapshot(ship(), other).eye_position_meters;
        assert!((eye[1] - body().radius_meters - PLAYER_EYE_HEIGHT_METERS).abs() < 1e-8);
        assert_eq!(p.eva_velocity(), None);
        p.position = PositionState::Inside {
            local: PLAYER_START,
        };
        p.set_nearby_surface(Some(body()));
        assert_eq!(p.nearby_surface, None);
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    #[test]
    fn space_access_preserves_height_blocks_hull_and_handles_closing_in_gate() {
        let mut controller = CharacterController {
            position: PositionState::Inside {
                local: [-7.0, PLAYER_START[1], 0.0],
            },
            ..CharacterController::default()
        };
        let backward = MovementInput {
            backward: true,
            ..MovementInput::default()
        };
        for _ in 0..12 {
            controller.advance(
                Duration::from_millis(16),
                backward,
                frame(),
                surface(),
                true,
                false,
            );
        }
        assert_eq!(controller.location(), CharacterLocation::Space);
        let before = controller.snapshot(frame(), surface());
        controller.advance(
            Duration::from_millis(16),
            MovementInput::default(),
            frame(),
            surface(),
            false,
            false,
        );
        let closed = controller.snapshot(frame(), surface());
        assert!(frame().world_to_local(closed.eye_position_meters)[0] <= EXTERIOR_AFT);
        assert!((closed.eye_position_meters[1] - before.eye_position_meters[1]).abs() < 1e-9);
        for _ in 0..30 {
            controller.advance(
                Duration::from_millis(16),
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                frame(),
                surface(),
                false,
                false,
            );
        }
        assert_eq!(controller.location(), CharacterLocation::Space);
        assert!(
            (frame().world_to_local(controller.snapshot(frame(), surface()).eye_position_meters)
                [0]
                - EXTERIOR_AFT)
                .abs()
                < 1e-7
        );
        // The open gate never makes its opaque rear windows passable.
        controller.position = PositionState::Space {
            world: frame().local_to_world([-9.0, PLAYER_START[1], 3.0]),
        };
        for _ in 0..30 {
            controller.advance(
                Duration::from_millis(16),
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                frame(),
                surface(),
                true,
                false,
            );
        }
        assert_eq!(controller.location(), CharacterLocation::Space);
        assert!(
            frame().world_to_local(controller.snapshot(frame(), surface()).eye_position_meters)[0]
                <= EXTERIOR_AFT + 1e-7
        );
    }

    #[test]
    fn moving_space_exit_inherits_velocity_and_keeps_world_motion_independent() {
        let velocity = [120.0, -7.0, 4.0];
        let mut ship = frame();
        let mut player = inside_at(-7.0, 0.0);
        let backward = MovementInput {
            backward: true,
            ..MovementInput::default()
        };
        for _ in 0..1 {
            ship.origin_meters = add(ship.origin_meters, scale(velocity, 0.016));
            player.advance_with_motion(
                Duration::from_millis(16),
                backward,
                (ship, velocity),
                surface(),
                true,
                false,
            );
        }
        assert_eq!(player.location(), CharacterLocation::Space);
        assert_eq!(player.eva_velocity(), Some(velocity));
        let exit = player.snapshot(ship, surface()).eye_position_meters;
        let local = ship.world_to_local(exit);
        for _ in 0..600 {
            ship.origin_meters = add(ship.origin_meters, scale(velocity, 0.016));
            player.advance_with_motion(
                Duration::from_millis(16),
                MovementInput::default(),
                (ship, velocity),
                surface(),
                true,
                false,
            );
        }
        let current = player.snapshot(ship, surface()).eye_position_meters;
        for (i, expected) in local.iter().enumerate() {
            assert!((ship.world_to_local(current)[i] - expected).abs() < 1e-8);
        }
        // Changing the ship velocity must not change an already detached player.
        ship.origin_meters = add(ship.origin_meters, [10.0, 0.0, 0.0]);
        player.advance_with_motion(
            Duration::from_millis(16),
            MovementInput::default(),
            (ship, [625.0, 0.0, 0.0]),
            surface(),
            true,
            false,
        );
        let next = player.snapshot(ship, surface()).eye_position_meters;
        for i in 0..3 {
            assert!((next[i] - current[i] - velocity[i] * 0.016).abs() < 1e-8);
        }
        assert_eq!(player.eva_velocity(), Some(velocity));
    }

    #[test]
    fn eva_three_axis_control_is_normalized_and_frame_rate_independent() {
        let mut results = Vec::new();
        for step_ms in [10, 20, 100] {
            let mut player = CharacterController {
                position: PositionState::Space {
                    world: [-30.0, 12.0, 0.0],
                },
                ..CharacterController::default()
            };
            player.apply_mouse_delta(100.0, -80.0);
            let before = player.snapshot(frame(), surface()).eye_position_meters;
            for _ in 0..(1000 / step_ms) {
                player.advance(
                    Duration::from_millis(step_ms),
                    MovementInput {
                        backward: true,
                        right: true,
                        jump: true,
                        ..MovementInput::default()
                    },
                    frame(),
                    surface(),
                    true,
                    false,
                );
            }
            let after = player.snapshot(frame(), surface()).eye_position_meters;
            let displacement = sub(after, before);
            assert!(
                (dot(displacement, displacement).sqrt() - WALK_SPEED_METERS_PER_SECOND).abs()
                    < 1e-8
            );
            assert!(displacement[1] > 0.0);
            results.push(after);
        }
        for result in &results[1..] {
            for (&actual, &reference) in result.iter().zip(results[0].iter()) {
                assert!((actual - reference).abs() < 1e-8);
            }
        }
    }

    #[test]
    fn eva_orientation_is_independent_of_ship_rotation() {
        let player = CharacterController {
            position: PositionState::Space {
                world: [-20.0, 12.0, 0.0],
            },
            ..CharacterController::default()
        };
        let before = player.snapshot(frame(), surface());
        let rotated = ShipFrame {
            axes: [[0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]],
            ..frame()
        };
        let after = player.snapshot(rotated, surface());
        assert_eq!(before.look_target_meters, after.look_target_meters);
        assert_eq!(before.up, after.up);
    }
}
