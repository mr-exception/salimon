use super::*;
use salimon_fabrication::{ConsumableSlot, FabricationCommand as C, JobId, RecipeId};
use salimon_survival::{Command as VitalCommand, Points};

fn consume(s: &mut GameplaySession, slot: ConsumableSlot) {
    let item = s.consumables.item(slot).unwrap();
    let selection = consumables::Selection {
        slot,
        job: item.job,
        unit: item.unit,
    };
    s.consume_selection(selection).unwrap();
    assert!(s.consume_selection(selection).is_err());
}

#[test]
fn new_run_crafts_exactly_one_extra_ration_and_resaves_without_regranting() {
    let mut s = GameplaySession::new_run();
    assert_eq!(
        s.snapshot().ship.total_mass_kg,
        salimon_ship::STARTER_DRY_MASS_KG + 0.5
    );
    assert_eq!(s.consumables.food().iter().flatten().count(), 2);
    assert_eq!(s.consumables.medical().iter().flatten().count(), 1);
    assert_eq!(s.survival.snapshot().active_time, Duration::ZERO);
    let start_energy = s.ship.snapshot().energy_core.stored_joules;
    assert_eq!(
        start_energy,
        GameplaySession::default()
            .ship
            .snapshot()
            .energy_core
            .stored_joules
    );
    let before = s.save_bytes(100).unwrap();
    s.restore_bytes(&before).unwrap();
    assert_eq!(s.save_bytes(100).unwrap(), before);
    s.execute_fabricator(C::Enqueue {
        recipe: RecipeId::new("food-ration").unwrap(),
        quantity: 1,
    })
    .unwrap();
    assert_eq!(
        s.snapshot().ship.total_mass_kg,
        salimon_ship::STARTER_DRY_MASS_KG
    );
    s.advance_automation(Duration::from_secs(10));
    let output = s.fabrication_snapshot().tray.unwrap();
    assert_eq!((output.job, output.unit), (JobId(3), 1));
    assert_eq!(
        s.snapshot().ship.total_mass_kg,
        salimon_ship::STARTER_DRY_MASS_KG + output.mass.kg()
    );
    s.collect_fabricator_output();
    assert_eq!(s.consumables.food().iter().flatten().count(), 3);
    assert_eq!(
        s.snapshot().ship.total_mass_kg,
        salimon_ship::STARTER_DRY_MASS_KG
    );
    assert!(
        s.fabrication_snapshot()
            .stored
            .iter()
            .all(|b| b.milligrams == 0)
    );
    assert!(
        s.execute_fabricator(C::Enqueue {
            recipe: RecipeId::new("food-ration").unwrap(),
            quantity: 1
        })
        .is_err()
    );
    consume(&mut s, ConsumableSlot::FoodOne);
    let saved = s.save_bytes(101).unwrap();
    for _ in 0..3 {
        // Even a pre-populated restore destination must use the saved vacancies.
        let mut restored = GameplaySession::new_run();
        restored.restore_bytes(&saved).unwrap();
        assert_eq!(restored.save_bytes(101).unwrap(), saved);
        assert!(restored.consumables.item(ConsumableSlot::FoodOne).is_none());
        s = restored;
    }
    let inventory = s.consumables.clone();
    let machine = s.fabricator.clone();
    for _ in 0..3 {
        s.character.enter_cockpit();
        s.character.leave_cockpit();
        s.clear_held_controls(); // Shared focus/cursor/lifecycle latch reset.
        s.toggle_view();
        s.toggle_view();
        s.advance_automation(Duration::from_millis(16));
    }
    assert_eq!(s.consumables, inventory);
    assert_eq!(s.fabricator, machine);
}

#[test]
fn old_empty_saves_and_launch_recovery_never_receive_new_run_grants() {
    for version in [1, 2, 3] {
        let empty = GameplaySession::default();
        let mut value: serde_json::Value =
            serde_json::from_slice(&empty.save_bytes(100).unwrap()).unwrap();
        value["schema_version"] = serde_json::json!(version);
        if version == 1 {
            value["state"].as_object_mut().unwrap().remove("survival");
        }
        let mut restored = GameplaySession::new_run();
        restored
            .restore_for_launch(&serde_json::to_vec(&value).unwrap(), 200)
            .unwrap();
        assert_eq!(restored.consumables, Default::default());
        assert!(
            restored
                .fabrication_snapshot()
                .stored
                .iter()
                .all(|b| b.milligrams == 0)
        );
        assert_eq!(
            restored.snapshot().ship.total_mass_kg,
            salimon_ship::STARTER_DRY_MASS_KG
        );
    }
}

#[test]
fn verified_native_launch_restart_and_reload_publish_supplies_once() {
    let directory = std::env::temp_dir().join(format!(
        "salimon-379-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut s = GameplaySession::new_run();
    let earth = *salimon_world::CELESTIAL_BODIES
        .iter()
        .find(|b| b.id == salimon_world::CelestialBodyId::Earth)
        .unwrap();
    s.enable_terrain(earth, [0.0, 1.0, 0.0]).unwrap();
    assert_eq!(
        s.ship.snapshot().total_mass_kg,
        salimon_ship::STARTER_DRY_MASS_KG + 0.5
    );
    let mut checkpoints =
        crate::checkpoints::Checkpoints::open_at(directory.clone(), &mut s, 100).unwrap();
    consume(&mut s, ConsumableSlot::FoodOne);
    consume(&mut s, ConsumableSlot::MedicalOne);
    checkpoints.finish(&s).unwrap();
    drop(checkpoints);
    let mut restored = GameplaySession::new_run();
    let mut checkpoints =
        crate::checkpoints::Checkpoints::open_at(directory.clone(), &mut restored, 100).unwrap();
    assert_eq!(restored.consumables, s.consumables);
    assert_eq!(restored.fabricator, s.fabricator);
    restored
        .survival
        .command(VitalCommand::Damage(Points::FULL))
        .unwrap();
    restored.advance_automation(Duration::from_millis(16));
    checkpoints.restart(&mut restored).unwrap();
    // Restart writes a real current-clock canonical journey; its fixture must
    // not reopen it in 1970 and bypass the verified-clock admission gate.
    let restarted_at = crate::checkpoints::now().unwrap();
    assert_eq!(restored.run.serial, 2);
    let expected = GameplaySession::new_run();
    assert_eq!(restored.consumables, expected.consumables);
    assert_eq!(restored.fabricator, expected.fabricator);
    assert!(checkpoints.restart(&mut restored).is_err());
    drop(checkpoints);
    // Both fallback generations, not just the newest, contain one starter grant.
    for slot in 0..2 {
        let single = directory.join(format!("only-slot-{slot}"));
        std::fs::create_dir_all(&single).unwrap();
        std::fs::copy(
            directory.join(format!("checkpoint-{slot}.sav")),
            single.join(format!("checkpoint-{slot}.sav")),
        )
        .unwrap();
        let mut recovered = GameplaySession::default();
        let checkpoint =
            crate::checkpoints::Checkpoints::open_at(single, &mut recovered, restarted_at).unwrap();
        assert_eq!(recovered.run.serial, 2);
        assert_eq!(recovered.consumables, expected.consumables);
        assert_eq!(recovered.fabricator, expected.fabricator);
        drop(checkpoint);
    }
    let mut reloaded = GameplaySession::default();
    let checkpoints =
        crate::checkpoints::Checkpoints::open_at(directory.clone(), &mut reloaded, restarted_at)
            .unwrap();
    assert_eq!(reloaded.consumables, expected.consumables);
    assert_eq!(reloaded.fabricator, expected.fabricator);
    drop(checkpoints);
    std::fs::remove_dir_all(directory).unwrap();
}
