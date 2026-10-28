//! Scanner composition: ordinary Core-funded service; discovery owns no catalog truth.
mod presentation;
pub(crate) mod save;
use super::GameplaySession;
use salimon_universe::{
    Catalog,
    scanner::{Contacts, Observation, Position, pulses::Parameters},
};
use std::{sync::LazyLock, time::Duration};
pub(crate) static CATALOG: LazyLock<Catalog> = LazyLock::new(|| {
    salimon_universe::starting_system::candidate_publication()
        .expect("checked permanent candidate publication")
        .catalog()
        .clone()
});
#[derive(Clone, Debug)]
pub(crate) struct Scanner {
    pub(crate) selected: usize,
    pub(crate) chart_mode: bool,
    pub(crate) chart_note: Option<&'static str>,
    pub(crate) contacts: Contacts,
    pub(crate) parameters: Parameters,
    pub(crate) utc_millis: u64,
    pub(crate) sub_millis_nanos: u32,
    pub(crate) powered: bool,
    pub(crate) error: Option<String>,
}
impl Default for Scanner {
    fn default() -> Self {
        Self {
            selected: 0,
            chart_mode: false,
            chart_note: None,
            contacts: Contacts::new(CATALOG.revision()).expect("checked revision"),
            parameters: serde_json::from_str(include_str!("../../config/scanner.json"))
                .expect("checked scanner configuration"),
            utc_millis: 0,
            sub_millis_nanos: 0,
            powered: false,
            error: None,
        }
    }
}
impl GameplaySession {
    pub(crate) fn advance_scanner(&mut self, delta: Duration) {
        // Historical compressed fixtures deliberately have no canonical scanner.
        if self.mining.terrain != salimon_world::terrain::TerrainMode::Canonical {
            return;
        }
        if let Err(e) = self.try_advance_scanner(delta) {
            self.scanner.powered = false;
            self.scanner.error = Some(e);
        }
    }
    fn try_advance_scanner(&mut self, delta: Duration) -> Result<(), String> {
        let nanos = delta.as_nanos() + u128::from(self.scanner.sub_millis_nanos);
        let millis = u64::try_from(nanos / 1_000_000).map_err(|_| "scanner clock overflow")?;
        let start = self.scanner.utc_millis;
        let end = start.checked_add(millis).ok_or("scanner clock overflow")?;
        let cost = self
            .scanner
            .parameters
            .cost(start, end)
            .map_err(|e| e.to_string())?;
        // Validate all source/evaluation work before debiting the real Core.
        let body = salimon_world::canonical::START.manifest().planet.id.clone();
        let ship = self.ship.snapshot();
        let target = Observation {
            id: body.clone(),
            position: Position::BodyLocal {
                body: body.clone(),
                meters: [0.; 3],
            },
            velocity_mps: [0.; 3],
        };
        // Online simulation samples the completed authoritative tick. Pulse
        // cadence is absolute; observed states are timestamped at this tick,
        // never backdated using an uncertified linear extrapolation.
        let observer = Position::BodyLocal {
            body,
            meters: ship.pose.position_meters,
        };
        let relative = target
            .position
            .relative_meters(&observer)
            .map_err(|e| e.to_string())?;
        let due = end / 5000 > start / 5000;
        let hit = due
            && relative[0].hypot(relative[1]).hypot(relative[2])
                <= self.scanner.parameters.range_meters;
        let available = self.ship.snapshot().energy_core.stored_joules as f64;
        let powered = ship.power_available && available >= cost;
        // Validate the contact transaction before spending Core joules. A
        // rejected saved counter/state must not repeatedly bill the same pulse.
        let refreshed = if powered && hit {
            let mut contacts = self.scanner.contacts.clone();
            contacts
                .refresh(&CATALOG, end, true, &[target])
                .map_err(|e| e.to_string())?;
            Some(contacts)
        } else {
            None
        };
        if !delta.is_zero() {
            self.ship
                .fund_auxiliary_work(cost, 0., delta)
                .map_err(|_| "scanner Core funding")?;
        }
        if let Some(contacts) = refreshed {
            self.scanner.contacts = contacts;
        }
        self.scanner
            .contacts
            .advance(end)
            .map_err(|e| e.to_string())?;
        if powered && hit {
            self.chart
                .observe(&self.scanner.contacts)
                .map_err(|e| e.to_string())?;
        }
        self.scanner.utc_millis = end;
        self.scanner.sub_millis_nanos = (nanos % 1_000_000) as u32;
        self.scanner.powered = powered;
        self.scanner.error = None;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scanner_real_session_five_second_core_draw_blackout_and_pause() {
        let mut s = GameplaySession::new_canonical_run().unwrap();
        let original = s.ship.snapshot().energy_core.stored_joules;
        s.advance_automation(Duration::from_millis(4992));
        assert_eq!(s.scanner.contacts.history().count(), 0);
        s.advance_automation(Duration::from_millis(16));
        assert_eq!(s.scanner.contacts.history().count(), 1);
        assert!(s.ship.snapshot().energy_core.stored_joules < original);
        let last = s
            .scanner
            .contacts
            .history()
            .next()
            .unwrap()
            .last_seen_utc_millis();
        s.ship.exchange_core_energy_joules(0., 1e30).unwrap();
        s.advance_automation(Duration::from_secs(5));
        assert!(!s.scanner.powered);
        assert_eq!(
            s.scanner
                .contacts
                .history()
                .next()
                .unwrap()
                .last_seen_utc_millis(),
            last
        );
        s.command(super::super::Command::TogglePause);
        let clock = s.scanner.utc_millis;
        s.advance_automation(Duration::from_secs(10));
        assert_eq!(s.scanner.utc_millis, clock);
    }
}
