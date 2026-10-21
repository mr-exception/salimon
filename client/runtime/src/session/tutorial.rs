//! Per-run, event-driven onboarding. No input capture, resources or clock authority.
use super::{GameplaySession, ViewMode};
use salimon_persistence::{SaveError, Validate};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Objective {
    Vitals,
    Wheel,
    Ration,
    Fabricator,
    Craft,
    Collect,
    SaveStatus,
}
const OBJECTIVES: [Objective; 7] = [
    Objective::Vitals,
    Objective::Wheel,
    Objective::Ration,
    Objective::Fabricator,
    Objective::Craft,
    Objective::Collect,
    Objective::SaveStatus,
];

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Tutorial {
    completed: u8,
    skipped: bool,
}
impl Validate for Tutorial {
    fn validate(&self) -> Result<(), SaveError> {
        if self.completed & !0x7f != 0 {
            return Err(salimon_persistence::invalid("unknown tutorial objective"));
        }
        Ok(())
    }
}
impl Tutorial {
    /// Legacy runs/isolated fixtures do not suddenly acquire an onboarding flow.
    pub(crate) fn dismissed() -> Self {
        Self {
            skipped: true,
            ..Self::default()
        }
    }
    pub(crate) fn record(&mut self, objective: Objective) {
        if !self.skipped {
            self.completed |= 1 << objective as u8;
        }
    }
    pub(crate) fn skip(&mut self) {
        self.skipped = true;
    }
    pub(crate) fn current(&self) -> Option<Objective> {
        (!self.skipped)
            .then(|| {
                OBJECTIVES
                    .into_iter()
                    .find(|o| self.completed & (1 << *o as u8) == 0)
            })
            .flatten()
    }
    pub(crate) fn text(&self) -> Option<String> {
        let objective = self.current()?;
        let prompt = match objective {
            Objective::Vitals => "H: inspect survival meters",
            Objective::Wheel => "Hold Q: open food and medicine",
            Objective::Ration => "Hold Q: select ration, click to eat",
            Objective::Fabricator => "E at Fabricator screen: recipes",
            Objective::Craft => "Fabricator: craft a basic ration",
            Objective::Collect => "E at tray: collect crafted ration",
            Objective::SaveStatus => "Escape: inspect checkpoint status",
        };
        Some(format!(
            "SURVIVAL {}/7: {prompt}\nT: skip tutorial",
            self.completed.count_ones() + 1
        ))
    }
}
impl GameplaySession {
    pub(crate) fn tutorial_text(&self) -> Option<String> {
        (self.view_mode == ViewMode::Gameplay
            && !self.is_paused()
            && !self.survival.snapshot().terminal
            && !self.wheel.open
            && !self.crafting_ui.open)
            .then(|| self.tutorial.text())
            .flatten()
    }
}
#[cfg(test)]
mod tests;
