//! Construction's chronological powered-work adapter, shared with future U32.
//! The current host retains its saved ship/region frame. No scanner/orbit logic.
pub(super) mod save;
use super::GameplaySession;
use salimon_persistence::{SaveError, Validate, invalid};
use serde::{Deserialize, Serialize};
use std::time::Duration;
const TICK_NANOS: u64 = 16_000_000;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OfflineClock {
    version: u32,
    pub(crate) watermark: u64,
    pub(crate) pending_nanos: u64,
    blueprint_version: u32,
    power_source: PowerSource,
    safety_reasons: std::collections::BTreeMap<u64, String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum PowerSource {
    OriginalCore,
}
impl Default for OfflineClock {
    fn default() -> Self {
        Self::at(0)
    }
}
impl OfflineClock {
    pub(crate) fn validate_sites(&self, ids: impl Iterator<Item = u64>) -> Result<(), SaveError> {
        let expected: std::collections::BTreeSet<_> = ids.collect();
        if expected != self.safety_reasons.keys().copied().collect() {
            return Err(invalid("offline safety record/project identity mismatch"));
        }
        Ok(())
    }
    pub(crate) fn at(watermark: u64) -> Self {
        Self {
            version: 1,
            watermark,
            pending_nanos: 0,
            blueprint_version: 1,
            power_source: PowerSource::OriginalCore,
            safety_reasons: Default::default(),
        }
    }
    pub(crate) fn captured(
        &self,
        utc: u64,
        safety_reasons: std::collections::BTreeMap<u64, String>,
    ) -> Self {
        Self {
            watermark: self.watermark.max(utc),
            safety_reasons,
            ..self.clone()
        }
    }
}
impl Validate for OfflineClock {
    fn validate(&self) -> Result<(), SaveError> {
        if self.version != 1
            || self.blueprint_version != 1
            || self.pending_nanos >= TICK_NANOS
            || self
                .safety_reasons
                .iter()
                .any(|(id, reason)| *id == 0 || reason.is_empty() || reason.len() > 256)
        {
            return Err(invalid("offline clock version/remainder"));
        }
        Ok(())
    }
}
/// Supplied only by the authoritative chronological host, after region/trajectory
/// verification. No event is inferred from wall time or a landing prediction.
#[derive(Clone, Debug)]
pub(crate) struct TimedEvent {
    pub(crate) after: Duration,
    pub(crate) event: Event,
}
#[derive(Clone, Debug)]
#[allow(dead_code)] // U32 host integration seam; exercised by owning tests now.
pub(crate) enum Event {
    Impact(super::construction::damage::Impact),
    Motion {
        pose: salimon_ship::ShipPose,
        velocity: [f64; 3],
        flight: salimon_ship::FlightState,
    },
}
impl GameplaySession {
    /// Stage the whole interval. A bad late event, save, collider or identity
    /// cannot expose an earlier completion or partially debited Core.
    pub(crate) fn reconcile_construction_offline(
        &mut self,
        interval: Duration,
        events: &[TimedEvent],
    ) -> Result<(), SaveError> {
        if interval
            .checked_add(Duration::from_nanos(self.offline.pending_nanos))
            .is_none()
        {
            return Err(invalid("offline interval overflow"));
        }
        let mut previous = Duration::ZERO;
        for e in events {
            if e.after < previous || e.after > interval {
                return Err(invalid("offline event order/range"));
            }
            previous = e.after;
        }
        if self.survival.snapshot().terminal {
            return Ok(());
        }
        let mut next = GameplaySession::default();
        next.restore_bytes(&self.save_bytes(0)?)?;
        // A launch is closed-game time, distinct from a live pause. The caller
        // never invokes this on an in-game pause frame.
        previous = Duration::ZERO;
        for e in events {
            next.advance_closed_interval(e.after - previous);
            // A verified event is an exact boundary, including a partial tick.
            next.finish_offline_remainder();
            match &e.event {
                Event::Impact(impact) => next.construction_impact(*impact).map_err(invalid)?,
                Event::Motion {
                    pose,
                    velocity,
                    flight,
                } => {
                    next.ship
                        .apply_verified_offline_motion(*pose, *velocity, *flight)
                        .map_err(|e| invalid(e.to_string()))?;
                    next.stop_airborne_construction();
                    if matches!(flight, salimon_ship::FlightState::GroundedAfterCrash { .. }) {
                        next.interrupt_construction_after_crash();
                    }
                    next.sync_carried();
                }
            }
            previous = e.after;
        }
        next.advance_closed_interval(interval - previous);
        if let Some(error) = &next.motion_error {
            return Err(invalid(format!("offline motion: {error}")));
        }
        // Verify the final authoritative source through the ordinary reconstruction
        // path before it is eligible for the host's durable publication.
        next.building.running = None;
        next.offline.safety_reasons = next.construction_safety_record();
        let mut verified = GameplaySession::default();
        verified.restore_bytes(&next.save_bytes(0)?)?;
        next.policy = self.policy;
        *self = next;
        Ok(())
    }
    fn advance_closed_interval(&mut self, interval: Duration) {
        let mut nanos = interval.as_nanos() + u128::from(self.offline.pending_nanos);
        self.offline.pending_nanos = 0;
        while nanos >= u128::from(TICK_NANOS) {
            let core = self.ship.snapshot().core_status;
            let possible = self.construction_offline_pending() || self.recovery_work_pending();
            let funded = self.ship.snapshot().energy_core.stored_joules as f64
                > self.recovery_reserve()
                || core.conversion_remaining_joules > 0.;
            // With no eligible auxiliary work (or future energy), ordinary power
            // and Fabricator already split at their exact source/output boundaries.
            // Skip quiet time, never grant work or cap real elapsed time.
            let moving = self.motion.is_some()
                && !matches!(
                    self.ship.snapshot().flight_state,
                    salimon_ship::FlightState::Landed { .. }
                        | salimon_ship::FlightState::GroundedAfterCrash { .. }
                );
            let ordinary_funded =
                self.ship.stored_core_joules_exact() > 0. || core.conversion_remaining_joules > 0.;
            let timed = self.motion.is_some()
                && (self.fabricator.power_demand().is_some()
                    || core.conversion_remaining_joules > 0.);
            let scanner_samples = self.motion.is_some() && !self.stationary_scanner_certified();
            if self.motion.is_some()
                && !moving
                && !possible
                && !scanner_samples
                && self.fabricator.power_demand().is_none()
                && core.conversion_remaining_joules == 0.
            {
                let step = nanos / u128::from(TICK_NANOS) * u128::from(TICK_NANOS);
                let delta =
                    Duration::new((step / 1_000_000_000) as u64, (step % 1_000_000_000) as u32);
                if let Err(error) = self.advance_closed_stationary(delta) {
                    self.motion_error = Some(error.to_string());
                    return;
                }
                nanos -= step;
                continue;
            }
            let step = if (possible && funded)
                || moving
                || ((timed || scanner_samples) && ordinary_funded)
            {
                u128::from(TICK_NANOS)
            } else {
                nanos / u128::from(TICK_NANOS) * u128::from(TICK_NANOS)
            };
            let delta = Duration::new((step / 1_000_000_000) as u64, (step % 1_000_000_000) as u32);
            self.closed_work_tick(delta);
            if self.motion_error.is_some() {
                return;
            }
            nanos -= step;
        }
        self.offline.pending_nanos = nanos as u64;
    }
    fn closed_work_tick(&mut self, delta: Duration) {
        self.advance_closed_sources(delta);
        if self.motion.is_some() && self.motion_error.is_none() {
            self.advance_scanner(delta);
            if let Some(error) = &self.scanner.error {
                self.motion_error = Some(error.clone());
            }
        }
    }
    pub(in crate::session) fn advance_closed_sources(&mut self, delta: Duration) {
        self.offline_power = true;
        let before = self.ship.snapshot().flight_state;
        self.advance_surface_age(delta);
        self.advance_fabrication(delta);
        let after = self.ship.snapshot().flight_state;
        if !matches!(before, salimon_ship::FlightState::GroundedAfterCrash { .. })
            && matches!(after, salimon_ship::FlightState::GroundedAfterCrash { .. })
        {
            self.interrupt_construction_after_crash();
        }
        self.stop_airborne_construction();
        self.advance_recovery(delta);
        self.advance_construction(delta);
        let powered = self.ship.snapshot().power_available;
        self.character.set_ship_gravity_enabled(powered);
        self.mining.ship_gravity_disabled = !powered;
        self.offline_power = false;
    }
    pub(super) fn finish_offline_remainder(&mut self) {
        let nanos = std::mem::take(&mut self.offline.pending_nanos);
        if nanos > 0 && !self.survival.snapshot().terminal {
            self.closed_work_tick(Duration::from_nanos(nanos));
        }
    }
}
#[cfg(test)]
mod tests;

#[cfg(test)]
mod motion_tests;
