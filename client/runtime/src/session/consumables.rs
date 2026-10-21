//! One exact-identity inventory/effect transaction for the Q-wheel host and protocol.
use super::{GameplaySession, ViewMode};
use salimon_fabrication::{ConsumableSlot, JobId, OutputCategory, OutputItem};
use salimon_survival::{consumables::Category, restrictions::Action};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Selection {
    pub(crate) slot: ConsumableSlot,
    pub(crate) job: JobId,
    pub(crate) unit: u32,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConsumeError {
    Unavailable,
    Empty,
    StaleSelection,
    UnknownEffect,
    CategoryMismatch,
    InvalidEffect,
}

impl GameplaySession {
    /// The caller identifies the item visible at selection time, not merely a
    /// reusable slot. Repeated clicks cannot eat a newly collected replacement.
    pub(crate) fn consume_selection(
        &mut self,
        selected: Selection,
    ) -> Result<OutputItem, ConsumeError> {
        if self.is_paused()
            || self.view_mode != ViewMode::Gameplay
            || self.crafting_ui.open
            || !self.restrictions().permits(Action::Consume)
        {
            return Err(ConsumeError::Unavailable);
        }
        let item = self
            .consumables
            .item(selected.slot)
            .ok_or(ConsumeError::Empty)?;
        if item.job != selected.job || item.unit != selected.unit {
            return Err(ConsumeError::StaleSelection);
        }
        let definition = self
            .consumable_effects
            .get(item.recipe.key())
            .ok_or(ConsumeError::UnknownEffect)?;
        if !matches!(
            (item.category, definition.category),
            (OutputCategory::Food, Category::Food) | (OutputCategory::Medical, Category::Medical)
        ) {
            return Err(ConsumeError::CategoryMismatch);
        }
        // Prepare potentially fallible effect registration/application before
        // changing either live owner. Single event-thread ownership makes commit infallible.
        let mut survival = self.survival.clone();
        survival
            .apply_consumable(definition)
            .map_err(|_| ConsumeError::InvalidEffect)?;
        let item = self
            .consumables
            .consume_slot(selected.slot)
            .expect("selected identity validated under exclusive session ownership");
        self.survival = survival;
        if item.recipe.key() == "food-ration" {
            self.tutorial.record(super::tutorial::Objective::Ration);
        }
        self.feedback = Some(match item.category {
            OutputCategory::Medical => "Medkit consumed",
            _ => "Food consumed",
        });
        Ok(item)
    }
}

#[cfg(test)]
mod tests;
