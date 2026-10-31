use super::*;
use salimon_fabrication::{FabricationCommand as C, RecipeId};
use salimon_world::resources::{RawMaterial, ResourceId};
use serde_json::Value;
fn grounded(energy: f64) -> GameplaySession {
    let mut s = GameplaySession::new_canonical_run().unwrap();
    s.bind_scanner_clock(100).unwrap();
    s.ship.exchange_core_energy_joules(0., 1e30).unwrap();
    s.ship.exchange_core_energy_joules(energy, 0.).unwrap();
    s
}
fn reload(s: &GameplaySession, saved: u64, launch: u64) -> GameplaySession {
    let mut out = GameplaySession::default();
    out.restore_for_launch(&s.save_bytes(saved).unwrap(), launch)
        .unwrap();
    out
}
#[test]
fn closed_stationary_pulses_funding_and_partitions_preserve_exact_source_history() {
    for energy in [0., 5000., 600_000., 10_000_000.] {
        let s = grounded(energy);
        let whole = reload(&s, 100, 120);
        let mut split = reload(&s, 100, 101);
        for time in 101..120 {
            split = reload(&split, time, time + 1);
        }
        assert_eq!(
            serde_json::to_value(whole.scanner.capture()).unwrap(),
            serde_json::to_value(split.scanner.capture()).unwrap(),
            "energy {energy}"
        );
        assert_eq!(
            whole.ship.stored_core_joules_exact(),
            split.ship.stored_core_joules_exact(),
            "energy {energy}"
        );
        assert_eq!(whole.motion, split.motion);
        assert_eq!(whole.scanner.utc_millis, 120_000);
        assert_eq!(whole.offline.pending_nanos, 0);
    }
}
#[test]
fn funded_early_pulse_survives_later_blackout_then_expires_after_three_hours() {
    let s = grounded(600_000.);
    let short = reload(&s, 100, 120);
    let e = short.scanner.contacts.history().next().unwrap();
    assert_eq!(e.last_seen_utc_millis(), 105_000);
    assert_eq!(e.successful_refreshes(), 1);
    assert_eq!(short.ship.stored_core_joules_exact(), 0.);
    let long = reload(&short, 120, 105 + 3 * 3600);
    assert_eq!(
        long.scanner
            .contacts
            .active(long.scanner.utc_millis)
            .count(),
        0
    );
    assert_eq!(
        long.scanner
            .contacts
            .history()
            .next()
            .unwrap()
            .last_seen_utc_millis(),
        105_000
    );
    assert_eq!(long.chart.entries().count(), 1);
}
#[test]
fn idle_year_uses_analytic_pulse_count_instead_of_replaying_frames() {
    let mut s = grounded(1e15);
    let energy = salimon_ship::EnergyConfig {
        capacity_joules: 1_000_000_000_000_000,
        gravity_idle_watts: 0.,
        cockpit_idle_watts: 0.,
        ..s.ship.energy_config()
    };
    s.ship.configure_energy_core(energy, 1e15).unwrap();
    let elapsed = 365 * 86400;
    let out = reload(&s, 100, 100 + elapsed);
    let e = out.scanner.contacts.history().next().unwrap();
    assert_eq!(e.successful_refreshes(), elapsed * 1000 / 5000);
    assert_eq!(e.last_seen_utc_millis(), (100 + elapsed) * 1000);
    assert_eq!(
        out.ship.stored_core_joules_exact(),
        1e15 - (elapsed as f64 * 1000. + (elapsed / 5) as f64 * 10000.)
    );
}
#[test]
fn fabrication_starvation_freezes_survival_and_pause_has_no_offline_credit() {
    let mut s = grounded(600_000.);
    s.execute_fabricator(C::Insert(
        RawMaterial::new(ResourceId::WaterIce, 1.).unwrap(),
    ))
    .unwrap();
    s.execute_fabricator(C::Insert(
        RawMaterial::new(ResourceId::OrganicBiomass, 1.).unwrap(),
    ))
    .unwrap();
    s.execute_fabricator(C::Enqueue {
        recipe: RecipeId::new("food-ration").unwrap(),
        quantity: 1,
    })
    .unwrap();
    s.survival
        .command(salimon_survival::Command::Damage(
            salimon_survival::Points::whole(10).unwrap(),
        ))
        .unwrap();
    let mut tired = serde_json::to_value(&s.survival).unwrap();
    tired["fatigue"] = serde_json::to_value(salimon_survival::Points::whole(20).unwrap()).unwrap();
    s.survival = serde_json::from_value(tired).unwrap();
    let before: Value = serde_json::to_value(&s.survival).unwrap();
    let out = reload(&s, 100, 160);
    let machine = out.fabricator.snapshot(false);
    assert!(machine.tray.is_none());
    assert!(machine.jobs[0].progress > Duration::ZERO);
    assert!(machine.jobs[0].progress < machine.jobs[0].unit_duration);
    assert!(out.survival.snapshot().fatigue > s.survival.snapshot().fatigue);
    let after: Value = serde_json::to_value(&out.survival).unwrap();
    for key in [
        "hunger",
        "health",
        "active_time",
        "starvation_elapsed",
        "pipeline",
    ] {
        assert_eq!(before[key], after[key], "frozen survival {key}");
    }
    s.toggle_pause();
    let bytes = s.save_bytes(100).unwrap();
    s.advance_automation(Duration::from_secs(60));
    assert_eq!(s.save_bytes(100).unwrap(), bytes);
    let whole = reload(&s, 100, 160);
    let first = reload(&s, 100, 130);
    let split = reload(&first, 130, 160);
    assert_eq!(
        serde_json::to_value(salimon_fabrication::FabricationSaveState::capture(
            &whole.fabricator,
            &whole.consumables
        ))
        .unwrap(),
        serde_json::to_value(salimon_fabrication::FabricationSaveState::capture(
            &split.fabricator,
            &split.consumables
        ))
        .unwrap()
    );
}
#[test]
fn fractional_core_storage_can_fund_a_real_fractional_pulse() {
    let mut s = grounded(10.5);
    s.scanner.parameters.idle_watts = 1.;
    s.scanner.parameters.pulse_joules = 10.;
    s.scanner.advance_closed(4999).unwrap();
    s.try_advance_scanner(Duration::from_millis(1)).unwrap();
    assert_eq!(s.scanner.contacts.history().count(), 1);
    assert!((s.ship.stored_core_joules_exact() - 0.499).abs() < 1e-10);
}

#[test]
fn curved_departure_keeps_an_early_in_range_pulse_after_leaving_range() {
    let mut s = grounded(1e9);
    let body = salimon_world::canonical::BODIES[0];
    let mut ship = salimon_ship::ShipController::flying_with_velocity(
        salimon_ship::ShipPose {
            position_meters: [0., body.radius_meters + 100., 0.],
            orientation: [0., 0., 0., 1.],
        },
        0,
        [0., 500., 0.],
    )
    .unwrap();
    ship.configure_terrain(
        salimon_world::terrain::TerrainMode::Canonical,
        crate::terrain::landing_footprint(),
    );
    s.ship = ship;
    s.sync_fabricator_mass();
    s.scanner.parameters.range_meters = body.radius_meters + 200.;
    s.scanner.advance_closed(4900).unwrap();
    let out = reload(&s, 100, 110);
    let e = out.scanner.contacts.history().next().unwrap();
    assert_eq!(e.successful_refreshes(), 1);
    assert!((105_000..105_016).contains(&e.last_seen_utc_millis()));
    assert!(out.ship.snapshot().pose.position_meters[1] > body.radius_meters + 200.);
}
#[test]
fn invalid_late_scanner_counter_rolls_back_the_whole_launch() {
    let mut s = grounded(1e9);
    s.try_advance_scanner(Duration::from_secs(5)).unwrap();
    let mut c = serde_json::to_value(s.scanner.contacts.checkpoint()).unwrap();
    c["encounters"][0]["successful_refreshes"] = serde_json::json!(u64::MAX);
    s.scanner.contacts =
        Contacts::from_checkpoint(&CATALOG, serde_json::from_value(c).unwrap()).unwrap();
    s.chart.observe(&s.scanner.contacts).unwrap();
    let mut target = GameplaySession::default();
    let original = target.save_bytes(105).unwrap();
    assert!(
        target
            .restore_for_launch(&s.save_bytes(105).unwrap(), 110)
            .is_err()
    );
    assert_eq!(target.save_bytes(105).unwrap(), original);
}
