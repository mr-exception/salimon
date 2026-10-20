//! Compose snapshot-derived permissions at the shared native/automation boundary.
use super::{GameplaySession, ShipControlInput, interaction::InteractionTarget};
use salimon_survival::restrictions::{Action, Restrictions};
use std::time::Duration;

impl GameplaySession {
    pub(crate) fn restrictions(&self) -> Restrictions {
        Restrictions::new(self.survival.snapshot(), self.restriction_config)
            .expect("session restriction tuning validated before installation")
    }

    pub(super) fn enforce_survival_restrictions(&mut self) {
        let rules = self.restrictions();
        self.character.set_movement_scale(if rules.terminal {
            0.0
        } else {
            rules.movement_scale()
        });
        if !rules.permits(Action::Pilot) {
            self.ship.revoke_pilot_control();
            self.ship_control_input = ShipControlInput::default();
        }
        if !rules.permits(Action::Mine) {
            self.mining.stop_mining();
        }
        if !rules.permits(Action::Fabricate) {
            self.crafting_ui.open = false;
        }
    }

    pub(super) fn admit_survival_action(&mut self, action: Action) -> bool {
        if self.is_paused() {
            return false;
        }
        let rules = self.restrictions();
        if !rules.permits(action) {
            self.feedback = Some("Too fatigued for this action");
            return false;
        }
        // Emergency actions and menus never share nonessential recovery time.
        if matches!(
            action,
            Action::Walk
                | Action::Consume
                | Action::Door
                | Action::Menu
                | Action::Pilot
                | Action::Mine
        ) {
            return true;
        }
        if self.elapsed < self.interaction_ready_at {
            self.feedback = Some("Recovering from interaction");
            return false;
        }
        self.interaction_ready_at = self.elapsed.saturating_add(rules.interaction_delay());
        true
    }

    pub(super) fn interaction_action(target: InteractionTarget) -> Action {
        match target {
            InteractionTarget::Cockpit => Action::Pilot,
            InteractionTarget::ExitDoor => Action::Door,
            InteractionTarget::Core => Action::FeedCore,
            InteractionTarget::Fabricator
            | InteractionTarget::FabricatorScreen
            | InteractionTarget::FabricatorTray => Action::Fabricate,
        }
    }

    pub(crate) fn permitted_interaction(&self, target: InteractionTarget) -> bool {
        self.restrictions()
            .permits(Self::interaction_action(target))
    }

    pub(super) fn mining_delta(&self, delta: Duration) -> Duration {
        // Quantize only voluntary extraction work, never the simulation clock.
        // World mining also retains its own cap, mass and interruption rules.
        let bps = self.restrictions().interaction_bps;
        let ns = delta
            .min(salimon_world::spherical_mining::SPHERE_TICK)
            .as_nanos()
            * u128::from(bps)
            / 10_000;
        Duration::from_nanos(ns as u64)
    }
}

#[cfg(test)]
mod tests;
