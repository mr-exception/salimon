//! Opt-in initial conditions for #489. Subsequent commands use production gates.
use crate::{e2e::Scenario, session::GameplaySession};
use salimon_fabrication::{FabricationCommand, RecipeId};

/// Explicit initial placement at the Archive; never opens the reader or marks records.
pub(crate) fn archive_initial_conditions(s: &mut GameplaySession) -> Result<(), String> {
    *s = GameplaySession::new_run();
    let mut source = serde_json::to_value(s.character.save_state()).map_err(|e| e.to_string())?;
    source["position"] = serde_json::json!({"Inside":{"local":[-3.0,1.8,-2.8]}});
    source["yaw_radians"] = serde_json::json!(std::f64::consts::PI * 1.5);
    source["pitch_radians"] = serde_json::json!(0.0);
    s.character = salimon_character::CharacterController::from_save(
        serde_json::from_value(source).map_err(|e| e.to_string())?,
        None,
    )
    .map_err(|e| e.to_string())?;
    s.save_bytes(100).map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn initialize(s: &mut GameplaySession, scenario: Scenario) -> Result<(), String> {
    *s = GameplaySession::new_run();
    // Reuse the canonical exact 2 kg held identity and Core aim fixture. This is
    // explicit fixture placement, never evidence of mining or physical travel.
    crate::flight_regression::initialize(s, Scenario::CoreRestart)?;
    if scenario == Scenario::ScoutWorkBlackout {
        s.execute_fabricator(FabricationCommand::Enqueue {
            recipe: RecipeId::new("food-ration").map_err(|e| format!("recipe: {e:?}"))?,
            quantity: 1,
        })
        .map_err(|e| format!("work fixture: {e:?}"))?;
    } else {
        s.ship = salimon_ship::ShipController::default();
        let mut source =
            serde_json::to_value(s.character.save_state()).map_err(|e| e.to_string())?;
        source["position"] = serde_json::json!({"Inside":{"local":[-3.8,1.8,1.3]}});
        source["yaw_radians"] = serde_json::json!(std::f64::consts::FRAC_PI_2);
        source["pitch_radians"] = serde_json::json!(0.0);
        s.character = salimon_character::CharacterController::from_save(
            serde_json::from_value(source).map_err(|e| e.to_string())?,
            None,
        )
        .map_err(|e| e.to_string())?;
        s.character.set_ship_gravity_enabled(true);
        s.mining.ship_gravity_disabled = false;
    }
    s.sync_carried();
    s.save_bytes(100)
        .map_err(|e| format!("invalid machinery fixture: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        e2e,
        session::{Command, crafting_ui::CraftingAction},
    };
    use salimon_renderer::MachineryState;
    use std::time::Duration;
    fn setup(scenario: Scenario) -> GameplaySession {
        let mut s = GameplaySession::default();
        e2e::initialize(
            &mut s,
            e2e::Config {
                scenario,
                seed: 0,
                step: Duration::from_millis(100),
                fragment_load: None,
            },
        )
        .unwrap();
        s
    }
    fn aim(s: &mut GameplaySession, target: [f64; 3]) {
        let snap = s.snapshot();
        let frame = crate::session::frames::character_ship_frame(snap.ship.pose);
        let eye = frame.world_to_local(snap.character.eye_position_meters);
        let look = frame.world_to_local(snap.character.look_target_meters);
        let old = salimon_math::sub(look, eye);
        let next = salimon_math::sub(target, eye);
        let yaw = (next[2].atan2(next[0]) - old[2].atan2(old[0]) + std::f64::consts::PI)
            .rem_euclid(std::f64::consts::TAU)
            - std::f64::consts::PI;
        let pitch = next[1].atan2(next[0].hypot(next[2])) - old[1].atan2(old[0].hypot(old[2]));
        s.command(Command::Look(yaw / 0.0022, -pitch / 0.0022));
    }
    fn feedback(s: &GameplaySession) -> salimon_renderer::MachineryFeedback {
        crate::scout_feedback::capture(s, s.ship.snapshot())
    }
    #[test]
    fn physical_intake_menu_work_pause_restore_blocked_and_exact_collection() {
        let mut s = setup(Scenario::ScoutFabricator);
        let id = s.mining.session.carried_id().unwrap();
        let mass = s.ship.snapshot().total_mass_kg;
        aim(&mut s, salimon_character::FABRICATOR_INPUT_MARKER_METERS);
        s.command(Command::Interact);
        assert!(s.mining.session.carried_id().is_none());
        assert!(!s.mining.session.fragments().iter().any(|p| p.id() == id));
        assert_eq!(mass, s.ship.snapshot().total_mass_kg);
        aim(&mut s, salimon_character::FABRICATOR_MONITOR_MARKER_METERS);
        s.command(Command::Interact);
        assert!(s.crafting_ui.open);
        s.crafting_action(CraftingAction::Add);
        assert_eq!(s.fabrication_snapshot().jobs.len(), 1);
        s.crafting_action(CraftingAction::Close);
        s.advance_game(Duration::from_secs(1));
        let active = feedback(&s);
        assert_eq!(active.fabricator, MachineryState::Working);
        assert!(active.fabrication_progress > 0.);
        s.toggle_pause();
        let bytes = s.save_bytes(100).unwrap();
        s.advance_automation(Duration::from_secs(30));
        assert_eq!(feedback(&s), active);
        assert_eq!(s.save_bytes(100).unwrap(), bytes);
        let mut restored = GameplaySession::default();
        restored.restore_bytes(&bytes).unwrap();
        assert_eq!(feedback(&restored), active);
        assert_eq!(restored.mining.session.carried_id(), None);
        s.toggle_pause();
        s.advance_game(Duration::from_secs(10));
        assert_eq!(feedback(&s).fabricator, MachineryState::Blocked);
        let item = s.fabrication_snapshot().tray.unwrap();
        let before = s.consumables.food().iter().flatten().count();
        aim(&mut s, salimon_character::FABRICATOR_TRAY_MARKER_METERS);
        s.command(Command::Interact);
        assert!(s.fabrication_snapshot().tray.is_none());
        assert_eq!(s.consumables.food().iter().flatten().count(), before + 1);
        assert!(
            s.consumables
                .food()
                .iter()
                .flatten()
                .any(|p| p.job == item.job && p.unit == item.unit)
        );
        let saved = s.save_bytes(100).unwrap();
        s.command(Command::Interact);
        assert_eq!(s.save_bytes(100).unwrap(), saved, "duplicate collection");
    }
    #[test]
    fn blackout_job_resumes_only_after_real_core_intake_and_funded_ticks() {
        let mut s = setup(Scenario::ScoutWorkBlackout);
        assert_eq!(feedback(&s).fabricator, MachineryState::Unpowered);
        let id = s.mining.session.carried_id().unwrap();
        s.advance_game(Duration::from_millis(100));
        assert_eq!(feedback(&s).fabrication_progress, 0.);
        assert!(!s.ship.snapshot().power_available);
        s.command(Command::Interact);
        assert!(s.mining.session.carried_id().is_none());
        assert!(!s.mining.session.fragments().iter().any(|p| p.id() == id));
        s.advance_game(Duration::from_millis(100));
        assert!(s.ship.snapshot().power_available);
        let on = feedback(&s);
        assert!(on.converting);
        assert_eq!(on.fabricator, MachineryState::Working);
        assert!(on.fabrication_progress > 0.);
        let bytes = s.save_bytes(100).unwrap();
        let mut restored = GameplaySession::default();
        restored.restore_bytes(&bytes).unwrap();
        assert_eq!(feedback(&restored), on);
    }
}
