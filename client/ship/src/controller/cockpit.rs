//! Cockpit authority and contextual messages.

use super::ShipController;
use crate::{FlightState, SteeringInput};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CockpitMessage {
    DoorLockedWhileInFlight,
    EnginesOff,
    CollisionSettling,
    GroundedAfterCrash,
    DoorObstructed,
    UnsafeLandingSite,
    InsufficientAssistEnergy,
    PressToLand,
    PressToTakeOff,
    CloseDoorBeforeTakeoff,
    LandingInProgress,
    TakeoffInProgress,
}

impl CockpitMessage {
    #[must_use]
    pub const fn text(self) -> &'static str {
        match self {
            Self::EnginesOff => "Engines off - press B to enable",
            Self::CollisionSettling => "Collision settling in progress",
            Self::GroundedAfterCrash => "Ship grounded after collision - press L to recover",
            Self::InsufficientAssistEnergy => "Insufficient Core energy for assistance",
            Self::UnsafeLandingSite => "Unsafe terrain: choose another landing site",
            Self::DoorObstructed => "Move clear of the boarding ramp",
            Self::DoorLockedWhileInFlight => "Door locked while in flight",
            Self::PressToLand => "Press L to land",
            Self::PressToTakeOff => "Press L to take off",
            Self::CloseDoorBeforeTakeoff => "Close door before takeoff",
            Self::LandingInProgress => "Assisted landing in progress",
            Self::TakeoffInProgress => "Assisted takeoff - flight controls locked",
        }
    }
}

impl ShipController {
    /// Survival authority loss: stop manual thrust/control without cancelling
    /// an already committed assist, changing heading or erasing world momentum.
    pub fn revoke_pilot_control(&mut self) {
        self.thruster_percentage = 0;
        self.set_cockpit_control(false);
    }

    pub fn set_cockpit_control(&mut self, active: bool) {
        self.cockpit_control_active = active;
        if !active {
            self.clear_flight_control_input();
        }
    }

    pub(super) fn clear_flight_control_input(&mut self) {
        self.clear_throttle_input();
        self.steering_target = SteeringInput::default();
        self.steering = SteeringInput::default();
    }

    pub fn clear_cockpit_message(&mut self) {
        self.cockpit_message = None;
    }

    #[must_use]
    pub fn contextual_cockpit_message(&self) -> Option<CockpitMessage> {
        if let Some(message) = self.cockpit_message {
            return Some(message);
        }
        if !self.energy_core.powered() {
            return None;
        }
        if self.cockpit_control_active && !self.engines_enabled {
            return Some(CockpitMessage::EnginesOff);
        }
        match self.flight_state {
            FlightState::Bouncing { .. } => Some(CockpitMessage::CollisionSettling),
            FlightState::GroundedAfterCrash { .. } => Some(CockpitMessage::GroundedAfterCrash),
            FlightState::AssistedLanding { .. } => Some(CockpitMessage::LandingInProgress),
            FlightState::AssistedTakeoff { .. } => Some(CockpitMessage::TakeoffInProgress),
            FlightState::Landed { .. } if self.cockpit_control_active => {
                Some(CockpitMessage::PressToTakeOff)
            }
            FlightState::Flying
                if self.cockpit_control_active && self.landing_target().is_some() =>
            {
                Some(CockpitMessage::PressToLand)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod blackout_tests {
    use super::*;
    use crate::{EnergyConfig, ShipPose, ThrottleDirection};
    use std::time::Duration;
    #[test]
    fn blackout_preserves_signed_selection_heading_and_coasting_then_restores_thrust() {
        for throttle in [-100, 100] {
            let mut ship = ShipController::flying_with_velocity(
                ShipPose {
                    position_meters: [0.0; 3],
                    orientation: [0.0, 0.0, 0.0, 1.0],
                },
                throttle,
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
                0.0,
            )
            .unwrap();
            let heading = ship.snapshot().pose.orientation;
            ship.set_throttle_key(ThrottleDirection::Up, true);
            assert!(!ship.adjust_thruster(1));
            ship.set_steering_input(SteeringInput {
                yaw: 1.0,
                ..Default::default()
            });
            ship.advance(Duration::from_secs(1));
            assert_eq!(ship.snapshot().thruster_percentage, throttle);
            assert_eq!(ship.snapshot().pose.orientation, heading);
            assert_eq!(ship.snapshot().velocity_meters_per_second, [10.0, 0.0, 0.0]);
            ship.set_cockpit_control(false);
            ship.exchange_core_energy_joules(500.0, 0.0).unwrap();
            ship.advance(Duration::from_secs(1));
            assert!(ship.snapshot().power_available);
            assert_eq!(ship.snapshot().thruster_percentage, throttle);
            assert_eq!(ship.snapshot().pose.orientation, heading);
            let expected = if throttle > 0 { 10.001 } else { 9.9995 };
            assert_eq!(ship.snapshot().velocity_meters_per_second[0], expected);
        }
    }
    #[test]
    fn steering_stops_at_power_boundary_and_exit_clears_ramp_residue() {
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
        );
        ship.configure_energy_core(
            EnergyConfig {
                capacity_joules: 1000,
                forward_watts: 0.0,
                reverse_watts: 0.0,
                gravity_idle_watts: 10.0,
                cockpit_idle_watts: 0.0,
            },
            5.0,
        )
        .unwrap();
        ship.set_steering_input(SteeringInput {
            yaw: 1.0,
            ..Default::default()
        });
        let mut boundary = ship.clone();
        boundary.advance(Duration::from_millis(500));
        ship.advance(Duration::from_secs(1));
        assert_eq!(
            ship.snapshot().pose.orientation,
            boundary.snapshot().pose.orientation
        );
        assert_eq!(ship.snapshot().steering, SteeringInput::default());
        let heading = ship.snapshot().pose.orientation;
        ship.exchange_core_energy_joules(100.0, 0.0).unwrap();
        ship.advance(Duration::from_secs(1));
        assert_eq!(ship.snapshot().pose.orientation, heading);
        ship.set_steering_input(SteeringInput {
            pitch: 1.0,
            ..Default::default()
        });
        ship.advance(Duration::from_millis(50));
        let exit_heading = ship.snapshot().pose.orientation;
        ship.set_cockpit_control(false);
        ship.advance(Duration::from_secs(1));
        assert_eq!(ship.snapshot().pose.orientation, exit_heading);
    }
}

#[cfg(test)]
mod survival_revocation_tests {
    use super::*;
    #[test]
    fn revocation_stops_manual_thrust_even_during_blackout_and_never_erases_velocity() {
        let mut ship = ShipController::flying_with_velocity(
            crate::ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            50,
            [10.0, 20.0, 30.0],
        )
        .unwrap();
        ship.set_stored_core_energy_joules(0);
        ship.revoke_pilot_control();
        ship.exchange_core_energy_joules(1e9, 0.0).unwrap();
        ship.advance(std::time::Duration::from_secs(1));
        let s = ship.snapshot();
        assert_eq!(s.thruster_percentage, 0);
        assert_eq!(s.velocity_meters_per_second, [10.0, 20.0, 30.0]);
        assert_eq!(s.pose.position_meters, [10.0, 20.0, 30.0]);
        assert!(!s.cockpit_control_active);
    }
}
