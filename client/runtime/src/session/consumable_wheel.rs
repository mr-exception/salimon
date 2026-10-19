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
}
impl GameplaySession {
    pub(super) fn wheel_action(&mut self, action: WheelAction) {
        use WheelAction::*;
        if let Hold(pressed) = action {
            if !pressed {
                self.wheel.held = false;
                self.wheel.open = false;
                self.wheel.hovered = None;
                return;
            }
            if self.wheel.held || self.crafting_ui.open || self.view_mode != ViewMode::Gameplay {
                return;
            }
            let primary_down = self.wheel.primary_down;
            self.clear_held_controls();
            self.wheel = WheelState {
                held: true,
                open: true,
                hovered: None,
                primary_down,
            };
            return;
        }
        if !self.wheel.open {
            return;
        }
        match action {
            Hover(slot) => self.wheel.hovered = slot,
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
                self.wheel.hovered = (1..=5)
                    .map(|step| ConsumableSlot::ALL[(start + offset * step) % 5])
                    .find(|slot| self.consumables.item(*slot).is_some());
            }
            Confirm => self.consume_wheel_selection(),
            Cancel => {
                self.wheel.open = false;
                self.wheel.hovered = None;
            }
            Hold(_) => unreachable!(),
        }
    }
    fn consume_wheel_selection(&mut self) {
        let Some(item) = self
            .wheel
            .hovered
            .and_then(|slot| self.consumables.consume_slot(slot))
        else {
            return;
        };
        self.wheel.open = false;
        self.wheel.hovered = None;
        // #368 owns applying healing/status effects. Do not invent survival state here.
        self.feedback = Some(match item.category {
            salimon_fabrication::OutputCategory::Medical => "Medkit consumed",
            _ => "Food ration consumed",
        });
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
            effect: "NO ACTIVE EFFECT".into(),
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
