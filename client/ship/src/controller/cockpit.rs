//! Cockpit authority and contextual messages.

use super::ShipController;
use crate::{FlightState, SteeringInput};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CockpitMessage {
    DoorLockedWhileInFlight,
    CollisionSettling,
    GroundedAfterCrash,
    DoorObstructed,
    UnsafeLandingSite,
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
            Self::CollisionSettling => "Collision settling in progress",
            Self::GroundedAfterCrash => "Ship grounded after collision",
            Self::UnsafeLandingSite => "Unsafe terrain: choose another landing site",
            Self::DoorObstructed => "Move clear of the boarding ramp",
            Self::DoorLockedWhileInFlight => "Door locked while in flight",
            Self::PressToLand => "Press L to land",
            Self::PressToTakeOff => "Press L to take off",
            Self::CloseDoorBeforeTakeoff => "Close door before takeoff",
            Self::LandingInProgress => "Assisted landing in progress",
            Self::TakeoffInProgress => "Assisted takeoff in progress",
        }
    }
}

impl ShipController {
    pub fn set_cockpit_control(&mut self, active: bool) {
        self.cockpit_control_active = active;
        if !active {
            self.clear_throttle_input();
            self.steering_target = SteeringInput::default();
        }
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
