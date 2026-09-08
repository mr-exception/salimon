use salimon_persistence::Validate;
use salimon_survival::{
    Points, Survival,
    restrictions::{Action, RestrictionConfig, Restrictions},
};
use std::time::Duration;

fn rules(hunger: u32, fatigue: u32) -> Restrictions {
    let mut snapshot = Survival::default().snapshot();
    snapshot.hunger = Points::milli(hunger).unwrap();
    snapshot.fatigue = Points::milli(fatigue).unwrap();
    Restrictions::new(snapshot, RestrictionConfig::default()).unwrap()
}

#[test]
fn fractional_boundaries_are_exact_and_penalties_multiply() {
    for (hunger, expected) in [
        (0, 7500),
        (19_999, 7500),
        (20_000, 9000),
        (49_999, 9000),
        (50_000, 10_000),
        (100_000, 10_000),
    ] {
        assert_eq!(rules(hunger, 100_000).movement_bps, expected);
    }
    for (fatigue, expected) in [
        (0, 3000),
        (1, 6500),
        (20_000, 6500),
        (20_001, 8500),
        (50_000, 8500),
        (50_001, 10_000),
        (100_000, 10_000),
    ] {
        assert_eq!(rules(100_000, fatigue).movement_bps, expected);
    }
    assert_eq!(rules(30_000, 30_000).movement_bps, 7650);
    assert_eq!(rules(0, 0).movement_bps, 2250);
    assert_eq!(rules(19_999, 20_000).interaction_bps, 5625);
}

#[test]
fn exhaustion_keeps_emergency_controls_and_terminal_allows_only_menus() {
    let rules = rules(0, 0);
    for action in [Action::Walk, Action::Consume, Action::Door, Action::Menu] {
        assert!(rules.permits(action));
    }
    for action in [
        Action::Pilot,
        Action::Mine,
        Action::Fabricate,
        Action::Carry,
        Action::FeedCore,
    ] {
        assert!(!rules.permits(action));
    }
    let mut snapshot = Survival::default().snapshot();
    snapshot.terminal = true;
    let terminal = Restrictions::new(snapshot, RestrictionConfig::default()).unwrap();
    assert!(!terminal.permits(Action::Consume));
    assert!(!terminal.permits(Action::Walk));
    assert!(terminal.permits(Action::Menu));
}

#[test]
fn tuning_is_validated_and_minimum_does_not_change_component_percentages() {
    let config = RestrictionConfig {
        minimum_movement_bps: 4000,
        low_hunger_interaction_bps: 5000,
        low_fatigue_interaction_bps: 8000,
        interaction_base: Duration::from_millis(100),
    };
    let mut snapshot = Survival::default().snapshot();
    snapshot.hunger = Points::ZERO;
    snapshot.fatigue = Points::ZERO;
    let r = Restrictions::new(snapshot, config).unwrap();
    assert_eq!(
        (
            r.hunger_movement_bps,
            r.fatigue_movement_bps,
            r.movement_bps
        ),
        (7500, 3000, 4000)
    );
    assert_eq!(r.interaction_bps, 4000);
    assert_eq!(r.interaction_delay(), Duration::from_millis(150));
    for value in [0, 10_001, u32::MAX] {
        assert!(
            RestrictionConfig {
                minimum_movement_bps: value,
                ..config
            }
            .validate()
            .is_err()
        );
        assert!(
            RestrictionConfig {
                low_hunger_interaction_bps: value,
                ..config
            }
            .validate()
            .is_err()
        );
        assert!(
            RestrictionConfig {
                low_fatigue_interaction_bps: value,
                ..config
            }
            .validate()
            .is_err()
        );
    }
    assert_eq!(rules(100_000, 100_000).interaction_delay(), Duration::ZERO);
}
