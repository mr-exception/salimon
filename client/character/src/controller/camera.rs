//! Mouse look and presentation basis for every movement mode.
use super::*;
impl CharacterController {
    pub fn apply_mouse_delta(&mut self, delta_x: f64, delta_y: f64) {
        if !delta_x.is_finite() || !delta_y.is_finite() {
            return;
        }
        self.yaw_radians =
            (self.yaw_radians + delta_x * LOOK_SENSITIVITY_RADIANS_PER_PIXEL).rem_euclid(TAU);
        self.pitch_radians = (self.pitch_radians - delta_y * LOOK_SENSITIVITY_RADIANS_PER_PIXEL)
            .clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
    }
    #[must_use]
    pub fn snapshot(&self, ship: ShipFrame, surface: SurfaceFrame) -> CharacterSnapshot {
        let surface = self.nearby_surface.unwrap_or(surface);
        let (eye, up, blend) = match self.position {
            PositionState::Cockpit => (ship.local_to_world(COCKPIT_POSITION), ship.axes[1], None),
            PositionState::Inside { local } => (ship.local_to_world(local), ship.axes[1], None),
            PositionState::Space { world } => (
                world,
                self.nearby_surface.map_or(self.eva_axes[1], |body| {
                    normalize(sub(world, body.body_center_meters))
                }),
                None,
            ),
            PositionState::Surface { world } => (
                world,
                normalize(sub(world, surface.body_center_meters)),
                None,
            ),
            PositionState::Doorway {
                world,
                elapsed,
                leaving_ship,
            } => {
                let fraction = (elapsed.as_secs_f64()
                    / DOORWAY_GRAVITY_BLEND_DURATION.as_secs_f64())
                .clamp(0.0, 1.0);
                let surface_up = normalize(sub(world, surface.body_center_meters));
                let amount = if leaving_ship {
                    fraction
                } else {
                    1.0 - fraction
                };
                (
                    world,
                    normalize(lerp(ship.axes[1], surface_up, amount)),
                    Some(fraction),
                )
            }
        };
        let view_axes = if matches!(self.position, PositionState::Space { .. }) {
            self.eva_axes
        } else {
            ship.axes
        };
        let base_forward = if matches!(self.position, PositionState::Surface { .. }) {
            surface_view_basis(ship.axes, up, self.yaw_radians).0
        } else {
            let (planar_forward, _) = planar_look(self.yaw_radians);
            normalize(add(
                scale(view_axes[0], planar_forward[0]),
                scale(view_axes[2], planar_forward[1]),
            ))
        };
        let right = normalize(cross(base_forward, up));
        let pitch = self.pitch_radians
            + if matches!(self.position, PositionState::Cockpit) {
                COCKPIT_VIEW_PITCH_RADIANS
            } else {
                0.0
            };
        let look = normalize(add(
            scale(base_forward, pitch.cos()),
            scale(up, pitch.sin()),
        ));
        let corrected_look = normalize(reject(look, right));

        CharacterSnapshot {
            location: self.location(),
            eye_position_meters: eye,
            look_target_meters: add(eye, corrected_look),
            up: up.map(|value| value as f32),
            local_ship_position_meters: self.local_ship_position(),
            doorway_blend_fraction: blend,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    #[test]
    fn horizontal_mouse_delta_turns_view_in_screen_direction_inside_and_in_cockpit() {
        let mut right = CharacterController::default();
        right.apply_mouse_delta(10.0, 0.0);
        let right_snapshot = right.snapshot(frame(), surface());
        assert!(right_snapshot.look_target_meters[2] > right_snapshot.eye_position_meters[2]);

        let mut left = CharacterController::default();
        left.apply_mouse_delta(-10.0, 0.0);
        let left_snapshot = left.snapshot(frame(), surface());
        assert!(left_snapshot.look_target_meters[2] < left_snapshot.eye_position_meters[2]);

        right.enter_cockpit();
        let cockpit_snapshot = right.snapshot(frame(), surface());
        assert!(cockpit_snapshot.look_target_meters[2] > cockpit_snapshot.eye_position_meters[2]);
    }

    #[test]
    fn vertical_mouse_delta_keeps_the_existing_pitch_direction() {
        let mut down = CharacterController::default();
        down.apply_mouse_delta(0.0, 10.0);
        let down_snapshot = down.snapshot(frame(), surface());
        assert!(down_snapshot.look_target_meters[1] < down_snapshot.eye_position_meters[1]);

        let mut up = CharacterController::default();
        up.apply_mouse_delta(0.0, -10.0);
        let up_snapshot = up.snapshot(frame(), surface());
        assert!(up_snapshot.look_target_meters[1] > up_snapshot.eye_position_meters[1]);
    }

    #[test]
    fn default_cockpit_view_keeps_the_forward_window_in_sight() {
        let mut controller = CharacterController::default();
        controller.enter_cockpit();
        let snapshot = controller.snapshot(frame(), surface());
        let look = sub(snapshot.look_target_meters, snapshot.eye_position_meters);

        assert!(look[0] > 0.99, "cockpit view must remain primarily forward");
        assert!(
            look[1] > -0.11,
            "cockpit view must clear the console and nose"
        );
        assert!(look[2].abs() < 1.0e-12);
    }
}
