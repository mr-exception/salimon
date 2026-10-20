use super::*;
use crate::session::{GameplaySession, ViewMode};
use std::time::Duration;
fn character(world: [f64; 3], local: Option<[f64; 3]>) -> CharacterSnapshot {
    CharacterSnapshot {
        location: if local.is_some() {
            CharacterLocation::InsideShip
        } else {
            CharacterLocation::Space
        },
        eye_position_meters: world,
        local_ship_position_meters: local,
        look_target_meters: world,
        up: [0., 1., 0.],
        doorway_blend_fraction: None,
    }
}
fn flying() -> GameplaySession {
    GameplaySession {
        ship: salimon_ship::ShipController::flying_with_velocity(
            salimon_ship::ShipPose {
                position_meters: [0.; 3],
                orientation: [0., 0., 0., 1.],
            },
            0,
            [100., 0., 0.],
        )
        .unwrap(),
        ..Default::default()
    }
}
#[test]
fn local_cabin_motion_adds_to_ship_once_at_solar_coordinates() {
    let mut j = JourneyState::default();
    let p = salimon_world::SURFACE_ANCHOR.meters();
    let q = [p[0] + 100., p[1], p[2]];
    j.record(
        p,
        q,
        character(q, Some([0.; 3])),
        character(q, Some([0.00001, 0., 0.])),
        true,
    );
    assert!((j.distance_meters - 100.00001).abs() < 1e-10);
    // Ship rotation/translation of an idle passenger adds zero character path.
    j.record(
        p,
        q,
        character(q, Some([1., 0., 0.])),
        character(p, Some([1., 0., 0.])),
        true,
    );
    assert!((j.distance_meters - 200.00001).abs() < 1e-10);
}
#[test]
fn doorway_frame_crossings_and_eva_have_no_coordinate_jump() {
    let mut j = JourneyState::default();
    let p = salimon_world::SURFACE_ANCHOR.meters();
    let q = [p[0] + 3., p[1] + 4., p[2]];
    j.record(
        p,
        p,
        character(p, Some([1., 0., 0.])),
        character(q, None),
        true,
    );
    assert_eq!(j.distance_meters, 5.);
    j.record(
        p,
        p,
        character(q, None),
        character(p, Some([1., 0., 0.])),
        true,
    );
    assert_eq!(j.distance_meters, 10.);
    j.record(p, q, character(p, None), character(q, None), true);
    assert_eq!(j.distance_meters, 20.); // separate ship and EVA paths
}
#[test]
fn invalid_and_inactive_samples_cannot_poison_counter() {
    let mut j = JourneyState::default();
    let c = character([0.; 3], None);
    j.record([0.; 3], [10., 0., 0.], c, c, false);
    j.record([0.; 3], [f64::INFINITY, 0., 0.], c, c, true);
    assert_eq!(j.distance_meters, 0.);
    assert!(j.validate().is_ok());
    j.tracking.as_mut().unwrap().last_ship_position_meters = Some([f64::NAN; 3]);
    assert!(j.validate().is_err());
}
#[test]
fn flight_roundtrip_offline_teleport_and_tour_preserve_only_real_travel() {
    let mut s = flying();
    s.advance_game(Duration::from_millis(16));
    let initial = s.run.journey.distance_meters;
    assert!((initial - 1.6).abs() < 0.001);
    let bytes = s.save_bytes(100).unwrap();
    let mut restored = GameplaySession::default();
    restored.restore_for_launch(&bytes, 10000).unwrap();
    assert_eq!(restored.run.journey.distance_meters, initial);
    assert_eq!(
        restored.survival.snapshot().active_time,
        Duration::from_millis(16)
    );
    restored.advance_game(Duration::from_millis(16));
    s.advance_game(Duration::from_millis(16));
    assert_eq!(
        restored.run.journey.distance_meters,
        s.run.journey.distance_meters
    );
    // Replacing a pose outside simulation is a teleport, never a travelled leg.
    restored.ship = salimon_ship::ShipController::default();
    restored.view_mode = ViewMode::PrecisionTour;
    let d = restored.run.journey.distance_meters;
    restored.advance_game(Duration::from_secs(1));
    assert_eq!(restored.run.journey.distance_meters, d);
}
#[test]
fn v2_migration_preserves_counter_and_v3_requires_sampling_source_transactionally() {
    let mut s = GameplaySession::default();
    s.run.journey.distance_meters = 42.;
    let mut v: serde_json::Value = serde_json::from_slice(&s.save_bytes(1).unwrap()).unwrap();
    v["state"]["survival"]["run"]["journey"]
        .as_object_mut()
        .unwrap()
        .remove("tracking");
    let original = s.save_bytes(1).unwrap();
    assert!(s.restore_bytes(&serde_json::to_vec(&v).unwrap()).is_err());
    assert_eq!(s.save_bytes(1).unwrap(), original);
    v["schema_version"] = serde_json::json!(2);
    s.restore_bytes(&serde_json::to_vec(&v).unwrap()).unwrap();
    assert_eq!(s.run.journey.distance_meters, 42.);
    assert!(s.run.journey.tracking.is_some());
    s.advance_game(Duration::from_millis(16));
    assert_eq!(s.run.journey.distance_meters, 42.);
}
#[test]
fn archive_contains_named_final_location_exact_duration_and_typed_cause() {
    let mut s = GameplaySession::default();
    s.advance_game(Duration::from_millis(16));
    s.survival
        .command(salimon_survival::Command::Damage(
            salimon_survival::Points::FULL,
        ))
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&s.ended_run_bytes().unwrap()).unwrap();
    assert_eq!(report["schema_version"], 2);
    assert_eq!(report["final_location"]["nearest_body"], "Earth");
    assert_eq!(report["final_location"]["name"], "Near Earth");
    assert_eq!(report["active_duration"]["nanos"], 16000000);
    assert_eq!(
        report["cause"],
        serde_json::to_value(s.survival.snapshot().death_cause.unwrap()).unwrap()
    );
    let fresh = s.fresh_after_death().unwrap();
    assert_eq!(fresh.run.journey.distance_meters, 0.);
    let space = NamedLocation::at([0.; 3]);
    assert!(space.name.starts_with("Space near "));
    for b in CELESTIAL_BODIES {
        let p = b.center.translated([0., b.radius_meters + 2., 0.]).meters();
        assert_eq!(NamedLocation::at(p).nearest_body, b.id);
    }
}
#[test]
fn takeoff_assist_counts_its_path_without_passenger_double_counting() {
    let mut s = GameplaySession::default();
    s.ship.set_cockpit_control(true);
    s.ship.trigger_landing_action();
    let start = s.ship.snapshot().pose.position_meters;
    for _ in 0..100 {
        s.advance_game(Duration::from_millis(16));
    }
    let travelled = distance(start, s.ship.snapshot().pose.position_meters);
    assert!(travelled > 0.);
    assert!((s.run.journey.distance_meters - travelled).abs() < 0.01);
}

#[test]
fn real_cabin_walking_adds_only_controller_path_and_restored_samples_remain_exact() {
    let mut s = GameplaySession::default();
    let start = s.snapshot().character.local_ship_position_meters.unwrap();
    s.movement_input.forward = true;
    for _ in 0..5 {
        s.advance_game(Duration::from_millis(16));
    }
    let end = s.snapshot().character.local_ship_position_meters.unwrap();
    assert!(s.run.journey.distance_meters > 0.);
    assert!((s.run.journey.distance_meters - distance(start, end)).abs() < 1e-9);
    let source = s.save_bytes(1).unwrap();
    let mut restored = GameplaySession::default();
    restored.restore_bytes(&source).unwrap();
    assert_eq!(restored.save_bytes(1).unwrap(), source);
}
#[test]
fn real_eva_drift_is_separate_from_ship_transport() {
    let mut s = flying();
    let mut c = serde_json::to_value(s.character.save_state()).unwrap();
    c["position"] = serde_json::json!({"Space":{"world":[100.,100.,100.]}});
    c["eva_inherited_velocity"] = serde_json::json!([10., 0., 0.]);
    s.character =
        salimon_character::CharacterController::from_save(serde_json::from_value(c).unwrap(), None)
            .unwrap();
    for _ in 0..10 {
        s.advance_game(Duration::from_millis(16));
    }
    assert_eq!(s.snapshot().character.location, CharacterLocation::Space);
    assert!((s.run.journey.distance_meters - 17.6).abs() < 1e-9);
}
