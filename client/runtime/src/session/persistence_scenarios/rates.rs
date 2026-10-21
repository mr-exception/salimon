//! Long durations exercise the survival domain directly; session routing and
//! fixed-tick partition invariance are covered separately below.
use super::*;
use salimon_survival::{Command as VitalCommand, Survival, restrictions::Action};

fn domain_advance(s: &mut Survival, duration: Duration, frame: Duration) {
    let mut remaining = duration;
    while !remaining.is_zero() {
        let step = remaining.min(frame);
        s.advance(step);
        remaining -= step;
    }
}

#[test]
fn persistence_scenario_exact_rates_survive_split_save_and_offline_recovery() {
    for frame in [
        Duration::from_millis(16),
        Duration::from_millis(33),
        Duration::from_millis(100),
    ] {
        let mut whole = Survival::default();
        let mut split = whole.clone();
        domain_advance(&mut split, Duration::from_secs(900), frame);
        split = serde_json::from_slice(&serde_json::to_vec(&split).unwrap()).unwrap();
        split.validate().unwrap();
        domain_advance(&mut split, Duration::from_secs(900), frame);
        whole.advance(Duration::from_secs(1800));
        assert_eq!(whole, split);
        assert_eq!(split.snapshot().hunger, Points::ZERO);
        split.advance(Duration::from_secs(10) - Duration::from_nanos(1));
        assert_eq!(split.snapshot().health, Points::FULL);
        split.advance(Duration::from_nanos(1));
        assert_eq!(split.snapshot().health, Points::whole(99).unwrap());
    }
    // Keep hunger funded to isolate the unmodified eighteen-hour fatigue rate.
    let mut s = Survival::default();
    for _ in 0..72 {
        s.command(VitalCommand::Feed(Points::FULL)).unwrap();
        s.advance(Duration::from_secs(900));
        s = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
        s.validate().unwrap();
    }
    assert_eq!(s.snapshot().fatigue, Points::ZERO);
    assert!(!s.snapshot().terminal);
    s.recover_offline(100, 21700);
    assert_eq!(s.snapshot().fatigue, Points::FULL);
}

#[test]
fn restored_exhaustion_gates_commands_and_q_remains_an_emergency_action() {
    let mut s = initialize(e2e::Scenario::FlightRegression);
    meters(&mut s, 10, 0);
    let saved = s.save_bytes(100).unwrap();
    s.restore_bytes(&saved).unwrap();
    for action in [
        Action::Pilot,
        Action::Mine,
        Action::Fabricate,
        Action::Carry,
    ] {
        assert!(!s.restrictions().permits(action));
    }
    assert!(s.restrictions().permits(Action::Consume));
    let hunger = s.survival.snapshot().hunger;
    for request in [
        json!({"op":"key","key":"consumable_wheel","pressed":true}),
        json!({"op":"wheel_hover","slot":0}),
        json!({"op":"mouse","button":"left","pressed":true}),
    ] {
        assert_eq!(protocol(&mut s, request)["ok"], true);
    }
    assert!(s.consumables.item(ConsumableSlot::FoodOne).is_none());
    assert_eq!(s.survival.snapshot().hunger, Points::whole(50).unwrap());
    assert!(s.survival.snapshot().hunger > hunger);
    assert!(!s.mining.input.held);
    assert!(!s.wheel.open);
}

#[test]
fn session_fixed_tick_partition_and_save_boundary_match_at_starvation_death() {
    let mut one = initialize(e2e::Scenario::CoastingSpace);
    meters(&mut one, 0, 100);
    one.survival
        .command(VitalCommand::Damage(Points::whole(99).unwrap()))
        .unwrap();
    let mut split = GameplaySession::default();
    split.restore_bytes(&one.save_bytes(100).unwrap()).unwrap();
    one.advance_automation(Duration::from_millis(10_000));
    for milliseconds in [3333, 17, 1499, 5151] {
        split.advance_automation(Duration::from_millis(milliseconds));
        let bytes = split.save_bytes(100).unwrap();
        split.restore_bytes(&bytes).unwrap();
    }
    assert!(one.survival.snapshot().terminal);
    assert_eq!(one.survival.snapshot().active_time, Duration::from_secs(10));
    assert_eq!(value(&one), value(&split));
    assert_eq!(
        one.ended_run_bytes().unwrap(),
        split.ended_run_bytes().unwrap()
    );
    let terminal = value(&one);
    one.advance_automation(Duration::from_secs(86400));
    assert_eq!(value(&one), terminal);
}

#[test]
#[ignore = "extended 18-hour frame partition matrix; nightly portable verification"]
fn persistence_extended_eighteen_hour_partition_matrix() {
    for frame in [
        Duration::from_millis(16),
        Duration::from_millis(33),
        Duration::from_millis(100),
    ] {
        let mut split = Survival::default();
        let mut whole = split.clone();
        for _ in 0..72 {
            for owner in [&mut split, &mut whole] {
                owner.command(VitalCommand::Feed(Points::FULL)).unwrap();
            }
            domain_advance(&mut split, Duration::from_secs(900), frame);
            whole.advance(Duration::from_secs(900));
            assert_eq!(split, whole);
        }
        assert_eq!(split.snapshot().fatigue, Points::ZERO);
    }
}
