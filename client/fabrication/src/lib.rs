//! Portable fabrication rules. Runtime owns physical intake, energy settlement,
//! inventory transfer and ship mass adapters; no clocks, GPU, IO or native types.
mod machine;
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
}

impl std::fmt::Display for FabricationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for FabricationError {}
