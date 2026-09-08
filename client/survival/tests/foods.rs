use salimon_persistence::Validate;
use salimon_survival::{Command, Points, Survival, consumables::*, effects::*, restrictions::*};
use std::time::Duration;
fn seconds(n: u64) -> Duration {
    Duration::from_secs(n)
}
fn meters(hunger: u32, fatigue: u32) -> Survival {
    let mut value = serde_json::to_value(Survival::default()).unwrap();
    value["hunger"] = serde_json::to_value(Points::whole(hunger).unwrap()).unwrap();
    value["fatigue"] = serde_json::to_value(Points::whole(fatigue).unwrap()).unwrap();
    let s: Survival = serde_json::from_value(value).unwrap();
    s.validate().unwrap();
    s
}
fn eat(s: &mut Survival, key: &str) {
    s.apply_consumable(Catalog::starter().get(key).unwrap())
        .unwrap();
}
fn restored(s: &Survival) -> Survival {
    let next: Survival = serde_json::from_slice(&serde_json::to_vec(s).unwrap()).unwrap();
    next.validate().unwrap();
    next
}
#[test]
fn gel_halves_only_active_drain_and_expires_exactly_after_ten_minutes() {
    let mut s = meters(30, 80);
    eat(&mut s, "nutrient-gel");
    assert_eq!(s.snapshot().hunger, Points::whole(50).unwrap());
    let mut baseline = meters(50, 80);
    s.advance(seconds(600));
    baseline.advance(seconds(300));
    assert_eq!(s.snapshot().fatigue, baseline.snapshot().fatigue);
    assert_eq!(s.modifier_bps(Modifier::FatigueDrain), 10_000);
    let mut split = meters(30, 80);
    eat(&mut split, "nutrient-gel");
    for ns in [1, 3, 999_999_999, 598_999_999_997] {
        split.advance(Duration::from_nanos(ns));
    }
    assert_eq!(s, split);
    s.advance(seconds(60));
    baseline.advance(seconds(60));
    assert_eq!(s.snapshot().fatigue, baseline.snapshot().fatigue);
}
#[test]
fn meal_feeds_exact_seventy_over_sixty_seconds_and_boosts_eligible_regen_only() {
    let mut s = meters(10, 80);
    eat(&mut s, "complete-meal");
    assert_eq!(s.snapshot().hunger, Points::whole(10).unwrap());
    s.advance(seconds(60));
    assert!((s.snapshot().hunger.value() - (80.0 - 60.0 / 18.0)).abs() < 1e-9);
    let mut s = meters(100, 80);
    s.command(Command::Damage(Points::whole(50).unwrap()))
        .unwrap();
    eat(&mut s, "complete-meal");
    s.advance(seconds(300));
    assert_eq!(s.snapshot().health, Points::whole(60).unwrap());
    assert_eq!(s.modifier_bps(Modifier::Regeneration), 10_000);
    s.advance(seconds(60));
    assert_eq!(s.snapshot().health, Points::whole(61).unwrap());
    let mut low = meters(0, 20);
    eat(&mut low, "complete-meal");
    low.command(Command::Damage(Points::whole(50).unwrap()))
        .unwrap();
    low.advance(seconds(60));
    assert_eq!(low.snapshot().health, Points::whole(50).unwrap());
}
#[test]
fn drink_relaxes_penalties_without_recharging_fatigue_or_bypassing_exhaustion() {
    let mut s = meters(80, 0);
    eat(&mut s, "energy-drink");
    assert_eq!(s.snapshot().fatigue, Points::ZERO);
    assert_eq!(s.snapshot().hunger, Points::whole(90).unwrap());
    let rules = Restrictions::with_fatigue_relief(
        s.snapshot(),
        RestrictionConfig::default(),
        s.modifier_bps(Modifier::FatigueRelief),
    )
    .unwrap();
    assert_eq!(rules.fatigue_movement_bps, 6500);
    assert_eq!(rules.interaction_bps, 8750);
    assert!(!rules.permits(Action::Pilot));
    assert!(rules.permits(Action::Consume));
    s.advance(seconds(300));
    assert_eq!(s.modifier_bps(Modifier::FatigueRelief), 0);
}
#[test]
fn all_food_timers_refresh_without_stacking_preserve_save_fraction_and_freeze_offline() {
    let mut s = meters(40, 80);
    for key in ["nutrient-gel", "complete-meal", "energy-drink"] {
        eat(&mut s, key);
    }
    s.advance(Duration::from_nanos(1));
    let mut loaded = restored(&s);
    loaded.recover_offline(100, 200);
    assert_eq!(
        loaded.active_effects().collect::<Vec<_>>(),
        s.active_effects().collect::<Vec<_>>()
    );
    assert_eq!(loaded.snapshot().hunger, s.snapshot().hunger);
    assert_eq!(loaded.snapshot().health, s.snapshot().health);
    // Compare exact continuation without the deliberately changed offline fatigue.
    let mut loaded = restored(&s);
    for key in ["nutrient-gel", "complete-meal", "energy-drink"] {
        eat(&mut s, key);
        eat(&mut loaded, key);
    }
    assert!(s.active_effects().all(|(_, stacks, _)| stacks == 1));
    s.advance(Duration::from_nanos(600_000_000_003));
    for ns in [1, 19, 1_000_000_000, 599_000_000_000 - 17] {
        loaded.advance(Duration::from_nanos(ns));
    }
    assert_eq!(s, loaded);
    s.validate().unwrap();
}
#[test]
fn malformed_buff_catalog_and_missing_old_remainder_are_handled() {
    let d = Catalog::starter().get("nutrient-gel").unwrap().clone();
    for (bps, duration) in [(10_001, seconds(600)), (5000, Duration::ZERO)] {
        let mut invalid = d.clone();
        if let Benefit::Modifier {
            basis_points,
            duration: time,
            ..
        } = &mut invalid.benefits[1]
        {
            *basis_points = bps;
            *time = duration;
        }
        let mut s = Survival::default();
        let before = s.clone();
        assert!(s.apply_consumable(&invalid).is_err());
        assert_eq!(s, before);
    }
    let mut legacy = serde_json::to_value(Survival::default()).unwrap();
    legacy.as_object_mut().unwrap().remove("fatigue_remainder");
    let s: Survival = serde_json::from_value(legacy).unwrap();
    s.validate().unwrap();
}
