//! Analytic funded contact windows for an idle ship fixed in its body frame.
//! Active work and curved trajectories remain on the chronological physical
//! scheduler. No affine chord is substituted for an orbital observer path.
use super::*;
use salimon_persistence::{SaveError, invalid};
use salimon_universe::scanner::pulses::{LinearTrack, Sweep};

impl GameplaySession {
    pub(in crate::session) fn stationary_scanner_certified(&self) -> bool {
        let radius = salimon_math::length(self.ship.snapshot().pose.position_meters);
        // Nacre stays outside range for every orbital direction, leaving only
        // the stationary Vesper target. Otherwise sample the actual curved path.
        self.scanner.parameters.range_meters + radius
            < salimon_universe::starting_motion::DEFINITION.semi_major_axis_meters
            || self.ship.stored_core_joules_exact() == 0.
    }
    pub(in crate::session) fn advance_closed_stationary(
        &mut self,
        delta: Duration,
    ) -> Result<(), SaveError> {
        let nanos = delta.as_nanos() + u128::from(self.scanner.sub_millis_nanos);
        let millis = u64::try_from(nanos / 1_000_000)
            .map_err(|_| invalid("offline scanner interval overflow"))?;
        let start = self.scanner.utc_millis;
        let end = start
            .checked_add(millis)
            .ok_or_else(|| invalid("offline scanner clock overflow"))?;
        let parameters = self.scanner.parameters;
        let scan_cost = parameters
            .cost(start, end)
            .map_err(|e| invalid(e.to_string()))?;
        let config = self.ship.energy_config();
        let service = config.gravity_idle_watts + config.cockpit_idle_watts;
        let available = self.ship.stored_core_joules_exact();
        let fully_funded = service * delta.as_secs_f64() + scan_cost <= available;
        let mut funded_millis = millis;
        if !fully_funded {
            // Monotone exact pulse-lattice cost, logarithmic in elapsed time.
            // Only fully funded pulses may refresh history. A failed later
            // pulse/idle slice cannot erase an earlier successful observation.
            let mut low = 0;
            let mut high = millis;
            while low < high {
                let mid = low + (high - low).div_ceil(2);
                let source_nanos = (u128::from(mid) * 1_000_000)
                    .saturating_sub(u128::from(self.scanner.sub_millis_nanos));
                let cost = service * (source_nanos as f64 * 1e-9)
                    + parameters
                        .cost(start, start + mid)
                        .map_err(|e| invalid(e.to_string()))?;
                if cost <= available {
                    low = mid;
                } else {
                    high = mid - 1;
                }
            }
            funded_millis = low;
        }
        let funded_end = start + funded_millis;
        let prefix_cost = parameters
            .cost(start, funded_end)
            .map_err(|e| invalid(e.to_string()))?;
        let body = salimon_world::canonical::START.manifest().planet.id.clone();
        let observer = LinearTrack {
            position: Position::BodyLocal {
                body: body.clone(),
                meters: self.ship.snapshot().pose.position_meters,
            },
            velocity_mps: [0.; 3],
        };
        let target = Observation {
            id: body.clone(),
            position: Position::BodyLocal {
                body,
                meters: [0.; 3],
            },
            velocity_mps: [0.; 3],
        };
        let mut contacts = self.scanner.contacts.clone();
        contacts
            .scan_linear(
                &CATALOG,
                Sweep {
                    start_ms: start,
                    end_ms: funded_end,
                    observer: &observer,
                    targets: &[target],
                    parameters,
                    powered: true,
                },
            )
            .map_err(|e| invalid(e.to_string()))?;
        contacts.advance(end).map_err(|e| invalid(e.to_string()))?;
        let mut chart = self.chart.clone();
        chart
            .observe(&contacts)
            .map_err(|e| invalid(e.to_string()))?;
        // No runnable Fabricator, conversion, recovery or construction exists
        // in this fast lane. Reserve the certified scanner prefix, settle idle
        // service, then debit the unfunded tail. This preserves the exact final
        // Core source and all funded contacts without replaying pulse frames.
        if !delta.is_zero() {
            self.ship
                .fund_auxiliary_work(prefix_cost, 0., delta)
                .map_err(|_| invalid("offline scanner prefix funding"))?;
        }
        self.advance_closed_sources(delta);
        if let Some(error) = &self.motion_error {
            return Err(invalid(error.clone()));
        }
        if !delta.is_zero() {
            self.ship
                .fund_auxiliary_work((scan_cost - prefix_cost).max(0.), 0., delta)
                .map_err(|_| invalid("offline scanner tail funding"))?;
        }
        self.scanner.contacts = contacts;
        self.scanner.utc_millis = end;
        self.scanner.sub_millis_nanos = (nanos % 1_000_000) as u32;
        self.scanner.powered = fully_funded && self.ship.snapshot().power_available;
        self.scanner.error = None;
        self.chart = chart;
        Ok(())
    }
}
