use super::*;
use crate::session::{Command, interaction::InteractionTarget};
fn aim(s: &mut GameplaySession, eye: [f64; 3], target: [f64; 3]) {
    let d = sub(target, eye);
    let mut v = serde_json::to_value(s.character.save_state()).unwrap();
    v["position"] = serde_json::json!({"Inside":{"local":eye}});
    v["yaw_radians"] = d[2].atan2(d[0]).rem_euclid(std::f64::consts::TAU).into();
    v["pitch_radians"] = (d[1] / length(d)).asin().into();
    s.character =
        salimon_character::CharacterController::from_save(serde_json::from_value(v).unwrap(), None)
            .unwrap();
    s.sync_mount_collision();
}
fn setup() -> GameplaySession {
    let mut s = GameplaySession::new_run();
    let p = s
        .cabin_interaction_anchor(InteractionTarget::CabinSeat)
        .unwrap();
    aim(&mut s, [0.8, 1.9973, -0.2], p);
    s
}
#[test]
fn seat_rest_free_look_restraint_blackout_and_exact_save_restore() {
    let mut s = setup();
    let snap = s.snapshot();
    let frame = character_ship_frame(snap.ship.pose);
    assert_eq!(
        s.automation_interaction(
            [0.8, 1.9973, -0.2],
            frame.world_to_local(snap.character.look_target_meters)
        ),
        Some("cabin_seat")
    );
    s.command(Command::Interact);
    s.advance_automation(Duration::from_millis(900));
    assert_eq!(
        s.habitation.cabin.as_ref().unwrap().motions[1].fraction,
        1.,
        "{:?}",
        s.feedback
    );
    let p = s
        .cabin_interaction_anchor(InteractionTarget::CabinSeat)
        .unwrap();
    aim(&mut s, [0.8, 1.9973, -0.2], p);
    s.command(Command::Cabin(CabinAction::Rest));
    s.advance_automation(Duration::from_millis(400));
    let c = s.habitation.cabin.as_ref().unwrap();
    assert!(c.rest.is_some(), "{:?}", s.feedback);
    assert_eq!(c.rest.as_ref().unwrap().fraction, 1.);
    assert_eq!(
        s.character.local_ship_position().unwrap(),
        rest_eye(RestKind::Seat)
    );
    let before = s.snapshot().character.look_target_meters;
    s.command(Command::Look(100., 0.));
    assert_ne!(s.snapshot().character.look_target_meters, before);
    s.command(Command::Cabin(CabinAction::Restraint));
    assert!(s.habitation.cabin.as_ref().unwrap().seat_harness);
    s.ship.set_stored_core_energy_joules(0);
    s.advance_automation(Duration::from_millis(160));
    assert_eq!(
        s.character.local_ship_position().unwrap(),
        rest_eye(RestKind::Seat)
    );
    let bytes = s.save_bytes(100).unwrap();
    let mut r = GameplaySession::default();
    r.restore_bytes(&bytes).unwrap();
    assert_eq!(r.save_bytes(100).unwrap(), bytes);
    r.command(Command::Cabin(CabinAction::Rest));
    assert!(
        r.habitation
            .cabin
            .as_ref()
            .unwrap()
            .rest
            .as_ref()
            .unwrap()
            .entering,
        "restrained body cannot get up"
    );
    r.command(Command::Cabin(CabinAction::Restraint));
    r.advance_automation(Duration::from_millis(500));
    assert!(r.habitation.cabin.as_ref().unwrap().rest.is_none());
}
#[test]
fn manual_shutter_locker_and_screen_reverse_in_blackout_with_motion_restore() {
    let mut s = setup();
    s.ship.set_stored_core_energy_joules(0);
    for k in [Furniture::Shutter, Furniture::Locker, Furniture::Screen] {
        let start = s.habitation.cabin.as_ref().unwrap().motions[k as usize];
        s.toggle_cabin_fixture(k);
        s.advance_automation(Duration::from_millis(160));
        let moved = s.habitation.cabin.as_ref().unwrap().motions[k as usize];
        assert_ne!(start.fraction, moved.fraction, "{k:?}: {:?}", s.feedback);
        let bytes = s.save_bytes(100).unwrap();
        let mut r = GameplaySession::default();
        r.restore_bytes(&bytes).unwrap();
        assert_eq!(r.save_bytes(100).unwrap(), bytes);
        s.toggle_cabin_fixture(k);
        s.advance_automation(Duration::from_millis(500));
        assert_eq!(
            s.habitation.cabin.as_ref().unwrap().motions[k as usize].fraction,
            start.fraction
        );
    }
}
#[test]
fn obstructed_seat_retains_belonging_and_can_reverse_safely() {
    let mut s = setup();
    let frame = character_ship_frame(s.ship.snapshot().pose);
    let world = frame.local_to_world([0.8, 0.70, -1.10]);
    let id = crate::sphere_fixtures::sphere(
        &mut s.mining.session,
        salimon_world::resources::ResourceId::IronOre,
        2.0,
        salimon_world::resources::ResourceTransform::new(
            salimon_world::WorldPosition::new(world[0], world[1], world[2]),
            [0., 0., 0., 1.],
        )
        .unwrap(),
    );
    let before = *s.mining.session.fragment(id).unwrap();
    s.toggle_cabin_fixture(Furniture::Seat);
    s.advance_cabin(Duration::from_secs(1));
    assert!(s.habitation.cabin.as_ref().unwrap().motions[1].fraction < 1.);
    assert_eq!(s.mining.session.fragment(id), Some(&before));
    assert!(s.feedback.unwrap().contains("obstructed"));
    s.toggle_cabin_fixture(Furniture::Seat);
    s.advance_cabin(Duration::from_secs(1));
    assert_eq!(s.habitation.cabin.as_ref().unwrap().motions[1].fraction, 0.);
}
#[test]
fn bunk_use_has_no_time_skip_and_screen_cannot_trap_a_resting_player() {
    let mut s = setup();
    aim(&mut s, [-0.8, 1.9973, -2.2], [-0.8, 1.05, -2.95]);
    s.command(Command::Interact);
    s.advance_automation(Duration::from_millis(800));
    assert_eq!(s.habitation.bunk_open_fraction, 1.);
    s.command(Command::Cabin(CabinAction::Rest));
    let elapsed = s.elapsed;
    s.advance_automation(Duration::from_millis(400));
    assert_eq!(s.elapsed, elapsed + Duration::from_millis(400));
    assert!(
        s.habitation.cabin.as_ref().unwrap().rest.is_some(),
        "{:?}",
        s.feedback
    );
    s.toggle_cabin_fixture(Furniture::Screen);
    s.advance_automation(Duration::from_millis(1100));
    assert_eq!(
        s.habitation.cabin.as_ref().unwrap().motions[2].fraction,
        0.,
        "{:?}",
        s.feedback
    );
    s.command(Command::Cabin(CabinAction::Rest));
    assert!(
        s.habitation
            .cabin
            .as_ref()
            .unwrap()
            .rest
            .as_ref()
            .unwrap()
            .entering
    );
    s.toggle_cabin_fixture(Furniture::Screen);
    s.advance_automation(Duration::from_millis(1100));
    s.command(Command::Cabin(CabinAction::Rest));
    s.advance_automation(Duration::from_millis(400));
    assert!(
        s.habitation.cabin.as_ref().unwrap().rest.is_none(),
        "{:?}",
        s.feedback
    );
}
#[test]
fn corrupted_posture_restore_is_atomic_and_previous_schema_has_no_new_fixtures() {
    let mut s = setup();
    let before = s.save_bytes(100).unwrap();
    let mut v: serde_json::Value = serde_json::from_slice(&before).unwrap();
    v["state"]["habitation"]["cabin"]["motions"][0]["fraction"] = 2.0.into();
    assert!(s.restore_bytes(&serde_json::to_vec(&v).unwrap()).is_err());
    assert_eq!(s.save_bytes(100).unwrap(), before);
    v["schema_version"] = 11.into();
    v["state"]["habitation"]
        .as_object_mut()
        .unwrap()
        .remove("cabin");
    s.restore_bytes(&serde_json::to_vec(&v).unwrap()).unwrap();
    assert!(s.habitation.cabin.is_none());
}

#[test]
fn authored_mesh_bounds_match_physical_proxies_and_swept_rotation_covers_arc_extrema() {
    use salimon_renderer::FixtureMesh as Mesh;
    for (kind, mesh) in [
        (Furniture::Table, Mesh::Table),
        (Furniture::Seat, Mesh::SeatPad),
        (Furniture::Locker, Mesh::LockerDoor),
    ] {
        let local = salimon_renderer::fixture_template_bounds(mesh).unwrap();
        for fraction in [0., 0.2, 0.5, 0.8, 1.] {
            let m = Mechanism {
                fraction,
                opening: true,
            };
            let (p, q) = pose(kind, m);
            let actual = transformed(p, q, local);
            let expected = mechanism_solids(kind, m)[0];
            for (a, b) in actual.into_iter().zip(expected) {
                assert!(
                    (a - b).abs() < 3e-7,
                    "{kind:?} geometry diverged from physical proxy"
                );
            }
        }
        let sweep = swept_solids(kind, Mechanism::default(), Mechanism::open())[0];
        for step in 0..=1000 {
            let sample = mechanism_solids(
                kind,
                Mechanism {
                    fraction: f64::from(step) / 1000.,
                    opening: true,
                },
            )[0];
            for i in 0..3 {
                assert!(sample[2 * i] >= sweep[2 * i] && sample[2 * i + 1] <= sweep[2 * i + 1]);
            }
        }
    }
}

#[test]
fn locker_is_hollow_and_closed_door_blocks_the_same_physical_contents() {
    let mut s = setup();
    let eye = [-4., 1.9973, -2.1];
    let local = [-4., 1.15, -3.6];
    aim(&mut s, eye, local);
    let frame = character_ship_frame(s.ship.snapshot().pose);
    let w = frame.local_to_world(local);
    let id = crate::sphere_fixtures::sphere(
        &mut s.mining.session,
        salimon_world::resources::ResourceId::IronOre,
        0.032,
        salimon_world::resources::ResourceTransform::new(
            salimon_world::WorldPosition::new(w[0], w[1], w[2]),
            [0., 0., 0., 1.],
        )
        .unwrap(),
    );
    let before = *s.mining.session.fragment(id).unwrap();
    assert_eq!(
        crate::carrying::target(&s.mining, s.snapshot().character, frame, false),
        None
    );
    s.toggle_cabin_fixture(Furniture::Locker);
    for _ in 0..50 {
        s.advance_cabin(Duration::from_millis(16));
    }
    s.sync_mount_collision();
    assert_eq!(
        s.habitation.cabin.as_ref().unwrap().motions[4].fraction,
        1.,
        "{:?}",
        s.feedback
    );
    assert_eq!(
        crate::carrying::target(&s.mining, s.snapshot().character, frame, false),
        Some(id)
    );
    assert_eq!(s.mining.session.fragment(id), Some(&before));
    assert!(!s.mining.interior_solids.iter().any(|b| {
        salimon_character::cargo::sphere_overlaps_box(
            local,
            crate::fragment_physics::sphere_radius(before),
            *b,
        )
    }));
}

#[test]
fn lying_player_can_aim_at_the_real_manual_shutter_handle() {
    let mut s = setup();
    s.habitation.bunk_open_fraction = 1.;
    s.habitation.bunk_deploying = true;
    let eye = rest_eye(RestKind::Bunk);
    aim(&mut s, eye, shutter_handle());
    assert_eq!(
        s.available_target(
            salimon_character::CharacterLocation::InsideShip,
            eye,
            shutter_handle()
        ),
        Some(InteractionTarget::ObservationShutter)
    );
}
