use salimon_persistence::Validate;
use salimon_survival::{Command, Points, Survival, consumables::*, effects::*};
use std::time::Duration;
fn p(n: u32) -> Points {
    Points::whole(n).unwrap()
}
fn seconds(n: u64) -> Duration {
    Duration::from_secs(n)
}
fn wounded() -> Survival {
    let mut s = Survival::default();
    s.command(Command::Damage(p(60))).unwrap();
    s
}
fn roundtrip(s: &Survival) -> Survival {
    let result: Survival = serde_json::from_slice(&serde_json::to_vec(s).unwrap()).unwrap();
    result.validate().unwrap();
    result
}
#[test]
fn starter_ration_clamps_hunger_and_medkit_delivers_exact_bounded_gradual_healing() {
    let catalog = Catalog::starter();
    let mut s = wounded();
    s.apply_consumable(catalog.get("food-ration").unwrap())
        .unwrap();
    assert_eq!(s.snapshot().hunger, Points::FULL);
    s.apply_consumable(catalog.get("medkit").unwrap()).unwrap();
    assert_eq!(s.snapshot().health, p(40));
    s.advance(seconds(1));
    assert_eq!(s.snapshot().health.value(), 42.5);
    s.advance(seconds(9));
    assert_eq!(s.snapshot().health, p(65));
    assert_eq!(s.active_effects().count(), 0);
    s.advance(seconds(1));
    assert_eq!(s.snapshot().health, p(65));
    let mut full = Survival::default();
    full.apply_consumable(catalog.get("medkit").unwrap())
        .unwrap();
    full.advance(seconds(10));
    assert_eq!(full.snapshot().health, Points::FULL);
}
#[test]
fn active_treatment_and_food_resume_exact_fractional_intervals_and_freeze_offline() {
    let definition = Definition {
        item_id: "recovery-meal".into(),
        category: Category::Food,
        benefits: vec![
            Benefit::OverTime {
                id: EffectId(100),
                recovery: Recovery::Hunger,
                milli_points: 1000,
                interval: seconds(1),
                duration: seconds(4),
                max_stacks: 1,
            },
            Benefit::OverTime {
                id: EffectId(101),
                recovery: Recovery::Health(Healing::Treatment),
                milli_points: 2000,
                interval: seconds(1),
                duration: seconds(4),
                max_stacks: 1,
            },
        ],
    };
    let mut s = wounded();
    s.damage(Damage {
        source: Source::Fall,
        amount: p(10),
        wound: true,
    })
    .unwrap();
    s.advance(seconds(900)); // Lower hunger; ignore base regen when measuring continuation.
    s.apply_consumable(&definition).unwrap();
    s.advance(Duration::from_millis(1375));
    let mut restored = roundtrip(&s);
    let mut offline = restored.clone();
    offline.recover_offline(100, 10_000);
    assert_eq!(offline.snapshot().hunger, restored.snapshot().hunger);
    assert_eq!(offline.snapshot().health, restored.snapshot().health);
    assert_eq!(
        offline.active_effects().collect::<Vec<_>>(),
        restored.active_effects().collect::<Vec<_>>()
    );
    let final_time = Duration::from_millis(2625);
    s.advance(final_time);
    for ms in [1, 19, 500, 1000, 1105] {
        restored.advance(Duration::from_millis(ms));
    }
    assert_eq!(s, restored);
    assert_eq!(s.snapshot().wounds, p(2)); // Four 2-point treatment ticks, no ordinary wound recovery.
    assert_eq!(s.active_effects().count(), 0);
}
#[test]
fn registry_conflicts_invalid_and_terminal_consumption_are_transactional() {
    let mut s = wounded();
    let catalog = Catalog::starter();
    let kit = catalog.get("medkit").unwrap();
    s.apply_consumable(kit).unwrap();
    let before = s.clone();
    let mut conflict = kit.clone();
    conflict.benefits.insert(
        0,
        Benefit::Instant {
            recovery: Recovery::Hunger,
            milli_points: 1000,
        },
    );
    if let Benefit::OverTime { milli_points, .. } = &mut conflict.benefits[1] {
        *milli_points = 3000;
    }
    assert!(s.apply_consumable(&conflict).is_err());
    assert_eq!(s, before);
    let mut bad = kit.clone();
    if let Benefit::OverTime { interval, .. } = &mut bad.benefits[0] {
        *interval = Duration::ZERO;
    }
    assert!(bad.validate().is_err());
    assert!(s.apply_consumable(&bad).is_err());
    assert_eq!(s, before);
    assert!(Catalog::new(vec![kit.clone(), kit.clone()]).is_err());
    let mut other = kit.clone();
    other.item_id = "other-kit".into();
    assert!(Catalog::new(vec![kit.clone(), other]).is_err());
    s.command(Command::Damage(Points::FULL)).unwrap();
    let dead = s.clone();
    assert!(
        s.apply_consumable(catalog.get("food-ration").unwrap())
            .is_err()
    );
    assert_eq!(s, dead);
}
#[test]
fn repeated_medkits_refresh_but_do_not_exceed_configured_stacks_or_replay_offline_ticks() {
    let mut s = wounded();
    let kit = Catalog::starter().get("medkit").unwrap().clone();
    s.apply_consumable(&kit).unwrap();
    s.advance(Duration::from_millis(500));
    s.apply_consumable(&kit).unwrap();
    assert_eq!(s.active_effects().next().unwrap().1, 1);
    s.advance(Duration::from_millis(500));
    assert_eq!(s.snapshot().health.value(), 42.5);
    let before = s.clone();
    s.recover_offline(0, 21_600);
    assert_eq!(s.snapshot().health, before.snapshot().health);
    assert_eq!(
        s.active_effects().collect::<Vec<_>>(),
        before.active_effects().collect::<Vec<_>>()
    );
}
#[test]
fn lethal_starvation_tick_precedes_periodic_food_and_healing_at_same_timestamp() {
    let mut s = Survival::default();
    let mut value = serde_json::to_value(&s).unwrap();
    value["hunger"] = serde_json::to_value(Points::ZERO).unwrap();
    s = serde_json::from_value(value).unwrap();
    s.command(Command::Damage(p(99))).unwrap();
    let d = Definition {
        item_id: "late-meal".into(),
        category: Category::Food,
        benefits: vec![Benefit::OverTime {
            id: EffectId(55),
            recovery: Recovery::Hunger,
            milli_points: 40_000,
            interval: seconds(10),
            duration: seconds(10),
            max_stacks: 1,
        }],
    };
    s.apply_consumable(&d).unwrap();
    s.advance(seconds(10));
    assert!(s.snapshot().terminal);
    assert_eq!(s.snapshot().hunger, Points::ZERO);
    assert_eq!(s.snapshot().death_cause, Some(Source::Starvation));
}
