//! Schema-1 validated machine/inventory source state, independent of disk IO.
use crate::{ConsumableInventory, Fabricator, Mass, OutputCategory, OutputItem, RecipeId};
use salimon_persistence::{SaveError, Validate, invalid};
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FabricationSaveState {
    pub machine: Fabricator,
    pub inventory: ConsumableInventory,
}
impl Validate for FabricationSaveState {
    fn validate(&self) -> Result<(), SaveError> {
        self.machine.validate_save()?;
        let mut identities = std::collections::HashSet::new();
        let validate_item = |item: &OutputItem| -> Result<(), SaveError> {
            RecipeId::new(item.recipe.key()).map_err(|_| invalid("item recipe ID"))?;
            Mass::from_milligrams(item.mass.milligrams()).map_err(|_| invalid("item mass"))?;
            if item.job.0 == 0 || item.job.0 >= self.machine.next_job_id() || item.unit == 0 {
                return Err(invalid("output identity high-water mark"));
            }
            let recipe = self
                .machine
                .recipes()
                .get(&item.recipe)
                .ok_or_else(|| invalid("unknown output recipe"))?;
            if item.mass != recipe.output_mass || item.category != recipe.output_category {
                return Err(invalid("output recipe mismatch"));
            }
            self.machine.validate_output_identity(item)?;
            Ok(())
        };
        for (slots, category) in [
            (self.inventory.food().as_slice(), OutputCategory::Food),
            (self.inventory.medical().as_slice(), OutputCategory::Medical),
        ] {
            for item in slots.iter().flatten() {
                validate_item(item)?;
                if item.category != category || !identities.insert((item.job.0, item.unit)) {
                    return Err(invalid("inventory category or duplicate output"));
                }
            }
        }
        if let Some(item) = self.machine.tray_item() {
            validate_item(item)?;
            if !identities.insert((item.job.0, item.unit)) {
                return Err(invalid("tray item already collected"));
            }
        }
        Ok(())
    }
}
impl FabricationSaveState {
    pub fn capture(machine: &Fabricator, inventory: &ConsumableInventory) -> Self {
        Self {
            machine: machine.clone(),
            inventory: inventory.clone(),
        }
    }
    pub fn restore(self) -> Result<(Fabricator, ConsumableInventory), SaveError> {
        self.validate()?;
        Ok((self.machine, self.inventory))
    }
}
