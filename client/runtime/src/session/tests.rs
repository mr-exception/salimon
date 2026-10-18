use super::*;
use crate::e2e::{self, Config, Scenario};
use serde_json::json;

fn initialized(scenario: Scenario) -> GameplaySession {
    let mut session = GameplaySession::default();
    e2e::initialize(
        &mut session,
        Config {
            scenario,
            seed: 309,
            step: Duration::from_millis(1003),
            fragment_load: None,
        },
    )
    .unwrap();
    session
}

#[test]
fn typed_and_protocol_commands_share_blackout_intake_power_and_backlog() {
    let mut typed = initialized(Scenario::CoreRestart);
    let mut protocol = initialized(Scenario::CoreRestart);
    assert!(!typed.snapshot().ship.power_available);
    assert!(typed.mining.session.carried_id().is_some());
    typed.command(Command::Interact);
    let response = crate::automation::execute_session(
        &mut protocol,
        &json!({"protocol":1,"id":1,"op":"interact"}).to_string(),
    );
    assert_eq!(response["ok"], true);
    assert!(typed.mining.session.fragments().is_empty());
    assert_eq!(
        typed.snapshot().ship.total_mass_kg,
        salimon_ship::ShipMass::default().total_kg()
    );
    assert!(
        typed
            .snapshot()
            .ship
            .core_status
            .conversion_remaining_joules
            > 0.0
    );
    for ms in [7, 201, 1, 394, 400] {
        typed.advance_automation(Duration::from_millis(ms));
    }
    let response = crate::automation::execute_session(
        &mut protocol,
        &json!({"protocol":1,"id":2,"op":"step","frames":1}).to_string(),
    );
    assert_eq!(response["ok"], true);
    let a = typed.snapshot();
    let b = protocol.snapshot();
    assert_eq!(a.ship, b.ship);
    assert_eq!(a.character, b.character);
    assert_eq!(a.backlog, Duration::from_millis(11));
    assert_eq!(a.backlog, b.backlog);
    assert!(a.ship.power_available);
    assert!(!typed.mining.ship_gravity_disabled);
    assert_eq!(
        typed.mining.session.fragments(),
        protocol.mining.session.fragments()
    );
}

#[test]
fn typed_controls_clear_on_view_reset_and_never_replay_denied_steering() {
    let mut session = initialized(Scenario::FlightRegression);
    session.command(Command::Control(ControlKey::Forward, true));
    session.command(Command::Control(ControlKey::Left, true));
    session.command(Command::Control(ControlKey::RollRight, true));
    let steering = session.ship_control_input.steering();
    assert_eq!(
        (steering.pitch, steering.yaw, steering.roll),
        (1.0, 1.0, 1.0)
    );
    session.command(Command::Control(ControlKey::Left, false));
    session.command(Command::Control(ControlKey::Right, true));
    assert_eq!(session.ship_control_input.steering().yaw, -1.0);
    session.ship.set_stored_core_energy_joules(0);
    session.command(Command::Control(ControlKey::Forward, true));
    assert_eq!(session.ship_control_input, ShipControlInput::default());
    session.ship.exchange_core_energy_joules(1e9, 0.0).unwrap();
    session.advance_automation(Duration::from_millis(16));
    assert_eq!(session.ship_control_input, ShipControlInput::default());
    session.command(Command::Throttle(salimon_ship::ThrottleDirection::Up, true));
    let throttle = session.ship.snapshot().thruster_percentage;
    session.command(Command::ToggleView);
    session.command(Command::Control(ControlKey::Forward, true));
    session.command(Command::Landing);
    assert_eq!(session.ship.snapshot().flight_state, FlightState::Flying);
    session.command(Command::Control(ControlKey::Forward, false));
    session.command(Command::ToggleView);
    session.advance_automation(Duration::from_millis(160));
    assert_eq!(session.ship_control_input, ShipControlInput::default());
    assert_eq!(session.ship.snapshot().thruster_percentage, throttle);
}

#[test]
fn benchmark_configuration_advances_the_production_ship_character_and_cargo() {
    let mut session = GameplaySession::default();
    let config = crate::benchmark::parse(&[
        "--benchmark".into(),
        "--mode".into(),
        "physics".into(),
        "--count".into(),
        "0".into(),
    ])
    .unwrap();
    crate::benchmark::initialize(&mut session, config).unwrap();
    session.ship = ShipController::flying_with_velocity(
        salimon_ship::ShipPose {
            position_meters: [1e12, 0.0, 0.0],
            orientation: [0.0, 0.0, 0.0, 1.0],
        },
        0,
        [40.0, 0.0, 0.0],
    )
    .unwrap();
    let before = session.snapshot();
    session.advance_automation(Duration::from_millis(16));
    let after = session.snapshot();
    assert!(after.ship.pose.position_meters[0] > before.ship.pose.position_meters[0]);
    assert_eq!(session.elapsed, Duration::from_millis(16));
    assert_eq!(
        after.ship.total_mass_kg,
        salimon_ship::ShipMass::default().total_kg()
    );
    assert!(
        after
            .character
            .eye_position_meters
            .iter()
            .all(|value| value.is_finite())
    );
}
