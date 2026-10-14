//! Core snapshot and nearby-body observation.

use super::ShipController;
use crate::{
    DEFAULT_CORE_ENERGY_CAPACITY_JOULES, DEFAULT_CORE_ENERGY_STORED_JOULES,
    NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS,
};
use salimon_world::WorldPosition;

/// Whole-joule display snapshot; authoritative storage retains fractional joules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnergyCoreState {
    pub capacity_joules: u64,
    pub stored_joules: u64,
}

impl Default for EnergyCoreState {
    fn default() -> Self {
        Self {
            capacity_joules: DEFAULT_CORE_ENERGY_CAPACITY_JOULES,
            stored_joules: DEFAULT_CORE_ENERGY_STORED_JOULES,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NearbyBodyTelemetry {
    pub name: &'static str,
    pub surface_distance_meters: f64,
    /// Negative approaches the body, positive recedes, and zero is stationary.
    pub radial_speed_meters_per_second: f64,
}

impl ShipController {
    pub fn configure_energy_core(
        &mut self,
        config: crate::EnergyConfig,
        stored_joules: f64,
    ) -> Result<(), crate::InvalidEnergy> {
        self.energy_core = crate::EnergyCore::new(config, stored_joules)?;
        if !self.energy_core.powered() {
            self.clear_throttle_input();
        }
        Ok(())
    }

    /// Scenario/import boundary, bounded by configured capacity.
    pub fn set_stored_core_energy_joules(&mut self, stored_joules: u64) {
        self.energy_core
            .set_stored_joules(stored_joules as f64)
            .expect("u64 joules are finite and nonnegative");
        if !self.energy_core.powered() {
            self.clear_throttle_input();
        }
    }

    /// Atomic E03 conversion/consumer handoff; positive charge automatically resumes
    /// the selected throttle on the next advancement, without resetting velocity.
    pub fn exchange_core_energy_joules(
        &mut self,
        incoming: f64,
        requested: f64,
    ) -> Result<f64, crate::InvalidEnergy> {
        let delivered = self.energy_core.exchange_joules(incoming, requested)?;
        if !self.energy_core.powered() {
            self.clear_throttle_input();
        }
        Ok(delivered)
    }

    pub(super) fn nearby_body_telemetry(
        &self,
        velocity_meters_per_second: [f64; 3],
    ) -> Option<NearbyBodyTelemetry> {
        let position = WorldPosition::new(
            self.pose.position_meters[0],
            self.pose.position_meters[1],
            self.pose.position_meters[2],
        );
        self.terrain
            .nearest_body(
                self.pose.position_meters,
                NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS,
                false,
            )
            .map(|(body, surface_distance_meters)| NearbyBodyTelemetry {
                name: body.name,
                surface_distance_meters,
                radial_speed_meters_per_second: body
                    .radial_speed_meters_per_second(position, velocity_meters_per_second),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::super::assist::body_definition;
    use super::*;
    use crate::{DEFAULT_CORE_ENERGY_CAPACITY_JOULES, EnergyCoreState, ShipPose};
    use salimon_math::add;
    use salimon_world::CelestialBodyId;

    #[test]
    fn core_energy_is_bounded_and_updates_live_snapshots() {
        let mut ship = ShipController::default();
        assert_eq!(ship.snapshot().energy_core, EnergyCoreState::default());

        ship.set_stored_core_energy_joules(25_000_000_000);
        assert_eq!(ship.snapshot().energy_core.stored_joules, 25_000_000_000);
        ship.set_stored_core_energy_joules(u64::MAX);
        assert_eq!(
            ship.snapshot().energy_core.stored_joules,
            DEFAULT_CORE_ENERGY_CAPACITY_JOULES
        );
    }

    #[test]
    fn nearby_body_telemetry_crosses_range_and_reports_radial_direction() {
        let earth = body_definition(CelestialBodyId::Earth);
        let position = add(
            earth.center.meters(),
            [
                earth.radius_meters + NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS,
                0.0,
                0.0,
            ],
        );
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: position,
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            10,
        );

        ship.velocity = [100.0, 0.0, 0.0];
        let receding = ship.snapshot().nearby_body.unwrap();
        assert_eq!(receding.name, "Earth");
        assert_eq!(
            receding.surface_distance_meters,
            NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS
        );
        assert!(receding.radial_speed_meters_per_second > 0.0);

        ship.pose.orientation = [0.0, 1.0, 0.0, 0.0];
        ship.velocity = [-100.0, 0.0, 0.0];
        assert!(
            ship.snapshot()
                .nearby_body
                .unwrap()
                .radial_speed_meters_per_second
                < 0.0
        );

        ship.thruster_percentage = 0;
        ship.velocity = [0.0; 3];
        assert_eq!(
            ship.snapshot()
                .nearby_body
                .unwrap()
                .radial_speed_meters_per_second,
            0.0
        );

        ship.pose.position_meters[0] += 1.0;
        assert!(ship.snapshot().nearby_body.is_none());
    }
}
