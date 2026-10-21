use crate::{FabricationError, Mass};
use salimon_world::resources::ResourceId;
use std::time::Duration;

#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeId(String);
impl RecipeId {
    pub fn new(key: impl Into<String>) -> Result<Self, FabricationError> {
        let key = key.into();
        if !valid_key(&key) {
            return Err(FabricationError::InvalidRecipeId);
        }
        Ok(Self(key))
    }
    pub fn key(&self) -> &str {
        &self.0
    }
}

fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 128
        && key
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        && !key.starts_with('-')
        && !key.ends_with('-')
}

/// Equipment/component are schema categories only; no upgrade gameplay here.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum OutputCategory {
    Food,
    Medical,
    Equipment,
    Component,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ingredient {
    pub resource: ResourceId,
    pub mass: Mass,
}

/// Schema for later progression. This wave only enables default-unlocked recipes;
/// it exposes no command to acquire research or upgrades.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnlockRequirements {
    pub unlocked_by_default: bool,
    pub prerequisites: Vec<String>,
}

/// Data input; RecipeBook validates the whole set before accepting any recipe.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeDefinition {
    pub id: RecipeId,
    pub ingredients: Vec<Ingredient>,
    pub duration: Duration,
    pub watts: f64,
    pub output_mass: Mass,
    pub output_category: OutputCategory,
    pub unlock: UnlockRequirements,
}

impl RecipeDefinition {
    fn validate(&self) -> Result<(), FabricationError> {
        if self.ingredients.is_empty()
            || self.duration.is_zero()
            || !self.watts.is_finite()
            || self.watts <= 0.0
            || !(self.watts * self.duration.as_secs_f64()).is_finite()
            || self.watts * self.duration.as_secs_f64() <= 0.0
        {
            return Err(FabricationError::InvalidRecipe);
        }
        for (index, ingredient) in self.ingredients.iter().enumerate() {
            if self.ingredients[..index]
                .iter()
                .any(|other| other.resource == ingredient.resource)
            {
                return Err(FabricationError::DuplicateIngredient);
            }
        }
        if self.unlock.unlocked_by_default && !self.unlock.prerequisites.is_empty() {
            return Err(FabricationError::InvalidUnlockRequirement);
        }
        for (index, key) in self.unlock.prerequisites.iter().enumerate() {
            if !valid_key(key) || self.unlock.prerequisites[..index].contains(key) {
                return Err(FabricationError::InvalidUnlockRequirement);
            }
        }
        Ok(())
    }
}

/// Immutable validated definitions, in caller order. Jobs capture their definition.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeBook(Vec<RecipeDefinition>);
impl RecipeBook {
    pub fn new(definitions: Vec<RecipeDefinition>) -> Result<Self, FabricationError> {
        for (index, definition) in definitions.iter().enumerate() {
            definition.validate()?;
            if definitions[..index]
                .iter()
                .any(|other| other.id == definition.id)
            {
                return Err(FabricationError::DuplicateRecipe);
            }
        }
        Ok(Self(definitions))
    }
    pub fn recipes(&self) -> &[RecipeDefinition] {
        &self.0
    }
    pub fn get(&self, id: &RecipeId) -> Option<&RecipeDefinition> {
        self.0.iter().find(|recipe| recipe.id == *id)
    }

    /// Proposed starter balancing values, not user-specified exact quantities.
    pub fn starter() -> Self {
        let unlocked = UnlockRequirements {
            unlocked_by_default: true,
            prerequisites: vec![],
        };
        let mut definitions = vec![
            RecipeDefinition {
                id: RecipeId::new("food-ration").expect("valid static recipe key"),
                ingredients: vec![
                    Ingredient {
                        resource: ResourceId::OrganicBiomass,
                        mass: Mass::from_milligrams(400_000).expect("positive constant"),
                    },
                    Ingredient {
                        resource: ResourceId::WaterIce,
                        mass: Mass::from_milligrams(100_000).expect("positive constant"),
                    },
                ],
                duration: Duration::from_secs(10),
                watts: 10_000.0,
                output_mass: Mass::from_milligrams(300_000).expect("positive constant"),
                output_category: OutputCategory::Food,
                unlock: unlocked.clone(),
            },
            RecipeDefinition {
                id: RecipeId::new("medkit").expect("valid static recipe key"),
                ingredients: vec![
                    Ingredient {
                        resource: ResourceId::MedicinalCompounds,
                        mass: Mass::from_milligrams(200_000).expect("positive constant"),
                    },
                    Ingredient {
                        resource: ResourceId::WaterIce,
                        mass: Mass::from_milligrams(100_000).expect("positive constant"),
                    },
                ],
                duration: Duration::from_secs(15),
                watts: 20_000.0,
                output_mass: Mass::from_milligrams(200_000).expect("positive constant"),
                output_category: OutputCategory::Medical,
                unlock: unlocked,
            },
        ];
        // Explicit provisional production tuning; advanced food stays locked.
        for (key, biomass, water, seconds, watts, output) in [
            ("nutrient-gel", 200_000, 150_000, 10, 10_000.0, 200_000),
            ("complete-meal", 700_000, 200_000, 20, 15_000.0, 500_000),
            ("energy-drink", 100_000, 250_000, 10, 10_000.0, 250_000),
        ] {
            definitions.push(RecipeDefinition {
                id: RecipeId::new(key).expect("static food key"),
                ingredients: vec![
                    Ingredient {
                        resource: ResourceId::OrganicBiomass,
                        mass: Mass::from_milligrams(biomass).expect("positive constant"),
                    },
                    Ingredient {
                        resource: ResourceId::WaterIce,
                        mass: Mass::from_milligrams(water).expect("positive constant"),
                    },
                ],
                duration: Duration::from_secs(seconds),
                watts,
                output_mass: Mass::from_milligrams(output).expect("positive constant"),
                output_category: OutputCategory::Food,
                unlock: UnlockRequirements {
                    unlocked_by_default: false,
                    prerequisites: vec![],
                },
            });
        }
        Self::new(definitions).expect("valid starter recipe definitions")
    }
}
