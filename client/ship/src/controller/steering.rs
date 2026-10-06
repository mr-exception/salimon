//! Cockpit steering input, ramps and local-axis rotations.

use super::ShipController;
use crate::orientation::{axis_angle_quaternion, normalized_quaternion, quaternion_multiply};
use crate::{FlightState, STEERING_RAMP_SECONDS, STEERING_RATE_RADIANS_PER_SECOND, SteeringInput};

fn approach(current: f64, target: f64, maximum_delta: f64) -> f64 {
    current + (target - current).clamp(-maximum_delta, maximum_delta)
}

impl ShipController {
    pub fn set_steering_input(&mut self, input: SteeringInput) {
        self.steering_target =
            if self.cockpit_control_active && self.flight_state == FlightState::Flying {
                SteeringInput {
                    pitch: input.pitch.clamp(-1.0, 1.0),
                    yaw: input.yaw.clamp(-1.0, 1.0),
                    roll: input.roll.clamp(-1.0, 1.0),
                }
            } else {
                SteeringInput::default()
            };
    }

    pub(super) fn advance_steering(&mut self, seconds: f64) {
        let step = seconds / STEERING_RAMP_SECONDS;
        self.steering.pitch = approach(self.steering.pitch, self.steering_target.pitch, step);
        self.steering.yaw = approach(self.steering.yaw, self.steering_target.yaw, step);
        self.steering.roll = approach(self.steering.roll, self.steering_target.roll, step);
        if self.flight_state != FlightState::Flying {
            return;
        }
        let angles = [
            self.steering.roll * STEERING_RATE_RADIANS_PER_SECOND * seconds,
            self.steering.yaw * STEERING_RATE_RADIANS_PER_SECOND * seconds,
            self.steering.pitch * STEERING_RATE_RADIANS_PER_SECOND * seconds,
        ];
        for (axis, angle) in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
            .into_iter()
            .zip(angles)
        {
            self.pose.orientation =
                quaternion_multiply(self.pose.orientation, axis_angle_quaternion(axis, angle));
        }
        self.pose.orientation = normalized_quaternion(self.pose.orientation);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ShipPose;
    use std::time::Duration;

    #[test]
    fn steering_ramps_then_holds_the_fixed_five_degree_rate() {
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
        );
        ship.set_steering_input(SteeringInput {
            pitch: 1.0,
            ..SteeringInput::default()
        });
        ship.advance(Duration::from_millis(60));
        assert_eq!(ship.snapshot().steering.pitch, 0.5);
        ship.advance(Duration::from_millis(60));
        assert_eq!(ship.snapshot().steering.pitch, 1.0);
        let before = ship.snapshot().pose.axes()[0];
        ship.advance(Duration::from_millis(100));
        let after = ship.snapshot().pose.axes()[0];
        let angle = before
            .iter()
            .zip(after)
            .map(|(left, right)| left * right)
            .sum::<f64>()
            .clamp(-1.0, 1.0)
            .acos();
        assert!((angle - 0.5_f64.to_radians()).abs() < 1.0e-10);
        assert!(after[1] > before[1], "positive pitch must raise the nose");
    }

    #[test]
    fn steering_target_clears_when_cockpit_is_left_without_changing_heading() {
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
        );
        ship.set_steering_input(SteeringInput {
            yaw: 1.0,
            ..SteeringInput::default()
        });
        ship.advance(Duration::from_millis(100));
        ship.set_cockpit_control(false);
        ship.advance(Duration::from_millis(100));
        ship.advance(Duration::from_millis(100));
        let settled = ship.snapshot();
        assert_eq!(settled.steering, SteeringInput::default());
        let orientation = settled.pose.orientation;
        ship.advance(Duration::from_secs(1));
        assert_eq!(ship.snapshot().pose.orientation, orientation);
    }
}
