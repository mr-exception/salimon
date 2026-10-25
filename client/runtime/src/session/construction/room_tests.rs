use super::tests::{move_player, piece};
use super::*;
use crate::session::{Command, ControlKey};
use std::time::Duration;

fn room() -> (GameplaySession, PlaceholderId) {
    let mut s = GameplaySession::default();
    let y = SHIP_FLOOR_HEIGHT_METERS;
    let target = [-14.5, y, -1.5];
    let id = s
        .building
        .source
        .projects
        .place(
            registered_catalog(),
            "two-deck-room",
            target,
            Rotation::Forward,
            &s.building.structure,
            &PlacementContext {
                actor: OWNER,
                owner: OWNER,
                flight: FlightPermission::Landed,
                eye_meters: [-7., y + 1.75, 0.],
                reach_meters: 5.,
                reachable: true,
                physical_support: true,
                obstacles: &static_obstacles(Footprint::Structural),
            },
        )
        .unwrap();
    // Four original 10 kg spheres supply the registered 40 kg room cost.
    for _ in 0..4 {
        piece(&mut s, salimon_world::resources::ResourceId::IronOre, 10.);
        let p = s.building.source.projects.funding_mut(id, OWNER).unwrap();
        p.funding
            .contribute(&mut s.mining.session, &p.blueprint.requirements, true)
            .unwrap();
    }
    s.sync_carried();
    s.sync_fabricator_mass();
    (s, id)
}
#[test]
fn one_funded_room_commits_four_cells_after_last_tick_and_roundtrips() {
    let (mut s, id) = room();
    let mass = s.ship.snapshot().total_mass_kg;
    let energy = s.ship.snapshot().energy_core.stored_joules;
    assert!(s.structural_solids().is_empty());
    s.advance_construction(Duration::from_secs(299));
    assert!(s.structural_solids().is_empty());
    assert_eq!(s.building.structure.parts().count(), 2);
    let mut r = GameplaySession::default();
    r.restore_bytes(&s.save_bytes(100).unwrap()).unwrap();
    r.advance_construction(Duration::from_secs(1));
    assert!(
        r.building
            .source
            .projects
            .inspect(id)
            .unwrap()
            .work
            .completed,
        "{}",
        r.construction_status(r.building.source.projects.inspect(id).unwrap())
    );
    assert_eq!(r.building.structure.parts().count(), 6);
    assert_eq!(r.building.structure.rooms().count(), 1);
    assert_eq!(r.ship.snapshot().total_mass_kg, mass);
    assert_eq!(
        energy - r.ship.snapshot().energy_core.stored_joules,
        3_000_000
    );
    assert!(r.structural_solids().len() > 25);
    let before = r.building.structure.clone();
    let mut again = GameplaySession::default();
    again.restore_bytes(&r.save_bytes(100).unwrap()).unwrap();
    assert_eq!(again.building.structure, before);
    assert_eq!(again.structural_solids(), r.structural_solids());
    assert_eq!(again.ship.snapshot().total_mass_kg, mass);
    again.advance_construction(Duration::from_secs(1000));
    assert_eq!(again.building.structure, before);
    assert!(again.cancel_construction(id).is_err());
}
#[test]
fn closed_gate_and_stairs_keep_full_body_clear_and_reach_upper_deck() {
    let (mut s, _) = room();
    s.advance_construction(Duration::from_secs(300));
    let y = SHIP_FLOOR_HEIGHT_METERS;
    move_player(&mut s, [-6.9, y + 1.75, 0.]);
    s.sync_mount_collision();
    s.command(Command::Control(ControlKey::Backward, true));
    s.advance_automation(Duration::from_secs(3));
    s.command(Command::Control(ControlKey::Backward, false));
    let closed = s.character.local_ship_position().unwrap();
    assert!(closed[0] > -7.9, "{closed:?}");
    // Use the real gate command and its full animation before crossing.
    s.ship.toggle_door();
    s.advance_automation(Duration::from_secs(3));
    s.command(Command::Control(ControlKey::Backward, true));
    s.advance_automation(Duration::from_secs(2));
    s.command(Command::Control(ControlKey::Backward, false));
    let entered = s.character.local_ship_position().unwrap();
    assert!(entered[0] < -8.6, "{entered:?}");
    // Explicit lower-deck fixture at stair foot; movement remains production WASD.
    move_player(&mut s, [-8.65, y + 1.85, -0.4]);
    s.sync_mount_collision();
    s.command(Command::Control(ControlKey::Backward, true));
    s.advance_automation(Duration::from_millis(1500));
    s.command(Command::Control(ControlKey::Backward, false));
    let upper = s.character.local_ship_position().unwrap();
    assert!(upper[1] > y + 4.6, "{upper:?}");
}

#[test]
fn room_evidence_uses_one_real_intake_and_preserves_powered_and_unpowered_movement() {
    let mut s = GameplaySession::default();
    evidence_room(&mut s).unwrap();
    assert!(
        (s.mining
            .session
            .fragment(s.mining.session.carried_id().unwrap())
            .unwrap()
            .material()
            .mass_kg()
            - 40.)
            .abs()
            < 1e-10
    );
    assert!(
        s.aimed_construction().is_some(),
        "{:?}",
        s.construction_inspect()
    );
    s.command(Command::Interact);
    let p = s.building.source.projects.jobs().next().unwrap();
    assert!(
        p.funding.funded(&p.blueprint.requirements),
        "{:?}",
        s.feedback
    );
    s.command(Command::Build(BuildAction::Toggle));
    s.advance_automation(Duration::from_secs(300));
    assert!(
        s.building
            .source
            .projects
            .jobs()
            .next()
            .unwrap()
            .work
            .completed
    );
    move_player(&mut s, [-10., SHIP_FLOOR_HEIGHT_METERS + 1.85, 0.6]);
    s.sync_mount_collision();
    s.command(Command::Control(ControlKey::Jump, true));
    s.advance_automation(Duration::from_millis(160));
    s.command(Command::Control(ControlKey::Jump, false));
    assert!(s.character.local_ship_position().unwrap()[1] > SHIP_FLOOR_HEIGHT_METERS + 2.);
    s.advance_automation(Duration::from_secs(2));
    let floor = s.character.local_ship_position().unwrap();
    assert!(
        (floor[1] - SHIP_FLOOR_HEIGHT_METERS - 1.85).abs() < 0.002,
        "{floor:?}"
    );
    s.ship.set_stored_core_energy_joules(0);
    s.command(Command::Control(ControlKey::Right, true));
    s.advance_automation(Duration::from_millis(160));
    s.command(Command::Control(ControlKey::Right, false));
    let a = s.character.local_ship_position().unwrap();
    s.advance_automation(Duration::from_millis(160));
    let b = s.character.local_ship_position().unwrap();
    assert!(
        salimon_math::length(salimon_math::sub(a, b)) > 0.001,
        "zero-G release retains drift"
    );
}

#[test]
fn expansion_opens_one_accessible_door_and_partition_edit_survives_save() {
    use salimon_ship::structure::{Direction, FaceKind, shell::PartitionEdit};
    let (mut s, _) = room();
    s.advance_construction(Duration::from_secs(300));
    let before = s.building.structure.clone();
    let target = [-17.5, SHIP_FLOOR_HEIGHT_METERS, -1.5];
    assert!(s.structural_support(Envelope {
        min: target,
        max: [-14.5, SHIP_FLOOR_HEIGHT_METERS + 3., 1.5]
    }));
    let id = s
        .building
        .source
        .projects
        .place(
            registered_catalog(),
            "hull-cell",
            target,
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
    for _ in 0..2 {
        piece(&mut s, salimon_world::resources::ResourceId::IronOre, 10.);
        let p = s.building.source.projects.funding_mut(id, OWNER).unwrap();
        p.funding
            .contribute(&mut s.mining.session, &p.blueprint.requirements, true)
            .unwrap();
    }
    s.sync_carried();
    s.advance_construction(Duration::from_secs(119));
    let [tip, _, _, top] = boarding_exit(&s.building.structure).unwrap();
    let floor = s
        .mining
        .modular_interior
        .support([tip + 0.3, top + 1., 0.], 0.24, top)
        .unwrap();
    move_player(&mut s, [tip + 0.3, floor + 1.75, 0.]);
    s.sync_mount_collision();
    let pending = s.building.source.clone();
    let energy = s.ship.snapshot().energy_core;
    s.advance_construction(Duration::from_secs(1));
    assert_eq!(
        s.building.source, pending,
        "occupied old boarding route cannot be rerouted"
    );
    assert_eq!(s.ship.snapshot().energy_core, energy);
    move_player(&mut s, [0.5, SHIP_FLOOR_HEIGHT_METERS + 1.75, -2.2]);
    s.sync_mount_collision();
    s.advance_construction(Duration::from_secs(1));
    assert!(
        s.building
            .source
            .projects
            .inspect(id)
            .unwrap()
            .work
            .completed
    );
    let (part, face) = s
        .building
        .structure
        .parts()
        .flat_map(|p| p.faces.iter().map(move |f| (p.id, f)))
        .find(|(_, f)| f.cell == Cell::new(-5, 0, 1) && f.direction == Direction::Forward)
        .unwrap();
    assert_eq!(face.kind, FaceKind::Doorway);
    let original_ids: Vec<_> = before.parts().map(|p| p.id).collect();
    assert!(
        original_ids
            .iter()
            .all(|id| s.building.structure.parts().any(|p| p.id == *id))
    );
    let mass = s.ship.snapshot().total_mass_kg;
    move_player(&mut s, [-13.5, SHIP_FLOOR_HEIGHT_METERS + 1.85, 0.9]);
    s.sync_mount_collision();
    s.command(Command::Look(std::f64::consts::PI / 0.0022, 0.));
    s.building.active = true;
    let checkpoint = s.checkpoint_events();
    s.command(Command::Build(BuildAction::RemovePartition));
    assert_eq!(
        s.building.source.removed_partitions,
        vec![PartitionEdit {
            part,
            direction: Direction::Forward
        }],
        "{:?}",
        s.feedback
    );
    assert_ne!(s.checkpoint_events(), checkpoint);
    let mut r = GameplaySession::default();
    r.restore_bytes(&s.save_bytes(100).unwrap()).unwrap();
    assert_eq!(r.building.structure, s.building.structure);
    assert_eq!(r.ship.snapshot().total_mass_kg, mass);
    let good = r.save_bytes(100).unwrap();
    let mut bad: serde_json::Value = serde_json::from_slice(&good).unwrap();
    bad["state"]["construction"]["removed_partitions"][0]["direction"] = "Down".into();
    assert!(r.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    assert_eq!(r.save_bytes(100).unwrap(), good);
    bad = serde_json::from_slice(&good).unwrap();
    bad["schema_version"] = 17.into();
    assert!(r.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
}

#[test]
fn occupied_connector_cannot_publish_the_last_joule_or_new_colliders() {
    let (mut s, id) = room();
    s.advance_construction(Duration::from_secs(299));
    move_player(&mut s, [-8., SHIP_FLOOR_HEIGHT_METERS + 1.75, 0.]);
    let before = s.building.source.clone();
    let energy = s.ship.snapshot().energy_core;
    s.advance_construction(Duration::from_secs(1));
    assert_eq!(s.building.source, before);
    assert_eq!(s.ship.snapshot().energy_core, energy);
    assert!(s.structural_solids().is_empty());
    move_player(&mut s, [-6.6, SHIP_FLOOR_HEIGHT_METERS + 1.75, 0.]);
    s.advance_construction(Duration::from_secs(1));
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
fn upper_deck_cargo_keeps_identity_collision_support_and_mass_across_reload() {
    let (mut s, _) = room();
    s.advance_construction(Duration::from_secs(300));
    piece(&mut s, salimon_world::resources::ResourceId::IronOre, 1.);
    move_player(&mut s, [-13.5, SHIP_FLOOR_HEIGHT_METERS + 4.85, 0.7]);
    s.sync_mount_collision();
    s.sync_carried();
    let id = s.mining.session.carried_id().unwrap();
    let mass = s.ship.snapshot().total_mass_kg;
    s.command(Command::GrabDrop(true));
    s.command(Command::GrabDrop(false));
    assert!(s.mining.session.carried_id().is_none(), "{:?}", s.feedback);
    s.advance_automation(Duration::from_secs(3));
    let local = s.mining.ship_fragments[&id].meters();
    let radius = crate::fragment_physics::sphere_radius(*s.mining.session.fragment(id).unwrap());
    assert!(
        (local[1] - SHIP_FLOOR_HEIGHT_METERS - 3.1 - radius).abs() < 0.003,
        "{local:?}"
    );
    assert_eq!(s.ship.snapshot().total_mass_kg, mass);
    let mut r = GameplaySession::default();
    r.restore_bytes(&s.save_bytes(100).unwrap()).unwrap();
    assert_eq!(r.ship.snapshot().total_mass_kg, mass);
    assert_eq!(r.mining.ship_fragments[&id], s.mining.ship_fragments[&id]);
}

#[test]
fn legacy_two_cell_reservation_retains_original_footprint_and_cancellable_receipts() {
    let mut s = GameplaySession::default();
    let y = SHIP_FLOOR_HEIGHT_METERS;
    s.building
        .source
        .projects
        .place(
            registered_catalog(),
            "two-deck-room",
            [-14.5, y, -1.5],
            Rotation::Forward,
            &s.building.structure,
            &PlacementContext {
                actor: OWNER,
                owner: OWNER,
                flight: FlightPermission::Landed,
                eye_meters: [-7., y + 1.75, 0.],
                reach_meters: 5.,
                reachable: true,
                physical_support: true,
                obstacles: &[],
            },
        )
        .unwrap();
    let mut v: serde_json::Value = serde_json::from_slice(&s.save_bytes(100).unwrap()).unwrap();
    v["schema_version"] = 17.into();
    let p = &mut v["state"]["construction"]["projects"]["jobs"][0];
    p["anchor_meters"][0] = (-11.5).into();
    p["space"]["min"][0] = (-11.5).into();
    p["blueprint"]["space"]["max"][0] = 3.into();
    p["blueprint"]["cells"] = serde_json::json!([{"x":0,"y":0,"z":0},{"x":0,"y":1,"z":0}]);
    p["cells"] = serde_json::json!([{"x":-3,"y":0,"z":1},{"x":-3,"y":1,"z":1}]);
    s.restore_bytes(&serde_json::to_vec(&v).unwrap()).unwrap();
    let p = s.building.source.projects.jobs().next().unwrap();
    assert_eq!(p.cells.len(), 2);
    assert!(!supports_completion(p));
    let id = p.id;
    s.cancel_construction(id).unwrap();
    assert_eq!(s.building.source.projects.jobs().count(), 0);
}

#[test]
fn completed_room_accepts_supported_furniture_without_its_old_reservation_occluding_intake() {
    let (mut s, _) = room();
    s.advance_construction(Duration::from_secs(300));
    let y = SHIP_FLOOR_HEIGHT_METERS + 0.1;
    let target = [-12., y, 0.5];
    let obstacles = s.construction_obstacles(Footprint::Interior);
    let id = s
        .building
        .source
        .projects
        .place(
            registered_catalog(),
            "stool",
            target,
            Rotation::Forward,
            &s.building.structure,
            &PlacementContext {
                actor: OWNER,
                owner: OWNER,
                flight: FlightPermission::Landed,
                eye_meters: [-10.5, y + 1.75, 0.75],
                reach_meters: 5.,
                reachable: true,
                physical_support: true,
                obstacles: &obstacles,
            },
        )
        .unwrap();
    move_player(&mut s, [-10.5, y + 1.75, 0.75]);
    s.sync_mount_collision();
    let d = [-1.25_f64, -1.25, 0.];
    s.command(Command::Look(
        std::f64::consts::PI / 0.0022,
        -d[1].atan2(d[0].abs()) / 0.0022,
    ));
    s.building.active = true;
    piece(&mut s, salimon_world::resources::ResourceId::IronOre, 1.);
    assert_eq!(s.aimed_construction(), Some(id));
    s.command(Command::Interact);
    assert!(
        s.building
            .source
            .projects
            .inspect(id)
            .unwrap()
            .funding
            .funded(&registered_catalog().find("stool").unwrap().requirements)
    );
    s.advance_construction(Duration::from_secs(10));
    assert!(
        s.building
            .source
            .projects
            .inspect(id)
            .unwrap()
            .work
            .completed
    );
    let mut r = GameplaySession::default();
    r.restore_bytes(&s.save_bytes(100).unwrap()).unwrap();
    assert_eq!(r.construction_solids(), s.construction_solids());
}

#[test]
fn rotated_internal_door_completes_and_manual_closure_respects_occupancy_and_save() {
    let mut s = GameplaySession::default();
    let mut result = None;
    'search: for x in -14..=12 {
        for z in -7..=7 {
            let target = [
                f64::from(x) * 0.5,
                SHIP_FLOOR_HEIGHT_METERS,
                f64::from(z) * 0.5,
            ];
            let obstacles = s.construction_obstacles(Footprint::Interior);
            if let Ok(id) = s.building.source.projects.place(
                registered_catalog(),
                "internal-door",
                target,
                Rotation::Port,
                &s.building.structure,
                &PlacementContext {
                    actor: OWNER,
                    owner: OWNER,
                    flight: FlightPermission::Landed,
                    eye_meters: target,
                    reach_meters: 5.,
                    reachable: true,
                    physical_support: true,
                    obstacles: &obstacles,
                },
            ) {
                let p = s.building.source.projects.inspect(id).unwrap();
                if has_physical_support(Footprint::Interior, p.space) {
                    result = Some(id);
                    break 'search;
                }
                s.building.source.projects.cancel(id, OWNER).unwrap();
            }
        }
    }
    let id = result.expect("safe rotated interior doorway location");
    piece(&mut s, salimon_world::resources::ResourceId::IronOre, 5.);
    let p = s.building.source.projects.funding_mut(id, OWNER).unwrap();
    p.funding
        .contribute(&mut s.mining.session, &p.blueprint.requirements, true)
        .unwrap();
    s.sync_carried();
    s.advance_construction(Duration::from_secs(30));
    assert!(
        s.building
            .source
            .projects
            .inspect(id)
            .unwrap()
            .work
            .completed
    );
    let closed = s.construction_solids();
    let checkpoint = s.checkpoint_events();
    s.toggle_completed_door(id);
    assert!(s.building.source.open_doors.contains(&id));
    assert_ne!(s.checkpoint_events(), checkpoint);
    let b = s.building.source.projects.inspect(id).unwrap().space;
    let eye = [
        (b.min[0] + b.max[0]) / 2.,
        b.min[1] + 1.75,
        (b.min[2] + b.max[2]) / 2.,
    ];
    move_player(&mut s, eye);
    s.toggle_completed_door(id);
    assert!(s.building.source.open_doors.contains(&id));
    move_player(&mut s, [0.5, SHIP_FLOOR_HEIGHT_METERS + 1.75, -2.2]);
    s.toggle_completed_door(id);
    assert!(!s.building.source.open_doors.contains(&id));
    assert_eq!(s.construction_solids(), closed);
    let mut r = GameplaySession::default();
    r.restore_bytes(&s.save_bytes(100).unwrap()).unwrap();
    assert_eq!(r.construction_solids(), closed);
}

#[test]
fn completed_room_preserves_a_surface_exit_and_body_clear_boarding_return() {
    let (mut s, _) = room();
    s.advance_construction(Duration::from_secs(300));
    move_player(&mut s, [-13., SHIP_FLOOR_HEIGHT_METERS + 1.85, 0.4]);
    s.sync_mount_collision();
    aim_at(&mut s, [-14.39, SHIP_FLOOR_HEIGHT_METERS + 1.2, 0.95]);
    s.command(Command::Interact);
    s.advance_automation(Duration::from_secs(3));
    assert!(
        crate::session::door_passable(s.ship.snapshot()),
        "{:?}",
        s.feedback
    );
    aim_at(&mut s, [-12., SHIP_FLOOR_HEIGHT_METERS + 1.85, 0.4]);
    s.command(Command::Control(ControlKey::Backward, true));
    s.advance_automation(Duration::from_secs(3));
    s.command(Command::Control(ControlKey::Backward, false));
    assert_eq!(
        s.character.location(),
        salimon_character::CharacterLocation::Surface,
        "{:?}",
        s.snapshot().character
    );
    s.command(Command::Control(ControlKey::Forward, true));
    s.advance_automation(Duration::from_secs(3));
    s.command(Command::Control(ControlKey::Forward, false));
    assert_eq!(
        s.character.location(),
        salimon_character::CharacterLocation::InsideShip,
        "{:?}",
        s.snapshot().character
    );
    assert!(s.character.local_ship_position().unwrap()[0] > -14.5);
}

fn aim_at(s: &mut GameplaySession, target: [f64; 3]) {
    let snap = s.snapshot();
    let f = character_ship_frame(snap.ship.pose);
    let eye = f.world_to_local(snap.character.eye_position_meters);
    let old = salimon_math::sub(f.world_to_local(snap.character.look_target_meters), eye);
    let d = salimon_math::sub(target, eye);
    let yaw = (d[2].atan2(d[0]) - old[2].atan2(old[0]) + std::f64::consts::PI)
        .rem_euclid(std::f64::consts::TAU)
        - std::f64::consts::PI;
    s.command(Command::Look(
        yaw / 0.0022,
        (old[1].atan2(old[0].hypot(old[2])) - d[1].atan2(d[0].hypot(d[2]))) / 0.0022,
    ));
}

#[test]
fn cargo_on_boarding_stairs_uses_actual_support_below_the_original_deck() {
    let (mut s, _) = room();
    s.advance_construction(Duration::from_secs(300));
    piece(&mut s, salimon_world::resources::ResourceId::IronOre, 1.);
    let floor = s
        .mining
        .modular_interior
        .support([-16., 2., 0.], 0.24, SHIP_FLOOR_HEIGHT_METERS)
        .unwrap();
    move_player(&mut s, [-16., floor + 1.75, 0.]);
    s.sync_mount_collision();
    s.sync_carried();
    let id = s.mining.session.carried_id().unwrap();
    s.command(Command::GrabDrop(true));
    s.command(Command::GrabDrop(false));
    assert!(s.mining.session.carried_id().is_none(), "{:?}", s.feedback);
    s.advance_automation(Duration::from_secs(3));
    let p = s.mining.ship_fragments[&id].meters();
    let radius = crate::fragment_physics::sphere_radius(*s.mining.session.fragment(id).unwrap());
    let h = s
        .mining
        .modular_interior
        .support(p, radius, p[1] - radius + 0.001)
        .unwrap();
    assert!(h < SHIP_FLOOR_HEIGHT_METERS);
    assert!((p[1] - radius - h).abs() < 0.003, "{p:?}, {h}");
}
