use super::*;
use crate::session::{Command, construction::room_tests::room};
use std::time::Duration;

#[test]
fn construction_active_stop_resume_takeoff_and_exact_save() {
    let (mut s, id) = room();
    s.advance_construction(Duration::from_secs(150));
    let source = s.building.source.projects.inspect(id).unwrap().clone();
    s.ship.set_cockpit_control(true);
    let landed = s.ship.snapshot();
    s.command(Command::Landing);
    assert_eq!(s.ship.snapshot().flight_state, landed.flight_state);
    assert_eq!(s.ship.snapshot().energy_core, landed.energy_core);
    assert_eq!(
        s.contextual_cockpit_message(),
        Some(CockpitMessage::StopExteriorConstruction)
    );
    s.ship.set_cockpit_control(false);
    s.ship.toggle_door();
    s.ship.advance(Duration::from_secs(2));
    // Use the real reachable, aimed builder route, including its ownership gate.
    s.character
        .set_supported_pose([-6.6, SHIP_FLOOR_HEIGHT_METERS + 1.75, 0.]);
    s.command(Command::Look(std::f64::consts::PI / 0.0022, 0.));
    s.building.active = true;
    assert_eq!(s.aimed_construction(), Some(id));
    s.construction_action(BuildAction::StopResume);
    let p = s.building.source.projects.inspect(id).unwrap();
    assert!(p.work.stopped);
    assert_eq!(p.funding, source.funding);
    assert_eq!(p.work.elapsed_nanos, source.work.elapsed_nanos);
    assert_eq!(s.construction_takeoff_blocker(), None);
    let energy = s.ship.snapshot().energy_core;
    s.advance_construction(Duration::from_secs(60));
    assert_eq!(s.ship.snapshot().energy_core, energy);
    let bytes = s.save_bytes(100).unwrap();
    let mut restored = GameplaySession::default();
    restored.restore_bytes(&bytes).unwrap();
    assert_eq!(restored.building.source, s.building.source);
    // Reopening an earlier envelope must not smuggle a stop latch.
    let mut old: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    old["schema_version"] = 18.into();
    assert!(
        restored
            .restore_bytes(&serde_json::to_vec(&old).unwrap())
            .is_err()
    );
    assert_eq!(restored.building.source, s.building.source);
    s.ship.toggle_door();
    s.ship.advance(Duration::from_secs(2));
    s.ship.set_cockpit_control(true);
    s.command(Command::Landing);
    assert!(matches!(
        s.ship.snapshot().flight_state,
        FlightState::AssistedTakeoff { .. }
    ));
}

#[test]
fn construction_stopped_unsecured_and_broken_completed_hull_block() {
    let (mut s, id) = room();
    let p = s.building.source.projects.funding_mut(id, OWNER).unwrap();
    p.work.stopped = true;
    p.work.damaged = true;
    assert_eq!(
        s.construction_takeoff_blocker(),
        Some(CockpitMessage::SecureExteriorConstruction)
    );
    s.building
        .source
        .projects
        .funding_mut(id, OWNER)
        .unwrap()
        .work
        .damaged = false;
    // Actual destruction of the authored support frame cannot be hidden by Stop.
    s.building.structure.damage_part(StructureId(2)).unwrap();
    assert_eq!(
        s.construction_takeoff_blocker(),
        Some(CockpitMessage::RepairStructuralHull)
    );
    let (mut s, id) = room();
    s.advance_construction(Duration::from_secs(300));
    assert_eq!(s.construction_takeoff_blocker(), None);
    let part = s
        .building
        .structure
        .parts()
        .find(|p| p.id.0 >= 1024)
        .unwrap()
        .clone();
    let face = part.faces.first().unwrap();
    s.building
        .structure
        .set_face(part.id, face.cell, face.direction, None)
        .unwrap();
    assert_eq!(
        s.construction_takeoff_blocker(),
        Some(CockpitMessage::RepairStructuralHull)
    );
    assert!(
        s.building
            .source
            .projects
            .inspect(id)
            .unwrap()
            .work
            .completed
    );
}

#[test]
fn construction_forced_flight_latches_without_momentum_loss_or_landing_restart() {
    let (mut s, id) = room();
    s.advance_construction(Duration::from_secs(100));
    let landed = s.ship.clone();
    let pose = s.ship.snapshot().pose;
    s.ship = salimon_ship::ShipController::flying_with_velocity(pose, 0, [5., 2., -1.]).unwrap();
    s.sync_fabricator_mass();
    let before = s.ship.snapshot();
    let work = s.building.source.projects.inspect(id).unwrap().clone();
    s.advance_construction(Duration::from_secs(1));
    assert_eq!(s.ship.snapshot(), before);
    assert!(s.building.source.projects.inspect(id).unwrap().work.stopped);
    s.ship = landed;
    s.advance_construction(Duration::from_secs(100));
    let p = s.building.source.projects.inspect(id).unwrap();
    assert_eq!(p.work.elapsed_nanos, work.work.elapsed_nanos);
    assert_eq!(p.funding, work.funding);
}

#[test]
fn construction_interior_coasting_and_explicit_resume_are_funded() {
    let (mut s, id) = crate::session::construction::tests::funded_stool();
    assert_eq!(s.construction_takeoff_blocker(), None);
    let pose = s.ship.snapshot().pose;
    s.ship = salimon_ship::ShipController::flying_with_velocity(pose, 0, [3., 0., 0.]).unwrap();
    s.advance_construction(Duration::from_secs(5));
    assert_eq!(
        s.building
            .source
            .projects
            .inspect(id)
            .unwrap()
            .funding
            .progress_ppm,
        500_000
    );
    assert!(!s.building.source.projects.inspect(id).unwrap().work.stopped);
    assert_eq!(s.ship.snapshot().velocity_meters_per_second, [3., 0., 0.]);
    s.building.active = true;
    s.construction_action(BuildAction::StopResume);
    assert!(s.building.source.projects.inspect(id).unwrap().work.stopped);
    s.construction_action(BuildAction::StopResume);
    assert!(!s.building.source.projects.inspect(id).unwrap().work.stopped);
    s.advance_construction(Duration::from_secs(5));
    assert!(
        s.building
            .source
            .projects
            .inspect(id)
            .unwrap()
            .work
            .completed
    );
}
#[test]
fn construction_cancel_half_work_refunds_once_and_preserves_structure() {
    let (mut s, id) = room();
    s.advance_construction(Duration::from_secs(150));
    s.building
        .source
        .projects
        .funding_mut(id, OWNER)
        .unwrap()
        .work
        .stopped = true;
    let before = s.building.structure.clone();
    s.cancel_construction(id).unwrap();
    assert_eq!(s.building.structure, before);
    assert_eq!(s.construction_takeoff_blocker(), None);
    assert_eq!(
        s.mining
            .session
            .fragments()
            .iter()
            .map(|f| f.material().mass_kg())
            .sum::<f64>(),
        20.
    );
    assert!(s.cancel_construction(id).is_err());
}
#[test]
fn construction_secured_stop_still_obeys_actual_ramp_core_and_engine_gates() {
    let (mut s, id) = room();
    s.building
        .source
        .projects
        .funding_mut(id, OWNER)
        .unwrap()
        .work
        .stopped = true;
    s.ship.set_cockpit_control(true);
    s.ship.toggle_door();
    s.ship.advance(Duration::from_secs(2));
    assert_eq!(
        s.contextual_cockpit_message(),
        Some(CockpitMessage::CloseDoorBeforeTakeoff)
    );
    s.command(Command::Landing);
    assert!(matches!(
        s.ship.snapshot().flight_state,
        FlightState::Landed { .. }
    ));
    s.ship.toggle_door();
    s.ship.advance(Duration::from_secs(2));
    s.ship.toggle_engines();
    s.command(Command::Landing);
    assert!(matches!(
        s.ship.snapshot().flight_state,
        FlightState::Landed { .. }
    ));
    s.ship.toggle_engines();
    s.ship.set_stored_core_energy_joules(0);
    s.command(Command::Landing);
    assert!(matches!(
        s.ship.snapshot().flight_state,
        FlightState::Landed { .. }
    ));
}

#[test]
fn construction_cancel_preserves_completed_room_identity_faces_and_mass() {
    let (mut s, room_id) = room();
    s.advance_construction(Duration::from_secs(300));
    let graph = s.building.structure.clone();
    let mass = s.ship.snapshot().total_mass_kg;
    let y = SHIP_FLOOR_HEIGHT_METERS;
    let id = s
        .building
        .source
        .projects
        .place(
            registered_catalog(),
            "hull-cell",
            [-17.5, y, -1.5],
            Rotation::Forward,
            &s.building.structure,
            &PlacementContext {
                actor: OWNER,
                owner: OWNER,
                flight: FlightPermission::Landed,
                eye_meters: [-14., 2., 0.],
                reach_meters: 5.,
                reachable: true,
                physical_support: true,
                obstacles: &[],
            },
        )
        .unwrap();
    assert_eq!(
        s.construction_takeoff_blocker(),
        Some(CockpitMessage::StopExteriorConstruction)
    );
    s.cancel_construction(id).unwrap();
    assert_eq!(s.building.structure, graph);
    assert_eq!(s.ship.snapshot().total_mass_kg, mass);
    assert!(
        s.building
            .source
            .projects
            .inspect(room_id)
            .unwrap()
            .work
            .completed
    );
    assert_eq!(s.construction_takeoff_blocker(), None);
}
