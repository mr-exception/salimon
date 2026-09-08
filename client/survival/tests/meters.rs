use salimon_persistence::Validate;
use salimon_survival::{Command, Config, OfflineResult, Points, Survival};
use std::time::Duration;
fn seconds(n: u64) -> Duration {
    Duration::from_secs(n)
}
fn points(n: u32) -> Points {
    Points::whole(n).unwrap()
}

#[test]
fn fresh_run_and_exact_hunger_starvation_death_boundaries() {
    let mut s = Survival::default();
    assert_eq!(s.snapshot().hunger, Points::FULL);
    assert_eq!(s.snapshot().fatigue, Points::FULL);
    assert_eq!(s.snapshot().health, Points::FULL);
    s.advance(seconds(1800) - Duration::from_nanos(1));
    assert!(s.snapshot().hunger > Points::ZERO);
    s.advance(Duration::from_nanos(1));
    assert_eq!(s.snapshot().hunger, Points::ZERO);
    assert_eq!(s.snapshot().health, Points::FULL);
    s.advance(seconds(10) - Duration::from_nanos(1));
    assert_eq!(s.snapshot().health, Points::FULL);
    s.advance(Duration::from_nanos(1));
    assert_eq!(s.snapshot().health, points(99));
    s.advance(Duration::MAX);
    assert_eq!(s.snapshot().active_time, seconds(2800));
    assert!(s.snapshot().terminal);
    let dead = s.snapshot();
    s.advance(seconds(100));
    s.command(Command::Heal(Points::FULL)).unwrap();
    s.command(Command::Feed(Points::FULL)).unwrap();
    assert_eq!(dead, s.snapshot());
    s.validate().unwrap();
    assert_eq!(Survival::default().snapshot().health, Points::FULL);
}
#[test]
fn fatigue_exact_eighteen_hours_with_regular_food_and_zero_is_not_death() {
    let mut s = Survival::default();
    for _ in 0..72 {
        s.command(Command::Feed(Points::FULL)).unwrap();
        s.advance(seconds(900));
    }
    assert_eq!(s.snapshot().fatigue, Points::ZERO);
    assert!(!s.snapshot().terminal);
    assert_eq!(s.snapshot().active_time, seconds(64800));
}
#[test]
fn partition_invariance_including_subnanopoint_drain_and_partial_starvation() {
    let mut whole = Survival::default();
    let mut split = whole.clone();
    whole.advance(seconds(1817) + Duration::from_nanos(13));
    for _ in 0..1817 {
        split.advance(seconds(1));
    }
    for _ in 0..13 {
        split.advance(Duration::from_nanos(1));
    }
    assert_eq!(whole, split);
    whole.command(Command::Feed(points(1))).unwrap();
    split.command(Command::Feed(points(1))).unwrap();
    whole.advance(seconds(29));
    for _ in 0..29 {
        split.advance(seconds(1));
    }
    assert_eq!(whole, split);
}
#[test]
fn offline_fatigue_only_once_capped_and_rollback_watermark_survives_reload() {
    let mut s = Survival::default();
    s.advance(seconds(1000));
    let before = s.snapshot();
    assert_eq!(
        s.recover_offline(1000, 1216),
        OfflineResult::Applied(seconds(216))
    );
    let after = s.snapshot();
    assert_eq!(after.fatigue.value(), before.fatigue.value() + 1.);
    assert_eq!(after.hunger, before.hunger);
    assert_eq!(after.health, before.health);
    assert_eq!(after.active_time, before.active_time);
    assert_eq!(
        s.recover_offline(1000, 99999),
        OfflineResult::AlreadyApplied
    );
    let mut loaded: Survival = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    loaded.validate().unwrap();
    assert_eq!(
        loaded.recover_offline(1000, 1210),
        OfflineResult::ClockRollback
    );
    let mut loaded: Survival =
        serde_json::from_slice(&serde_json::to_vec(&loaded).unwrap()).unwrap();
    assert_eq!(
        loaded.recover_offline(1000, 1216),
        OfflineResult::Applied(Duration::ZERO)
    );
    let mut loaded: Survival =
        serde_json::from_slice(&serde_json::to_vec(&loaded).unwrap()).unwrap();
    assert_eq!(
        loaded.recover_offline(1216, u64::MAX),
        OfflineResult::Applied(seconds(21600))
    );
    assert_eq!(loaded.snapshot().fatigue, Points::FULL);
}
#[test]
fn saved_fraction_continues_exactly_and_invalid_state_is_rejected() {
    let mut s = Survival::default();
    s.advance(seconds(1805) + Duration::from_nanos(7));
    let mut loaded: Survival = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    loaded.validate().unwrap();
    s.advance(seconds(5));
    loaded.advance(seconds(5));
    assert_eq!(s, loaded);
    let mut json = serde_json::to_value(&s).unwrap();
    json["health"] = serde_json::json!(u64::MAX);
    assert!(
        serde_json::from_value::<Survival>(json)
            .unwrap()
            .validate()
            .is_err()
    );
    assert!(Points::whole(101).is_err());
    assert!(Points::milli(u32::MAX).is_err());
    let config = Config {
        hunger_full_seconds: 0,
        ..Default::default()
    };
    assert!(Survival::new(config).is_err());
    let config = Config {
        hunger_full_seconds: 7,
        ..Default::default()
    };
    assert!(Survival::new(config).is_err());
}
#[test]
fn zero_elapsed_and_dead_offline_do_not_change_state() {
    let mut s = Survival::default();
    let fresh = s.clone();
    s.advance(Duration::ZERO);
    assert_eq!(fresh, s);
    s.command(Command::Damage(Points::FULL)).unwrap();
    let dead = s.snapshot();
    assert_eq!(s.recover_offline(0, 1000), OfflineResult::Terminal);
    assert_eq!(dead, s.snapshot());
}
