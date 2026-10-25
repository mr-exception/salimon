use super::*;
use crate::session::{Command, ControlKey};
use std::time::Duration;
fn prepare() -> GameplaySession {
    let mut s = GameplaySession::default();
    s.command(Command::Control(ControlKey::Right, true));
    s.advance_automation(Duration::from_millis(72 * 16));
    s.command(Command::Control(ControlKey::Right, false));
    let snapshot = s.snapshot();
    let frame = character_ship_frame(snapshot.ship.pose);
    let eye = frame.world_to_local(snapshot.character.eye_position_meters);
    let d = salimon_math::sub([-1.4, SHIP_FLOOR_HEIGHT_METERS, 3.2], eye);
    s.command(Command::Look(
        d[2].atan2(d[0]) / 0.0022,
        -d[1].atan2(d[0].hypot(d[2])) / 0.0022,
    ));
    s.command(Command::Build(BuildAction::Toggle));
    s
}
#[test]
fn production_preview_place_inspect_cancel_preserves_physical_mass_and_materials() {
    let mut s = prepare();
    let before_mass = s.ship.snapshot().total_mass_kg;
    assert!(s.building.active);
    let preview = s.construction_preview().unwrap();
    assert_eq!(preview.rejection, None, "{preview:?}");
    s.command(Command::Mining(true));
    s.command(Command::Mining(false));
    assert_eq!(s.building.source.projects.jobs().count(), 1);
    assert_eq!(s.ship.snapshot().total_mass_kg, before_mass);
    assert!(s.mining.session.fragments().is_empty());
    assert!(s.construction_inspect()["aimed_project"].is_u64());
    s.advance_automation(Duration::from_millis(300));
    s.command(Command::Mining(true));
    s.command(Command::Mining(false));
    assert_eq!(s.building.source.projects.jobs().count(), 1);
    s.command(Command::Interact);
    assert!(!s.archive_reader.open);
    assert!(!s.crafting_ui.open);
    s.command(Command::Build(BuildAction::Cancel));
    assert_eq!(s.building.source.projects.jobs().count(), 0);
    assert_eq!(s.ship.snapshot().total_mass_kg, before_mass);
}
#[test]
fn placeholder_restore_is_exact_and_ui_is_transient_without_advancing_progress() {
    let mut s = prepare();
    s.command(Command::Mining(true));
    s.command(Command::Mining(false));
    let bytes = s.save_bytes(100).unwrap();
    let source = s.building.source.clone();
    let elapsed = s.elapsed;
    let mut restored = GameplaySession::default();
    restored.restore_bytes(&bytes).unwrap();
    assert_eq!(restored.building.source, source);
    assert_eq!(restored.elapsed, elapsed);
    assert!(!restored.building.active);
    assert_eq!(restored.construction_boxes().len(), 1);
    restored.building.active = true;
    restored.command(Command::Build(BuildAction::Cancel));
    assert_eq!(restored.building.source.projects.jobs().count(), 0);
}
#[test]
fn schema_eight_requires_source_and_legacy_seven_migrates_empty_atomically() {
    let mut s = prepare();
    s.command(Command::Mining(true));
    s.command(Command::Mining(false));
    let good = s.save_bytes(100).unwrap();
    let mut current: serde_json::Value = serde_json::from_slice(&good).unwrap();
    assert_eq!(
        current["schema_version"],
        salimon_persistence::SCHEMA_VERSION
    );
    current["state"]
        .as_object_mut()
        .unwrap()
        .remove("construction");
    assert!(
        s.restore_bytes(&serde_json::to_vec(&current).unwrap())
            .is_err()
    );
    assert_eq!(s.save_bytes(100).unwrap(), good);
    current["schema_version"] = 7.into();
    s.restore_bytes(&serde_json::to_vec(&current).unwrap())
        .unwrap();
    assert_eq!(s.building.source.projects.jobs().count(), 0);
}
#[test]
fn wrong_owner_catalog_or_cells_never_replace_a_live_session() {
    let mut s = prepare();
    s.command(Command::Mining(true));
    s.command(Command::Mining(false));
    let good = s.save_bytes(100).unwrap();
    for (key, value) in [
        ("owner", serde_json::json!(2)),
        ("cells", serde_json::json!([{ "x":100,"y":0,"z":0}])),
    ] {
        let mut source: serde_json::Value = serde_json::from_slice(&good).unwrap();
        source["state"]["construction"]["projects"]["jobs"][0][key] = value;
        assert!(
            s.restore_bytes(&serde_json::to_vec(&source).unwrap())
                .is_err()
        );
        assert_eq!(s.save_bytes(100).unwrap(), good);
    }
}
#[test]
fn normal_ship_and_placeholders_do_not_install_structure_or_consume_resources() {
    let mut s = prepare();
    let parts = s.building.structure.parts().count();
    s.command(Command::Mining(true));
    s.command(Command::Mining(false));
    s.advance_automation(Duration::from_secs(10));
    assert_eq!(s.building.structure.parts().count(), parts);
    assert_eq!(s.building.source.projects.jobs().count(), 1);
    assert!(s.building.catalog.approved_economy);
    assert_eq!(
        s.ship.snapshot().total_mass_kg,
        salimon_ship::STARTER_DRY_MASS_KG
    );
}

#[test]
fn saved_projects_cannot_bypass_the_registered_catalog_or_protected_routes() {
    use salimon_persistence::Validate;
    let structure = starter_structure();
    let mut source = ConstructionSave::default();
    let context = PlacementContext {
        actor: OWNER,
        owner: OWNER,
        flight: FlightPermission::Landed,
        eye_meters: [0.5, 1.9, -2.2],
        reach_meters: REACH_METERS,
        reachable: true,
        physical_support: true,
        obstacles: &[],
    };
    source
        .projects
        .place(
            registered_catalog(),
            "stool",
            [0.5, SHIP_FLOOR_HEIGHT_METERS, -2.2],
            Rotation::Forward,
            &structure,
            &context,
        )
        .unwrap();
    assert!(
        source.validate().is_err(),
        "a forged trusted-host context must not survive restore"
    );
    let mut s = prepare();
    s.command(Command::Mining(true));
    s.command(Command::Mining(false));
    let good = s.save_bytes(100).unwrap();
    let mut bad: serde_json::Value = serde_json::from_slice(&good).unwrap();
    bad["state"]["construction"]["projects"]["jobs"][0]["blueprint"]["requirements"][0]["milligrams"] =
        1.into();
    assert!(s.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    assert_eq!(s.save_bytes(100).unwrap(), good);
}

pub(super) fn piece(
    s: &mut GameplaySession,
    resource: salimon_world::resources::ResourceId,
    mass: f64,
) {
    use salimon_world::{resources::*, spherical_mining::*};
    let frame = character_ship_frame(s.ship.snapshot().pose);
    let w = frame.local_to_world([-1., 1.3, 2.5]);
    let pose = ResourceTransform::new(
        salimon_world::WorldPosition::new(w[0], w[1], w[2]),
        [0., 0., 0., 1.],
    )
    .unwrap();
    let mut deposit = ResourceDeposit::new(
        DepositId {
            body: salimon_world::CelestialBodyId::Earth,
            local: 543 + s.mining.session.save_state().next_id,
        },
        RawMaterial::new(resource, mass).unwrap(),
        pose.position(),
        mass,
    )
    .unwrap();
    let mut id = None;
    for _ in 0..400 {
        id = s
            .mining
            .session
            .tick_spherical(
                true,
                Some(&mut deposit),
                SPHERE_TICK,
                MiningConfig::default(),
                pose,
            )
            .or(id);
        if id.is_some() {
            break;
        }
    }
    assert!(s.mining.session.pick_up(id.unwrap()));
    s.equipment.stow();
    s.sync_carried();
}
#[test]
fn intake_rejection_receipts_save_refund_and_mass_are_atomic() {
    use salimon_world::resources::ResourceId::*;
    let mut s = prepare();
    s.command(Command::Mining(true));
    s.command(Command::Mining(false));
    piece(&mut s, IronOre, 0.5);
    let before = s.ship.snapshot().total_mass_kg;
    s.building.catalog.approved_economy = false;
    s.command(Command::Interact);
    assert!(s.mining.session.carried_id().is_some());
    s.building.catalog.approved_economy = true;
    s.command(Command::Interact);
    assert!(s.mining.session.carried_id().is_none(), "{:?}", s.feedback);
    assert_eq!(s.ship.snapshot().total_mass_kg, before);
    let bytes = s.save_bytes(100).unwrap();
    let mut r = GameplaySession::default();
    r.restore_bytes(&bytes).unwrap();
    assert_eq!(r.ship.snapshot().total_mass_kg, before);
    assert_eq!(r.building.source, s.building.source);
    piece(&mut s, SilicateRock, 0.5);
    s.command(Command::Interact);
    assert!(s.mining.session.carried_id().is_some());
    let id = s.mining.session.carried_id().unwrap();
    s.mining.session.remove_fragment(id);
    s.sync_carried();
    piece(&mut s, IronOre, 0.75);
    let f = s.mining.session.carried_id();
    s.command(Command::Interact);
    assert_eq!(s.mining.session.carried_id(), f);
    let id = f.unwrap();
    s.mining.session.remove_fragment(id);
    s.sync_carried();
    let id = s.building.source.projects.jobs().next().unwrap().id;
    let original = s
        .building
        .source
        .projects
        .inspect(id)
        .unwrap()
        .funding
        .receipts[0];
    s.cancel_construction(id).unwrap();
    assert_eq!(
        s.mining.session.fragment(original.id()).unwrap().material(),
        original.material()
    );
    assert_eq!(s.ship.snapshot().total_mass_kg, before);
    let bytes = s.save_bytes(100).unwrap();
    assert!(s.cancel_construction(id).is_err());
    assert_eq!(s.save_bytes(100).unwrap(), bytes);
}
#[test]
fn fully_funded_partial_cancel_and_corrupt_cross_owner_receipts() {
    use salimon_world::resources::ResourceId::*;
    let mut s = prepare();
    s.command(Command::Mining(true));
    s.command(Command::Mining(false));
    s.building.catalog.approved_economy = true;
    piece(&mut s, IronOre, 1.);
    let mass = s.ship.snapshot().total_mass_kg;
    s.command(Command::Interact);
    let id = s.building.source.projects.jobs().next().unwrap().id;
    let p = s.building.source.projects.funding_mut(id, OWNER).unwrap();
    assert!(p.funding.funded(&p.blueprint.requirements));
    p.funding
        .set_progress(500_000, &p.blueprint.requirements)
        .unwrap();
    let good = s.save_bytes(100).unwrap();
    let mut bad: serde_json::Value = serde_json::from_slice(&good).unwrap();
    bad["state"]["construction"]["projects"]["jobs"][0]["funding"]["receipts"][0]["id"] =
        999.into();
    assert!(s.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    assert_eq!(s.save_bytes(100).unwrap(), good);
    s.cancel_construction(id).unwrap();
    assert_eq!(s.ship.snapshot().total_mass_kg, mass - 0.5);
    assert_eq!(s.mining.session.fragments()[0].material().mass_kg(), 0.5);
}

pub(super) fn funded_stool() -> (GameplaySession, PlaceholderId) {
    let mut s = prepare();
    s.command(Command::Build(BuildAction::Place));
    piece(&mut s, salimon_world::resources::ResourceId::IronOre, 1.);
    s.command(Command::Interact);
    let p = s.building.source.projects.jobs().next().unwrap();
    assert!(p.funding.funded(&p.blueprint.requirements));
    let id = p.id;
    (s, id)
}
pub(super) fn move_player(s: &mut GameplaySession, eye: [f64; 3]) {
    let mut v = serde_json::to_value(s.character.save_state()).unwrap();
    v["position"] = serde_json::json!({"Inside":{"local":eye}});
    s.character =
        salimon_character::CharacterController::from_save(serde_json::from_value(v).unwrap(), None)
            .unwrap();
}
#[test]
fn paid_construction_automatically_completes_once_without_tool_or_nearby_player() {
    let (mut s, id) = funded_stool();
    let mass = s.ship.snapshot().total_mass_kg;
    let energy = s.ship.snapshot().energy_core.stored_joules;
    s.command(Command::Build(BuildAction::Toggle));
    move_player(
        &mut s,
        [
            4.,
            SHIP_FLOOR_HEIGHT_METERS + salimon_character::PLAYER_EYE_HEIGHT_METERS,
            0.,
        ],
    );
    assert!(s.construction_solids().is_empty());
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
    assert!(
        s.construction_solids().is_empty(),
        "partial build never solid"
    );
    assert_eq!(energy - s.ship.snapshot().energy_core.stored_joules, 5_000);
    let saved = s.save_bytes(100).unwrap();
    let mut restored = GameplaySession::default();
    restored.restore_bytes(&saved).unwrap();
    assert_eq!(restored.building.source, s.building.source);
    restored.advance_construction(Duration::from_secs(500));
    assert!(
        restored
            .building
            .source
            .projects
            .inspect(id)
            .unwrap()
            .work
            .completed
    );
    assert_eq!(restored.construction_solids().len(), 5);
    assert_eq!(restored.ship.snapshot().total_mass_kg, mass);
    assert_eq!(
        energy - restored.ship.snapshot().energy_core.stored_joules,
        10_000
    );
    assert!(restored.cancel_construction(id).is_err());
    let after = restored.save_bytes(100).unwrap();
    restored.advance_construction(Duration::from_secs(500));
    assert_eq!(restored.save_bytes(100).unwrap(), after);
    let mut again = GameplaySession::default();
    again.restore_bytes(&after).unwrap();
    assert_eq!(again.construction_solids(), restored.construction_solids());
    assert!(
        again
            .mining
            .interior_solids
            .iter()
            .any(|b| again.construction_solids().contains(b))
    );
}
#[test]
fn construction_safety_damage_pause_and_exact_partial_energy_admission() {
    let (mut s, id) = funded_stool();
    let source = s.building.source.clone();
    let energy = s.ship.snapshot().energy_core.stored_joules;
    let space = s.building.source.projects.inspect(id).unwrap().space;
    let original = s.snapshot().character;
    let frame = character_ship_frame(s.ship.snapshot().pose);
    let old_eye = frame.world_to_local(original.eye_position_meters);
    move_player(
        &mut s,
        [
            (space.min[0] + space.max[0]) / 2.,
            SHIP_FLOOR_HEIGHT_METERS + salimon_character::PLAYER_EYE_HEIGHT_METERS,
            (space.min[2] + space.max[2]) / 2.,
        ],
    );
    s.advance_construction(Duration::from_secs(10));
    assert_eq!(s.building.source, source);
    assert_eq!(s.ship.snapshot().energy_core.stored_joules, energy);
    assert!(
        s.construction_inspect()["status"][id.0.to_string()]
            .as_str()
            .unwrap()
            .contains("Player")
    );
    move_player(&mut s, old_eye);
    s.building
        .source
        .projects
        .funding_mut(id, OWNER)
        .unwrap()
        .work
        .damaged = true;
    s.advance_construction(Duration::from_secs(10));
    assert_eq!(s.ship.snapshot().energy_core.stored_joules, energy);
    s.building
        .source
        .projects
        .funding_mut(id, OWNER)
        .unwrap()
        .work
        .damaged = false;
    s.paused = true;
    s.advance_construction(Duration::from_secs(10));
    assert_eq!(s.building.source, source);
    s.paused = false;
    let reserve = s.recovery_reserve();
    s.ship.exchange_core_energy_joules(0., f64::MAX).unwrap();
    s.advance_construction(Duration::from_secs(5));
    assert_eq!(s.building.source, source);
    s.ship
        .exchange_core_energy_joules(reserve + 2_500., 0.)
        .unwrap();
    s.advance_construction(Duration::from_secs(5));
    assert_eq!(s.ship.snapshot().energy_core.stored_joules, reserve as u64);
    assert_eq!(
        s.building
            .source
            .projects
            .inspect(id)
            .unwrap()
            .funding
            .progress_ppm,
        250_000
    );
    let progress = s.building.source.clone();
    s.advance_construction(Duration::from_secs(5));
    assert_eq!(s.building.source, progress);
    s.ship.exchange_core_energy_joules(7_500., 0.).unwrap();
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
}
#[test]
fn paid_partial_cancel_returns_proportional_real_mass_and_never_replays() {
    let (mut s, id) = funded_stool();
    s.advance_construction(Duration::from_millis(2_500));
    let mass = s.ship.snapshot().total_mass_kg;
    s.cancel_construction(id).unwrap();
    assert_eq!(s.mining.session.fragments().len(), 1);
    assert_eq!(s.mining.session.fragments()[0].material().mass_kg(), 0.75);
    assert_eq!(s.ship.snapshot().total_mass_kg, mass - 0.25);
    let saved = s.save_bytes(100).unwrap();
    assert!(s.cancel_construction(id).is_err());
    assert_eq!(s.save_bytes(100).unwrap(), saved);
}
#[test]
fn timer_partitioning_save_validation_and_full_scheduler_pause_are_exact() {
    let (mut a, id) = funded_stool();
    let bytes = a.save_bytes(100).unwrap();
    let mut b = GameplaySession::default();
    b.restore_bytes(&bytes).unwrap();
    a.advance_construction(Duration::from_millis(1_234));
    for _ in 0..1_234 {
        b.advance_construction(Duration::from_millis(1));
    }
    assert_eq!(a.building.source, b.building.source);
    assert_eq!(a.ship.snapshot().energy_core, b.ship.snapshot().energy_core);
    let good = a.save_bytes(100).unwrap();
    for (key, value) in [
        ("elapsed_nanos", serde_json::json!(300_000_000_001_u64)),
        ("completed", serde_json::json!(true)),
    ] {
        let mut bad: serde_json::Value = serde_json::from_slice(&good).unwrap();
        bad["state"]["construction"]["projects"]["jobs"][0]["work"][key] = value;
        assert!(a.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
        assert_eq!(a.save_bytes(100).unwrap(), good);
    }
    let mut bad: serde_json::Value = serde_json::from_slice(&good).unwrap();
    bad["schema_version"] = 16.into();
    assert!(a.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    a.paused = true;
    let before = a.building.source.clone();
    a.advance_automation(Duration::from_secs(5));
    assert_eq!(a.building.source, before);
    a.paused = false;
    a.advance_automation(Duration::from_millis(16));
    assert!(
        a.building
            .source
            .projects
            .inspect(id)
            .unwrap()
            .funding
            .progress_ppm
            > 123_400
    );
}

#[test]
fn cargo_obstruction_prevents_final_install_and_spends_nothing_until_clear() {
    let (mut s, id) = funded_stool();
    s.advance_construction(Duration::from_millis(9_984));
    piece(&mut s, salimon_world::resources::ResourceId::IronOre, 0.1);
    let f = s.mining.session.carried_id().unwrap();
    let p = s.building.source.projects.inspect(id).unwrap();
    let local = std::array::from_fn(|i| (p.space.min[i] + p.space.max[i]) / 2.);
    let w = character_ship_frame(s.ship.snapshot().pose).local_to_world(local);
    let pose = salimon_world::resources::ResourceTransform::new(
        salimon_world::WorldPosition::new(w[0], w[1], w[2]),
        [0., 0., 0., 1.],
    )
    .unwrap();
    s.mining.session.move_carried(pose);
    let before = s.building.source.clone();
    let energy = s.ship.snapshot().energy_core;
    s.advance_construction(Duration::from_secs(20));
    assert_eq!(s.building.source, before);
    assert_eq!(s.ship.snapshot().energy_core, energy);
    assert!(s.construction_solids().is_empty());
    s.mining.session.remove_fragment(f);
    s.sync_carried();
    let checkpoint = s.checkpoint_events();
    s.advance_construction(Duration::from_millis(16));
    assert!(
        s.building
            .source
            .projects
            .inspect(id)
            .unwrap()
            .work
            .completed
    );
    assert_ne!(
        s.checkpoint_events(),
        checkpoint,
        "completion triggers durable checkpoint"
    );
}

#[test]
fn complex_installation_stops_cancellable_before_completion_without_fake_solids() {
    let mut s = prepare();
    let mut placed = None;
    'search: for x in -14..=12 {
        for z in -8..=8 {
            let target = [
                f64::from(x) * 0.5,
                SHIP_FLOOR_HEIGHT_METERS,
                f64::from(z) * 0.5,
            ];
            let obstacles = s.construction_obstacles(Footprint::Interior);
            let c = PlacementContext {
                actor: OWNER,
                owner: OWNER,
                flight: FlightPermission::Landed,
                eye_meters: target,
                reach_meters: REACH_METERS,
                reachable: true,
                physical_support: true,
                obstacles: &obstacles,
            };
            if let Ok(id) = s.building.source.projects.place(
                &s.building.catalog,
                "utility-module",
                target,
                Rotation::Forward,
                &s.building.structure,
                &c,
            ) {
                let p = s.building.source.projects.inspect(id).unwrap();
                if has_physical_support(Footprint::Interior, p.space) {
                    placed = Some(id);
                    break 'search;
                }
                s.building.source.projects.cancel(id, OWNER).unwrap();
            }
        }
    }
    let id = placed.expect("a clear supported module location exists in the starter");
    piece(&mut s, salimon_world::resources::ResourceId::IronOre, 3.);
    let p = s.building.source.projects.funding_mut(id, OWNER).unwrap();
    p.funding
        .contribute(&mut s.mining.session, &p.blueprint.requirements, true)
        .unwrap();
    s.sync_carried();
    let before = s.ship.snapshot().energy_core.stored_joules;
    s.advance_construction(Duration::from_secs(100));
    let p = s.building.source.projects.inspect(id).unwrap();
    assert_eq!(p.funding.progress_ppm, 999_999);
    assert!(!p.work.completed);
    assert!(s.construction_solids().is_empty());
    assert!(
        s.construction_status(p)
            .contains("installation unavailable")
    );
    assert!(before - s.ship.snapshot().energy_core.stored_joules <= 180_000);
    let after = s.ship.snapshot().energy_core;
    let source = s.building.source.clone();
    s.advance_construction(Duration::from_secs(100));
    assert_eq!(s.building.source, source);
    assert_eq!(s.ship.snapshot().energy_core, after);
    s.cancel_construction(id).unwrap();
    assert_eq!(
        s.mining.session.fragments()[0].material().mass_kg(),
        0.000003
    );
}
