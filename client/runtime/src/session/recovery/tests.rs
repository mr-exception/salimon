use super::*;
use salimon_fabrication::{FabricationCommand as C, RecipeId};
use salimon_world::{
    CelestialBodyId, WorldPosition,
    resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId, ResourceTransform},
    spherical_mining::{MiningConfig, SPHERE_TICK},
};
fn new_session() -> GameplaySession {
    let mut s = GameplaySession::default();
    crate::e2e::initialize(
        &mut s,
        crate::e2e::Config {
            scenario: crate::e2e::Scenario::LandedEarth,
            seed: 468,
            step: Duration::from_millis(20),
            fragment_load: None,
        },
    )
    .unwrap();
    s
}
fn enqueue(
    s: &mut GameplaySession,
    quantity: u32,
) -> Result<salimon_fabrication::CommandResult, salimon_fabrication::FabricationError> {
    s.execute_fabricator(C::Enqueue {
        recipe: RecipeId::new("data-recovery-module").unwrap(),
        quantity,
    })
}
fn install() -> GameplaySession {
    let mut s = new_session();
    for r in [ResourceId::IronOre, ResourceId::SilicateRock] {
        s.execute_fabricator(C::Insert(RawMaterial::new(r, 2.0).unwrap()))
            .unwrap();
    }
    enqueue(&mut s, 1).unwrap();
    s.advance_fabrication(Duration::from_secs(60));
    assert!(s.fabricator_output_component());
    let before = s.ship.snapshot().total_mass_kg;
    s.collect_fabricator_output();
    assert_eq!(s.ship.snapshot().total_mass_kg, before);
    assert!(s.recovery.module.is_some());
    assert!(!s.fabricator_output_present());
    s
}
#[test]
fn module_recipe_commit_install_mass_and_duplicate_requests_are_atomic() {
    let mut s = new_session();
    let bytes = s.save_bytes(100).unwrap();
    assert!(enqueue(&mut s, 1).is_err());
    assert_eq!(s.save_bytes(100).unwrap(), bytes);
    s = install();
    let bytes = s.save_bytes(100).unwrap();
    assert!(enqueue(&mut s, 1).is_err());
    assert!(enqueue(&mut s, 2).is_err());
    s.collect_fabricator_output();
    assert_eq!(s.save_bytes(100).unwrap(), bytes);
    let mut restored = new_session();
    restored.restore_bytes(&bytes).unwrap();
    assert_eq!(
        restored.ship.snapshot().total_mass_kg,
        s.ship.snapshot().total_mass_kg
    );
    assert_eq!(restored.save_bytes(100).unwrap(), bytes);
}
#[test]
fn progress_is_exactly_core_funded_and_stops_at_emergency_floor_without_refund() {
    let mut s = install();
    s.toggle_recovery("received.damaged-01");
    let reserve = s.recovery_reserve();
    s.ship.exchange_core_energy_joules(0.0, f64::MAX).unwrap();
    s.advance_recovery(Duration::from_secs(30));
    assert_eq!(s.recovery.spent_joules, [0.0; 2]);
    s.ship
        .exchange_core_energy_joules(reserve + 10_000_000.0, 0.0)
        .unwrap();
    let before = s.ship.snapshot().energy_core.stored_joules;
    s.advance_recovery(Duration::from_secs(3));
    assert_eq!(s.recovery.spent_joules[0], 10_000_000.0);
    assert_eq!(s.ship.snapshot().energy_core.stored_joules, reserve as u64);
    assert_eq!(
        before - s.ship.snapshot().energy_core.stored_joules,
        10_000_000
    );
    let progress = s.recovery.spent_joules;
    s.toggle_recovery("received.damaged-01");
    s.advance_recovery(Duration::from_secs(600));
    assert_eq!(s.recovery.spent_joules, progress);
    s.toggle_recovery("received.damaged-01");
    s.advance_recovery(Duration::from_secs(600));
    assert_eq!(s.recovery.spent_joules, progress);
}
#[test]
fn save_restore_offline_freeze_legacy_and_bad_sources_are_transactional() {
    let mut s = install();
    s.toggle_recovery("received.damaged-01");
    s.advance_recovery(Duration::from_secs(7));
    let bytes = s.save_bytes(100).unwrap();
    let mut restored = new_session();
    restored.restore_bytes(&bytes).unwrap();
    assert_eq!(restored.recovery.spent_joules, s.recovery.spent_joules);
    assert_eq!(restored.recovery.active, s.recovery.active);
    assert_eq!(restored.save_bytes(100).unwrap(), bytes);
    {
        let field = "recovery";
        let mut bad: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        bad["state"][field] = serde_json::Value::Null;
        assert!(
            restored
                .restore_bytes(&serde_json::to_vec(&bad).unwrap())
                .is_err()
        );
        assert_eq!(restored.save_bytes(100).unwrap(), bytes);
    }
    let mut bad: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    bad["state"]["recovery"]["spent_joules"] = serde_json::json!([-1.0, 0.0]);
    assert!(
        restored
            .restore_bytes(&serde_json::to_vec(&bad).unwrap())
            .is_err()
    );
    let mut legacy: serde_json::Value =
        serde_json::from_slice(&new_session().save_bytes(100).unwrap()).unwrap();
    legacy["schema_version"] = 6.into();
    legacy["state"].as_object_mut().unwrap().remove("recovery");
    legacy["state"]["fabrication"]["machine"]["recipes"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["id"] != "data-recovery-module");
    restored
        .restore_bytes(&serde_json::to_vec(&legacy).unwrap())
        .unwrap();
    assert!(restored.recovery.module.is_none());
    assert!(
        restored
            .fabricator
            .recipes()
            .get(&RecipeId::new("data-recovery-module").unwrap())
            .is_some()
    );
}
fn mine(
    s: &mut GameplaySession,
    resource: ResourceId,
    kg: f64,
    key: u64,
) -> salimon_world::resources::FragmentId {
    let pose =
        ResourceTransform::new(WorldPosition::new(0.0, 0.0, 0.0), [0.0, 0.0, 0.0, 1.0]).unwrap();
    let mut deposit = ResourceDeposit::new(
        DepositId {
            body: CelestialBodyId::Earth,
            local: key,
        },
        RawMaterial::new(resource, kg).unwrap(),
        pose.position(),
        kg,
    )
    .unwrap();
    let mut id = None;
    for _ in 0..500 {
        id = s.mining.session.tick_spherical(
            true,
            Some(&mut deposit),
            SPHERE_TICK,
            MiningConfig::default(),
            pose,
        );
        if id.is_some() {
            break;
        }
    }
    let id = id.unwrap();
    assert!(s.mining.session.pick_up(id));
    s.sync_carried();
    id
}
#[test]
fn mined_carried_fabricated_installed_and_core_funded_first_record_loop() {
    let mut s = new_session();
    for (key, r, kg) in [
        (4680, ResourceId::IronOre, 1.0),
        (4681, ResourceId::SilicateRock, 0.5),
    ] {
        let id = mine(&mut s, r, kg, key);
        s.insert_fabricator_fragment();
        assert!(s.mining.session.fragment(id).is_none());
    }
    enqueue(&mut s, 1).unwrap();
    s.advance_fabrication(Duration::from_secs(60));
    s.collect_fabricator_output();
    s.ship.set_stored_core_energy_joules(0);
    let id = mine(&mut s, ResourceId::IronOre, 10.0, 4682);
    assert_eq!(
        s.ship.intake_carried_fragment(&mut s.mining.session),
        salimon_ship::IntakeResult::Accepted
    );
    assert!(s.mining.session.fragment(id).is_none());
    s.sync_carried();
    s.toggle_recovery("received.damaged-01");
    for _ in 0..650 {
        s.advance_fabrication(Duration::from_secs(1));
        s.advance_recovery(Duration::from_secs(1));
    }
    assert!(s.recovery.complete(0));
    assert_eq!(
        s.recovery.spent_joules[0],
        s.recovery.config.record_cost_joules[0]
    );
    assert!(
        s.recovery
            .record_text("received.damaged-01")
            .unwrap()
            .contains("UNVERIFIED")
    );
    assert!(!s.recovery.complete(1));
}

#[test]
fn native_reader_controls_pause_and_durable_restore_do_not_replay_or_bank_work() {
    let mut s = install();
    let mut value = serde_json::to_value(s.character.save_state()).unwrap();
    value["position"] = serde_json::json!({"Inside":{"local":[-3.0,1.8,-2.8]}});
    value["yaw_radians"] = serde_json::json!(std::f64::consts::PI * 1.5);
    s.character = salimon_character::CharacterController::from_save(
        serde_json::from_value(value).unwrap(),
        None,
    )
    .unwrap();
    s.open_archive();
    for _ in 0..5 {
        s.archive_action(super::super::archive::ReaderAction::Next);
    }
    crate::app::input::dispatch_session_key(
        &mut s,
        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyR),
        true,
    );
    assert_eq!(s.recovery.active, Some(0));
    s.advance_automation(Duration::from_millis(160));
    let spent = s.recovery.spent_joules;
    assert!(spent[0] > 0.0);
    s.command(super::super::Command::TogglePause);
    s.advance_automation(Duration::from_secs(100));
    assert_eq!(s.recovery.spent_joules, spent);
    s.command(super::super::Command::TogglePause);
    let before = s.save_bytes(100).unwrap();
    let mut loaded = new_session();
    loaded.restore_bytes(&before).unwrap();
    assert_eq!(loaded.recovery.spent_joules, spent);
    assert!(!loaded.archive_reader.open);
    loaded.advance_automation(Duration::from_millis(160));
    s.advance_automation(Duration::from_millis(160));
    assert_eq!(loaded.recovery.spent_joules, s.recovery.spent_joules);
    let mut invalid: serde_json::Value =
        serde_json::from_slice(&s.save_bytes(100).unwrap()).unwrap();
    let module = invalid["state"]["recovery"]["module"].clone();
    invalid["state"]["fabrication"]["machine"]["tray"] = module;
    assert!(
        s.restore_bytes(&serde_json::to_vec(&invalid).unwrap())
            .is_err()
    );
}

#[test]
fn concurrent_thrust_crafting_and_recovery_spend_disjoint_exact_joules() {
    let mut s = install();
    for (resource, kg) in [
        (ResourceId::OrganicBiomass, 0.4),
        (ResourceId::WaterIce, 0.1),
    ] {
        s.execute_fabricator(C::Insert(RawMaterial::new(resource, kg).unwrap()))
            .unwrap();
    }
    s.execute_fabricator(C::Enqueue {
        recipe: RecipeId::new("food-ration").unwrap(),
        quantity: 1,
    })
    .unwrap();
    s.ship = salimon_ship::ShipController::flying(
        salimon_ship::ShipPose {
            position_meters: [0.; 3],
            orientation: [0., 0., 0., 1.],
        },
        100,
    );
    s.sync_carried();
    s.toggle_recovery("received.damaged-01");
    let before = s.ship.snapshot();
    let watts = 100_000_000.0 + 110_000.0 + s.fabrication_snapshot().requested_watts;
    for _ in 0..6 {
        s.advance_game(Duration::from_millis(16));
    }
    let after = s.ship.snapshot();
    assert_eq!(s.recovery.spent_joules[0], 5_000_000.0 * 0.096);
    assert_eq!(
        (before.energy_core.stored_joules - after.energy_core.stored_joules) as f64,
        (watts + 5_000_000.0) * 0.096
    );
    assert!(
        (after.velocity_meters_per_second[0] - 100_000_000.0 * 0.096 / before.total_mass_kg).abs()
            < 1e-9
    );
    assert_eq!(
        s.fabrication_snapshot().jobs[0].progress,
        Duration::from_millis(96)
    );
}

#[test]
fn schema_six_initializes_recovery_without_changing_existing_job_resources_or_core() {
    let mut s = new_session();
    for (resource, kg) in [
        (ResourceId::OrganicBiomass, 0.4),
        (ResourceId::WaterIce, 0.1),
        (ResourceId::IronOre, 2.0),
    ] {
        s.execute_fabricator(C::Insert(RawMaterial::new(resource, kg).unwrap()))
            .unwrap();
    }
    s.execute_fabricator(C::Enqueue {
        recipe: RecipeId::new("food-ration").unwrap(),
        quantity: 1,
    })
    .unwrap();
    s.advance_game(Duration::from_millis(16));
    let mut legacy: serde_json::Value =
        serde_json::from_slice(&s.save_bytes(100).unwrap()).unwrap();
    legacy["schema_version"] = 6.into();
    legacy["state"].as_object_mut().unwrap().remove("recovery");
    legacy["state"]["fabrication"]["machine"]["recipes"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["id"] != "data-recovery-module");
    let mut expected = legacy["state"]["fabrication"]["machine"].clone();
    expected.as_object_mut().unwrap().remove("recipes");
    let core = s.ship.snapshot().energy_core;
    s.restore_bytes(&serde_json::to_vec(&legacy).unwrap())
        .unwrap();
    let migrated: serde_json::Value = serde_json::from_slice(&s.save_bytes(100).unwrap()).unwrap();
    let mut actual = migrated["state"]["fabrication"]["machine"].clone();
    actual.as_object_mut().unwrap().remove("recipes");
    assert_eq!(actual, expected);
    assert_eq!(s.ship.snapshot().energy_core, core);
    assert!(s.recovery.module.is_none());
    assert_eq!(s.recovery.spent_joules, [0.0; 2]);
    assert_eq!(s.recovery.active, None);
}

#[test]
fn save_now_disk_relaunch_and_restart_never_bank_or_duplicate_recovery_work() {
    use crate::checkpoints::{
        Checkpoints,
        control::{Request, Status},
    };
    use std::time::Instant;
    let path = std::env::temp_dir().join(format!("salimon-515-{}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    let mut s = install();
    let mut host = Checkpoints::open_at(path.clone(), &mut s, 100).unwrap();
    s.toggle_recovery("received.damaged-01");
    s.advance_game(Duration::from_millis(16));
    let source = serde_json::to_value(&s.recovery).unwrap();
    let mass = s.ship.snapshot().total_mass_kg;
    let energy = s.ship.snapshot().energy_core;
    s.toggle_pause();
    host.request(Request::SaveNow, &s);
    let deadline = Instant::now() + Duration::from_secs(10);
    while !matches!(host.status().0, Status::Saved { .. }) {
        assert!(
            Instant::now() < deadline,
            "recovery Save Now timeout: {:?}",
            host.status()
        );
        host.poll(&s);
        std::thread::yield_now();
    }
    let utc = host.status().1.unwrap();
    drop(host);
    let mut loaded = GameplaySession::default();
    // Real closed-game service/Recovery work settles before publication.
    let mut host = Checkpoints::open_at(path.clone(), &mut loaded, utc + 7 * 86400).unwrap();
    assert!(loaded.recovery.complete(0));
    assert!(loaded.recovery.spent_joules[0] >= source["spent_joules"][0].as_f64().unwrap());
    assert_eq!(loaded.ship.snapshot().total_mass_kg, mass);
    assert_eq!(loaded.ship.snapshot().energy_core.stored_joules, 0);
    assert!(loaded.ship.snapshot().energy_core.stored_joules <= energy.stored_joules);
    assert!(!loaded.archive_reader.open);
    loaded
        .survival
        .command(salimon_survival::Command::Damage(
            salimon_survival::Points::FULL,
        ))
        .unwrap();
    host.restart(&mut loaded).unwrap();
    assert!(loaded.recovery.module.is_none());
    assert_eq!(loaded.recovery.spent_joules, [0.; 2]);
    assert_eq!(loaded.recovery.active, None);
    drop(host);
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
#[ignore = "writes CPU Archive raster evidence; set SALIMON_RECOVERY_EVIDENCE"]
fn recovery_ui_evidence() {
    let dir = std::path::PathBuf::from(std::env::var("SALIMON_RECOVERY_EVIDENCE").unwrap());
    std::fs::create_dir_all(&dir).unwrap();
    let mut s = install();
    let mut v = serde_json::to_value(s.character.save_state()).unwrap();
    v["position"] = serde_json::json!({"Inside":{"local":[-3.0,1.8,-2.8]}});
    v["yaw_radians"] = serde_json::json!(std::f64::consts::PI * 1.5);
    s.character =
        salimon_character::CharacterController::from_save(serde_json::from_value(v).unwrap(), None)
            .unwrap();
    s.open_archive();
    for _ in 0..5 {
        s.archive_action(super::super::archive::ReaderAction::Next);
    }
    s.recovery.spent_joules[0] = s.recovery.config.record_cost_joules[0];
    let text = s.archive_text().unwrap();
    assert!(text.lines().all(|l| l.len() <= 58));
    for scale in [2, 3] {
        let (w, h, pixels) = crate::action_bar::rasterize_action_bar(&text, scale);
        assert!(w <= 1248 && h <= 768);
        std::fs::write(dir.join(format!("recovered-{scale}.rgba")), pixels).unwrap();
        std::fs::write(
            dir.join(format!("recovered-{scale}.size")),
            format!("{w} {h}"),
        )
        .unwrap();
    }
}
