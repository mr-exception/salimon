use salimon_persistence::Validate;
use salimon_survival::{Command, Config, Points, Survival, effects::*};
use std::time::Duration;
fn sec(n: u64) -> Duration {
    Duration::from_secs(n)
}
fn p(n: u32) -> Points {
    Points::whole(n).unwrap()
}
fn effect(id: u16, behavior: Behavior, lifetime: Lifetime, stacks: u16) -> EffectRule {
    let interval = (!matches!(behavior, Behavior::Amplify { .. })).then_some(sec(1));
    EffectRule {
        id: EffectId(id),
        behavior,
        lifetime,
        interval,
        max_stacks: stacks,
        removal: Removal::Dispellable,
    }
}
fn damage(source: Source, n: u32, wound: bool) -> Damage {
    Damage {
        source,
        amount: p(n),
        wound,
    }
}
fn dot(source: Source, n: u32) -> Behavior {
    Behavior::Damage {
        source,
        amount: p(n),
        wound: false,
    }
}
fn state(effects: Vec<EffectRule>) -> Survival {
    Survival::with_rules(
        Config::default(),
        Rules {
            effects,
            ..Default::default()
        },
    )
    .unwrap()
}
#[test]
fn regen_exact_sixty_seconds_thresholds_and_fractional_save_continuation() {
    let mut s = Survival::default();
    s.damage(damage(Source::Fall, 20, false)).unwrap();
    s.advance(sec(59));
    assert_eq!(s.snapshot().health, p(80));
    let mut restored: Survival = serde_json::from_value(serde_json::to_value(&s).unwrap()).unwrap();
    restored.validate().unwrap();
    s.advance(sec(1));
    restored.advance(sec(1));
    assert_eq!(s, restored);
    assert_eq!(s.snapshot().health, p(81));
    s.advance(sec(480)); // Hunger exactly 70; ninth regen tick is included.
    assert_eq!(s.snapshot().hunger, p(70));
    assert_eq!(s.snapshot().health, p(89));
    s.advance(sec(61));
    assert_eq!(s.snapshot().health, p(89));
    // Fatigue below 50 independently blocks regen with hunger restored.
    for _ in 0..36 {
        s.command(Command::Feed(Points::FULL)).unwrap();
        s.advance(sec(900));
    }
    s.command(Command::Feed(Points::FULL)).unwrap();
    s.damage(damage(Source::Fall, 10, false)).unwrap();
    let before = s.snapshot().health;
    s.advance(sec(60));
    assert_eq!(s.snapshot().health, before);
}
#[test]
fn wounds_block_only_the_affected_debt_until_treatment() {
    let mut s = Survival::default();
    s.damage(damage(Source::Fall, 20, true)).unwrap();
    s.damage(damage(Source::Environment, 10, false)).unwrap();
    s.heal(Heal {
        amount: Points::FULL,
        mode: Healing::Ordinary,
    })
    .unwrap();
    assert_eq!(s.snapshot().health, p(80));
    assert_eq!(s.snapshot().wounds, p(20));
    s.advance(sec(60));
    assert_eq!(s.snapshot().health, p(80));
    s.heal(Heal {
        amount: p(5),
        mode: Healing::Treatment,
    })
    .unwrap();
    assert_eq!(s.snapshot().health, p(85));
    assert_eq!(s.snapshot().wounds, p(15));
    s.validate().unwrap();
}
#[test]
fn deterministic_modifier_order_source_specific_amplification_and_cap() {
    let mut rules = Rules::default();
    let source = rules
        .sources
        .iter_mut()
        .find(|r| r.source == Source::Fall)
        .unwrap();
    source.defense = Defense {
        flat: p(2),
        resistance_bps: 5000,
    };
    source.amplification_bps = 5000;
    source.damage_cap = p(10);
    rules.effects.push(effect(
        1,
        Behavior::Amplify {
            source: Source::Fall,
            basis_points: 5000,
        },
        Lifetime::Persistent,
        2,
    ));
    let mut s = Survival::with_rules(Config::default(), rules).unwrap();
    s.set_armor(Defense {
        flat: p(2),
        resistance_bps: 5000,
    })
    .unwrap();
    s.apply_effect(EffectId(1)).unwrap();
    // (20-2-2) * .5 * .5 * (1+.5+.5) = 8.
    s.damage(damage(Source::Fall, 20, false)).unwrap();
    assert_eq!(s.snapshot().health, p(92));
    s.apply_effect(EffectId(1)).unwrap(); // 4 * 2.5 = 10 cap
    s.damage(damage(Source::Fall, 100, false)).unwrap();
    assert_eq!(s.snapshot().health, p(82));
    s.damage(damage(Source::Radiation, 20, false)).unwrap(); // armor only =>9
    assert_eq!(s.snapshot().health, p(73));
}
#[test]
fn exposure_radiation_stops_immediately_burn_lingers_and_offline_freezes_all_effects() {
    let mut s = state(vec![
        effect(
            1,
            dot(Source::Radiation, 1),
            Lifetime::Exposure {
                linger: Duration::ZERO,
            },
            1,
        ),
        effect(
            2,
            dot(Source::Burning, 2),
            Lifetime::Exposure { linger: sec(3) },
            1,
        ),
    ]);
    for id in [1, 2] {
        s.set_exposure(EffectId(id), true).unwrap();
        s.set_exposure(EffectId(id), true).unwrap();
    }
    assert!(s.active_effects().all(|(_, stacks, _)| stacks == 1));
    s.advance(sec(1));
    assert_eq!(s.snapshot().health, p(97));
    for id in [1, 2] {
        s.set_exposure(EffectId(id), false).unwrap();
    }
    s.advance(sec(1));
    assert_eq!(s.snapshot().health, p(95));
    let before = serde_json::to_value(&s).unwrap();
    s.recover_offline(1000, 99999);
    let after = serde_json::to_value(&s).unwrap();
    assert_eq!(before["pipeline"], after["pipeline"]);
    s.advance(sec(10));
    assert_eq!(s.snapshot().health, p(91));
    assert_eq!(s.active_effects().count(), 0);
}
#[test]
fn timed_stacking_refresh_cap_dispel_and_treatment_only_rules() {
    let mut r = effect(1, dot(Source::Burning, 1), Lifetime::Timed(sec(3)), 2);
    r.removal = Removal::TreatmentOnly;
    let mut s = state(vec![r]);
    s.apply_effect(EffectId(1)).unwrap();
    s.advance(sec(1));
    for _ in 0..10 {
        s.apply_effect(EffectId(1)).unwrap();
    }
    assert_eq!(s.active_effects().next().unwrap().1, 2);
    assert!(!s.remove_effect(EffectId(1), false).unwrap());
    let mut split = s.clone();
    s.advance(sec(3));
    for _ in 0..3000 {
        split.advance(Duration::from_millis(1));
    }
    assert_eq!(s, split);
    assert_eq!(s.snapshot().health, p(93));
    assert_eq!(s.active_effects().count(), 0);
    s.apply_effect(EffectId(1)).unwrap();
    assert!(s.remove_effect(EffectId(1), true).unwrap());
}
#[test]
fn lethal_timestamp_damage_precedes_heals_and_cause_uses_explicit_priority() {
    let mut s = state(vec![
        effect(1, dot(Source::Burning, 100), Lifetime::Persistent, 1),
        effect(2, dot(Source::Radiation, 100), Lifetime::Persistent, 1),
        effect(
            3,
            Behavior::Heal {
                amount: p(100),
                mode: Healing::Treatment,
            },
            Lifetime::Persistent,
            1,
        ),
    ]);
    for id in [3, 2, 1] {
        s.apply_effect(EffectId(id)).unwrap();
    }
    s.advance(sec(100));
    assert!(s.snapshot().terminal);
    assert_eq!(s.snapshot().active_time, sec(1));
    assert_eq!(s.snapshot().death_cause, Some(Source::Burning));
    let dead = s.clone();
    s.advance(sec(100));
    s.set_exposure(EffectId(999), false).unwrap_err();
    assert_eq!(s, dead);
    s.validate().unwrap();
}
#[test]
fn starvation_and_periodic_heal_compete_with_stable_damage_first_order() {
    let mut r = effect(
        1,
        Behavior::Heal {
            amount: p(1),
            mode: Healing::Ordinary,
        },
        Lifetime::Persistent,
        1,
    );
    r.interval = Some(sec(10));
    let mut s = state(vec![r]);
    s.advance(sec(1800));
    s.damage(damage(Source::Fall, 99, false)).unwrap();
    s.apply_effect(EffectId(1)).unwrap();
    s.advance(sec(10));
    assert!(s.snapshot().terminal);
    assert_eq!(s.snapshot().death_cause, Some(Source::Starvation));
}
#[test]
fn registry_validation_custom_sources_and_corrupt_status_rejection() {
    let mut rules = Rules::default();
    rules
        .sources
        .push(SourceRule::unmodified(Source::Registered(7), 900));
    let mut s = Survival::with_rules(Config::default(), rules.clone()).unwrap();
    s.damage(damage(Source::Registered(7), 100, false)).unwrap();
    assert_eq!(s.snapshot().death_cause, Some(Source::Registered(7)));
    rules.sources.push(rules.sources[0].clone());
    assert!(rules.validate().is_err());
    let mut r = effect(1, dot(Source::Fall, 1), Lifetime::Timed(sec(1)), 1);
    r.interval = Some(Duration::ZERO);
    assert!(
        Rules {
            effects: vec![r],
            ..Default::default()
        }
        .validate()
        .is_err()
    );
    let mut s = state(vec![effect(
        1,
        dot(Source::Fall, 1),
        Lifetime::Timed(sec(3)),
        1,
    )]);
    s.apply_effect(EffectId(1)).unwrap();
    let mut json = serde_json::to_value(s).unwrap();
    json["pipeline"]["active"]["1"]["stacks"] = serde_json::json!(1000);
    assert!(
        serde_json::from_value::<Survival>(json)
            .unwrap()
            .validate()
            .is_err()
    );
}
