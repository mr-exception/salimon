use super::*;
use salimon_ship::{FlightState, ShipController, ShipPose};
use salimon_world::terrain::TerrainMode;
use serde_json::Value;
fn flying() -> GameplaySession {
    let mut s = GameplaySession::new_canonical_run().unwrap();
    let body = salimon_world::canonical::BODIES[0];
    let mut ship = ShipController::flying_with_velocity(
        ShipPose {
            position_meters: [0., body.radius_meters + 500_000., 0.],
            orientation: [0., 0., 0., 1.],
        },
        75,
        [200., 0., 0.],
    )
    .unwrap();
    ship.configure_terrain(TerrainMode::Canonical, crate::terrain::landing_footprint());
    s.ship = ship;
    s.sync_fabricator_mass();
    s.bind_scanner_clock(100).unwrap();
    s
}
#[test]
fn schema_29_migration_preserves_geography_resources_and_body_relative_motion() {
    let s = flying();
    let before = s.snapshot().ship;
    let mut old: Value = serde_json::from_slice(&s.save_bytes(100).unwrap()).unwrap();
    old["schema_version"] = 29.into();
    old["state"].as_object_mut().unwrap().remove("motion");
    let mut loaded = GameplaySession::default();
    loaded
        .restore_bytes(&serde_json::to_vec(&old).unwrap())
        .unwrap();
    assert_eq!(loaded.snapshot().ship, before);
    assert_eq!(
        loaded.motion.as_ref().unwrap(),
        &salimon_universe::starting_motion::Clock::at(100).unwrap()
    );
    let current: Value = serde_json::from_slice(&loaded.save_bytes(100).unwrap()).unwrap();
    assert_eq!(current["state"]["physical"], old["state"]["physical"]);
    assert_eq!(current["schema_version"], 30);
    let bytes = loaded.save_bytes(100).unwrap();
    for path in ["/state/motion/version", "/state/motion/definition_sha256"] {
        let mut bad = current.clone();
        *bad.pointer_mut(path).unwrap() = serde_json::json!("invalid");
        assert!(
            loaded
                .restore_bytes(&serde_json::to_vec(&bad).unwrap())
                .is_err()
        );
        assert_eq!(loaded.save_bytes(100).unwrap(), bytes);
    }
    let mut missing = current;
    missing["state"].as_object_mut().unwrap().remove("motion");
    assert!(
        loaded
            .restore_bytes(&serde_json::to_vec(&missing).unwrap())
            .is_err()
    );
    assert_eq!(loaded.save_bytes(100).unwrap(), bytes);
}
#[test]
fn launch_shuts_engines_retains_inertia_and_applies_verified_gravity() {
    let s = flying();
    let before = s.snapshot().ship;
    let bytes = s.save_bytes(100).unwrap();
    let mut loaded = GameplaySession::default();
    loaded.restore_for_launch(&bytes, 102).unwrap();
    let after = loaded.snapshot().ship;
    assert!(!after.engines_enabled);
    assert_eq!(after.thruster_percentage, 0);
    assert_eq!(after.pose.orientation, before.pose.orientation);
    assert!((after.pose.position_meters[0] - 400.).abs() < 0.1);
    assert!(after.velocity_meters_per_second[1] < -0.1);
    assert_eq!(
        loaded.motion.as_ref().unwrap(),
        &s.motion
            .as_ref()
            .unwrap()
            .advanced(Duration::from_secs(2))
            .unwrap()
    );
    let credited = loaded.save_bytes(102).unwrap();
    loaded.restore_for_launch(&credited, 102).unwrap();
    assert_eq!(loaded.save_bytes(102).unwrap(), credited);
}
#[test]
fn long_grounded_launch_follows_orbit_without_moving_body_local_geography() {
    let mut s = GameplaySession::new_canonical_run().unwrap();
    s.bind_scanner_clock(100).unwrap();
    s.ship.exchange_core_energy_joules(0., 1e30).unwrap();
    let before = s.snapshot().ship;
    let original = s.motion.as_ref().unwrap().clone();
    let bytes = s.save_bytes(100).unwrap();
    s.restore_for_launch(&bytes, 100 + 365 * 86400).unwrap();
    assert_eq!(s.snapshot().ship.pose, before.pose);
    assert_eq!(s.snapshot().ship.flight_state, before.flight_state);
    assert_eq!(
        s.motion.as_ref().unwrap(),
        &original.advanced(Duration::from_secs(365 * 86400)).unwrap()
    );
    assert_ne!(
        salimon_universe::starting_motion::planet(original.epoch)
            .unwrap()
            .position_meters,
        salimon_universe::starting_motion::planet(s.motion.as_ref().unwrap().epoch)
            .unwrap()
            .position_meters
    );
}
#[test]
fn closed_descent_uses_actual_terrain_contact_and_never_grants_landing_or_thrust() {
    let mut s = flying();
    let body = salimon_world::canonical::BODIES[0];
    let mut ship = ShipController::flying_with_velocity(
        ShipPose {
            position_meters: [0., body.radius_meters + 100., 0.],
            orientation: [0., 0., 0., 1.],
        },
        100,
        [0., -100., 0.],
    )
    .unwrap();
    ship.configure_terrain(TerrainMode::Canonical, crate::terrain::landing_footprint());
    s.ship = ship;
    let bytes = s.save_bytes(100).unwrap();
    s.restore_for_launch(&bytes, 110).unwrap();
    assert!(matches!(
        s.ship.snapshot().flight_state,
        FlightState::GroundedAfterCrash { .. }
    ));
    assert!(s.ship.snapshot().pose.position_meters[1] >= body.radius_meters + 15.99);
    assert!(!s.ship.snapshot().engines_enabled);
    assert_eq!(s.ship.snapshot().thruster_percentage, 0);
}
