//! Cockpit authority and contextual messages.

use super::ShipController;
use crate::{FlightState, SteeringInput};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CockpitMessage {
    DoorLockedWhileInFlight,
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
        match self.flight_state {
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
