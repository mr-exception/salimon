//! Authoritative private ship state and update dispatch.

use crate::{
    CockpitMessage, DoorState, FlightState, LANDED_ORIGIN_HEIGHT_METERS, ShipPose, ShipSnapshot,
    SteeringInput,
};
use salimon_world::{CELESTIAL_BODIES, CelestialBodyId};
use std::time::Duration;

pub(crate) mod assist;
pub(crate) mod cockpit;
mod collision;
mod door;
mod flight;
mod offline;
pub mod persistence;
mod steering;
pub(crate) mod telemetry;
pub mod throttle;
use assist::AssistTransition;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
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
    pub(crate) thruster_percentage: i8,
    engines_enabled: bool,
    #[serde(skip)]
    throttle_input: throttle::HeldThrottle,
    energy_core: crate::EnergyCore,
    fabricator_watts: f64,
    #[serde(skip)]
    auxiliary_watts: f64,
    #[serde(skip)]
    fabricator_powered_seconds: f64,
    pub(crate) conversion: crate::conversion::Conversion,
    #[serde(skip)]
    cockpit_message: Option<CockpitMessage>,
    #[serde(skip)]
    steering_target: SteeringInput,
    steering: SteeringInput,
    pub(crate) assist: Option<AssistTransition>,
    assist_config: crate::AssistConfig,
    bounce: Option<collision::Bounce>,
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
            throttle_input: throttle::HeldThrottle::default(),
            thruster_percentage: 0,
            engines_enabled: true,
            energy_core: crate::EnergyCore::default(),
            fabricator_watts: 0.0,
            auxiliary_watts: 0.0,
            fabricator_powered_seconds: 0.0,
            conversion: crate::conversion::Conversion::default(),
            cockpit_message: None,
            steering_target: SteeringInput::default(),
            steering: SteeringInput::default(),
            assist: None,
            assist_config: crate::AssistConfig::default(),
            bounce: None,
        }
    }
}

impl ShipController {
    #[must_use]
    pub fn flying(pose: ShipPose, thruster_percentage: i8) -> Self {
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
            throttle_input: throttle::HeldThrottle::default(),
            thruster_percentage: thruster_percentage.clamp(-100, 100),
            engines_enabled: true,
            energy_core: crate::EnergyCore::default(),
            fabricator_watts: 0.0,
            auxiliary_watts: 0.0,
            fabricator_powered_seconds: 0.0,
            conversion: crate::conversion::Conversion::default(),
            cockpit_message: None,
            steering_target: SteeringInput::default(),
            steering: SteeringInput::default(),
            assist: None,
            assist_config: crate::AssistConfig::default(),
            bounce: None,
        }
    }

    /// A scenario/import boundary; selected throttle never implies initial speed.
    pub fn flying_with_velocity(
        pose: ShipPose,
        throttle: i8,
        velocity: [f64; 3],
    ) -> Result<Self, crate::InvalidFlightMotion> {
        if !velocity.iter().all(|v| v.is_finite()) {
            return Err(crate::InvalidFlightMotion);
        }
        let mut ship = Self::flying(pose, throttle);
        ship.velocity = velocity;
        Ok(ship)
    }

    /// Event-driven cached mass update preserving thrust capacities and velocity.
    pub fn set_total_mass_kg(&mut self, mass_kg: f64) -> Result<(), crate::InvalidFlightMotion> {
        self.propulsion = self.propulsion.with_total_mass(mass_kg)?;
        Ok(())
    }

    /// Validated scenario/tuning boundary. Does not scan cargo or reset velocity.
    pub fn set_propulsion(&mut self, propulsion: crate::Propulsion) {
        self.propulsion = propulsion;
    }

    /// Register requested recipe watts independently of the existing ship consumers.
    pub fn set_fabricator_watts(&mut self, watts: f64) -> Result<(), crate::InvalidEnergy> {
        let total = self.energy_core.config().demand_watts(100, true) + watts;
        let reverse = self.energy_core.config().demand_watts(-100, true) + watts;
        if !watts.is_finite() || watts < 0.0 || !total.is_finite() || !reverse.is_finite() {
            return Err(crate::InvalidEnergy);
        }
        self.fabricator_watts = watts;
        Ok(())
    }

    /// Funded interval from the last advance; no duration is banked between ticks.
    pub fn fabricator_powered_duration(&self, delta: Duration) -> Duration {
        // Floor at the Duration boundary: never grant even a nanosecond of free work.
        let nanos = (self.fabricator_powered_seconds * 1_000_000_000.0).floor();
        Duration::from_nanos(nanos.clamp(0.0, u64::MAX as f64) as u64).min(delta)
    }

    pub fn advance(&mut self, delta: Duration) {
        self.fabricator_powered_seconds = 0.0;
        self.auxiliary_watts = 0.0;
        let converting = self.conversion.remaining_duration().min(delta);
        let incoming = self.conversion.advance(converting);
        self.advance_with_core_input(converting, incoming)
            .expect("validated conversion supplies finite joules");
        if converting < delta {
            self.advance_with_core_input(delta - converting, 0.0)
                .expect("zero Core input is valid");
        }
    }

    /// Incoming converted joules at the step boundary, settled against this step
    /// consumption before storage clipping. Invalid input leaves all ship state intact.
    pub fn advance_with_core_input(
        &mut self,
        delta: Duration,
        incoming_joules: f64,
    ) -> Result<(), crate::InvalidEnergy> {
        if !incoming_joules.is_finite() || incoming_joules < 0.0 {
            return Err(crate::InvalidEnergy);
        }
        if let Some(boundary) = self.throttle_boundary()
            && !boundary.is_zero()
            && delta > boundary
        {
            let first_input = incoming_joules * (boundary.as_secs_f64() / delta.as_secs_f64());
            self.advance_with_core_input(boundary, first_input)?;
            return self.advance_with_core_input(delta - boundary, incoming_joules - first_input);
        }
        if let Some(remaining) = self.assist_remaining()
            && delta > remaining
            && !remaining.is_zero()
        {
            let fraction = remaining.as_secs_f64() / delta.as_secs_f64();
            let first_input = incoming_joules * fraction;
            self.advance_with_core_input(remaining, first_input)?;
            return self.advance_with_core_input(delta - remaining, incoming_joules - first_input);
        }
        self.advance_door(delta);
        let effective_throttle = self.effective_throttle();
        let propulsion_watts = self
            .energy_core
            .config()
            .propulsion_watts(effective_throttle, self.flight_state == FlightState::Flying);
        let powered_seconds = self.energy_core.advance_with_load(
            delta,
            effective_throttle,
            self.flight_state == FlightState::Flying,
            incoming_joules,
            self.fabricator_watts,
        )?;
        if self.fabricator_watts > 0.0 {
            self.fabricator_powered_seconds += powered_seconds;
        }
        // Integrate steering only over the powered part of the interval, then
        // clear input residue at blackout so restoration preserves the heading.
        self.advance_steering(powered_seconds);
        if !self.energy_core.powered() {
            self.clear_flight_control_input();
        }
        self.settle_assist_energy(delta);
        match self.flight_state {
            FlightState::Flying => self.advance_powered_flight(
                delta,
                powered_seconds,
                propulsion_watts * powered_seconds * f64::from(effective_throttle.signum()),
            ),
            FlightState::AssistedLanding { body } => self.advance_landing(body, delta),
            FlightState::AssistedTakeoff { body } => self.advance_takeoff(body, delta),
            FlightState::Bouncing { body } => self.advance_bounce(body, delta),
            FlightState::Landed { .. } | FlightState::GroundedAfterCrash { .. } => {}
        }
        self.advance_throttle(delta);
        Ok(())
    }

    pub(super) fn effective_throttle(&self) -> i8 {
        if self.engines_enabled {
            self.thruster_percentage
        } else {
            0
        }
    }
    /// Explicit engine switch; zero selected throttle is still engine-on.
    pub fn toggle_engines(&mut self) -> bool {
        if !self.cockpit_control_active
            || !self.energy_core.powered()
            || self.assist.is_some()
            || matches!(self.flight_state, FlightState::Bouncing { .. })
        {
            return false;
        }
        self.engines_enabled = !self.engines_enabled;
        self.cockpit_message = None;
        self.clear_throttle_input();
        true
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
            engines_enabled: self.engines_enabled,
            total_mass_kg: self.propulsion.total_mass_kg(),
            speed_meters_per_second: self.speed_meters_per_second(),
            velocity_meters_per_second,
            navigation: self.navigation_telemetry(),
            energy_core: self.energy_core.snapshot(),
            core_status: self.core_status(),
            power_available: self.energy_core.powered(),
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

#[cfg(test)]
mod impulse_tests;
