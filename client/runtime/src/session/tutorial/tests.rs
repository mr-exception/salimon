use super::*;
use crate::session::{Command, consumable_wheel::WheelAction};
use salimon_fabrication::{ConsumableSlot, FabricationCommand, RecipeId};
use std::time::Duration;

fn eat(s: &mut GameplaySession) {
    for action in [
        WheelAction::Hold(true),
        WheelAction::Hover(Some(ConsumableSlot::FoodOne)),
        WheelAction::Primary(true),
        WheelAction::Primary(false),
        WheelAction::Hold(false),
    ] {
        s.command(Command::Wheel(action));
    }
}
#[test]
fn early_actions_complete_once_and_survive_reload_without_grants() {
    let mut s = GameplaySession::new_run();
    eat(&mut s); // Actions count even before their guided step.
    assert_eq!(s.tutorial.completed, 0b110);
    assert!(s.tutorial_text().unwrap().contains("H:"));
    let inventory = s.consumables.clone();
    s.execute_fabricator(FabricationCommand::Enqueue {
        recipe: RecipeId::new("food-ration").unwrap(),
        quantity: 1,
    })
    .unwrap();
    s.advance_automation(Duration::from_secs(10));
    assert_ne!(s.tutorial.completed & (1 << Objective::Craft as u8), 0);
    s.collect_fabricator_output();
    let bytes = s.save_bytes(100).unwrap();
    let mut loaded = GameplaySession::new_run();
    loaded.restore_bytes(&bytes).unwrap();
    assert_eq!(loaded.save_bytes(100).unwrap(), bytes);
    assert_eq!(loaded.tutorial, s.tutorial);
    assert_eq!(loaded.consumables.food().iter().flatten().count(), 2);
    assert_eq!(inventory.food().iter().flatten().count(), 1);
    loaded.command(Command::InspectVitals);
    assert!(matches!(
        loaded.tutorial.current(),
        Some(Objective::Fabricator)
    ));
    let eye = [-3.0, 1.65, 2.0];
    let target = salimon_character::FABRICATOR_MONITOR_MARKER_METERS;
    let d: [f64; 3] = std::array::from_fn(|i| target[i] - eye[i]);
    let mut character = serde_json::to_value(loaded.character.save_state()).unwrap();
    character["position"] = serde_json::json!({"Inside":{"local":eye}});
    character["yaw_radians"] = d[2].atan2(d[0]).into();
    character["pitch_radians"] = d[1].atan2(d[0].hypot(d[2])).into();
    loaded.character = salimon_character::CharacterController::from_save(
        serde_json::from_value(character).unwrap(),
        None,
    )
    .unwrap();
    loaded.command(Command::Interact);
    assert!(loaded.crafting_ui.open);
    loaded.clear_held_controls();
    assert!(matches!(
        loaded.tutorial.current(),
        Some(Objective::SaveStatus)
    ));
    loaded.command(Command::TogglePause);
    assert!(loaded.tutorial.current().is_none());
    loaded.command(Command::TogglePause);
    assert!(loaded.tutorial_text().is_none());
}
#[test]
fn skip_is_durable_focus_and_modal_safe_and_restart_is_fresh() {
    let mut s = GameplaySession::new_run();
    let before = s.consumables.clone();
    s.command(Command::Wheel(WheelAction::Hold(true)));
    s.command(Command::SkipTutorial);
    assert!(s.wheel.open); // Skip has no input/clock authority.
    s.clear_held_controls();
    assert!(s.tutorial_text().is_none());
    let bytes = s.save_bytes(200).unwrap();
    s.restore_bytes(&bytes).unwrap();
    s.command(Command::InspectVitals);
    assert!(s.tutorial_text().is_none());
    assert_eq!(before, s.consumables);
    s.survival
        .command(salimon_survival::Command::Damage(
            salimon_survival::Points::FULL,
        ))
        .unwrap();
    assert!(s.tutorial_text().is_none());
    let fresh = s.fresh_after_death().unwrap();
    assert!(matches!(fresh.tutorial.current(), Some(Objective::Vitals)));
    assert_eq!(fresh.tutorial.completed, 0);
}
#[test]
fn legacy_migration_invalid_state_and_missing_source_are_transactional() {
    let mut s = GameplaySession::new_run();
    let before = s.save_bytes(100).unwrap();
    for version in [1, 2, 3] {
        let mut old: serde_json::Value = serde_json::from_slice(&before).unwrap();
        old["schema_version"] = version.into();
        old["state"].as_object_mut().unwrap().remove("mounts");
        old["state"]["habitation"]
            .as_object_mut()
            .unwrap()
            .remove("cabin");
        old["state"].as_object_mut().unwrap().remove("tutorial");
        if version == 1 {
            old["state"].as_object_mut().unwrap().remove("survival");
        }
        s.restore_bytes(&serde_json::to_vec(&old).unwrap()).unwrap();
        assert!(s.tutorial.current().is_none());
    }
    s.restore_bytes(&before).unwrap();
    for payload in [
        serde_json::Value::Null,
        serde_json::json!({"completed":128,"skipped":false}),
    ] {
        let mut bad: serde_json::Value = serde_json::from_slice(&before).unwrap();
        bad["state"]["tutorial"] = payload;
        assert!(s.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
        assert_eq!(s.save_bytes(100).unwrap(), before);
    }
}
#[test]
fn tutorial_never_pauses_or_blocks_world_and_modal_overlays_hide_it() {
    let mut s = GameplaySession::new_run();
    s.command(Command::Wheel(WheelAction::Hold(true)));
    assert!(s.tutorial_text().is_none());
    s.advance_automation(Duration::from_secs(1));
    assert!(!s.elapsed.is_zero());
    s.clear_held_controls();
    s.crafting_ui.open = true;
    assert!(s.tutorial_text().is_none());
    s.clear_held_controls();
    s.command(Command::TogglePause);
    let frozen = s.save_bytes(100).unwrap();
    s.command(Command::SkipTutorial);
    s.advance_automation(Duration::from_secs(300));
    assert_eq!(s.save_bytes(100).unwrap(), frozen);
    s.command(Command::TogglePause);
    s.command(Command::Control(crate::session::ControlKey::Forward, true));
    assert!(s.input.movement.forward);
}
#[test]
fn all_prompts_fit_small_drawables_above_the_toolbar() {
    let mut tutorial = Tutorial::default();
    for objective in OBJECTIVES {
        let text = tutorial.text().unwrap();
        let (w, h, pixels) = crate::action_bar::rasterize_action_bar(&text, 2);
        // Renderer fits to its 16px margins; two lines stay clear of the HUD.
        assert!(w <= 608 && h <= 48, "{w}x{h}: {text}");
        assert_eq!(pixels.len(), w as usize * h as usize * 4);
        tutorial.record(objective);
    }
    assert!(tutorial.text().is_none());
}
