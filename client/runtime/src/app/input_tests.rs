//! Portable production dispatch/lifecycle regressions; OS event delivery is #312.
use super::*;
use crate::session::ShipControlInput;
use salimon_ship::FlightState;

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
    let mut session = crate::session::GameplaySession::default();
    session.ship = ShipController::flying(
        salimon_ship::ShipPose {
            position_meters: [1e12, 0.0, 0.0],
            orientation: [0.0, 0.0, 0.0, 1.0],
        },
        0,
    );
    let mut app = ClientApplication {
        session,
        native_focused: true,
        cursor_captured: true,
        ..ClientApplication::default()
    };
    app.session.character.enter_cockpit();
    app.session.ship.set_cockpit_control(true);
    app
}

#[test]
fn escape_then_w_e_l_and_throttle_do_not_mutate_gameplay_until_recapture() {
    let mut app = pilot();
    native(&mut app, KeyCode::KeyW, true);
    native(&mut app, KeyCode::ArrowUp, true);
    app.session.advance_game(Duration::from_millis(16));
    app.release_input_capture(); // Same reset invoked by real initial Escape.
    let before = app.session.ship.snapshot();
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
    assert_eq!(app.session.ship.snapshot(), before);
    assert_eq!(app.session.character.location(), CharacterLocation::Cockpit);
    assert_eq!(app.session.ship_control_input, ShipControlInput::default());
    assert!(!app.session.movement_input.forward);
    app.cursor_captured = true;
    native(&mut app, KeyCode::ArrowUp, true);
    assert_eq!(
        app.session.ship.snapshot().thruster_percentage,
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
    assert!(walker.session.movement_input.forward);
    native(&mut walker, KeyCode::Digit1, true);
    walker.session.mining_mouse_input(true);
    assert!(walker.session.mining.input.held);
    assert!(walker.session.mining.press_f());
    native(&mut walker, KeyCode::F2, true);
    assert!(!walker.session.movement_input.forward);
    assert!(!walker.session.mining.input.held);
    assert!(walker.session.mining.press_f(), "view reset clears F latch");
    native(&mut walker, KeyCode::KeyF, false);
    native(&mut walker, KeyCode::KeyW, false);
    native(&mut walker, KeyCode::F2, true);
    assert!(!walker.session.movement_input.forward);

    let mut app = pilot();
    native(&mut app, KeyCode::KeyW, true);
    native(&mut app, KeyCode::ArrowUp, true);
    app.session.advance_game(Duration::from_millis(16));
    let selected = app.session.ship.snapshot().thruster_percentage;
    native(&mut app, KeyCode::F2, true);
    let pose = app.session.ship.snapshot().pose;
    for key in [KeyCode::KeyW, KeyCode::ArrowUp] {
        native(&mut app, key, false);
    }
    app.e2e_config = Some(e2e::Config {
        scenario: e2e::Scenario::CoreRestart,
        seed: 0,
        step: Duration::from_millis(16),
        fragment_load: None,
    });
    app.session.automation_step = app.e2e_step();
    // Tour actions and protocol action aliases cannot activate gameplay.
    let before = app.session.ship.snapshot();
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
    assert_eq!(app.session.ship.snapshot(), before);
    native(&mut app, KeyCode::F2, true);
    app.session.advance_game(Duration::from_millis(160));
    assert_eq!(
        app.session.ship.snapshot().pose.orientation,
        pose.orientation
    );
    assert_eq!(app.session.ship.snapshot().thruster_percentage, selected);
    assert_eq!(app.session.ship_control_input, ShipControlInput::default());
}

#[test]
fn lifecycle_resets_and_focus_replay_cannot_reinstall_held_controls() {
    let mut app = pilot();
    for transition in 0..3 {
        native(&mut app, KeyCode::KeyW, true);
        native(&mut app, KeyCode::ArrowUp, true);
        app.session.advance_game(Duration::from_millis(16));
        let selected = app.session.ship.snapshot().thruster_percentage;
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
        let pose = app.session.ship.snapshot().pose;
        app.session.advance_game(Duration::from_millis(160));
        assert_eq!(
            app.session.ship.snapshot().pose.orientation,
            pose.orientation
        );
        assert_eq!(app.session.ship.snapshot().thruster_percentage, selected);
        assert_eq!(app.session.view_mode, ViewMode::Gameplay);
        assert_eq!(app.session.ship_control_input, ShipControlInput::default());
    }
}

#[test]
fn automation_bypasses_desktop_capture_but_preserves_power_and_assist_authority() {
    let mut app = pilot();
    app.release_input_capture();
    app.set_input_focus(false);
    app.automation_key(PhysicalKey::Code(KeyCode::ArrowUp), true);
    assert_eq!(app.session.ship.snapshot().thruster_percentage, 1);
    app.session.ship.set_stored_core_energy_joules(0);
    app.automation_key(PhysicalKey::Code(KeyCode::ArrowUp), true);
    app.automation_key(PhysicalKey::Code(KeyCode::KeyW), true);
    assert_eq!(app.session.ship.snapshot().thruster_percentage, 1);
    assert_eq!(app.session.ship_control_input, ShipControlInput::default());
    app.session.ship = ShipController::default();
    app.session.ship.set_cockpit_control(true);
    app.automation_key(PhysicalKey::Code(KeyCode::KeyL), true);
    assert!(matches!(
        app.session.ship.snapshot().flight_state,
        FlightState::AssistedTakeoff { .. }
    ));
    app.automation_key(PhysicalKey::Code(KeyCode::KeyW), true);
    app.automation_key(PhysicalKey::Code(KeyCode::ArrowUp), true);
    assert_eq!(app.session.ship.snapshot().thruster_percentage, 0);
    assert_eq!(app.session.ship_control_input, ShipControlInput::default());
}

#[test]
fn native_protocol_adds_only_shell_observations_and_expires_session_feedback() {
    let config = e2e::Config {
        scenario: e2e::Scenario::CoreRestart,
        seed: 309,
        step: Duration::from_millis(16),
        fragment_load: None,
    };
    let mut app = ClientApplication::default();
    e2e::initialize(&mut app.session, config).unwrap();
    app.e2e_config = Some(config);
    let portable = automation::inspect_session(&app.session);
    let inspect = automation::execute(&mut app, r#"{"protocol":1,"id":1,"op":"inspect"}"#);
    assert_eq!(inspect["ok"], true);
    for key in [
        "ship",
        "player",
        "world",
        "mining",
        "equipment",
        "view_mode",
    ] {
        assert_eq!(inspect["result"][key], portable[key], "{key}");
    }
    assert!(inspect["result"]["window"].is_null());
    assert!(inspect["result"]["terrain"]["gpu_ready"].is_boolean());
    assert_eq!(
        automation::execute(&mut app, r#"{"protocol":1,"id":2,"op":"interact"}"#)["ok"],
        true
    );
    assert!(app.action_bar.has_transient());
    assert!(
        app.session.feedback.is_none(),
        "the shell consumes feedback once"
    );
    assert_eq!(
        automation::execute(
            &mut app,
            r#"{"protocol":1,"id":3,"op":"step","frames":200}"#
        )["ok"],
        true
    );
    assert!(
        !app.action_bar.has_transient(),
        "expiry uses completed automation ticks"
    );
}

#[test]
fn wheel_released_pointer_still_admits_q_release_and_focus_loss_cancels_without_replay() {
    let mut app = pilot();
    native(&mut app, KeyCode::KeyW, true);
    native(&mut app, KeyCode::KeyQ, true);
    assert!(app.session.wheel.open);
    assert_eq!(app.session.ship_control_input, ShipControlInput::default());
    // The native pointer is released while the wheel has keyboard focus.
    app.cursor_captured = false;
    native(&mut app, KeyCode::KeyQ, false);
    assert!(!app.session.wheel.open);
    app.cursor_captured = true;
    native(&mut app, KeyCode::KeyQ, true);
    app.set_input_focus(false);
    assert!(!app.session.wheel.open);
    assert!(!app.session.wheel.held);
    native(&mut app, KeyCode::Enter, true);
    assert!(!app.session.wheel.open);
    app.set_input_focus(true);
    native(&mut app, KeyCode::KeyQ, true); // No capture: no synthetic reopen.
    assert!(!app.session.wheel.open);
}
