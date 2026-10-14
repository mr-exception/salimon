//! Inertial force-based flight and solid-body collision correction.

use super::ShipController;
use crate::FlightState;
use salimon_math::{add, scale};
use std::time::Duration;

impl ShipController {
    pub(super) fn advance_powered_flight(&mut self, delta: Duration, powered_seconds: f64) {
        let start = self.pose.position_meters;
        let velocity = self.velocity;
        self.integrate_thrust(powered_seconds, self.thruster_percentage);
        self.integrate_thrust((delta.as_secs_f64() - powered_seconds).max(0.0), 0);
        self.sweep_contact(start, self.pose.position_meters, velocity, delta);
    }

    fn integrate_thrust(&mut self, seconds: f64, throttle: i8) {
        let acceleration = scale(
            self.pose.axes()[0],
            self.propulsion
                .acceleration_meters_per_second_squared(throttle),
        );
        self.pose.position_meters = add(
            self.pose.position_meters,
            add(
                scale(self.velocity, seconds),
                scale(acceleration, 0.5 * seconds * seconds),
            ),
        );
        self.velocity = add(self.velocity, scale(acceleration, seconds));
    }

    #[must_use]
    pub fn speed_meters_per_second(&self) -> f64 {
        let [x, y, z] = self.velocity_meters_per_second();
        x.hypot(y).hypot(z)
    }

    #[must_use]
    pub fn velocity_meters_per_second(&self) -> [f64; 3] {
        match self.flight_state {
            FlightState::Flying | FlightState::Bouncing { .. } => self.velocity,
            FlightState::AssistedLanding { .. } | FlightState::AssistedTakeoff { .. } => {
                let assist = self.assist.as_ref().expect("active assistance has a path");
                assist.velocity_meters_per_second
            }
            FlightState::Landed { .. } | FlightState::GroundedAfterCrash { .. } => [0.0; 3],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{COLLISION_RADIUS_METERS, ShipPose};
    use salimon_math::length as vector_length;
    use salimon_world::CELESTIAL_BODIES;
    use salimon_world::CelestialBodyId;
    use std::time::Duration;

    #[test]
    fn long_direct_updates_do_not_discard_travel() {
        let pose = ShipPose {
            position_meters: [0.0; 3],
            orientation: [0.0, 0.0, 0.0, 1.0],
        };
        let mut whole = ShipController::flying(pose, 10);
        let mut split = whole.clone();
        whole.advance(Duration::from_secs(2));
        for _ in 0..20 {
            split.advance(Duration::from_millis(100));
        }
        assert!(
            (whole.snapshot().pose.position_meters[0] - split.snapshot().pose.position_meters[0])
                .abs()
                < 1e-9
        );
        assert!(whole.snapshot().pose.position_meters[0] > 0.0);
    }

    #[test]
    fn leaving_cockpit_does_not_change_flying_motion_or_thruster() {
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: [10.0, 20.0, 30.0],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            37,
        );
        let before = ship.snapshot();
        ship.set_cockpit_control(false);
        ship.advance(Duration::from_millis(100));
        let after = ship.snapshot();
        assert_eq!(after.thruster_percentage, before.thruster_percentage);
        assert_eq!(after.pose.orientation, before.pose.orientation);
        assert!(after.speed_meters_per_second > before.speed_meters_per_second);
        assert!(after.pose.position_meters[0] > before.pose.position_meters[0]);
        assert!(!after.cockpit_control_active);
    }

    #[test]
    fn thruster_changes_by_exact_points_only_with_cockpit_authority() {
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            99,
        );
        ship.adjust_thruster(1);
        assert_eq!(ship.snapshot().thruster_percentage, 100);
        assert_eq!(ship.snapshot().speed_meters_per_second, 0.0);
        ship.adjust_thruster(1);
        assert_eq!(ship.snapshot().thruster_percentage, 100);
        ship.adjust_thruster(-1);
        assert_eq!(ship.snapshot().thruster_percentage, 99);
        ship.set_cockpit_control(false);
        ship.adjust_thruster(-1);
        assert_eq!(ship.snapshot().thruster_percentage, 99);
    }

    #[test]
    fn solid_body_penetration_is_corrected_to_nearest_valid_position() {
        let earth = CELESTIAL_BODIES
            .iter()
            .find(|body| body.id == CelestialBodyId::Earth)
            .unwrap();
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: earth.center.meters(),
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
        );
        ship.advance(Duration::ZERO);
        let position = ship.snapshot().pose.position_meters;
        let distance = vector_length([
            position[0] - earth.center.meters()[0],
            position[1] - earth.center.meters()[1],
            position[2] - earth.center.meters()[2],
        ]);
        assert!((distance - earth.radius_meters - COLLISION_RADIUS_METERS).abs() < 1.0e-3);
    }
}

#[cfg(test)]
mod inertial_tests {
    use super::*;
    use crate::{Propulsion, ShipPose, SteeringInput};
    use std::time::Duration;

    fn ship(throttle: i8, velocity: [f64; 3]) -> ShipController {
        ShipController::flying_with_velocity(
            ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            throttle,
            velocity,
        )
        .unwrap()
    }

    #[test]
    fn turning_while_coasting_preserves_world_momentum_and_true_speed() {
        let mut ship = ship(0, [300.0, -400.0, 0.0]);
        ship.set_steering_input(SteeringInput {
            yaw: 1.0,
            ..Default::default()
        });
        for _ in 0..100 {
            ship.advance(Duration::from_millis(16));
        }
        let state = ship.snapshot();
        assert_ne!(state.pose.orientation, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(state.velocity_meters_per_second, [300.0, -400.0, 0.0]);
        assert_eq!(state.speed_meters_per_second, 500.0);
        assert!((state.pose.position_meters[0] - 480.0).abs() < 1e-9);
    }

    #[test]
    fn forward_reverse_and_mass_obey_force_over_mass() {
        let mut forward = ship(100, [0.0; 3]);
        forward.advance(Duration::from_secs(1));
        assert_eq!(
            forward.snapshot().velocity_meters_per_second,
            [40.0, 0.0, 0.0]
        );
        assert_eq!(forward.snapshot().pose.position_meters, [20.0, 0.0, 0.0]);
        let mut reverse = ship(0, [0.0; 3]);
        reverse.integrate_thrust(1.0, -100);
        assert_eq!(
            reverse.snapshot().velocity_meters_per_second,
            [-20.0, 0.0, 0.0]
        );
        assert_eq!(reverse.snapshot().pose.position_meters, [-10.0, 0.0, 0.0]);
        forward.set_propulsion(Propulsion::new(200_000.0, 4_000_000.0, 2_000_000.0).unwrap());
        forward.advance(Duration::from_secs(1));
        assert_eq!(
            forward.snapshot().velocity_meters_per_second,
            [60.0, 0.0, 0.0]
        );
        assert_eq!(forward.snapshot().total_mass_kg, 200_000.0);
    }

    #[test]
    fn arbitrary_partitions_and_high_speed_keep_all_travel() {
        let mut whole = ship(100, [10_000_000.0, 0.0, 0.0]);
        let mut split = whole.clone();
        whole.advance(Duration::from_millis(1003));
        for ms in [1, 307, 90, 5, 600] {
            split.advance(Duration::from_millis(ms));
        }
        assert!(
            (whole.snapshot().pose.position_meters[0] - split.snapshot().pose.position_meters[0])
                .abs()
                < 1e-7
        );
        assert!(
            (whole.snapshot().speed_meters_per_second - split.snapshot().speed_meters_per_second)
                .abs()
                < 1e-7
        );
        let velocity = whole.velocity_meters_per_second();
        whole.adjust_thruster(-100);
        whole.set_cockpit_control(false);
        whole.advance(Duration::from_secs(1));
        assert_eq!(whole.velocity_meters_per_second(), velocity);
        assert!(ShipController::flying_with_velocity(whole.pose, 0, [f64::NAN; 3]).is_err());
    }
}

#[cfg(test)]
mod energy_flight_tests {
    use super::*;
    use crate::{EnergyConfig, ShipPose, SteeringInput};
    #[test]
    fn converted_charge_and_consumers_share_one_step_without_capacity_clipping() {
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            100,
        );
        ship.configure_energy_core(
            EnergyConfig {
                capacity_joules: 100,
                forward_watts: 100.0,
                reverse_watts: 50.0,
                gravity_idle_watts: 0.0,
                cockpit_idle_watts: 0.0,
            },
            100.0,
        )
        .unwrap();
        ship.advance_with_core_input(Duration::from_secs(1), 100.0)
            .unwrap();
        assert_eq!(ship.snapshot().energy_core.stored_joules, 100);
        assert_eq!(ship.snapshot().velocity_meters_per_second, [40.0, 0.0, 0.0]);
        ship.set_stored_core_energy_joules(0);
        ship.advance_with_core_input(Duration::from_secs(1), 50.0)
            .unwrap();
        assert_eq!(ship.snapshot().velocity_meters_per_second, [60.0, 0.0, 0.0]);
        assert_eq!(ship.snapshot().energy_core.stored_joules, 0);
        let before = ship.snapshot();
        assert!(
            ship.advance_with_core_input(Duration::from_secs(1), f64::NAN)
                .is_err()
        );
        assert_eq!(ship.snapshot(), before);
    }

    #[test]
    fn exhaustion_coasts_without_losing_selection_or_velocity_then_charge_resumes() {
        let mut ship = ShipController::flying_with_velocity(
            ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            100,
            [10.0, 0.0, 0.0],
        )
        .unwrap();
        ship.configure_energy_core(
            EnergyConfig {
                capacity_joules: 1000,
                forward_watts: 100.0,
                reverse_watts: 50.0,
                gravity_idle_watts: 0.0,
                cockpit_idle_watts: 0.0,
            },
            50.0,
        )
        .unwrap();
        let mut split = ship.clone();
        ship.advance(Duration::from_secs(1));
        for ms in [123, 377, 500] {
            split.advance(Duration::from_millis(ms));
        }
        let state = ship.snapshot();
        assert!(!state.power_available);
        assert_eq!(state.thruster_percentage, 100);
        assert_eq!(state.velocity_meters_per_second, [30.0, 0.0, 0.0]);
        assert_eq!(state.pose.position_meters, [25.0, 0.0, 0.0]);
        assert!((split.snapshot().pose.position_meters[0] - 25.0).abs() < 1e-10);
        assert!(!ship.adjust_thruster(-100));
        ship.set_steering_input(SteeringInput {
            yaw: 1.0,
            ..Default::default()
        });
        ship.advance(Duration::from_secs(1));
        assert_eq!(ship.snapshot().pose.orientation, state.pose.orientation);
        assert_eq!(
            ship.snapshot().velocity_meters_per_second,
            state.velocity_meters_per_second
        );
        ship.exchange_core_energy_joules(100.0, 0.0).unwrap();
        assert!(ship.snapshot().power_available);
        ship.advance(Duration::from_millis(500));
        assert_eq!(ship.snapshot().velocity_meters_per_second, [50.0, 0.0, 0.0]);
        assert_eq!(ship.snapshot().energy_core.stored_joules, 50);
    }
}
