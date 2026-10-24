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

/// Core-local observation. Depletion assumes unchanged ordinary bus demand and
/// includes only the currently accepted fragment, not future refueling.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoreStatus {
    pub consumption_watts: f64,
    /// Actual instantaneous recipe draw; zero without power or a runnable job.
    pub fabricator_watts: f64,
    pub conversion_watts: f64,
    pub conversion_remaining_joules: f64,
    pub depletion_seconds: Option<f64>,
    pub low_energy: bool,
}

/// Observational navigation in ship-local axes (+X forward, +Y up, -Z starboard).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavigationTelemetry {
    pub velocity_direction_local: Option<[f64; 3]>,
    pub heading_velocity_angle_degrees: Option<f64>,
    /// Instantaneous powered engine acceleration; unavailable during captured assists/crashes.
    pub acceleration_meters_per_second_squared: Option<f64>,
    pub braking: BrakingEstimate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BrakingEstimate {
    Stationary,
    /// Full reverse at the current heading, with no steering or future conversion credited.
    Aligned {
        seconds: f64,
        distance_meters: f64,
        energy_joules: f64,
        sufficient_energy: bool,
    },
    AlignHeading,
    Unavailable,
}

impl ShipController {
    pub(super) fn navigation_telemetry(&self) -> NavigationTelemetry {
        use salimon_math::dot;
        let velocity = self.velocity_meters_per_second();
        let speed = self.speed_meters_per_second();
        let axes = self.pose.axes();
        let direction =
            (speed > 0.0 && speed.is_finite()).then(|| velocity.map(|component| component / speed));
        let local = direction.map(|v| axes.map(|axis| dot(v, axis)));
        let free = self.flight_state == crate::FlightState::Flying;
        let powered = self.energy_core.powered();
        let acceleration = free.then(|| {
            if powered {
                self.propulsion.acceleration_meters_per_second_squared(
                    self.energy_core
                        .config()
                        .propulsion_watts(self.effective_throttle(), true)
                        * f64::from(self.effective_throttle().signum()),
                )
            } else {
                0.0
            }
        });
        let reverse_acceleration = self
            .propulsion
            .acceleration_meters_per_second_squared(self.energy_core.config().reverse_watts);
        let braking = if speed == 0.0 {
            BrakingEstimate::Stationary
        } else if !free
            || !powered
            || !self.engines_enabled
            || !speed.is_finite()
            || reverse_acceleration <= 0.0
        {
            BrakingEstimate::Unavailable
        } else if local.is_none_or(|v| v[0] <= 0.0 || v[1].hypot(v[2]) > 1e-9) {
            BrakingEstimate::AlignHeading
        } else {
            let seconds = speed / reverse_acceleration;
            let distance_meters = 0.5 * speed * seconds;
            let energy_joules = seconds
                * (self.energy_core.config().demand_watts(-100, true) + self.fabricator_watts);
            if [seconds, distance_meters, energy_joules]
                .iter()
                .all(|v| v.is_finite())
            {
                BrakingEstimate::Aligned {
                    seconds,
                    distance_meters,
                    energy_joules,
                    sufficient_energy: energy_joules <= self.energy_core.stored_joules(),
                }
            } else {
                BrakingEstimate::Unavailable
            }
        };
        NavigationTelemetry {
            velocity_direction_local: local,
            heading_velocity_angle_degrees: local
                .map(|v| v[0].clamp(-1.0, 1.0).acos().to_degrees()),
            acceleration_meters_per_second_squared: acceleration,
            braking,
        }
    }

    pub(super) fn core_status(&self) -> CoreStatus {
        let demand = self.energy_core.config().demand_watts(
            self.effective_throttle(),
            self.flight_state == crate::FlightState::Flying,
        ) + self.fabricator_watts
            + self.auxiliary_watts;
        let stored = self.energy_core.stored_joules();
        let conversion = self.conversion.remaining_joules();
        let rate = self.conversion.active_watts();
        let depletion_seconds = if demand == 0.0 {
            None
        } else if rate < demand
            && stored / (demand - rate) <= self.conversion.remaining_duration().as_secs_f64()
        {
            Some(stored / (demand - rate))
        } else {
            let duration = self.conversion.remaining_duration().as_secs_f64();
            let after = (stored + conversion - demand * duration)
                .clamp(0.0, self.energy_core.config().capacity_joules as f64);
            Some(duration + after / demand)
        };
        CoreStatus {
            fabricator_watts: if self.energy_core.powered() {
                self.fabricator_watts
            } else {
                0.0
            },
            consumption_watts: if self.energy_core.powered() {
                demand
            } else {
                0.0
            } + self.active_assist_watts(),
            conversion_watts: rate,
            conversion_remaining_joules: conversion,
            depletion_seconds,
            low_energy: stored / self.energy_core.config().capacity_joules as f64 <= 0.10,
        }
    }

    /// Capacity-only transaction: no charge, reservation or input mutation.
    pub fn increase_core_capacity(
        &mut self,
        expected: u64,
        capacity: u64,
    ) -> Result<(), crate::InvalidEnergy> {
        self.energy_core.increase_capacity(expected, capacity)
    }

    pub fn configure_energy_core(
        &mut self,
        config: crate::EnergyConfig,
        stored_joules: f64,
    ) -> Result<(), crate::InvalidEnergy> {
        self.energy_core = crate::EnergyCore::new(config, stored_joules)?;
        if !self.energy_core.powered() {
            self.clear_flight_control_input();
        }
        Ok(())
    }

    /// Scenario/import boundary, bounded by configured capacity.
    pub fn set_stored_core_energy_joules(&mut self, stored_joules: u64) {
        self.energy_core
            .set_stored_joules(stored_joules as f64)
            .expect("u64 joules are finite and nonnegative");
        if !self.energy_core.powered() {
            self.clear_flight_control_input();
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
            self.clear_flight_control_input();
        }
        Ok(delivered)
    }

    /// Low-priority post-bus work, settled after ordinary loads/conversion.
    /// Only stored unreserved energy above the emergency floor may be spent.
    pub fn energy_config(&self) -> crate::EnergyConfig {
        self.energy_core.config()
    }
    pub fn fund_auxiliary_work(
        &mut self,
        requested: f64,
        reserve: f64,
        elapsed: std::time::Duration,
    ) -> Result<f64, crate::InvalidEnergy> {
        if !requested.is_finite() || requested < 0.0 || !reserve.is_finite() || reserve < 0.0 {
            return Err(crate::InvalidEnergy);
        }
        if elapsed.is_zero() {
            return if requested == 0.0 {
                Ok(0.0)
            } else {
                Err(crate::InvalidEnergy)
            };
        }
        let available = (self.energy_core.stored_joules() - reserve).max(0.0);
        let funded = self.exchange_core_energy_joules(0.0, requested.min(available))?;
        self.auxiliary_watts = funded / elapsed.as_secs_f64();
        Ok(funded)
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

#[cfg(test)]
mod core_status_tests {
    use super::*;
    use crate::EnergyConfig;
    #[test]
    fn core_status_uses_fractional_authority_and_idle_demand() {
        let mut ship = ShipController::default();
        ship.configure_energy_core(
            EnergyConfig {
                capacity_joules: 1000,
                forward_watts: 100.0,
                reverse_watts: 50.0,
                gravity_idle_watts: 10.0,
                cockpit_idle_watts: 0.0,
            },
            0.5,
        )
        .unwrap();
        let snapshot = ship.snapshot();
        assert_eq!(snapshot.energy_core.stored_joules, 0);
        assert!(snapshot.power_available);
        assert_eq!(snapshot.core_status.consumption_watts, 10.0);
        assert_eq!(snapshot.core_status.depletion_seconds, Some(0.05));
        assert!(snapshot.core_status.low_energy);
        ship.set_stored_core_energy_joules(0);
        assert!(!ship.snapshot().power_available);
        assert_eq!(ship.snapshot().core_status.consumption_watts, 0.0);
        ship.exchange_core_energy_joules(1000.0, 0.0).unwrap();
        assert!(!ship.snapshot().core_status.low_energy);
        assert_eq!(ship.snapshot().core_status.depletion_seconds, Some(100.0));
    }
}

#[cfg(test)]
mod navigation_tests {
    use super::*;
    use crate::{FlightState, Propulsion, ShipPose};
    fn moving(v: [f64; 3]) -> ShipController {
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
        );
        ship.velocity = v;
        ship
    }
    #[test]
    fn vector_angle_and_reverse_budget_use_full_velocity_and_mass() {
        let mut ship = moving([100.0, 0.0, 0.0]);
        ship.propulsion = Propulsion::default().with_total_mass(200_000.0).unwrap();
        ship.thruster_percentage = -50;
        let n = ship.snapshot().navigation;
        assert_eq!(n.velocity_direction_local, Some([1.0, 0.0, 0.0]));
        assert_eq!(n.heading_velocity_angle_degrees, Some(0.0));
        assert_eq!(n.acceleration_meters_per_second_squared, Some(-125.0));
        assert_eq!(
            n.braking,
            BrakingEstimate::Aligned {
                seconds: 0.4,
                distance_meters: 20.0,
                energy_joules: 20_044_000.0,
                sufficient_energy: true
            }
        );
        ship.set_stored_core_energy_joules(1);
        assert!(matches!(
            ship.snapshot().navigation.braking,
            BrakingEstimate::Aligned {
                sufficient_energy: false,
                ..
            }
        ));
        ship.set_stored_core_energy_joules(0);
        assert_eq!(
            ship.snapshot().navigation.braking,
            BrakingEstimate::Unavailable
        );
        assert_eq!(
            ship.snapshot()
                .navigation
                .acceleration_meters_per_second_squared,
            Some(0.0)
        );
    }
    #[test]
    fn tangential_and_backward_motion_never_claim_a_reverse_stop() {
        for velocity in [[100.0, 1.0, 0.0], [0.0, 100.0, 0.0], [-100.0, 0.0, 0.0]] {
            assert_eq!(
                moving(velocity).snapshot().navigation.braking,
                BrakingEstimate::AlignHeading
            );
        }
        let mut ship = moving([100.0, 0.0, 0.0]);
        ship.pose.orientation = [
            0.0,
            0.0,
            std::f64::consts::FRAC_1_SQRT_2,
            std::f64::consts::FRAC_1_SQRT_2,
        ];
        let n = ship.snapshot().navigation;
        assert!((n.heading_velocity_angle_degrees.unwrap() - 90.0).abs() < 1e-10);
        assert_eq!(n.braking, BrakingEstimate::AlignHeading);
        assert_eq!(ship.snapshot().speed_meters_per_second, 100.0);
    }
    #[test]
    fn zero_unsupported_and_overflow_are_explicit() {
        let n = moving([0.0; 3]).snapshot().navigation;
        assert_eq!(n.velocity_direction_local, None);
        assert_eq!(n.heading_velocity_angle_degrees, None);
        assert_eq!(n.braking, BrakingEstimate::Stationary);
        let mut ship = moving([100.0, 0.0, 0.0]);
        ship.propulsion = Propulsion::new(100_000.0, 0.0).unwrap();
        assert_eq!(
            ship.snapshot().navigation.braking,
            BrakingEstimate::Unavailable
        );
        ship.flight_state = FlightState::Bouncing {
            body: salimon_world::CelestialBodyId::Earth,
        };
        assert_eq!(
            ship.snapshot()
                .navigation
                .acceleration_meters_per_second_squared,
            None
        );
        assert_eq!(
            moving([f64::MAX, 0.0, 0.0]).snapshot().navigation.braking,
            BrakingEstimate::Unavailable
        );
    }
}

#[cfg(test)]
mod tiny_velocity_test {
    use crate::{ShipController, ShipPose};
    #[test]
    fn subnormal_velocity_has_a_finite_direction() {
        let ship = ShipController::flying_with_velocity(
            ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
            [1e-320, 0.0, 0.0],
        )
        .unwrap();
        let n = ship.snapshot().navigation;
        assert_eq!(n.velocity_direction_local, Some([1.0, 0.0, 0.0]));
        assert_eq!(n.heading_velocity_angle_degrees, Some(0.0));
    }
}

#[cfg(test)]
mod auxiliary_tests {
    use std::time::Duration;
    #[test]
    fn auxiliary_draw_protects_reserve_is_reported_and_invalid_requests_are_atomic() {
        let mut ship = crate::ShipController::default();
        ship.set_stored_core_energy_joules(1000);
        assert_eq!(
            ship.fund_auxiliary_work(900.0, 400.0, Duration::from_secs(2))
                .unwrap(),
            600.0
        );
        assert_eq!(ship.snapshot().energy_core.stored_joules, 400);
        assert_eq!(ship.auxiliary_watts, 300.0);
        let restored = crate::ShipController::from_save(ship.save_state()).unwrap();
        assert_eq!(restored.auxiliary_watts, 0.0);
        assert_eq!(restored.snapshot().energy_core.stored_joules, 400);
        for (draw, reserve) in [(f64::NAN, 0.0), (-1.0, 0.0), (1.0, f64::INFINITY)] {
            assert!(
                ship.fund_auxiliary_work(draw, reserve, Duration::from_secs(1))
                    .is_err()
            );
            assert_eq!(ship.snapshot().energy_core.stored_joules, 400);
        }
        assert!(ship.fund_auxiliary_work(1.0, 0.0, Duration::ZERO).is_err());
        assert_eq!(
            ship.fund_auxiliary_work(100.0, 500.0, Duration::from_secs(1))
                .unwrap(),
            0.0
        );
    }
}
