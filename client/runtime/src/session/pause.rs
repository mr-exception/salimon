//! Transient modal authority. A paused snapshot remains saveable; the menu is never saved.
use super::{GameplaySession, ViewMode};
impl GameplaySession {
    pub(crate) fn is_paused(&self) -> bool {
        self.paused
    }
    pub(crate) fn toggle_pause(&mut self) {
        if self.view_mode != ViewMode::Gameplay || self.survival.snapshot().terminal {
            return;
        }
        self.clear_held_controls();
        self.update_clock.reset();
        self.reset_presentation();
        self.paused = !self.paused;
    }
    pub(crate) fn pause_text(&self) -> Option<String> {
        self.paused.then(|| "SALIMON / PAUSED\n\nENTER / ESC   RESUME\nS   ACCESSIBILITY SETTINGS\n\nAll simulation is frozen.\nF1 releases the cursor during gameplay.".into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{Command, ControlKey};
    use std::time::Duration;
    #[test]
    fn pause_freezes_full_save_and_rejects_inputs_without_resume_backlog() {
        let mut s = GameplaySession::new_run();
        s.execute_fabricator(salimon_fabrication::FabricationCommand::Enqueue {
            recipe: salimon_fabrication::RecipeId::new("food-ration").unwrap(),
            quantity: 1,
        })
        .unwrap();
        s.command(Command::Control(ControlKey::Forward, true));
        s.update_clock.accumulate(Duration::from_secs(4));
        s.command(Command::TogglePause);
        assert!(s.is_paused());
        let before = s.save_bytes(123).unwrap();
        for c in [
            Command::Control(ControlKey::Forward, true),
            Command::Interact,
            Command::Look(100.0, 100.0),
            Command::Mining(true),
            Command::Landing,
            Command::ToggleView,
            Command::GrabDrop(true),
        ] {
            s.command(c);
        }
        s.advance_game(Duration::from_secs(86400));
        s.advance_automation(Duration::from_secs(86400));
        assert_eq!(s.save_bytes(123).unwrap(), before);
        assert_eq!(s.update_clock.backlog(), Duration::ZERO);
        assert!(!s.movement_input.forward);
        let mut restored = GameplaySession::default();
        restored.restore_bytes(&before).unwrap();
        assert!(!restored.is_paused());
        s.command(Command::TogglePause);
        s.advance_automation(Duration::from_millis(16));
        assert_eq!(s.elapsed, Duration::from_millis(16));
        assert!(!s.movement_input.forward);
    }
    #[test]
    fn tour_pause_is_independent() {
        let mut s = GameplaySession::default();
        s.command(Command::ToggleView);
        s.command(Command::TogglePause);
        assert!(!s.is_paused());
        assert_eq!(s.view_mode, ViewMode::PrecisionTour);
    }
}
