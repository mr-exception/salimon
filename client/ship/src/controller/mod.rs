//! Authoritative private ship state and update dispatch.

use crate::{
    CockpitMessage, DoorState, EnergyCoreState, FlightState, LANDED_ORIGIN_HEIGHT_METERS, ShipPose,
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
    pub(crate) pose: ShipPose,
    pub(crate) velocity: [f64; 3],
    pub(crate) propulsion: crate::Propulsion,
    pub(crate) terrain: salimon_world::terrain::TerrainMode,
    pub(crate) landing_footprint: Option<crate::LandingFootprint>,
    pub(crate) flight_state: FlightState,
    door_state: DoorState,
    door_open_fraction: f64,
    cockpit_control_active: bool,
    pub(crate) thruster_percentage: u8,
    energy_core: EnergyCoreState,
    cockpit_message: Option<CockpitMessage>,
    steering_target: SteeringInput,
    steering: SteeringInput,
    pub(crate) assist: Option<AssistTransition>,
}

impl Default for ShipController {
    fn default() -> Self {
        let earth = CELESTIAL_BODIES
            .iter()
            .find(|body| body.id == CelestialBodyId::Earth)
            .expect("world catalog always contains Earth");
        let mut position_meters = earth.center.meters();
        position_meters[1] += earth.radius_meters + LANDED_ORIGIN_HEIGHT_METERS;
        Self {
            velocity: [0.0; 3],
            propulsion: crate::Propulsion::default(),
            terrain: salimon_world::terrain::TerrainMode::Flat,
            landing_footprint: None,
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
            velocity: [0.0; 3],
            propulsion: crate::Propulsion::default(),
            terrain: salimon_world::terrain::TerrainMode::Flat,
            landing_footprint: None,
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

    /// A scenario/import boundary; selected throttle never implies initial speed.
    pub fn flying_with_velocity(
        pose: ShipPose,
        throttle: u8,
        velocity: [f64; 3],
    ) -> Result<Self, crate::InvalidFlightMotion> {
        if !velocity.iter().all(|v| v.is_finite()) {
            return Err(crate::InvalidFlightMotion);
        }
        let mut ship = Self::flying(pose, throttle);
        ship.velocity = velocity;
        Ok(ship)
    }

    /// Event-driven cached mass/thrust update. Does not scan cargo or reset momentum.
    pub fn set_propulsion(&mut self, propulsion: crate::Propulsion) {
        self.propulsion = propulsion;
    }

    pub fn advance(&mut self, delta: Duration) {
        self.advance_door(delta);
        let seconds = delta.as_secs_f64();
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
            total_mass_kg: self.propulsion.total_mass_kg(),
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
            (snapshot.pose.position_meters[1]
                - LANDED_ORIGIN_HEIGHT_METERS
                - (earth.center.meters()[1] + earth.radius_meters))
                .abs()
                < 1.0e-9
        );
    }
}
