//! Opaque schema-1 logical state; no input latch or presentation feedback.
use super::ShipController;
use salimon_persistence::{SaveError, Validate, invalid};
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ShipSaveState(ShipController);
impl Validate for ShipSaveState {
    fn validate(&self) -> Result<(), SaveError> {
        self.0.validate_save()
    }
}
impl ShipController {
    pub fn save_state(&self) -> ShipSaveState {
        ShipSaveState(self.clone())
    }
    pub fn from_save(state: ShipSaveState) -> Result<Self, SaveError> {
        state.validate()?;
        let mut ship = state.0;
        ship.throttle_input = Default::default();
        ship.steering_target = Default::default();
        ship.cockpit_message = None;
        ship.fabricator_powered_seconds = 0.;
        Ok(ship)
    }
    fn validate_save(&self) -> Result<(), SaveError> {
        let pose = self.pose;
        let norm: f64 = pose.orientation.iter().map(|v| v * v).sum();
        if !pose
            .position_meters
            .iter()
            .chain(self.velocity.iter())
            .all(|v| v.is_finite())
            || !norm.is_finite()
            || (norm - 1.).abs() > 1e-9
            || !(-100..=100).contains(&self.thruster_percentage)
            || !(0.0..=1.0).contains(&self.door_open_fraction)
            || ![self.steering.pitch, self.steering.yaw, self.steering.roll]
                .iter()
                .all(|v| v.is_finite() && v.abs() <= 1.)
        {
            return Err(invalid("ship pose, motion, door or control range"));
        }
        self.propulsion.validate_save()?;
        self.energy_core.validate_save()?;
        self.conversion.validate_save()?;
        if !self.fabricator_watts.is_finite() || self.fabricator_watts < 0. {
            return Err(invalid("Fabricator demand"));
        }
        if !self.assist_config.landing_range_radius_fraction.is_finite()
            || !(0.0..=1.0).contains(&self.assist_config.landing_range_radius_fraction)
            || self.assist_config.landing_range_radius_fraction == 0.
            || !self
                .assist_config
                .maximum_landing_speed_meters_per_second
                .is_finite()
            || self.assist_config.maximum_landing_speed_meters_per_second < 0.
        {
            return Err(invalid("assist config"));
        }
        let assisted = matches!(
            self.flight_state,
            crate::FlightState::AssistedLanding { .. } | crate::FlightState::AssistedTakeoff { .. }
        );
        if assisted != self.assist.is_some() {
            return Err(invalid("assist/flight state mismatch"));
        }
        if let Some(a) = &self.assist {
            a.validate_save()?;
        }
        if let Some(b) = &self.bounce {
            b.validate_save()?;
        }
        if let Some(f) = &self.landing_footprint
            && [&f.feet, &f.ramp, &f.hull]
                .iter()
                .any(|points| points.is_empty() || points.iter().flatten().any(|v| !v.is_finite()))
        {
            return Err(invalid("landing footprint"));
        }
        Ok(())
    }
}
