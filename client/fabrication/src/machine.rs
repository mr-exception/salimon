use crate::{FabricationError, Mass, OutputCategory, RecipeBook, RecipeDefinition, RecipeId};
use salimon_world::resources::{RESOURCE_CATALOG, RawMaterial, ResourceId};
use std::{collections::VecDeque, time::Duration};

pub const STORAGE_CAPACITY_MILLIGRAMS: u64 = 20_000_000;
/// Technical memory bound, not a gameplay upgrade/capacity decision.
pub const DEFAULT_QUEUE_CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JobId(pub u64);

#[derive(Clone, Debug, PartialEq)]
pub enum FabricationCommand {
    Insert(RawMaterial),
    Enqueue { recipe: RecipeId, quantity: u32 },
    Cancel(JobId),
    Collect,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommandResult {
    Inserted(Insertion),
    Enqueued(JobId),
    Cancelled,
    Collected(OutputItem),
}

/// Success means the adapter must remove the ENTIRE source fragment, with any overflow discarded. Errors mean it must preserve the source fragment unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Insertion {
    pub resource: ResourceId,
    pub accepted_milligrams: u64,
    pub discarded_milligrams: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputItem {
    pub recipe: RecipeId,
    pub job: JobId,
    /// One-based completed unit within its job; (job, unit) is a stable item key.
    pub unit: u32,
    pub mass: Mass,
    pub category: OutputCategory,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredResource {
    pub resource: ResourceId,
    pub milligrams: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobSnapshot {
    pub id: JobId,
    pub recipe: RecipeId,
    pub remaining_quantity: u32,
    pub completed_quantity: u32,
    /// Progress on the current unit only. Waiting jobs always have zero progress.
    pub progress: Duration,
    pub unit_duration: Duration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FabricatorStatus {
    Idle,
    TrayBlocked,
    NoPower,
    Working,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FabricatorSnapshot {
    pub stored: Vec<StoredResource>,
    pub jobs: Vec<JobSnapshot>,
    pub tray: Option<OutputItem>,
    pub status: FabricatorStatus,
    /// Demand if powered, zero while idle/blocked. Core funding is runtime-owned.
    pub requested_watts: f64,
    pub queue_capacity: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PowerDemand {
    pub watts: f64,
    /// Runtime must split here or at earlier Core exhaustion before funding time.
    pub until_output: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepResult {
    pub worked: Duration,
    /// Not banked. Adapters may use this interval for OTHER simulation consumers.
    pub unused: Duration,
    pub consumed_joules: f64,
    pub produced_output: bool,
}

/// Separate integer components avoid overflow even with very large custom output
/// mass. Reserved inputs, jobs and collected inventory deliberately contribute zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MassContribution {
    pub stored_milligrams: u64,
    pub tray_milligrams: u64,
}
impl MassContribution {
    pub fn kg(self) -> f64 {
        (self.stored_milligrams as f64 + self.tray_milligrams as f64) / 1_000_000.0
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Job {
    id: JobId,
    recipe: RecipeDefinition,
    remaining: u32,
    completed: u32,
    progress: Duration,
}

/// In-memory authoritative domain; snapshots are observations, not a save format.
#[derive(Clone, Debug, PartialEq)]
pub struct Fabricator {
    recipes: RecipeBook,
    stored: Vec<StoredResource>,
    jobs: VecDeque<Job>,
    tray: Option<OutputItem>,
    queue_capacity: usize,
    next_job: u64,
}

impl Fabricator {
    pub fn new(recipes: RecipeBook, queue_capacity: usize) -> Result<Self, FabricationError> {
        if queue_capacity == 0 {
            return Err(FabricationError::InvalidQueueCapacity);
        }
        Ok(Self {
            recipes,
            stored: RESOURCE_CATALOG
                .iter()
                .map(|r| StoredResource {
                    resource: r.id,
                    milligrams: 0,
                })
                .collect(),
            jobs: VecDeque::new(),
            tray: None,
            queue_capacity,
            next_job: 1,
        })
    }

    pub fn starter() -> Self {
        Self::new(RecipeBook::starter(), DEFAULT_QUEUE_CAPACITY).expect("nonzero static capacity")
    }

    pub fn recipes(&self) -> &RecipeBook {
        &self.recipes
    }

    pub fn execute(
        &mut self,
        command: FabricationCommand,
    ) -> Result<CommandResult, FabricationError> {
        match command {
            FabricationCommand::Insert(material) => {
                self.insert(material).map(CommandResult::Inserted)
            }
            FabricationCommand::Enqueue { recipe, quantity } => {
                self.enqueue(&recipe, quantity).map(CommandResult::Enqueued)
            }
            FabricationCommand::Cancel(id) => self.cancel(id).map(|()| CommandResult::Cancelled),
            FabricationCommand::Collect => self.collect().map(CommandResult::Collected),
        }
    }

    pub fn insert(&mut self, material: RawMaterial) -> Result<Insertion, FabricationError> {
        let mass = Mass::from_kg(material.mass_kg())?.milligrams();
        let stored = self
            .stored
            .iter_mut()
            .find(|s| s.resource == material.resource())
            .expect("all world resources initialized");
        let accepted = mass.min(STORAGE_CAPACITY_MILLIGRAMS - stored.milligrams);
        if accepted == 0 {
            return Err(FabricationError::StorageFull(material.resource()));
        }
        stored.milligrams += accepted;
        Ok(Insertion {
            resource: material.resource(),
            accepted_milligrams: accepted,
            discarded_milligrams: mass - accepted,
        })
    }

    /// All fallible checks happen before mutating balances, queue or identity.
    pub fn enqueue(
        &mut self,
        recipe_id: &RecipeId,
        quantity: u32,
    ) -> Result<JobId, FabricationError> {
        if quantity == 0 {
            return Err(FabricationError::InvalidQuantity);
        }
        if self.jobs.len() >= self.queue_capacity {
            return Err(FabricationError::QueueFull);
        }
        let recipe = self
            .recipes
            .get(recipe_id)
            .ok_or(FabricationError::UnknownRecipe)?;
        if !recipe.unlock.unlocked_by_default {
            return Err(FabricationError::LockedRecipe);
        }
        let next = self
            .next_job
            .checked_add(1)
            .ok_or(FabricationError::ArithmeticOverflow)?;
        let mut costs = Vec::with_capacity(recipe.ingredients.len());
        for ingredient in &recipe.ingredients {
            let cost = ingredient
                .mass
                .milligrams()
                .checked_mul(u64::from(quantity))
                .ok_or(FabricationError::ArithmeticOverflow)?;
            let index = self
                .stored
                .iter()
                .position(|s| s.resource == ingredient.resource)
                .expect("all world resources initialized");
            if self.stored[index].milligrams < cost {
                return Err(FabricationError::InsufficientMaterial(ingredient.resource));
            }
            costs.push((index, cost));
        }
        let job = Job {
            id: JobId(self.next_job),
            recipe: recipe.clone(),
            remaining: quantity,
            completed: 0,
            progress: Duration::ZERO,
        };
        for (index, cost) in costs {
            self.stored[index].milligrams -= cost;
        }
        let id = job.id;
        self.next_job = next;
        self.jobs.push_back(job);
        Ok(id)
    }

    pub fn cancel(&mut self, id: JobId) -> Result<(), FabricationError> {
        let index = self
            .jobs
            .iter()
            .position(|j| j.id == id)
            .ok_or(FabricationError::UnknownJob)?;
        self.jobs.remove(index);
        // Inputs were committed on enqueue. Completed tray output is independent.
        Ok(())
    }

    /// Runtime must validate a free matching inventory slot before collecting.
    pub fn collect(&mut self) -> Result<OutputItem, FabricationError> {
        self.tray.take().ok_or(FabricationError::EmptyTray)
    }

    pub fn power_demand(&self) -> Option<PowerDemand> {
        if self.tray.is_some() {
            return None;
        }
        self.jobs.front().map(|job| PowerDemand {
            watts: job.recipe.watts,
            until_output: job.recipe.duration - job.progress,
        })
    }

    /// `powered` means the caller funds ALL returned `worked` time at demand watts.
    /// For partial Core exhaustion, call with the funded subinterval then the
    /// unpowered remainder. Never infer a full frame from a nonzero battery.
    pub fn step(&mut self, delta: Duration, powered: bool) -> StepResult {
        let mut result = StepResult {
            worked: Duration::ZERO,
            unused: delta,
            consumed_joules: 0.0,
            produced_output: false,
        };
        let Some(demand) = self.power_demand().filter(|_| powered) else {
            return result;
        };
        let worked = delta.min(demand.until_output);
        result.worked = worked;
        result.unused = delta - worked;
        result.consumed_joules = demand.watts * worked.as_secs_f64();
        let job = self.jobs.front_mut().expect("demand requires a head job");
        job.progress += worked;
        if job.progress == job.recipe.duration {
            job.completed += 1;
            job.remaining -= 1;
            job.progress = Duration::ZERO;
            self.tray = Some(OutputItem {
                recipe: job.recipe.id.clone(),
                job: job.id,
                unit: job.completed,
                mass: job.recipe.output_mass,
                category: job.recipe.output_category,
            });
            result.produced_output = true;
            if job.remaining == 0 {
                self.jobs.pop_front();
            }
        }
        result
    }

    pub fn snapshot(&self, powered: bool) -> FabricatorSnapshot {
        let status = if self.tray.is_some() {
            FabricatorStatus::TrayBlocked
        } else if self.jobs.is_empty() {
            FabricatorStatus::Idle
        } else if !powered {
            FabricatorStatus::NoPower
        } else {
            FabricatorStatus::Working
        };
        FabricatorSnapshot {
            stored: self.stored.clone(),
            jobs: self
                .jobs
                .iter()
                .map(|j| JobSnapshot {
                    id: j.id,
                    recipe: j.recipe.id.clone(),
                    remaining_quantity: j.remaining,
                    completed_quantity: j.completed,
                    progress: j.progress,
                    unit_duration: j.recipe.duration,
                })
                .collect(),
            tray: self.tray.clone(),
            status,
            requested_watts: self.power_demand().map_or(0.0, |d| d.watts),
            queue_capacity: self.queue_capacity,
        }
    }

    pub fn mass_contribution(&self) -> MassContribution {
        MassContribution {
            // Each of the fixed world catalog bins is bounded to 20 million mg.
            stored_milligrams: self.stored.iter().map(|s| s.milligrams).sum(),
            tray_milligrams: self.tray.as_ref().map_or(0, |item| item.mass.milligrams()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhausted_identity_space_rejects_without_losing_inputs_or_reusing_ids() {
        let mut machine = Fabricator::starter();
        for resource in RESOURCE_CATALOG {
            machine
                .insert(RawMaterial::new(resource.id, 20.0).unwrap())
                .unwrap();
        }
        machine.next_job = u64::MAX;
        let original = machine.clone();
        assert_eq!(
            machine.enqueue(&RecipeId::new("food-ration").unwrap(), 1),
            Err(FabricationError::ArithmeticOverflow)
        );
        assert_eq!(machine, original);
    }
}
