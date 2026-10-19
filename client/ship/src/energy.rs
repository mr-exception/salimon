//! Authoritative joule storage, validated watt consumers and atomic energy exchange.
use crate::EnergyCoreState;
use std::{fmt, time::Duration};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidEnergy;
impl fmt::Display for InvalidEnergy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Core capacity must be positive; energy/watts must be finite and nonnegative with finite total demand")
    }
}
impl std::error::Error for InvalidEnergy {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnergyConfig {
    pub capacity_joules: u64,
    pub forward_watts: f64,
    pub reverse_watts: f64,
    pub gravity_idle_watts: f64,
    pub cockpit_idle_watts: f64,
}
impl Default for EnergyConfig {
    fn default() -> Self {
        Self {
            capacity_joules: crate::DEFAULT_CORE_ENERGY_CAPACITY_JOULES,
            forward_watts: 100_000_000.0,
            reverse_watts: 50_000_000.0,
            gravity_idle_watts: 100_000.0,
            cockpit_idle_watts: 10_000.0,
        }
    }
}
impl EnergyConfig {
    fn validate(self) -> Result<Self, InvalidEnergy> {
        let values = [
            self.forward_watts,
            self.reverse_watts,
            self.gravity_idle_watts,
            self.cockpit_idle_watts,
        ];
        if self.capacity_joules == 0
            || values.iter().any(|v| !v.is_finite() || *v < 0.0)
            || !(self.forward_watts.max(self.reverse_watts)
                + self.gravity_idle_watts
                + self.cockpit_idle_watts)
                .is_finite()
        {
            return Err(InvalidEnergy);
        }
        Ok(self)
    }
    pub fn demand_watts(self, throttle: i8, free_flight: bool) -> f64 {
        let fraction = if free_flight {
            f64::from(throttle.clamp(-100, 100)).abs() / 100.0
        } else {
            0.0
        };
        let propulsion = if throttle < 0 {
            self.reverse_watts
        } else {
            self.forward_watts
        };
        fraction * propulsion + self.gravity_idle_watts + self.cockpit_idle_watts
    }
}

/// An isolated assist budget. Only the Core can reserve/refund it; consumers
/// cannot borrow from it. It is crate-private to prevent duplicated spending.
#[derive(Clone, Debug)]
pub(crate) struct AssistEnergyReservation {
    remaining_joules: f64,
}
impl AssistEnergyReservation {
    pub(crate) fn spend(&mut self, joules: f64) {
        self.remaining_joules = (self.remaining_joules - joules).max(0.0);
    }
}

#[derive(Clone, Debug)]
pub struct EnergyCore {
    config: EnergyConfig,
    stored_joules: f64,
}
impl Default for EnergyCore {
    fn default() -> Self {
        Self::new(
            EnergyConfig::default(),
            crate::DEFAULT_CORE_ENERGY_STORED_JOULES as f64,
        )
        .expect("valid starter Core")
    }
}
impl EnergyCore {
    pub fn new(config: EnergyConfig, stored_joules: f64) -> Result<Self, InvalidEnergy> {
        let config = config.validate()?;
        if !stored_joules.is_finite() || stored_joules < 0.0 {
            return Err(InvalidEnergy);
        }
        Ok(Self {
            config,
            stored_joules: stored_joules.min(config.capacity_joules as f64),
        })
    }
    pub(crate) fn reserve_assist(&mut self, joules: f64) -> Option<AssistEnergyReservation> {
        if !joules.is_finite() || joules < 0.0 || joules > self.stored_joules {
            return None;
        }
        self.stored_joules -= joules;
        Some(AssistEnergyReservation {
            remaining_joules: joules,
        })
    }
    pub(crate) fn refund_assist(&mut self, reservation: AssistEnergyReservation) {
        // Capacity clipping applies even if conversion filled the bus during assist.
        self.stored_joules = (self.stored_joules + reservation.remaining_joules)
            .min(self.config.capacity_joules as f64);
    }
    pub fn config(&self) -> EnergyConfig {
        self.config
    }
    pub fn stored_joules(&self) -> f64 {
        self.stored_joules
    }
    pub fn powered(&self) -> bool {
        self.stored_joules > 0.0
    }
    pub fn snapshot(&self) -> EnergyCoreState {
        EnergyCoreState {
            capacity_joules: self.config.capacity_joules,
            stored_joules: (self.stored_joules as u64).min(self.config.capacity_joules),
        }
    }
    pub fn set_stored_joules(&mut self, joules: f64) -> Result<(), InvalidEnergy> {
        if !joules.is_finite() || joules < 0.0 {
            return Err(InvalidEnergy);
        }
        self.stored_joules = joules.min(self.config.capacity_joules as f64);
        Ok(())
    }
    /// Atomic conversion/consumer exchange: incoming energy is available to the load
    /// before surplus storage is capped. Returns joules actually delivered to the load.
    /// This is an energy transaction, not an assumed continuous charging rate.
    pub fn exchange_joules(&mut self, incoming: f64, requested: f64) -> Result<f64, InvalidEnergy> {
        if !incoming.is_finite() || incoming < 0.0 || !requested.is_finite() || requested < 0.0 {
            return Err(InvalidEnergy);
        }
        // Subtract the load before adding storage to avoid overflow at f64::MAX.
        let from_incoming = incoming.min(requested);
        let from_storage = self.stored_joules.min(requested - from_incoming);
        self.stored_joules = ((self.stored_joules - from_storage) + (incoming - from_incoming))
            .min(self.config.capacity_joules as f64);
        Ok(from_incoming + from_storage)
    }
    /// E03 supplies converted joules at this step boundary. Consumers and incoming
    /// energy settle together, so a full battery does not discard charge before draw.
    pub fn advance_with_input(
        &mut self,
        delta: Duration,
        throttle: i8,
        free_flight: bool,
        incoming_joules: f64,
    ) -> Result<f64, InvalidEnergy> {
        if !incoming_joules.is_finite() || incoming_joules < 0.0 {
            return Err(InvalidEnergy);
        }
        if incoming_joules == 0.0 {
            return Ok(self.advance(delta, throttle, free_flight));
        }
        let seconds = delta.as_secs_f64();
        let watts = self.config.demand_watts(throttle, free_flight);
        if watts == 0.0 {
            self.exchange_joules(incoming_joules, 0.0)?;
            return Ok(if self.powered() { seconds } else { 0.0 });
        }
        let requested = (watts * seconds).min(f64::MAX);
        let delivered = self.exchange_joules(incoming_joules, requested)?;
        Ok((delivered / watts).min(seconds))
    }

    /// Shared continuous load. Every consumer receives the same powered interval;
    /// isolated assist reservations are never accessible on this bus.
    pub fn advance_with_load(
        &mut self,
        delta: Duration,
        throttle: i8,
        free_flight: bool,
        incoming_joules: f64,
        additional_watts: f64,
    ) -> Result<f64, InvalidEnergy> {
        let watts = self.config.demand_watts(throttle, free_flight) + additional_watts;
        if !additional_watts.is_finite()
            || additional_watts < 0.0
            || !watts.is_finite()
            || !incoming_joules.is_finite()
            || incoming_joules < 0.0
        {
            return Err(InvalidEnergy);
        }
        // Keep the historical zero-load path and its rounding behavior unchanged.
        if additional_watts == 0.0 {
            return self.advance_with_input(delta, throttle, free_flight, incoming_joules);
        }
        let seconds = delta.as_secs_f64();
        let delivered = self.exchange_joules(incoming_joules, (watts * seconds).min(f64::MAX))?;
        Ok((delivered / watts).min(seconds))
    }

    /// All consumers share one bus; no consumer receives energy after exhaustion.
    /// Exact powered duration permits thrust integration followed by inertial coasting.
    pub fn advance(&mut self, delta: Duration, throttle: i8, free_flight: bool) -> f64 {
        let seconds = delta.as_secs_f64();
        if !self.powered() {
            return 0.0;
        }
        let watts = self.config.demand_watts(throttle, free_flight);
        if watts == 0.0 {
            return seconds;
        }
        let powered_seconds = seconds.min(self.stored_joules / watts);
        if powered_seconds < seconds || seconds >= self.stored_joules / watts {
            self.stored_joules = 0.0;
        } else {
            self.stored_joules = (self.stored_joules - watts * seconds).max(0.0);
        }
        powered_seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> EnergyConfig {
        EnergyConfig {
            capacity_joules: 1000,
            forward_watts: 100.0,
            reverse_watts: 50.0,
            gravity_idle_watts: 10.0,
            cockpit_idle_watts: 5.0,
        }
    }
    #[test]
    fn signed_throttle_idle_and_zero_crossing() {
        let mut core = EnergyCore::new(config(), 115.0).unwrap();
        assert_eq!(core.advance(Duration::from_secs(2), 100, true), 1.0);
        assert_eq!(core.stored_joules(), 0.0);
        assert_eq!(core.advance(Duration::from_secs(1), -100, true), 0.0);
        core.exchange_joules(100.0, 0.0).unwrap();
        core.advance(Duration::from_secs(1), -100, true);
        assert_eq!(core.stored_joules(), 35.0);
        core.advance(Duration::from_secs(1), 100, false);
        assert_eq!(core.stored_joules(), 20.0);
        core.advance(Duration::from_secs(1), 0, true);
        assert_eq!(core.stored_joules(), 5.0);
    }
    #[test]
    fn exchange_precedence_capacity_bootstrap_and_extreme_numbers() {
        let mut core = EnergyCore::new(config(), 990.0).unwrap();
        assert_eq!(core.exchange_joules(100.0, 90.0).unwrap(), 90.0);
        assert_eq!(core.stored_joules(), 1000.0);
        core.set_stored_joules(0.0).unwrap();
        assert_eq!(core.exchange_joules(50.0, 60.0).unwrap(), 50.0);
        assert!(!core.powered());
        core.exchange_joules(f64::MAX, f64::MAX).unwrap();
        assert_eq!(core.stored_joules(), 0.0);
        core.exchange_joules(f64::MAX, 0.0).unwrap();
        assert_eq!(core.stored_joules(), 1000.0);
        core.advance(Duration::MAX, 100, true);
        assert_eq!(core.stored_joules(), 0.0);
        for v in [f64::NAN, f64::INFINITY, -1.0] {
            assert!(core.exchange_joules(v, 0.0).is_err());
            assert!(core.set_stored_joules(v).is_err());
            assert!(
                EnergyCore::new(
                    EnergyConfig {
                        gravity_idle_watts: v,
                        ..config()
                    },
                    0.0
                )
                .is_err()
            );
        }
        assert!(
            EnergyCore::new(
                EnergyConfig {
                    capacity_joules: 0,
                    ..config()
                },
                0.0
            )
            .is_err()
        );
    }
    #[test]
    fn fractional_joules_and_partitions_are_preserved() {
        let mut whole = EnergyCore::new(config(), 300.25).unwrap();
        let mut split = whole.clone();
        whole.advance(Duration::from_secs(2), 37, true);
        for ms in [1, 399, 1000, 600] {
            split.advance(Duration::from_millis(ms), 37, true);
        }
        assert!((whole.stored_joules() - split.stored_joules()).abs() < 1e-10);
        let mut core = EnergyCore::new(
            EnergyConfig {
                forward_watts: 0.1,
                gravity_idle_watts: 0.0,
                cockpit_idle_watts: 0.0,
                ..config()
            },
            1.0,
        )
        .unwrap();
        for _ in 0..10 {
            core.advance(Duration::from_millis(100), 100, true);
        }
        assert!((core.stored_joules() - 0.9).abs() < 1e-12);
        assert!(core.powered());
    }
}

#[cfg(test)]
mod reservation_tests {
    use super::*;
    #[test]
    fn reservation_is_atomic_isolated_and_refunds_only_unused_energy() {
        let mut core = EnergyCore::new(EnergyConfig::default(), 100.0).unwrap();
        assert!(core.reserve_assist(101.0).is_none());
        assert_eq!(core.stored_joules(), 100.0);
        let mut budget = core.reserve_assist(80.0).unwrap();
        assert!(core.reserve_assist(30.0).is_none());
        assert_eq!(core.exchange_joules(0.0, 100.0).unwrap(), 20.0);
        budget.spend(30.0);
        core.refund_assist(budget);
        assert_eq!(core.stored_joules(), 50.0);
        for invalid in [f64::NAN, f64::INFINITY, -1.0] {
            assert!(core.reserve_assist(invalid).is_none());
            assert_eq!(core.stored_joules(), 50.0);
        }
    }
}

#[cfg(test)]
mod machine_load_tests {
    use super::*;
    #[test]
    fn shared_load_is_atomic_and_cannot_spend_reserved_assist_energy() {
        let mut core = EnergyCore::default();
        core.set_stored_joules(1000.0).unwrap();
        let reservation = core.reserve_assist(880.0).unwrap();
        let funded = core
            .advance_with_load(Duration::from_secs(1), 0, false, 0.0, 10_000.0)
            .unwrap();
        assert_eq!(funded, 0.001);
        assert_eq!(core.stored_joules(), 0.0);
        core.refund_assist(reservation);
        assert_eq!(core.stored_joules(), 880.0);
        for watts in [f64::NAN, f64::INFINITY, -1.0] {
            assert!(
                core.advance_with_load(Duration::from_secs(1), 100, true, 20.0, watts)
                    .is_err()
            );
            assert_eq!(core.stored_joules(), 880.0);
        }
        // Simultaneous forward thrust, gravity, cockpit and independent machine load.
        core.set_stored_joules(1000.0).unwrap();
        let seconds = core
            .advance_with_load(Duration::from_secs(1), 100, true, 0.0, 10_000.0)
            .unwrap();
        assert_eq!(seconds, 1000.0 / 100_120_000.0);
    }
}
