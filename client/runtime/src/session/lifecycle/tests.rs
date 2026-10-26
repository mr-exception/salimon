use super::*;
use crate::session::{Command, ControlKey};
use salimon_survival::{Command as VitalCommand, Points};
use serde_json::json;
fn low(s: &mut GameplaySession) {
    let mut v = serde_json::to_value(&s.survival).unwrap();
    v["hunger"] = serde_json::to_value(Points::ZERO).unwrap();
    v["health"] = serde_json::to_value(Points::whole(1).unwrap()).unwrap();
    v["starvation_elapsed"] = serde_json::to_value(Duration::from_millis(9995)).unwrap();
    s.survival = serde_json::from_value(v).unwrap();
    s.survival.validate().unwrap();
}
#[test]
fn death_truncates_world_to_lethal_boundary_and_freezes_all_future_work_and_commands() {
    let mut s = GameplaySession {
        ship: salimon_ship::ShipController::flying_with_velocity(
            salimon_ship::ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
            [100.0, 0.0, 0.0],
        )
        .unwrap(),
        ..Default::default()
    };
    low(&mut s);
    s.advance_game(Duration::from_millis(16));
    assert!(s.survival.snapshot().terminal);
    assert_eq!(s.elapsed, Duration::from_millis(5));
    assert_eq!(s.snapshot().ship.pose.position_meters, [0.5, 0.0, 0.0]);
    let frozen = s.save_bytes(1).unwrap();
    for command in [
        Command::Control(ControlKey::Forward, true),
        Command::Look(10.0, 20.0),
        Command::Interact,
        Command::Landing,
        Command::Mining(true),
        Command::ToggleView,
    ] {
        s.command(command);
    }
    s.apply_look_delta(100.0, 100.0);
    s.advance_automation(Duration::from_secs(100));
    assert!(s.save_bytes(1).unwrap() == frozen);
    assert_eq!(s.update_clock.backlog(), Duration::ZERO);
}
#[test]
fn whole_game_roundtrip_includes_active_healing_fraction_configuration_and_journey_source() {
    let mut s = GameplaySession::default();
    s.survival
        .command(VitalCommand::Damage(Points::whole(60).unwrap()))
        .unwrap();
    s.survival
        .apply_consumable(s.consumable_effects.get("medkit").unwrap())
        .unwrap();
    s.advance_automation(Duration::from_millis(1375));
    s.run.journey.distance_meters = 123.5;
    let save = s.save_bytes(100).unwrap();
    let mut restored = GameplaySession::default();
    restored.restore_bytes(&save).unwrap();
    assert!(restored.save_bytes(100).unwrap() == save);
    for ms in [9, 100, 1003] {
        s.advance_automation(Duration::from_millis(ms));
        restored.advance_automation(Duration::from_millis(ms));
    }
    assert!(s.save_bytes(100).unwrap() == restored.save_bytes(100).unwrap());
    assert_eq!(restored.run.journey.distance_meters, 123.5);
}
#[test]
fn schema_one_migrates_only_missing_survival_and_schema_two_requires_complete_state() {
    let mut s = GameplaySession::default();
    s.advance_automation(Duration::from_secs(3));
    let expected = s.snapshot().ship;
    let mut v: serde_json::Value = serde_json::from_slice(&s.save_bytes(10).unwrap()).unwrap();
    v["schema_version"] = json!(1);
    v["state"].as_object_mut().unwrap().remove("surface_age");
    v["state"].as_object_mut().unwrap().remove("survival");
    let mut restored = GameplaySession::default();
    restored
        .restore_bytes(&serde_json::to_vec(&v).unwrap())
        .unwrap();
    assert_eq!(restored.snapshot().ship, expected);
    assert_eq!(restored.survival.snapshot().hunger, Points::FULL);
    assert_eq!(restored.survival.snapshot().active_time, Duration::ZERO);
    let original = restored.save_bytes(10).unwrap();
    v["schema_version"] = json!(2);
    assert!(
        restored
            .restore_bytes(&serde_json::to_vec(&v).unwrap())
            .is_err()
    );
    assert!(original == restored.save_bytes(10).unwrap());
    v["schema_version"] = json!(salimon_persistence::SCHEMA_VERSION + 1);
    assert!(matches!(
        restored.restore_bytes(&serde_json::to_vec(&v).unwrap()),
        Err(SaveError::UnsupportedVersion(v)) if v == salimon_persistence::SCHEMA_VERSION + 1
    ));
}
#[test]
fn active_menu_time_advances_but_restore_resets_ui_and_terminal_backlog() {
    let mut s = GameplaySession::default();
    s.toggle_view();
    s.advance_automation(Duration::from_secs(1));
    assert_eq!(
        s.survival.snapshot().active_time,
        Duration::from_millis(992)
    );
    s.crafting_ui.open = true;
    s.input.movement.forward = true;
    let bytes = s.save_bytes(1).unwrap();
    let mut restored = GameplaySession::default();
    restored.restore_bytes(&bytes).unwrap();
    assert_eq!(restored.view_mode, ViewMode::Gameplay);
    assert!(!restored.crafting_ui.open);
    assert!(!restored.input.movement.forward);
    restored
        .survival
        .command(VitalCommand::Damage(Points::FULL))
        .unwrap();
    restored.update_clock.accumulate(Duration::from_secs(100));
    let dead = restored.save_bytes(1).unwrap();
    s.restore_bytes(&dead).unwrap();
    assert!(s.survival.snapshot().terminal);
    assert_eq!(s.update_clock.backlog(), Duration::ZERO);
}
#[test]
fn fresh_restart_rebuilds_state_and_monotonic_run_identity_without_reusing_old_inputs() {
    let mut s = GameplaySession::default();
    assert!(s.fresh_after_death().is_err());
    s.seed = 371;
    s.run.journey.distance_meters = 100.0;
    s.survival
        .command(VitalCommand::Damage(Points::FULL))
        .unwrap();
    s.input.movement.forward = true;
    let fresh = s.fresh_after_death().unwrap();
    assert_eq!(fresh.seed, 371);
    assert_eq!(fresh.run.serial, 2);
    assert_eq!(fresh.run.journey.distance_meters, 0.0);
    assert_eq!(fresh.elapsed, Duration::ZERO);
    assert!(!fresh.input.movement.forward);
    assert_eq!(fresh.survival.snapshot().health, Points::FULL);
    assert!(fresh.mining.session.fragments().is_empty());
    assert_eq!(fresh.consumables, GameplaySession::new_run().consumables);
    assert_eq!(s.ended_run_bytes().unwrap(), s.ended_run_bytes().unwrap());
}

#[test]
fn consumed_crafted_medkit_and_partial_healing_share_one_whole_game_checkpoint() {
    use salimon_fabrication::{ConsumableSlot, FabricationCommand as C, RecipeId};
    use salimon_world::resources::{RawMaterial, ResourceId};
    let mut s = GameplaySession::default();
    for id in [ResourceId::MedicinalCompounds, ResourceId::WaterIce] {
        s.execute_fabricator(C::Insert(RawMaterial::new(id, 1.0).unwrap()))
            .unwrap();
    }
    s.execute_fabricator(C::Enqueue {
        recipe: RecipeId::new("medkit").unwrap(),
        quantity: 1,
    })
    .unwrap();
    s.advance_automation(Duration::from_millis(15008));
    s.collect_fabricator_output();
    s.survival
        .command(VitalCommand::Damage(Points::whole(60).unwrap()))
        .unwrap();
    let item = s.consumables.item(ConsumableSlot::MedicalOne).unwrap();
    let selection = crate::session::consumables::Selection {
        slot: ConsumableSlot::MedicalOne,
        job: item.job,
        unit: item.unit,
    };
    s.consume_selection(selection).unwrap();
    s.advance_automation(Duration::from_millis(1375));
    let saved = s.save_bytes(100).unwrap();
    let mut restored = GameplaySession::default();
    restored.restore_bytes(&saved).unwrap();
    assert!(restored.consumables.medical().iter().all(Option::is_none));
    assert!(restored.consume_selection(selection).is_err());
    assert!(restored.save_bytes(100).unwrap() == saved);
    let mut bad: serde_json::Value = serde_json::from_slice(&saved).unwrap();
    bad["state"]["survival"]["catalog"][1]["benefits"][0]["OverTime"]["milli_points"] = json!(3000);
    assert!(
        restored
            .restore_bytes(&serde_json::to_vec(&bad).unwrap())
            .is_err()
    );
    assert!(restored.save_bytes(100).unwrap() == saved);
    for owner in [&mut s, &mut restored] {
        owner.advance_automation(Duration::from_millis(8640));
    }
    assert_eq!(
        restored.survival.snapshot().health,
        Points::whole(65).unwrap()
    );
    assert!(restored.save_bytes(100).unwrap() == s.save_bytes(100).unwrap());
}
