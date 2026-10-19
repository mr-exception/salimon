//! Exact threshold and permission policy; no clocks, input or movement ownership.
use crate::{Points, Snapshot};
use salimon_persistence::{SaveError, Validate, invalid};
use std::time::Duration;

const ONE: u32 = 10_000;

/// Numeric tuning, separate from meter depletion and physical simulation time.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictionConfig {
    pub minimum_movement_bps: u32,
    pub low_hunger_interaction_bps: u32,
    pub low_fatigue_interaction_bps: u32,
    pub interaction_base: Duration,
}
impl Default for RestrictionConfig {
    fn default() -> Self {
        Self {
            minimum_movement_bps: 1000,
            low_hunger_interaction_bps: 7500,
            low_fatigue_interaction_bps: 7500,
            interaction_base: Duration::from_millis(250),
        }
    }
}
impl Validate for RestrictionConfig {
    fn validate(&self) -> Result<(), SaveError> {
        if [
            self.minimum_movement_bps,
            self.low_hunger_interaction_bps,
            self.low_fatigue_interaction_bps,
        ]
        .iter()
        .any(|v| !(1..=ONE).contains(v))
            || self.interaction_base > Duration::from_secs(60)
        {
            return Err(invalid("invalid survival restriction tuning"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Walk,
    Consume,
    Door,
    Menu,
    Pilot,
    Mine,
    Fabricate,
    Carry,
    FeedCore,
}

/// Renderer-independent exact percentages for HUD/debuff labels and authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Restrictions {
    pub hunger_movement_bps: u32,
    pub fatigue_movement_bps: u32,
    pub movement_bps: u32,
    pub interaction_bps: u32,
    pub exhausted: bool,
    pub terminal: bool,
    interaction_base: Duration,
}
impl Restrictions {
    pub fn new(snapshot: Snapshot, config: RestrictionConfig) -> Result<Self, SaveError> {
        config.validate()?;
        let hunger = snapshot.hunger;
        let fatigue = snapshot.fatigue;
        let point = |n| Points::whole(n).expect("constant threshold in 0..100");
        let hunger_movement_bps = if hunger < point(20) {
            7500
        } else if hunger < point(50) {
            9000
        } else {
            ONE
        };
        let fatigue_movement_bps = if fatigue == Points::ZERO {
            3000
        } else if fatigue <= point(20) {
            6500
        } else if fatigue <= point(50) {
            8500
        } else {
            ONE
        };
        let mut interaction_bps = ONE;
        if hunger < point(20) {
            interaction_bps = config.low_hunger_interaction_bps;
        }
        if fatigue <= point(20) {
            interaction_bps = (u64::from(interaction_bps)
                * u64::from(config.low_fatigue_interaction_bps)
                / u64::from(ONE)) as u32;
        }
        Ok(Self {
            hunger_movement_bps,
            fatigue_movement_bps,
            movement_bps: (hunger_movement_bps * fatigue_movement_bps / ONE)
                .max(config.minimum_movement_bps),
            interaction_bps: interaction_bps.max(1),
            exhausted: fatigue == Points::ZERO,
            terminal: snapshot.terminal,
            interaction_base: config.interaction_base,
        })
    }
    pub fn permits(self, action: Action) -> bool {
        if self.terminal {
            return action == Action::Menu;
        }
        !self.exhausted
            || matches!(
                action,
                Action::Walk | Action::Consume | Action::Door | Action::Menu
            )
    }
    pub fn movement_scale(self) -> f64 {
        f64::from(self.movement_bps) / f64::from(ONE)
    }
    /// Additional recovery between discrete nonessential actions; never slows
    /// emergency consumption/doors or world/ship/crafting-job clocks.
    pub fn interaction_delay(self) -> Duration {
        let ns = self.interaction_base.as_nanos() * u128::from(ONE - self.interaction_bps)
            / u128::from(self.interaction_bps);
        Duration::from_nanos(ns.min(u128::from(u64::MAX)) as u64)
    }
}
