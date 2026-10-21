//! Exclusive consumable input; the authoritative simulation clock keeps running.
use super::{GameplaySession, ViewMode};
use salimon_fabrication::ConsumableSlot;
#[derive(Clone, Copy, Debug)]
pub(crate) enum WheelAction {
    Hold(bool),
    Hover(Option<ConsumableSlot>),
    Primary(bool),
    Next,
    Previous,
    Confirm,
    Cancel,
}
#[derive(Default)]
pub(crate) struct WheelState {
    pub(crate) held: bool,
    pub(crate) open: bool,
    pub(crate) hovered: Option<ConsumableSlot>,
    pub(crate) primary_down: bool,
    selection: Option<super::consumables::Selection>,
}
impl GameplaySession {
    pub(super) fn wheel_action(&mut self, action: WheelAction) {
        use WheelAction::*;
        if let Hold(pressed) = action {
            if !pressed {
                self.wheel.held = false;
                self.wheel.open = false;
                self.wheel.hovered = None;
                self.wheel.selection = None;
                return;
            }
            if self.wheel.held
                || self.crafting_ui.open
                || self.archive_reader.open
                || self.view_mode != ViewMode::Gameplay
            {
                return;
            }
            let primary_down = self.wheel.primary_down;
            self.clear_held_controls();
            self.tutorial.record(super::tutorial::Objective::Wheel);
            self.wheel = WheelState {
                held: true,
                open: true,
                hovered: None,
                primary_down,
                selection: None,
            };
            return;
        }
        if !self.wheel.open {
            return;
        }
        match action {
            Hover(slot) => self.select_wheel_slot(slot),
            Primary(pressed) => {
                let was_down = std::mem::replace(&mut self.wheel.primary_down, pressed);
                if pressed && !was_down {
                    self.consume_wheel_selection();
                }
            }
            Next | Previous => {
                let offset = if matches!(action, Next) { 1 } else { 4 };
                let start = self
                    .wheel
                    .hovered
                    .map_or(if offset == 1 { 4 } else { 0 }, ConsumableSlot::index);
                let slot = (1..=5)
                    .map(|step| ConsumableSlot::ALL[(start + offset * step) % 5])
                    .find(|slot| self.consumables.item(*slot).is_some());
                self.select_wheel_slot(slot);
            }
            Confirm => self.consume_wheel_selection(),
            Cancel => {
                self.wheel.open = false;
                self.wheel.hovered = None;
            }
            Hold(_) => unreachable!(),
        }
    }
    fn select_wheel_slot(&mut self, slot: Option<ConsumableSlot>) {
        self.wheel.hovered = slot;
        self.wheel.selection = slot.and_then(|slot| {
            self.consumables
                .item(slot)
                .map(|item| super::consumables::Selection {
                    slot,
                    job: item.job,
                    unit: item.unit,
                })
        });
    }
    fn consume_wheel_selection(&mut self) {
        let Some(selected) = self.wheel.selection else {
            return;
        };
        if self.consume_selection(selected).is_err() {
            return;
        }
        self.wheel.open = false;
        self.wheel.hovered = None;
        self.wheel.selection = None;
    }
    pub(crate) fn wheel_presentation(&self) -> salimon_renderer::ConsumableWheel {
        salimon_renderer::ConsumableWheel {
            slots: self.equipment_presentation().consumables,
            hovered: self.wheel.hovered.map(ConsumableSlot::index),
            names: std::array::from_fn(|i| {
                self.consumables
                    .item(ConsumableSlot::ALL[i])
                    .map_or_else(|| "EMPTY SLOT".into(), |item| item.recipe.key().to_owned())
            }),
            effect: self
                .wheel
                .hovered
                .and_then(|slot| self.consumables.item(slot))
                .and_then(|item| self.consumable_effects.get(item.recipe.key()))
                .map_or_else(
                    || "SELECT AN ITEM".into(),
                    |definition| {
                        definition
                            .benefits
                            .iter()
                            .map(benefit_text)
                            .collect::<Vec<_>>()
                            .join(" / ")
                    },
                ),
        }
    }
}
fn benefit_text(benefit: &salimon_survival::consumables::Benefit) -> String {
    use salimon_survival::consumables::{Benefit, Recovery};
    let name = |recovery| match recovery {
        Recovery::Hunger => "HUNGER",
        Recovery::Health(salimon_survival::effects::Healing::Ordinary) => "HEALTH",
        Recovery::Health(salimon_survival::effects::Healing::Treatment) => "WOUND CARE",
    };
    match *benefit {
        Benefit::Instant {
            recovery,
            milli_points,
        } => format!("{} +{}", name(recovery), milli_points as f64 / 1000.0),
        Benefit::OverTime {
            recovery,
            milli_points,
            interval,
            duration,
            ..
        } => format!(
            "{} +{}/{}s FOR {}s",
            name(recovery),
            milli_points as f64 / 1000.0,
            interval.as_secs_f64(),
            duration.as_secs_f64()
        ),
        Benefit::Modifier {
            kind,
            basis_points,
            duration,
            ..
        } => {
            use salimon_survival::effects::Modifier;
            let name = match kind {
                Modifier::FatigueDrain => "FATIGUE DRAIN",
                Modifier::Regeneration => "REGEN",
                Modifier::FatigueRelief => "FATIGUE PENALTY",
            };
            format!(
                "{name} {}% {}s",
                basis_points as f64 / 100.0,
                duration.as_secs_f64()
            )
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{Command, ControlKey};
    use salimon_fabrication::{JobId, Mass, OutputCategory, OutputItem, RecipeId};
    fn stocked() -> GameplaySession {
        let mut s = GameplaySession::default();
        for i in 0..5 {
            s.consumables
                .insert(OutputItem {
                    recipe: RecipeId::new(if i < 3 { "food-ration" } else { "medkit" }).unwrap(),
                    job: JobId(i + 1),
                    unit: 1,
                    mass: Mass::from_milligrams(1000).unwrap(),
                    category: if i < 3 {
                        OutputCategory::Food
                    } else {
                        OutputCategory::Medical
                    },
                })
                .unwrap();
        }
        s
    }
    fn action(s: &mut GameplaySession, a: WheelAction) {
        s.command(Command::Wheel(a));
    }
    #[test]
    fn tap_release_cancel_repeats_and_selected_consumption_once_without_compaction() {
        let mut s = stocked();
        let original = s.consumables.clone();
        action(&mut s, WheelAction::Hold(true));
        action(&mut s, WheelAction::Hold(false));
        assert_eq!(s.consumables, original);
        assert!(!s.wheel.open);
        action(&mut s, WheelAction::Hold(true));
        action(&mut s, WheelAction::Hover(Some(ConsumableSlot::FoodTwo)));
        let mass = s.ship.snapshot().total_mass_kg;
        let gear = s.equipment.selected();
        for _ in 0..4 {
            action(&mut s, WheelAction::Primary(true));
            action(&mut s, WheelAction::Hold(true));
        }
        assert!(!s.wheel.open);
        assert!(s.consumables.food()[1].is_none());
        assert!(s.consumables.food()[0].is_some());
        assert!(s.consumables.food()[2].is_some());
        assert_eq!(s.ship.snapshot().total_mass_kg, mass);
        assert_eq!(s.equipment.selected(), gear);
        action(&mut s, WheelAction::Hold(false));
        action(&mut s, WheelAction::Hold(true));
        action(&mut s, WheelAction::Hover(Some(ConsumableSlot::MedicalTwo)));
        action(&mut s, WheelAction::Primary(false));
        action(&mut s, WheelAction::Primary(true));
        assert!(s.consumables.medical()[1].is_none());
        assert!(s.consumables.medical()[0].is_some());
    }
    #[test]
    fn clears_held_controls_and_gates_world_actions_but_advances_clock_and_flight() {
        let mut s = stocked();
        s.ship = salimon_ship::ShipController::flying_with_velocity(
            s.ship.snapshot().pose,
            0,
            [10.0, 0.0, 0.0],
        )
        .unwrap();
        s.command(Command::Control(ControlKey::Forward, true));
        s.command(Command::Mining(true));
        action(&mut s, WheelAction::Hold(true));
        assert!(!s.movement_input.forward);
        let before = s.snapshot();
        let selected = s.equipment.selected();
        let elapsed = s.elapsed;
        for command in [
            Command::Control(ControlKey::Forward, true),
            Command::Look(100.0, 100.0),
            Command::Select(crate::equipment::ToolbarSlot::One),
            Command::Interact,
            Command::Landing,
            Command::GrabDrop(true),
            Command::Mining(true),
        ] {
            s.command(command);
        }
        assert!(!s.movement_input.forward);
        assert_eq!(s.equipment.selected(), selected);
        assert_eq!(s.snapshot().character, before.character);
        s.advance_automation(std::time::Duration::from_millis(100));
        assert!(s.elapsed > elapsed);
        assert_ne!(
            s.ship.snapshot().pose.position_meters,
            before.ship.pose.position_meters
        );
        assert!(s.wheel.open);
        // A mouse held before opening cannot consume until released and pressed again.
        action(&mut s, WheelAction::Hover(Some(ConsumableSlot::FoodOne)));
        action(&mut s, WheelAction::Primary(true));
        assert!(s.consumables.food()[0].is_some());
        action(&mut s, WheelAction::Primary(false));
        action(&mut s, WheelAction::Primary(true));
        assert!(s.consumables.food()[0].is_none());
    }
    #[test]
    fn focus_crafting_tour_empty_slots_and_keyboard_fallback_are_safe() {
        let mut s = stocked();
        action(&mut s, WheelAction::Hold(true));
        s.clear_held_controls();
        assert!(!s.wheel.open);
        assert!(!s.wheel.held);
        let original = s.consumables.clone();
        action(&mut s, WheelAction::Primary(true));
        assert_eq!(s.consumables, original);
        s.crafting_ui.open = true;
        action(&mut s, WheelAction::Hold(true));
        assert!(!s.wheel.open);
        s.clear_held_controls();
        s.toggle_view();
        action(&mut s, WheelAction::Hold(true));
        assert!(!s.wheel.open);
        s.toggle_view();
        s.consumables.consume_slot(ConsumableSlot::FoodOne);
        action(&mut s, WheelAction::Hold(true));
        action(&mut s, WheelAction::Hover(Some(ConsumableSlot::FoodOne)));
        action(&mut s, WheelAction::Confirm);
        assert!(s.wheel.open);
        action(&mut s, WheelAction::Next);
        assert_eq!(s.wheel.hovered, Some(ConsumableSlot::FoodTwo));
        action(&mut s, WheelAction::Confirm);
        assert!(s.consumables.food()[1].is_none());
        assert!(!s.wheel.open);
        action(&mut s, WheelAction::Hold(false));
        action(&mut s, WheelAction::Hold(true));
        action(&mut s, WheelAction::Cancel);
        action(&mut s, WheelAction::Hold(true));
        assert!(!s.wheel.open);
    }
}

#[cfg(test)]
mod survival_integration_tests {
    use super::*;
    use crate::session::Command;
    use salimon_survival::{Command as VitalCommand, Points};
    use std::time::Duration;
    fn use_item(s: &mut GameplaySession, slot: ConsumableSlot) {
        for c in [
            Command::Mining(false),
            Command::Wheel(WheelAction::Hold(false)),
            Command::Wheel(WheelAction::Hold(true)),
            Command::Wheel(WheelAction::Hover(Some(slot))),
            Command::Wheel(WheelAction::Confirm),
        ] {
            s.command(c);
        }
    }
    #[test]
    fn real_ration_medkit_and_full_vitals_consumption() {
        let mut s = GameplaySession::new_run();
        let mut v = serde_json::to_value(&s.survival).unwrap();
        v["hunger"] = serde_json::to_value(Points::whole(10).unwrap()).unwrap();
        s.survival = serde_json::from_value(v).unwrap();
        s.survival
            .command(VitalCommand::Damage(Points::whole(30).unwrap()))
            .unwrap();
        use_item(&mut s, ConsumableSlot::FoodOne);
        assert_eq!(s.survival.snapshot().hunger, Points::whole(50).unwrap());
        assert!(s.consumables.food()[0].is_none() && !s.wheel.open);
        use_item(&mut s, ConsumableSlot::MedicalOne);
        let before = s.survival.snapshot().health;
        s.advance_automation(Duration::from_secs(6));
        assert!(s.survival.snapshot().health > before);
        assert!(s.consumables.medical()[0].is_none());
        let mut full = GameplaySession::new_run();
        use_item(&mut full, ConsumableSlot::FoodTwo);
        assert!(full.consumables.food()[1].is_none());
        assert_eq!(full.survival.snapshot().hunger, Points::FULL);
    }
    #[test]
    fn exhaustion_allows_use_but_pause_death_and_reload_cancel_selection() {
        let mut s = GameplaySession::new_run();
        let mut v = serde_json::to_value(&s.survival).unwrap();
        v["fatigue"] = serde_json::to_value(Points::ZERO).unwrap();
        s.survival = serde_json::from_value(v).unwrap();
        use_item(&mut s, ConsumableSlot::FoodOne);
        assert!(s.consumables.food()[0].is_none());
        s.command(Command::Wheel(WheelAction::Hold(false)));
        s.command(Command::Wheel(WheelAction::Hold(true)));
        s.command(Command::Wheel(WheelAction::Hover(Some(
            ConsumableSlot::FoodTwo,
        ))));
        let original = s.consumables.clone();
        s.command(Command::TogglePause);
        assert!(!s.wheel.open);
        s.command(Command::Wheel(WheelAction::Hold(true)));
        s.command(Command::Wheel(WheelAction::Confirm));
        assert_eq!(s.consumables, original);
        s.command(Command::TogglePause);
        s.command(Command::Wheel(WheelAction::Hold(true)));
        let bytes = s.save_bytes(123).unwrap();
        s.restore_bytes(&bytes).unwrap();
        assert!(!s.wheel.open && s.wheel.selection.is_none());
        s.command(Command::Wheel(WheelAction::Confirm));
        assert_eq!(s.consumables, original);
        s.command(Command::Wheel(WheelAction::Hold(true)));
        s.survival
            .command(VitalCommand::Damage(Points::FULL))
            .unwrap();
        s.command(Command::Wheel(WheelAction::Confirm));
        assert!(!s.wheel.open);
        assert_eq!(s.consumables, original);
    }
    #[test]
    fn replaced_selection_and_unknown_effect_do_not_consume_inventory() {
        let mut s = GameplaySession::new_run();
        let slot = ConsumableSlot::FoodOne;
        s.command(Command::Wheel(WheelAction::Hold(true)));
        s.command(Command::Wheel(WheelAction::Hover(Some(slot))));
        let original = s.consumables.consume_slot(slot).unwrap();
        let replacement = s.consumables.consume_slot(ConsumableSlot::FoodTwo).unwrap();
        s.consumables.insert(replacement).unwrap();
        s.consumables.insert(original).unwrap();
        let before = s.save_bytes(1).unwrap();
        s.command(Command::Wheel(WheelAction::Confirm));
        assert_eq!(s.save_bytes(1).unwrap(), before);
        assert!(s.wheel.open);
        s.command(Command::Wheel(WheelAction::Hover(Some(slot))));
        s.consumable_effects = salimon_survival::consumables::Catalog::new(vec![]).unwrap();
        s.command(Command::Wheel(WheelAction::Confirm));
        assert!(s.consumables.item(slot).is_some() && s.wheel.open);
    }
    #[test]
    fn hover_uses_configured_recovery_definitions() {
        let mut s = GameplaySession::new_run();
        s.command(Command::Wheel(WheelAction::Hold(true)));
        s.command(Command::Wheel(WheelAction::Hover(Some(
            ConsumableSlot::FoodOne,
        ))));
        assert_eq!(s.wheel_presentation().effect, "HUNGER +40");
        s.command(Command::Wheel(WheelAction::Hover(Some(
            ConsumableSlot::MedicalOne,
        ))));
        assert_eq!(s.wheel_presentation().effect, "HEALTH +2.5/1s FOR 10s");
    }
}
