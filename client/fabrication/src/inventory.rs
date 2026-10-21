//! Typed receiving slots, independent of gear, world carrying and ship mass.
use crate::{OutputCategory, OutputItem};

pub const FOOD_SLOT_COUNT: usize = 3;
pub const MEDICAL_SLOT_COUNT: usize = 2;

/// Exact one-item slot selection; invalid numeric indexes never enter the domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsumableSlot {
    FoodOne,
    FoodTwo,
    FoodThree,
    MedicalOne,
    MedicalTwo,
}
impl ConsumableSlot {
    pub const ALL: [Self; 5] = [
        Self::FoodOne,
        Self::FoodTwo,
        Self::FoodThree,
        Self::MedicalOne,
        Self::MedicalTwo,
    ];
    pub fn index(self) -> usize {
        match self {
            Self::FoodOne => 0,
            Self::FoodTwo => 1,
            Self::FoodThree => 2,
            Self::MedicalOne => 3,
            Self::MedicalTwo => 4,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsumableCategory {
    Food,
    Medical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InventoryError {
    Full,
    UnsupportedCategory,
    AlreadyStored,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsumableInventory {
    food: [Option<OutputItem>; FOOD_SLOT_COUNT],
    medical: [Option<OutputItem>; MEDICAL_SLOT_COUNT],
}

impl ConsumableInventory {
    pub fn item(&self, slot: ConsumableSlot) -> Option<&OutputItem> {
        match slot.index() {
            i @ 0..=2 => self.food[i].as_ref(),
            i => self.medical[i - 3].as_ref(),
        }
    }
    /// Consume precisely the chosen slot; never compact survivors or check vitals.
    pub fn consume_slot(&mut self, slot: ConsumableSlot) -> Option<OutputItem> {
        match slot.index() {
            i @ 0..=2 => self.food[i].take(),
            i => self.medical[i - 3].take(),
        }
    }

    /// Removes the first occupied matching slot, without shifting/stacking items.
    /// No health/hunger or future full-vitals gate participates in this phase.
    pub fn consume_first(&mut self, category: ConsumableCategory) -> Option<OutputItem> {
        let slots: &mut [Option<OutputItem>] = match category {
            ConsumableCategory::Food => &mut self.food,
            ConsumableCategory::Medical => &mut self.medical,
        };
        slots
            .iter_mut()
            .find(|slot| slot.is_some())
            .and_then(Option::take)
    }

    pub fn food(&self) -> &[Option<OutputItem>; FOOD_SLOT_COUNT] {
        &self.food
    }
    pub fn medical(&self) -> &[Option<OutputItem>; MEDICAL_SLOT_COUNT] {
        &self.medical
    }
    /// Atomic first-vacant placement. Each typed slot holds exactly one item.
    pub fn insert(&mut self, item: OutputItem) -> Result<usize, InventoryError> {
        if self
            .food
            .iter()
            .chain(&self.medical)
            .flatten()
            .any(|stored| stored.job == item.job && stored.unit == item.unit)
        {
            return Err(InventoryError::AlreadyStored);
        }
        let slots: &mut [Option<OutputItem>] = match item.category {
            OutputCategory::Food => &mut self.food,
            OutputCategory::Medical => &mut self.medical,
            _ => return Err(InventoryError::UnsupportedCategory),
        };
        let index = slots
            .iter()
            .position(Option::is_none)
            .ok_or(InventoryError::Full)?;
        slots[index] = Some(item);
        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{JobId, Mass, RecipeId};
    fn item(id: u64, category: OutputCategory) -> OutputItem {
        OutputItem {
            recipe: RecipeId::new("test-item").unwrap(),
            job: JobId(id),
            unit: 1,
            mass: Mass::from_milligrams(1000).unwrap(),
            category,
        }
    }
    #[test]
    fn fixed_one_item_categories_preserve_order_reuse_first_vacancy_and_deplete() {
        let mut inventory = ConsumableInventory::default();
        assert_eq!(inventory.food().len(), FOOD_SLOT_COUNT);
        assert_eq!(inventory.medical().len(), MEDICAL_SLOT_COUNT);
        assert_eq!(inventory.consume_first(ConsumableCategory::Food), None);
        assert_eq!(inventory.consume_first(ConsumableCategory::Medical), None);
        for id in 1..=3 {
            assert_eq!(
                inventory.insert(item(id, OutputCategory::Food)),
                Ok((id - 1) as usize)
            );
        }
        for id in 4..=5 {
            assert_eq!(
                inventory.insert(item(id, OutputCategory::Medical)),
                Ok((id - 4) as usize)
            );
        }
        let full = inventory.clone();
        for category in [OutputCategory::Food, OutputCategory::Medical] {
            assert_eq!(
                inventory.insert(item(6, category)),
                Err(InventoryError::Full)
            );
            assert_eq!(inventory, full);
        }
        assert_eq!(
            inventory
                .consume_first(ConsumableCategory::Food)
                .unwrap()
                .job,
            JobId(1)
        );
        assert!(inventory.food()[0].is_none());
        assert_eq!(inventory.food()[1].as_ref().unwrap().job, JobId(2));
        assert_eq!(inventory.insert(item(6, OutputCategory::Food)), Ok(0));
        for id in [6, 2, 3] {
            assert_eq!(
                inventory
                    .consume_first(ConsumableCategory::Food)
                    .unwrap()
                    .job,
                JobId(id)
            );
        }
        for id in [4, 5] {
            assert_eq!(
                inventory
                    .consume_first(ConsumableCategory::Medical)
                    .unwrap()
                    .job,
                JobId(id)
            );
        }
        assert_eq!(inventory, ConsumableInventory::default());
    }
    #[test]
    fn unsupported_categories_and_duplicate_resident_identities_are_atomic() {
        let mut inventory = ConsumableInventory::default();
        inventory.insert(item(1, OutputCategory::Food)).unwrap();
        let original = inventory.clone();
        for category in [OutputCategory::Equipment, OutputCategory::Component] {
            assert_eq!(
                inventory.insert(item(2, category)),
                Err(InventoryError::UnsupportedCategory)
            );
            assert_eq!(inventory, original);
        }
        assert_eq!(
            inventory.insert(item(1, OutputCategory::Medical)),
            Err(InventoryError::AlreadyStored)
        );
        assert_eq!(inventory, original);
    }
}
