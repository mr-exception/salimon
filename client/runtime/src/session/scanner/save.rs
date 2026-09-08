//! Schema-25 scanner source; derived power, errors and presentation are transient.
use super::*;
use salimon_persistence::{SaveError, Validate, invalid};
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScannerSave {
    version: u32,
    contacts: salimon_universe::scanner::checkpoint::Checkpoint,
    parameters: Parameters,
    utc_millis: u64,
    sub_millis_nanos: u32,
}
impl Validate for ScannerSave {
    fn validate(&self) -> Result<(), SaveError> {
        if self.contacts.catalog_revision != CATALOG.revision() {
            return Err(SaveError::IncompatibleCatalog(
                "scanner catalog revision".into(),
            ));
        }
        if self.version != 1
            || self.sub_millis_nanos >= 1_000_000
            || self.contacts.watermark_utc_millis != self.utc_millis
        {
            return Err(invalid("scanner version/clock/remainder"));
        }
        self.parameters
            .validate()
            .map_err(|e| invalid(e.to_string()))?;
        Contacts::from_checkpoint(&CATALOG, self.contacts.clone())
            .map_err(|e| invalid(e.to_string()))?;
        Ok(())
    }
}
impl Scanner {
    pub(crate) fn capture(&self) -> ScannerSave {
        ScannerSave {
            version: 1,
            contacts: self.contacts.checkpoint(),
            parameters: self.parameters,
            utc_millis: self.utc_millis,
            sub_millis_nanos: self.sub_millis_nanos,
        }
    }
    pub(crate) fn restore(
        source: Option<ScannerSave>,
        schema: u32,
        canonical: bool,
        saved_at: u64,
    ) -> Result<Self, SaveError> {
        match (canonical, schema, source) {
            (true, 25.., Some(s)) => {
                s.validate()?;
                Ok(Self {
                    selected: 0,
                    chart_mode: false,
                    chart_note: None,
                    contacts: Contacts::from_checkpoint(&CATALOG, s.contacts)
                        .map_err(|e| invalid(e.to_string()))?,
                    parameters: s.parameters,
                    utc_millis: s.utc_millis,
                    sub_millis_nanos: s.sub_millis_nanos,
                    powered: false,
                    error: None,
                })
            }
            (true, 1..=24, None) => {
                let mut scanner = Self::default();
                scanner.advance_closed(
                    saved_at
                        .checked_mul(1000)
                        .ok_or_else(|| invalid("scanner UTC overflow"))?,
                )?;
                Ok(scanner)
            }
            (false, _, None) => Ok(Self::default()),
            _ => Err(invalid(
                "scanner source missing or incompatible with schema/world",
            )),
        }
    }
    pub(crate) fn advance_closed(&mut self, elapsed_millis: u64) -> Result<(), SaveError> {
        let end = self
            .utc_millis
            .checked_add(elapsed_millis)
            .ok_or_else(|| invalid("scanner UTC overflow"))?;
        self.contacts
            .advance(end)
            .map_err(|e| invalid(e.to_string()))?;
        self.utc_millis = end;
        self.powered = false;
        Ok(())
    }
}
impl GameplaySession {
    pub(crate) fn bind_scanner_clock(&mut self, utc: u64) -> Result<(), SaveError> {
        if let Some(clock) = &mut self.motion {
            // This entry point is called for a new journey, never ordinary
            // restore. Native startup binds both clocks before first publication.
            *clock = salimon_universe::starting_motion::Clock::at(utc)
                .map_err(|e| invalid(e.to_string()))?;
        }
        if self.mining.terrain == salimon_world::terrain::TerrainMode::Canonical
            && self.scanner.utc_millis == 0
            && self.scanner.contacts.history().next().is_none()
        {
            self.scanner.advance_closed(
                utc.checked_mul(1000)
                    .ok_or_else(|| invalid("scanner initial UTC overflow"))?,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scanner_persistence_roundtrip_launch_expiry_rediscovery_and_no_replay() {
        let mut s = GameplaySession::new_canonical_run().unwrap();
        s.bind_scanner_clock(100).unwrap();
        s.advance_automation(Duration::from_millis(5008));
        let at = s
            .scanner
            .contacts
            .history()
            .next()
            .unwrap()
            .last_seen_utc_millis();
        let bytes = s.save_bytes(105).unwrap();
        let mut loaded = GameplaySession::default();
        loaded.restore_bytes(&bytes).unwrap();
        assert_eq!(loaded.save_bytes(105).unwrap(), bytes);
        assert!(!loaded.scanner.powered);
        // Expiry is exercised through the real launch adapter. Power is removed
        // to keep the long construction reconciliation in its no-work fast lane.
        s.ship.exchange_core_energy_joules(0., 1e30).unwrap();
        let bytes = s.save_bytes(105).unwrap();
        loaded.restore_for_launch(&bytes, 105 + 3 * 3600).unwrap();
        assert_eq!(
            loaded
                .scanner
                .contacts
                .active(loaded.scanner.utc_millis)
                .count(),
            0
        );
        assert_eq!(
            loaded
                .scanner
                .contacts
                .history()
                .next()
                .unwrap()
                .last_seen_utc_millis(),
            at
        );
        let credited = loaded.save_bytes(105 + 3 * 3600).unwrap();
        let clock = loaded.scanner.utc_millis;
        loaded
            .restore_for_launch(&credited, 105 + 3 * 3600)
            .unwrap();
        assert_eq!(loaded.scanner.utc_millis, clock);
        loaded.ship.exchange_core_energy_joules(1e8, 0.).unwrap();
        loaded.advance_automation(Duration::from_secs(5));
        assert_eq!(
            loaded
                .scanner
                .contacts
                .active(loaded.scanner.utc_millis)
                .count(),
            1
        );
    }
    #[test]
    fn scanner_persistence_invalid_state_catalog_and_schema_are_atomic() {
        let mut s = GameplaySession::new_canonical_run().unwrap();
        s.advance_automation(Duration::from_millis(5008));
        let original = s.save_bytes(100).unwrap();
        let source: serde_json::Value = serde_json::from_slice(&original).unwrap();
        for (path, value) in [
            ("/state/scanner/version", serde_json::json!(2)),
            (
                "/state/scanner/sub_millis_nanos",
                serde_json::json!(1_000_000),
            ),
            (
                "/state/scanner/contacts/catalog_revision",
                serde_json::json!("wrong"),
            ),
            (
                "/state/scanner/contacts/encounters/0/last_seen_utc_millis",
                serde_json::json!(u64::MAX),
            ),
            (
                "/state/scanner/parameters/range_meters",
                serde_json::json!(-1),
            ),
        ] {
            let mut bad = source.clone();
            *bad.pointer_mut(path).unwrap() = value;
            assert!(
                s.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err(),
                "{path}"
            );
            assert_eq!(s.save_bytes(100).unwrap(), original);
        }
        let mut missing = source.clone();
        missing["state"].as_object_mut().unwrap().remove("scanner");
        assert!(
            s.restore_bytes(&serde_json::to_vec(&missing).unwrap())
                .is_err()
        );
        missing["schema_version"] = 24.into();
        missing["state"].as_object_mut().unwrap().remove("chart");
        s.restore_bytes(&serde_json::to_vec(&missing).unwrap())
            .unwrap();
        assert_eq!(s.scanner.contacts.history().count(), 0);
        assert_eq!(s.scanner.utc_millis, 100_000);
    }
}
