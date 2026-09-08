use super::*;
use crate::session::input::{Key, dispatch};
use std::time::Duration;
fn cockpit() -> GameplaySession {
    let mut s = GameplaySession::new_canonical_run().unwrap();
    s.character = Default::default();
    s.character.enter_cockpit();
    s.ship.set_cockpit_control(true);
    s
}
#[test]
fn chart_discovery_ui_route_expiry_resume_and_legacy_upgrade() {
    let mut s = cockpit();
    assert_eq!(s.chart.entries().count(), 0);
    dispatch(&mut s, Key::TourPause, true);
    assert!(s.scanner.chart_mode);
    assert!(s.scanner_hint().unwrap().contains("EMPTY"));
    dispatch(&mut s, Key::Confirm, true);
    assert!(s.chart.routes().is_empty());
    s.advance_automation(Duration::from_millis(5008));
    assert_eq!(s.chart.entries().count(), 1);
    let before = s.ship.snapshot();
    dispatch(&mut s, Key::Confirm, true);
    assert_eq!(s.chart.routes().len(), 1);
    assert_eq!(s.ship.snapshot(), before);
    let bytes = s.save_bytes(100).unwrap();
    let mut loaded = GameplaySession::default();
    loaded.restore_bytes(&bytes).unwrap();
    assert_eq!(loaded.save_bytes(100).unwrap(), bytes);
    assert!(!loaded.scanner.chart_mode);
    assert_eq!(loaded.chart.routes().len(), 1);
    loaded
        .scanner
        .advance_closed(salimon_universe::scanner::CONTACT_TTL_MILLIS)
        .unwrap();
    loaded.chart.observe(&loaded.scanner.contacts).unwrap();
    assert_eq!(
        loaded
            .scanner
            .contacts
            .active(loaded.scanner.utc_millis)
            .count(),
        0
    );
    assert_eq!(loaded.chart.entries().count(), 1);
    loaded.scanner.chart_mode = true;
    assert!(loaded.chart_hint().contains("HISTORICAL"));
    let mut legacy: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    legacy["schema_version"] = 25.into();
    legacy["state"].as_object_mut().unwrap().remove("chart");
    loaded
        .restore_bytes(&serde_json::to_vec(&legacy).unwrap())
        .unwrap();
    assert_eq!(loaded.chart.entries().count(), 1);
    assert_eq!(loaded.chart.routes().len(), 0);
}
#[test]
fn chart_corrupt_checkpoint_or_forged_discovery_never_publishes() {
    let mut s = cockpit();
    s.advance_automation(Duration::from_millis(5008));
    let bytes = s.save_bytes(100).unwrap();
    let source: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    for (path, value) in [
        ("/state/chart/version", serde_json::json!(99)),
        ("/state/chart/catalog_revision", serde_json::json!("wrong")),
        (
            "/state/chart/entries/0/observed/last_seen_utc_millis",
            serde_json::json!(u64::MAX),
        ),
        ("/state/chart/entries", serde_json::json!([])),
    ] {
        let mut bad = source.clone();
        *bad.pointer_mut(path).unwrap() = value;
        assert!(
            s.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "{path}"
        );
        assert_eq!(s.save_bytes(100).unwrap(), bytes);
    }
    let mut bad = source;
    bad["state"].as_object_mut().unwrap().remove("chart");
    assert!(s.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
}
#[test]
fn chart_input_respects_pause_modal_and_outside_cockpit() {
    let mut s = cockpit();
    s.command(crate::session::Command::TogglePause);
    dispatch(&mut s, Key::TourPause, true);
    assert!(!s.scanner.chart_mode);
    s.command(crate::session::Command::TogglePause);
    s.archive_reader.open = true;
    dispatch(&mut s, Key::TourPause, true);
    assert!(!s.scanner.chart_mode);
    s.archive_reader.open = false;
    s.character.leave_cockpit();
    dispatch(&mut s, Key::TourPause, true);
    assert!(!s.scanner.chart_mode);
}

#[test]
fn chart_range_change_rediscovery_preserves_routes_and_recovered_lead_uncertainty() {
    use salimon_universe::scanner::chart::{CoordinateUncertainty, Lead};
    let mut s = cockpit();
    s.advance_automation(Duration::from_millis(5008));
    s.chart_toggle();
    s.chart_route_selected();
    let routes = s.chart.routes().to_vec();
    s.scanner
        .advance_closed(salimon_universe::scanner::CONTACT_TTL_MILLIS)
        .unwrap();
    s.scanner.parameters.range_meters *= 2.;
    s.advance_automation(Duration::from_millis(5008));
    assert_eq!(s.chart.routes(), routes);
    assert_eq!(s.chart.entries().count(), 1);
    assert_eq!(
        s.chart
            .entries()
            .next()
            .unwrap()
            .observed()
            .unwrap()
            .successful_refreshes(),
        2
    );
    let id = CATALOG
        .records()
        .iter()
        .find(|r| matches!(r, salimon_universe::Record::Star(_)))
        .unwrap()
        .id()
        .clone();
    let lead = Lead {
        position: Position::BodyLocal {
            body: salimon_world::canonical::START.manifest().planet.id.clone(),
            meters: [1000., 0., 0.],
        },
        source: "fixture-recovered-warning-v1".into(),
        received_utc_millis: s.scanner.utc_millis,
        inferred: false,
        uncertainty: CoordinateUncertainty::Unknown,
    };
    s.chart_receive_lead(id.clone(), lead.clone()).unwrap();
    s.chart_receive_lead(id.clone(), lead.clone()).unwrap();
    assert_eq!(s.chart.get(&id).unwrap().leads().len(), 1);
    s.scanner.selected = s.chart.entries().position(|e| e.id == id).unwrap();
    assert!(s.chart_hint().contains("TRANSMITTED"));
    assert!(s.chart_hint().contains("UNCERTAINTY UNKNOWN"));
    s.chart_route_selected();
    assert_eq!(s.chart.routes(), routes);
    let before = s.chart.clone();
    let mut future = lead;
    future.source = "fixture-future-v2".into();
    future.received_utc_millis += 1;
    assert!(s.chart_receive_lead(id.clone(), future).is_err());
    assert_eq!(s.chart, before);
    let bytes = s.save_bytes(100).unwrap();
    s.restore_bytes(&bytes).unwrap();
    assert_eq!(s.scanner.parameters.range_meters, 2e9);
    assert_eq!(
        s.chart.get(&id).unwrap().leads()[0].uncertainty,
        CoordinateUncertainty::Unknown
    );
}

#[test]
fn rejected_saved_refresh_counter_never_bills_or_replays_the_same_scanner_pulse() {
    let mut s = cockpit();
    s.advance_automation(Duration::from_millis(5008));
    let mut source: serde_json::Value =
        serde_json::from_slice(&s.save_bytes(100).unwrap()).unwrap();
    source["state"]["scanner"]["contacts"]["encounters"][0]["successful_refreshes"] =
        u64::MAX.into();
    source["state"]["chart"]["entries"][0]["observed"]["successful_refreshes"] = u64::MAX.into();
    s.restore_bytes(&serde_json::to_vec(&source).unwrap())
        .unwrap();
    let bytes = s.save_bytes(100).unwrap();
    let energy = s.ship.snapshot().energy_core.stored_joules;
    for _ in 0..2 {
        s.advance_scanner(Duration::from_secs(5));
        assert!(s.scanner.error.is_some());
        assert_eq!(s.ship.snapshot().energy_core.stored_joules, energy);
        assert_eq!(s.save_bytes(100).unwrap(), bytes);
    }
}
