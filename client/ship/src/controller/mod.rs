//! Authoritative private ship state and update dispatch.

use crate::{
    CockpitMessage, DoorState, EnergyCoreState, FlightState, LOWEST_LOCAL_Y_METERS, ShipPose,
    ShipSnapshot, SteeringInput,
};
use salimon_world::{CELESTIAL_BODIES, CelestialBodyId};
use std::time::Duration;

pub(crate) mod assist;
pub(crate) mod cockpit;
mod door;
mod flight;
mod steering;
pub(crate) mod telemetry;
use assist::AssistTransition;

#[derive(Clone, Debug)]
pub struct ShipController {
    pose: ShipPose,
    flight_state: FlightState,
    door_state: DoorState,
    door_open_fraction: f64,
    cockpit_control_active: bool,
    thruster_percentage: u8,
    energy_core: EnergyCoreState,
    cockpit_message: Option<CockpitMessage>,
    steering_target: SteeringInput,
    steering: SteeringInput,
    assist: Option<AssistTransition>,
}

impl Default for ShipController {
    fn default() -> Self {
        let earth = CELESTIAL_BODIES
            .iter()
            .find(|body| body.id == CelestialBodyId::Earth)
            .expect("world catalog always contains Earth");
        let mut position_meters = earth.center.meters();
        position_meters[1] += earth.radius_meters - LOWEST_LOCAL_Y_METERS;
        Self {
            pose: ShipPose {
                position_meters,
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            flight_state: FlightState::Landed {
                body: CelestialBodyId::Earth,
            },
            door_state: DoorState::Closed,
            door_open_fraction: 0.0,
            cockpit_control_active: false,
            thruster_percentage: 0,
            energy_core: EnergyCoreState::default(),
            cockpit_message: None,
            steering_target: SteeringInput::default(),
            steering: SteeringInput::default(),
            assist: None,
        }
    }
}

impl ShipController {
    #[must_use]
    pub fn flying(pose: ShipPose, thruster_percentage: u8) -> Self {
        Self {
            pose,
            flight_state: FlightState::Flying,
            door_state: DoorState::Closed,
            door_open_fraction: 0.0,
            cockpit_control_active: true,
            thruster_percentage: thruster_percentage.min(100),
            energy_core: EnergyCoreState::default(),
            cockpit_message: None,
            steering_target: SteeringInput::default(),
            steering: SteeringInput::default(),
            assist: None,
        }
    }

    pub fn advance(&mut self, delta: Duration) {
        self.advance_door(delta);
        let seconds = delta.as_secs_f64().min(0.1);
        self.advance_steering(seconds);
        match self.flight_state {
            FlightState::Flying => self.advance_flight(seconds),
            FlightState::AssistedLanding { body } => self.advance_landing(body, delta),
            FlightState::AssistedTakeoff { body } => self.advance_takeoff(body, delta),
            FlightState::Landed { .. } => {}
        }
    }

    #[must_use]
    pub fn snapshot(&self) -> ShipSnapshot {
        let velocity_meters_per_second = self.velocity_meters_per_second();
        ShipSnapshot {
            pose: self.pose,
            flight_state: self.flight_state,
            door_state: self.door_state,
            door_open_fraction: self.door_open_fraction,
            cockpit_control_active: self.cockpit_control_active,
            thruster_percentage: self.thruster_percentage,
            speed_meters_per_second: self.speed_meters_per_second(),
            velocity_meters_per_second,
            energy_core: self.energy_core,
            nearby_body: self.nearby_body_telemetry(velocity_meters_per_second),
            cockpit_message: self.cockpit_message,
            steering: self.steering,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ship_is_landed_on_earth_with_closed_door() {
        let snapshot = ShipController::default().snapshot();
        assert_eq!(
            snapshot.flight_state,
            FlightState::Landed {
                body: CelestialBodyId::Earth
            }
        );
        assert_eq!(snapshot.door_state, DoorState::Closed);
        assert_eq!(snapshot.thruster_percentage, 0);
        let earth = CELESTIAL_BODIES
            .iter()
            .find(|body| body.id == CelestialBodyId::Earth)
            .unwrap();
        assert_eq!(snapshot.pose.orientation, [0.0, 0.0, 0.0, 1.0]);
        assert!(
            (snapshot.pose.position_meters[1] + LOWEST_LOCAL_Y_METERS
                - (earth.center.meters()[1] + earth.radius_meters))
                .abs()
                < 1.0e-9
        );
    }
}
