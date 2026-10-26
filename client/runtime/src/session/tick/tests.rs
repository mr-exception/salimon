use super::*;
use crate::e2e::{self, Config, Scenario};
use crate::session::{Command, ControlKey};

// Captured from parent 8c13f72 before changing state composition or tick stages.
const PARENT_TICK: &[u8] = include_bytes!("../../../fixtures/tick-schema23-construction.json");

fn construction() -> GameplaySession {
    let mut session = GameplaySession::default();
    e2e::initialize(
        &mut session,
        Config {
            scenario: Scenario::ConstructionBuild,
            seed: 642,
            step: Duration::from_millis(100),
            fragment_load: None,
        },
    )
    .unwrap();
    session
}

#[test]
fn ordered_stages_and_partitioned_admission_match_parent_construction_save() {
    let mut whole = construction();
    whole.advance_automation(Duration::from_secs(12));
    assert_eq!(whole.save_bytes(100).unwrap(), PARENT_TICK);
    let mut split = construction();
    for _ in 0..120 {
        split.advance_automation(Duration::from_millis(100));
    }
    assert_eq!(split.save_bytes(100).unwrap(), PARENT_TICK);
}

#[test]
fn restore_preserves_host_policy_and_clears_held_input() {
    let mut session = GameplaySession::new_run();
    session
        .policy
        .set_automation_step(Some(Duration::from_millis(32)));
    session.policy.set_interpolation(false);
    session.policy.set_physics_measurement(true);
    let policy = session.policy;
    session.command(Command::Control(ControlKey::Forward, true));
    assert!(session.input.movement().forward);
    let bytes = include_bytes!("../../../fixtures/save-schema23-new-run.json");
    session.restore_for_launch(bytes, 100).unwrap();
    assert_eq!(session.policy, policy);
    assert_eq!(session.e2e_step(), Some(Duration::from_millis(32)));
    assert!(!session.input.movement().forward);
    assert_eq!(
        session.input.steering(),
        crate::session::ShipControlInput::default()
    );
    // Launch legitimately reconciles offline metadata. A different host policy
    // must produce the same authoritative save as an ordinary launch.
    let mut ordinary = GameplaySession::default();
    ordinary.restore_for_launch(bytes, 100).unwrap();
    assert_eq!(
        session.save_bytes(100).unwrap(),
        ordinary.save_bytes(100).unwrap()
    );
}
