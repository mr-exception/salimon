//! Portable production dispatch/lifecycle regressions; OS event delivery is #312.
use super::*;

fn native(app: &mut ClientApplication, key: KeyCode, pressed: bool) {
    app.dispatch_key(
        PhysicalKey::Code(key),
        pressed,
        false,
        false,
        InputSource::Native,
    );
}
fn pilot() -> ClientApplication {
    let mut app = ClientApplication {
        ship: ShipController::flying(
            salimon_ship::ShipPose {
                position_meters: [1e12, 0.0, 0.0],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
        ),
        native_focused: true,
        cursor_captured: true,
        ..ClientApplication::default()
    };
    app.character.enter_cockpit();
    app.ship.set_cockpit_control(true);
    app
}

#[test]
fn escape_then_w_e_l_and_throttle_do_not_mutate_gameplay_until_recapture() {
    let mut app = pilot();
    native(&mut app, KeyCode::KeyW, true);
    native(&mut app, KeyCode::ArrowUp, true);
    app.advance_game(Duration::from_millis(16));
    app.release_input_capture(); // Same reset invoked by real initial Escape.
    let before = app.ship.snapshot();
    for key in [
        KeyCode::KeyW,
        KeyCode::KeyE,
        KeyCode::KeyL,
        KeyCode::KeyF,
        KeyCode::Digit1,
        KeyCode::ArrowUp,
    ] {
        native(&mut app, key, true);
    }
    assert_eq!(app.ship.snapshot(), before);
    assert_eq!(app.character.location(), CharacterLocation::Cockpit);
    assert_eq!(app.ship_control_input, ShipControlInput::default());
    assert!(!app.movement_input.forward);
    app.cursor_captured = true;
    native(&mut app, KeyCode::ArrowUp, true);
    assert_eq!(
        app.ship.snapshot().thruster_percentage,
        before.thruster_percentage + 1
    );
}

#[test]
fn view_switch_clears_walking_steering_mining_and_throttle_without_replay() {
    let mut walker = ClientApplication {
        native_focused: true,
        cursor_captured: true,
        ..ClientApplication::default()
    };
    native(&mut walker, KeyCode::KeyW, true);
    assert!(walker.movement_input.forward);
    native(&mut walker, KeyCode::Digit1, true);
    walker.mining_mouse_input(true);
    assert!(walker.mining.held);
    assert!(walker.mining.press_f());
    native(&mut walker, KeyCode::F2, true);
    assert!(!walker.movement_input.forward);
    assert!(!walker.mining.held);
    assert!(walker.mining.press_f(), "view reset clears F latch");
    native(&mut walker, KeyCode::KeyF, false);
    native(&mut walker, KeyCode::KeyW, false);
    native(&mut walker, KeyCode::F2, true);
    assert!(!walker.movement_input.forward);

    let mut app = pilot();
    native(&mut app, KeyCode::KeyW, true);
    native(&mut app, KeyCode::ArrowUp, true);
    app.advance_game(Duration::from_millis(16));
    let selected = app.ship.snapshot().thruster_percentage;
    native(&mut app, KeyCode::F2, true);
    let pose = app.ship.snapshot().pose;
    for key in [KeyCode::KeyW, KeyCode::ArrowUp] {
        native(&mut app, key, false);
    }
    app.e2e_config = Some(e2e::Config {
        scenario: e2e::Scenario::CoreRestart,
        seed: 0,
        step: Duration::from_millis(16),
        fragment_load: None,
    });
    // Tour actions and protocol action aliases cannot activate gameplay.
    let before = app.ship.snapshot();
    for value in [
        serde_json::json!({"op":"interact"}),
        serde_json::json!({"op":"landing"}),
    ] {
        let mut value = value;
        value["protocol"] = 1.into();
        value["id"] = 1.into();
        let result = automation::execute(&mut app, &value.to_string());
        assert_eq!(result["result"]["applied"], false, "{result}");
    }
    assert_eq!(app.ship.snapshot(), before);
    native(&mut app, KeyCode::F2, true);
    app.advance_game(Duration::from_millis(160));
    assert_eq!(app.ship.snapshot().pose.orientation, pose.orientation);
    assert_eq!(app.ship.snapshot().thruster_percentage, selected);
    assert_eq!(app.ship_control_input, ShipControlInput::default());
}

#[test]
fn lifecycle_resets_and_focus_replay_cannot_reinstall_held_controls() {
    let mut app = pilot();
    for transition in 0..3 {
        native(&mut app, KeyCode::KeyW, true);
        native(&mut app, KeyCode::ArrowUp, true);
        app.advance_game(Duration::from_millis(16));
        let selected = app.ship.snapshot().thruster_percentage;
        match transition {
            0 => {
                app.set_input_focus(false);
                app.set_input_focus(true);
            }
            1 => {
                app.occluded = true;
                app.reset_update_clock();
                app.occluded = false;
            }
            _ => {
                app.set_input_focus(false);
                app.reset_update_clock();
                app.set_input_focus(true);
            }
        }
        app.cursor_captured = true;
        for key in [
            KeyCode::KeyW,
            KeyCode::ArrowUp,
            KeyCode::KeyE,
            KeyCode::KeyL,
            KeyCode::F2,
        ] {
            app.dispatch_key(
                PhysicalKey::Code(key),
                true,
                false,
                true,
                InputSource::Native,
            );
            app.dispatch_key(
                PhysicalKey::Code(key),
                true,
                true,
                false,
                InputSource::Native,
            );
            native(&mut app, key, false);
        }
        let pose = app.ship.snapshot().pose;
        app.advance_game(Duration::from_millis(160));
        assert_eq!(app.ship.snapshot().pose.orientation, pose.orientation);
        assert_eq!(app.ship.snapshot().thruster_percentage, selected);
        assert_eq!(app.view_mode, ViewMode::Gameplay);
        assert_eq!(app.ship_control_input, ShipControlInput::default());
    }
}

#[test]
fn automation_bypasses_desktop_capture_but_preserves_power_and_assist_authority() {
    let mut app = pilot();
    app.release_input_capture();
    app.set_input_focus(false);
    app.automation_key(PhysicalKey::Code(KeyCode::ArrowUp), true);
    assert_eq!(app.ship.snapshot().thruster_percentage, 1);
    app.ship.set_stored_core_energy_joules(0);
    app.automation_key(PhysicalKey::Code(KeyCode::ArrowUp), true);
    app.automation_key(PhysicalKey::Code(KeyCode::KeyW), true);
    assert_eq!(app.ship.snapshot().thruster_percentage, 1);
    assert_eq!(app.ship_control_input, ShipControlInput::default());
    app.ship = ShipController::default();
    app.ship.set_cockpit_control(true);
    app.automation_key(PhysicalKey::Code(KeyCode::KeyL), true);
    assert!(matches!(
        app.ship.snapshot().flight_state,
        FlightState::AssistedTakeoff { .. }
    ));
    app.automation_key(PhysicalKey::Code(KeyCode::KeyW), true);
    app.automation_key(PhysicalKey::Code(KeyCode::ArrowUp), true);
    assert_eq!(app.ship.snapshot().thruster_percentage, 0);
    assert_eq!(app.ship_control_input, ShipControlInput::default());
}
