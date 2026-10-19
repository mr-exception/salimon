//! Typed receiving slots, independent of gear, world carrying and ship mass.
use crate::{OutputCategory, OutputItem};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InventoryError {
    Full,
    UnsupportedCategory,
    AlreadyStored,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConsumableInventory {
    food: [Option<OutputItem>; 3],
    medical: [Option<OutputItem>; 2],
}

impl ConsumableInventory {
    pub fn food(&self) -> &[Option<OutputItem>; 3] {
        &self.food
    }
    pub fn medical(&self) -> &[Option<OutputItem>; 2] {
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
