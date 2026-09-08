use super::*;
use crate::{EnergyConfig, Propulsion, ThrottleDirection};
fn ship(throttle: i8, velocity: [f64; 3]) -> ShipController {
    let mut s = ShipController::flying_with_velocity(
        ShipPose {
            position_meters: [0.0; 3],
            orientation: [0.0, 0.0, 0.0, 1.0],
        },
        throttle,
        velocity,
    )
    .unwrap();
    s.configure_energy_core(
        EnergyConfig {
            gravity_idle_watts: 0.0,
            cockpit_idle_watts: 0.0,
            ..Default::default()
        },
        45e9,
    )
    .unwrap();
    s
}
#[test]
fn exact_examples_independent_of_starting_velocity_with_mass_and_axis_changes() {
    for velocity in [0.0, 100_000.0] {
        let mut s = ship(100, [velocity, 0.0, 0.0]);
        let before = s.energy_core.stored_joules();
        s.advance(Duration::from_secs(10));
        assert_eq!(s.velocity, [velocity + 10_000.0, 0.0, 0.0]);
        assert_eq!(before - s.energy_core.stored_joules(), 1e9);
    }
    let mut s = ship(-100, [0.0; 3]);
    s.advance(Duration::from_secs(1));
    assert_eq!(s.velocity, [-500.0, 0.0, 0.0]);
    let mut s = ship(100, [0.0; 3]);
    s.set_total_mass_kg(200_000.0).unwrap();
    s.advance(Duration::from_secs(1));
    assert_eq!(s.velocity, [500.0, 0.0, 0.0]);
    s.pose.orientation = [
        0.0,
        0.0,
        std::f64::consts::FRAC_1_SQRT_2,
        std::f64::consts::FRAC_1_SQRT_2,
    ];
    s.advance(Duration::from_secs(1));
    assert!((s.velocity[0] - 500.0).abs() < 1e-10 && (s.velocity[1] - 500.0).abs() < 1e-10);
    s.set_propulsion(Propulsion::new(200_000.0, 2.0).unwrap());
    s.advance(Duration::from_secs(1));
    assert!((s.velocity[1] - 1500.0).abs() < 1e-10);
}
#[test]
fn partial_shared_funding_and_conversion_do_not_mint_service_or_assist_impulse() {
    let mut whole = ship(100, [0.0; 3]);
    whole
        .configure_energy_core(
            EnergyConfig {
                forward_watts: 100.0,
                reverse_watts: 50.0,
                gravity_idle_watts: 25.0,
                cockpit_idle_watts: 25.0,
                ..Default::default()
            },
            100.0,
        )
        .unwrap();
    whole.set_fabricator_watts(50.0).unwrap(); // 200W shared; 0.5 seconds funded.
    let mut split = whole.clone();
    whole.advance(Duration::from_secs(1));
    for ms in [123, 377, 500] {
        split.advance(Duration::from_millis(ms));
    }
    assert_eq!(whole.velocity, [0.0005, 0.0, 0.0]); // Only 50J propulsion /100t.
    assert_eq!(whole.velocity, split.velocity);
    assert!((whole.pose.position_meters[0] - split.pose.position_meters[0]).abs() < 1e-15);
    whole.advance(Duration::from_secs(1));
    assert_eq!(whole.velocity, split.velocity);
    whole
        .advance_with_core_input(Duration::from_secs(1), 200.0)
        .unwrap();
    assert_eq!(whole.velocity, [0.0015, 0.0, 0.0]);
    let mut idle = ship(0, [10.0, 0.0, 0.0]);
    idle.advance(Duration::from_secs(1));
    assert_eq!(idle.velocity, [10.0, 0.0, 0.0]);
}
#[test]
fn held_throttle_bills_each_repeat_at_its_actual_boundary() {
    let mut whole = ship(0, [0.0; 3]);
    whole.set_throttle_key(ThrottleDirection::Up, true);
    let mut split = whole.clone();
    let energy = whole.energy_core.stored_joules();
    whole.advance(Duration::from_millis(300));
    for ms in [13, 87, 41, 59, 100] {
        split.advance(Duration::from_millis(ms));
    }
    // 1%, 2%, 3% each for 0.1 seconds; selection reaches 4% at the endpoint.
    assert_eq!(energy - whole.energy_core.stored_joules(), 600_000.0);
    assert_eq!(whole.thruster_percentage, 4);
    assert!((whole.velocity[0] - 6.0).abs() < 1e-12);
    assert_eq!(
        whole.energy_core.stored_joules(),
        split.energy_core.stored_joules()
    );
    assert!((whole.velocity[0] - split.velocity[0]).abs() < 1e-12);
}
#[test]
fn engine_switch_is_distinct_durable_and_not_reenabled_by_power_restore() {
    let mut s = ship(100, [42.0, 0.0, 0.0]);
    assert!(s.toggle_engines());
    assert!(!s.snapshot().engines_enabled);
    s.advance(Duration::from_secs(1));
    assert_eq!(s.velocity, [42.0, 0.0, 0.0]);
    assert_eq!(
        s.snapshot()
            .navigation
            .acceleration_meters_per_second_squared,
        Some(0.0)
    );
    assert_eq!(
        s.snapshot().navigation.braking,
        crate::BrakingEstimate::Unavailable
    );
    let saved = s.save_state();
    let mut restored = ShipController::from_save(saved).unwrap();
    restored.set_stored_core_energy_joules(0);
    restored.set_stored_core_energy_joules(1_000_000_000);
    restored.advance(Duration::from_secs(1));
    assert_eq!(restored.velocity, s.velocity);
    assert!(restored.toggle_engines());
    restored.advance(Duration::from_secs(1));
    assert_eq!(restored.velocity[0], 1042.0);
    let zero = ship(0, [0.0; 3]);
    assert!(zero.snapshot().engines_enabled);
    let mut landed = ShipController::default();
    landed.set_cockpit_control(true);
    assert!(landed.toggle_engines());
    let before = landed.energy_core.stored_joules();
    landed.trigger_landing_action();
    assert!(matches!(landed.flight_state, FlightState::Landed { .. }));
    assert_eq!(before, landed.energy_core.stored_joules());
    assert!(landed.toggle_engines());
    landed.trigger_landing_action();
    assert!(landed.assist.is_some());
    assert!(!landed.toggle_engines()); // Cannot cancel a committed assist.
}
