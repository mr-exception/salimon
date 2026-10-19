//! Portable fabrication rules. Runtime owns physical intake, energy settlement,
//! inventory transfer and ship mass adapters; no clocks, GPU, IO or native types.
mod inventory;
mod machine;
pub use inventory::{
    ConsumableCategory, ConsumableInventory, FOOD_SLOT_COUNT, InventoryError, MEDICAL_SLOT_COUNT,
};
mod mass;
mod recipes;

pub use machine::*;
pub use mass::*;
pub use recipes::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FabricationError {
    InvalidMass,
    InvalidRecipeId,
    InvalidRecipe,
    DuplicateRecipe,
    DuplicateIngredient,
    InvalidUnlockRequirement,
    InvalidQueueCapacity,
    UnknownRecipe,
    LockedRecipe,
    InvalidQuantity,
    QueueFull,
    ArithmeticOverflow,
    InsufficientMaterial(salimon_world::resources::ResourceId),
    UnknownJob,
    EmptyTray,
    StorageFull(salimon_world::resources::ResourceId),
}

impl std::fmt::Display for FabricationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for FabricationError {}

mod save;
pub use save::FabricationSaveState;
