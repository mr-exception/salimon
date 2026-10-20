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
fn cursor_release_then_w_e_l_and_throttle_do_not_mutate_gameplay_until_recapture() {
    let mut app = pilot();
    native(&mut app, KeyCode::KeyW, true);
    native(&mut app, KeyCode::ArrowUp, true);
    app.session.advance_game(Duration::from_millis(16));
    app.release_input_capture(); // Same reset invoked by F1 cursor release.
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
fn journey_ended_restart_uses_verified_archive_and_retries_failure_without_resurrection() {
    use crate::checkpoints::Checkpoints;
    let root = std::env::temp_dir().join(format!("salimon-374-restart-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut app = ClientApplication::default();
    app.checkpoints = Some(Checkpoints::open_at(root.clone(), &mut app.session, 100).unwrap());
    app.session
        .survival
        .command(salimon_survival::Command::Damage(
            salimon_survival::Points::FULL,
        ))
        .unwrap();
    let before = app.session.save_bytes(1).unwrap();
    let serial = app.session.run.serial;
    // Stale/unfocused/occluded/synthetic/repeated presses cannot restart.
    native(&mut app, KeyCode::Enter, true);
    assert_eq!(app.session.run.serial, serial);
    app.native_focused = true;
    app.occluded = true;
    native(&mut app, KeyCode::Enter, true);
    app.occluded = false;
    for (repeat, synthetic, pressed) in [
        (true, false, true),
        (false, true, true),
        (false, false, false),
    ] {
        app.dispatch_key(
            PhysicalKey::Code(KeyCode::Enter),
            pressed,
            repeat,
            synthetic,
            InputSource::Native,
        );
        assert_eq!(app.session.run.serial, serial);
    }
    for key in [
        KeyCode::F2,
        KeyCode::F3,
        KeyCode::KeyH,
        KeyCode::KeyW,
        KeyCode::KeyE,
    ] {
        native(&mut app, key, true);
    }
    assert_eq!(app.session.save_bytes(1).unwrap(), before);
    let archive = root.join(format!("journeys/v2/run-{serial}.json"));
    std::fs::create_dir_all(archive.parent().unwrap()).unwrap();
    std::fs::write(&archive, b"conflict").unwrap();
    native(&mut app, KeyCode::Enter, true);
    assert!(app.session.survival.snapshot().terminal);
    assert_eq!(app.session.save_bytes(1).unwrap(), before);
    assert!(app.checkpoints.as_ref().unwrap().warning.is_some());
    std::fs::remove_file(&archive).unwrap();
    native(&mut app, KeyCode::NumpadEnter, true);
    assert!(!app.session.survival.snapshot().terminal);
    assert_eq!(app.session.run.serial, serial + 1);
    assert_eq!(
        app.session.survival.snapshot().hunger,
        salimon_survival::Points::FULL
    );
    assert_eq!(
        app.session.survival.snapshot().health,
        salimon_survival::Points::FULL
    );
    assert_eq!(
        app.session.survival.snapshot().fatigue,
        salimon_survival::Points::FULL
    );
    assert_eq!(app.session.run.journey.distance_meters, 0.);
    assert_eq!(app.session.update_clock.backlog(), Duration::ZERO);
    assert!(!app.cursor_captured);
    assert!(archive.exists());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&archive).unwrap()).unwrap();
    assert_eq!(report["serial"], serial);
    // A second activation in fresh gameplay cannot restart a living run.
    assert!(!app.restart_terminal());
    assert_eq!(app.session.run.serial, serial + 1);
    drop(app);
    let mut restored = crate::session::GameplaySession::default();
    let saves = Checkpoints::open_at(root.clone(), &mut restored, 100).unwrap();
    assert_eq!(restored.run.serial, serial + 1);
    assert!(!restored.survival.snapshot().terminal);
    drop(saves);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn journey_ended_terminal_relaunch_and_missing_storage_never_grant_unverified_gameplay() {
    use crate::checkpoints::Checkpoints;
    let root = std::env::temp_dir().join(format!("salimon-374-terminal-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut app = ClientApplication::default();
    let mut saves = Checkpoints::open_at(root.clone(), &mut app.session, 100).unwrap();
    app.session
        .survival
        .command(salimon_survival::Command::Damage(
            salimon_survival::Points::FULL,
        ))
        .unwrap();
    saves.finish(&app.session).unwrap();
    drop(saves);
    drop(app);
    let mut app = ClientApplication::default();
    let saves = Checkpoints::open_at(root.clone(), &mut app.session, 100).unwrap();
    assert!(app.session.survival.snapshot().terminal);
    let model = crate::journey_ended::Report::from(&app.session.ended_run().unwrap());
    assert!(
        app.journey_ended
            .image(Some(model), [640, 360], 1.)
            .is_some()
    );
    // Dropping the store models an unavailable restart backend, not permission to bypass it.
    drop(saves);
    let serial = app.session.run.serial;
    assert!(!app.restart_terminal());
    assert_eq!(app.session.run.serial, serial);
    assert!(app.session.survival.snapshot().terminal);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn pause_native_background_focus_and_protocol_share_freeze() {
    let mut app = pilot();
    native(&mut app, KeyCode::KeyW, true);
    native(&mut app, KeyCode::ArrowUp, true);
    native(&mut app, KeyCode::Escape, true);
    assert!(app.session.is_paused());
    let throttle = app.session.ship.snapshot().thruster_percentage;
    let frozen = app.session.save_bytes(123).unwrap();
    for key in [
        KeyCode::KeyW,
        KeyCode::KeyL,
        KeyCode::F2,
        KeyCode::KeyP,
        KeyCode::KeyE,
    ] {
        native(&mut app, key, true);
    }
    assert_eq!(
        app.advance_native(Duration::from_secs(7200)),
        (Duration::ZERO, 0)
    );
    app.set_input_focus(false);
    app.reset_update_clock();
    app.set_input_focus(true);
    assert!(app.session.is_paused());
    assert_eq!(app.session.save_bytes(123).unwrap(), frozen);
    // A focus-generated Escape must never resume.
    app.dispatch_key(
        PhysicalKey::Code(KeyCode::Escape),
        true,
        false,
        true,
        InputSource::Native,
    );
    assert!(app.session.is_paused());
    native(&mut app, KeyCode::Enter, true);
    assert!(!app.session.is_paused());
    assert_eq!(app.session.ship.snapshot().thruster_percentage, throttle);
    assert_eq!(app.session.ship_control_input, ShipControlInput::default());
}

#[test]
fn isolated_e2e_ignores_desktop_keys_and_preserves_protocol_holds_on_lifecycle_reset() {
    let mut app = pilot();
    app.e2e_input_isolated = true;
    native(&mut app, KeyCode::KeyW, true);
    assert_eq!(app.session.ship_control_input, ShipControlInput::default());
    app.automation_key(PhysicalKey::Code(KeyCode::KeyW), true);
    let scripted = app.session.ship_control_input;
    app.session
        .update_clock
        .accumulate(Duration::from_millis(5));
    assert_ne!(scripted, ShipControlInput::default());
    app.set_input_focus(false);
    app.reset_update_clock();
    assert_eq!(app.session.ship_control_input, scripted);
    assert_eq!(app.session.update_clock.backlog(), Duration::from_millis(5));
    app.automation_key(PhysicalKey::Code(KeyCode::KeyW), false);
    assert_eq!(app.session.ship_control_input, ShipControlInput::default());
    app.e2e_input_isolated = false;
    app.native_focused = true;
    app.cursor_captured = true;
    native(&mut app, KeyCode::KeyW, true);
    assert_ne!(app.session.ship_control_input, ShipControlInput::default());
    app.set_input_focus(false);
    assert_eq!(app.session.ship_control_input, ShipControlInput::default());
}
