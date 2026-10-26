use super::*;

#[test]
fn modal_priority_clears_movement_and_construction_cancel_does_not_pause() {
    let mut session = GameplaySession::new_run();
    dispatch(&mut session, Key::Forward, true);
    assert!(session.movement_input.forward);
    dispatch(&mut session, Key::Wheel, true);
    assert!(session.wheel.open);
    assert!(!session.movement_input.forward);
    dispatch(&mut session, Key::Build, true);
    assert!(!session.building.active);
    dispatch(&mut session, Key::Wheel, false);
    assert!(!session.wheel.open);
    dispatch(&mut session, Key::Build, true);
    assert!(session.building.active);
    dispatch(&mut session, Key::Cancel, true);
    assert!(!session.building.active);
    assert!(!session.is_paused());
    dispatch(&mut session, Key::Cancel, true);
    assert!(session.is_paused());
    dispatch(&mut session, Key::Build, true);
    assert!(!session.building.active);
    dispatch(&mut session, Key::Confirm, true);
    assert!(!session.is_paused());
    assert!(!session.movement_input.forward);
}
