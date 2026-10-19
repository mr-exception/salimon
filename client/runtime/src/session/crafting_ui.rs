//! Modal, keyboard-accessible machine UI. Domain commands remain transactional.
use super::GameplaySession;
use salimon_fabrication::{FabricationCommand, FabricationError};
#[derive(Clone, Copy, Debug)]
pub(crate) enum CraftingAction {
    Close,
    PreviousRecipe,
    NextRecipe,
    Less,
    More,
    NextJob,
    Cancel,
    Add,
}

#[derive(Default)]
pub(crate) struct CraftingUi {
    pub(crate) open: bool,
    recipe: usize,
    quantity: u32,
    job: usize,
    message: String,
}

impl GameplaySession {
    pub(super) fn validate_crafting_focus(&mut self) {
        if !self.crafting_ui.open {
            return;
        }
        let snapshot = self.snapshot();
        let frame = super::frames::character_ship_frame(snapshot.ship.pose);
        let eye = frame.world_to_local(snapshot.character.eye_position_meters);
        let target = salimon_character::FABRICATOR_MONITOR_MARKER_METERS;
        if self.character.location() != salimon_character::CharacterLocation::InsideShip
            || !super::interaction::machine_target(eye, target, target)
        {
            self.crafting_ui.open = false;
        }
    }

    pub(crate) fn crafting_action(&mut self, key: CraftingAction) {
        if !self.crafting_ui.open {
            return;
        }
        // Close and navigation remain safe menu actions. Mutation uses authority.
        if !self
            .restrictions()
            .permits(salimon_survival::restrictions::Action::Fabricate)
        {
            self.crafting_ui.open = false;
            return;
        }
        if matches!(key, CraftingAction::Add | CraftingAction::Cancel)
            && !self.admit_survival_action(salimon_survival::restrictions::Action::Fabricate)
        {
            return;
        }
        let recipes = self.fabricator.recipes().recipes();
        let snapshot = self
            .fabricator
            .snapshot(self.ship.snapshot().power_available);
        self.crafting_ui.quantity = self.crafting_ui.quantity.max(1);
        let command = match key {
            CraftingAction::Close => {
                self.crafting_ui.open = false;
                None
            }
            CraftingAction::PreviousRecipe => {
                self.crafting_ui.recipe = self.crafting_ui.recipe.saturating_sub(1);
                None
            }
            CraftingAction::NextRecipe => {
                self.crafting_ui.recipe =
                    (self.crafting_ui.recipe + 1).min(recipes.len().saturating_sub(1));
                None
            }
            CraftingAction::Less => {
                self.crafting_ui.quantity = self.crafting_ui.quantity.saturating_sub(1).max(1);
                None
            }
            CraftingAction::More => {
                self.crafting_ui.quantity = self.crafting_ui.quantity.saturating_add(1);
                None
            }
            CraftingAction::NextJob => {
                self.crafting_ui.job = (self.crafting_ui.job + 1) % snapshot.jobs.len().max(1);
                None
            }
            CraftingAction::Cancel => snapshot
                .jobs
                .get(self.crafting_ui.job)
                .map(|job| FabricationCommand::Cancel(job.id)),
            CraftingAction::Add => {
                recipes
                    .get(self.crafting_ui.recipe)
                    .map(|recipe| FabricationCommand::Enqueue {
                        recipe: recipe.id.clone(),
                        quantity: self.crafting_ui.quantity,
                    })
            }
        };
        if let Some(command) = command {
            self.crafting_ui.message = match self.execute_fabricator(command) {
                Ok(_) => "Accepted. Committed ingredients are never refunded.".into(),
                Err(FabricationError::InsufficientMaterial(resource)) => {
                    let recipe = &self.fabricator.recipes().recipes()[self.crafting_ui.recipe];
                    let ingredient = recipe
                        .ingredients
                        .iter()
                        .find(|i| i.resource == resource)
                        .expect("error identifies recipe ingredient");
                    let required = u128::from(ingredient.mass.milligrams())
                        * u128::from(self.crafting_ui.quantity);
                    let stored = snapshot
                        .stored
                        .iter()
                        .find(|s| s.resource == resource)
                        .expect("catalog bin")
                        .milligrams;
                    format!(
                        "Need {:.3} kg {:?}; stored {:.3} kg",
                        required as f64 / 1e6,
                        resource,
                        stored as f64 / 1e6
                    )
                }
                Err(error) => format!("Cannot craft: {error}"),
            };
        }
    }

    pub(crate) fn crafting_text(&self) -> Option<String> {
        if !self.crafting_ui.open {
            return None;
        }
        let s = self
            .fabricator
            .snapshot(self.ship.snapshot().power_available);
        let mut lines = vec![
            "FABRICATOR / WALL MONITOR".into(),
            "UP/DOWN recipe  LEFT/RIGHT quantity  ENTER add".into(),
            "TAB next job  DELETE cancel (no refund)  E/ESC close".into(),
            format!(
                "Status: {:?} / quantity {}",
                s.status,
                self.crafting_ui.quantity.max(1)
            ),
        ];
        for bin in &s.stored {
            lines.push(format!(
                "{:?}: {:.3} / 20 kg",
                bin.resource,
                bin.milligrams as f64 / 1e6
            ));
        }
        let recipes = self.fabricator.recipes().recipes();
        if let Some(recipe) = recipes.get(self.crafting_ui.recipe) {
            lines.push(format!(
                "RECIPE {}/{}: {} [{}]",
                self.crafting_ui.recipe + 1,
                recipes.len(),
                recipe.id.key(),
                if recipe.unlock.unlocked_by_default {
                    "READY"
                } else {
                    "LOCKED"
                }
            ));
            lines.push(format!(
                "Each: {:.1} seconds / {:.1} kW",
                recipe.duration.as_secs_f64(),
                recipe.watts / 1000.0
            ));
            for ingredient in &recipe.ingredients {
                lines.push(format!(
                    "  {:?}: {:.3} kg each",
                    ingredient.resource,
                    ingredient.mass.kg()
                ));
            }
            if !recipe.unlock.unlocked_by_default {
                lines.push(format!(
                    "Requires: {}",
                    recipe.unlock.prerequisites.join(", ")
                ));
            }
        }
        lines.push(format!(
            "QUEUE: {} / {} jobs (TAB to inspect)",
            s.jobs.len(),
            s.queue_capacity
        ));
        // Selected and active rows stay readable even at the technical queue limit.
        for (index, job) in s
            .jobs
            .iter()
            .enumerate()
            .filter(|(i, _)| *i == 0 || *i == self.crafting_ui.job)
        {
            lines.push(format!(
                "{} #{} {} x{} / {:.1}s left",
                if index == self.crafting_ui.job {
                    ">"
                } else {
                    " "
                },
                job.id.0,
                job.recipe.key(),
                job.remaining_quantity,
                (job.unit_duration - job.progress).as_secs_f64()
            ));
        }
        if let Some(item) = s.tray {
            lines.push(format!("TRAY: {} / collect physically", item.recipe.key()));
        }
        // Wrap error/requirement strings to a fixed readable width.
        lines.push(self.crafting_ui.message.clone());
        Some(
            lines
                .into_iter()
                .flat_map(|line| {
                    let chars: Vec<_> = line.chars().collect();
                    chars
                        .chunks(66)
                        .map(|chunk| chunk.iter().collect::<String>())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

impl GameplaySession {
    pub(crate) fn fabricator_instruments(&self) -> salimon_renderer::FabricatorInstruments {
        use salimon_fabrication::FabricatorStatus as S;
        let s = self
            .fabricator
            .snapshot(self.ship.snapshot().power_available);
        salimon_renderer::FabricatorInstruments {
            status: match s.status {
                S::Idle => 0,
                S::Working => 1,
                S::NoPower => 2,
                S::TrayBlocked => 3,
            },
            jobs: s.jobs.len() as u32,
            remaining_seconds: s.jobs.first().map_or(0, |j| {
                (j.unit_duration - j.progress).as_secs_f64().ceil() as u32
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_world::resources::{RESOURCE_CATALOG, RawMaterial};
    #[test]
    fn modal_commands_validate_costs_and_capture_gameplay_without_reordering() {
        let mut s = GameplaySession::default();
        s.crafting_ui.open = true;
        s.crafting_action(CraftingAction::Add);
        assert!(
            s.crafting_text()
                .unwrap()
                .contains("Need 0.400 kg OrganicBiomass")
        );
        assert!(s.fabricator.snapshot(true).jobs.is_empty());
        for r in RESOURCE_CATALOG {
            s.fabricator
                .insert(RawMaterial::new(r.id, 2.0).unwrap())
                .unwrap();
        }
        s.crafting_action(CraftingAction::More);
        s.crafting_action(CraftingAction::Add);
        assert_eq!(s.fabricator.snapshot(true).jobs[0].remaining_quantity, 2);
        let equipment = s.equipment.selected();
        s.command(super::super::Command::Select(
            crate::equipment::ToolbarSlot::Four,
        ));
        assert_eq!(s.equipment.selected(), equipment);
        s.crafting_action(CraftingAction::Cancel);
        assert!(s.fabricator.snapshot(true).jobs.is_empty());
        s.crafting_action(CraftingAction::Close);
        assert!(s.crafting_text().is_none());
    }
    #[test]
    fn locked_recipe_status_and_departure_are_observations_without_free_work() {
        let mut s = GameplaySession::default();
        let mut recipe = salimon_fabrication::RecipeBook::starter().recipes()[0].clone();
        recipe.unlock.unlocked_by_default = false;
        recipe.unlock.prerequisites = vec!["future-research".into()];
        s.fabricator = salimon_fabrication::Fabricator::new(
            salimon_fabrication::RecipeBook::new(vec![recipe]).unwrap(),
            8,
        )
        .unwrap();
        s.crafting_ui.open = true;
        assert!(s.crafting_text().unwrap().contains("LOCKED"));
        assert!(s.crafting_text().unwrap().contains("future-research"));
        s.crafting_action(CraftingAction::Add);
        assert!(s.crafting_text().unwrap().contains("LockedRecipe"));
        assert!(s.fabricator.snapshot(true).jobs.is_empty());
        // Starting location is away from the machine: no stale modal survives a tick.
        s.validate_crafting_focus();
        assert!(!s.crafting_ui.open);
    }
    #[test]
    fn raster_fits_gameplay_resolution_and_uses_large_high_contrast_glyphs() {
        let mut s = GameplaySession::default();
        s.crafting_ui.open = true;
        let (w, h, pixels) =
            crate::action_bar::rasterize_action_bar(&s.crafting_text().unwrap(), 3);
        assert!(w <= 1248 && h <= 768, "{w}x{h}");
        assert!(pixels.as_chunks::<4>().0.contains(&[248, 250, 252, 255]));
    }
}
