//! Deterministic survival authority. Hosts supply admitted active durations and UTC
//! samples; this crate never reads a clock, advances a world, or performs IO.
pub mod consumables;
pub mod effects;
mod pipeline;
pub mod restrictions;

use salimon_persistence::{SaveError, Validate, invalid};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// One point is 648 seconds of nanoseconds. Default drains/recovery are integral
// (36/1/3 units per nanosecond), so split updates lose no fractional progress.
const POINT: u64 = 648_000_000_000;
const FULL: u64 = 100 * POINT;
const SCALE_SECONDS: u32 = 64_800;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Points(u64);
impl Points {
    pub const ZERO: Self = Self(0);
    pub const FULL: Self = Self(FULL);
    pub fn whole(value: u32) -> Result<Self, SaveError> {
        Self::milli(
            value
                .checked_mul(1000)
                .ok_or_else(|| invalid("points overflow"))?,
        )
    }
    pub fn milli(value: u32) -> Result<Self, SaveError> {
        if value > 100_000 {
            return Err(invalid("points outside 0..100"));
        }
        Ok(Self(u64::from(value) * (POINT / 1000)))
    }
    /// Presentation only; authority never accumulates floating-point values.
    pub fn value(self) -> f64 {
        self.0 as f64 / POINT as f64
    }
    pub(crate) fn subtract(&mut self, amount: Self) {
        self.0 = self.0.saturating_sub(amount.0);
    }
    pub(crate) fn add(&mut self, amount: Self) {
        self.0 = self.0.saturating_add(amount.0).min(FULL);
    }
}
impl Validate for Points {
    fn validate(&self) -> Result<(), SaveError> {
        if self.0 > FULL {
            Err(invalid("points outside 0..100"))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub hunger_full_seconds: u32,
    pub fatigue_full_seconds: u32,
    pub fatigue_recovery_seconds: u32,
    pub starvation_interval: Duration,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            hunger_full_seconds: 1800,
            fatigue_full_seconds: 64_800,
            fatigue_recovery_seconds: 21_600,
            starvation_interval: Duration::from_secs(10),
        }
    }
}
impl Validate for Config {
    fn validate(&self) -> Result<(), SaveError> {
        for seconds in [
            self.hunger_full_seconds,
            self.fatigue_full_seconds,
            self.fatigue_recovery_seconds,
        ] {
            if seconds == 0 || !SCALE_SECONDS.is_multiple_of(seconds) {
                return Err(invalid(
                    "meter durations must be positive whole-second divisors of 64800",
                ));
            }
        }
        if self.starvation_interval.is_zero()
            || self.starvation_interval > Duration::from_secs(3600)
        {
            return Err(invalid("starvation interval outside (0,3600] seconds"));
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Snapshot {
    pub hunger: Points,
    pub fatigue: Points,
    pub health: Points,
    pub terminal: bool,
    pub active_time: Duration,
    pub death_cause: Option<effects::Source>,
    pub wounds: Points,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    Feed(Points),
    Heal(Points),
    Damage(Points),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OfflineResult {
    Applied(Duration),
    ClockRollback,
    AlreadyApplied,
    Terminal,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Survival {
    config: Config,
    pipeline: effects::Pipeline,
    hunger: Points,
    fatigue: Points,
    health: Points,
    starvation_elapsed: Duration,
    active_time: Duration,
    terminal: bool,
    // A receipt protects replay in one launch. The host persists the credited
    // state and watermark atomically before exposing it; #371 owns that IO.
    offline_watermark: u64,
    #[serde(skip)]
    offline_applied: bool,
}
impl Default for Survival {
    fn default() -> Self {
        Self::new(Config::default()).expect("default survival config")
    }
}
impl Survival {
    pub fn new(config: Config) -> Result<Self, SaveError> {
        config.validate()?;
        Ok(Self {
            config,
            pipeline: Default::default(),
            hunger: Points::FULL,
            fatigue: Points::FULL,
            health: Points::FULL,
            starvation_elapsed: Duration::ZERO,
            active_time: Duration::ZERO,
            terminal: false,
            offline_watermark: 0,
            offline_applied: false,
        })
    }
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            hunger: self.hunger,
            fatigue: self.fatigue,
            health: self.health,
            terminal: self.terminal,
            active_time: self.active_time,
            death_cause: self.pipeline.death_cause,
            wounds: self.pipeline.wounds,
        }
    }
    pub fn command(&mut self, command: Command) -> Result<(), SaveError> {
        let amount = match command {
            Command::Feed(p) | Command::Heal(p) | Command::Damage(p) => p,
        };
        amount.validate()?;
        if self.terminal {
            return Ok(());
        }
        match command {
            Command::Feed(p) => {
                self.hunger.add(p);
                if self.hunger != Points::ZERO {
                    self.starvation_elapsed = Duration::ZERO;
                }
            }
            Command::Heal(p) => self.heal(effects::Heal {
                amount: p,
                mode: effects::Healing::Ordinary,
            })?,
            Command::Damage(p) => self.damage(effects::Damage {
                source: effects::Source::Environment,
                amount: p,
                wound: false,
            })?,
        }
        Ok(())
    }
    /// Exact elapsed accounting, independent of render cadence. Terminal updates
    /// stop at the lethal boundary, even when a host supplies a very large delta.
    pub fn advance(&mut self, elapsed: Duration) {
        self.advance_pipeline(elapsed);
    }
    fn drain_meters(&mut self, elapsed_ns: u128) {
        self.hunger.0 = self.hunger.0.saturating_sub(
            (elapsed_ns * u128::from(SCALE_SECONDS / self.config.hunger_full_seconds))
                .min(u128::from(FULL)) as u64,
        );
        self.fatigue.0 = self.fatigue.0.saturating_sub(
            (elapsed_ns * u128::from(SCALE_SECONDS / self.config.fatigue_full_seconds))
                .min(u128::from(FULL)) as u64,
        );
        self.active_time = self.active_time.saturating_add(nanos(elapsed_ns));
    }
    /// Called once by a launch transaction, never on resume, pause, or saving.
    /// A crash uses the last verified checkpoint as the conservative exit bound.
    /// Forward jumps grant at most six configured recovery hours; rollback grants
    /// nothing and retains the high watermark, including after another restart.
    pub fn recover_offline(&mut self, checkpoint_utc: u64, launch_utc: u64) -> OfflineResult {
        if self.offline_applied {
            return OfflineResult::AlreadyApplied;
        }
        self.offline_applied = true;
        if self.terminal {
            return OfflineResult::Terminal;
        }
        let start = checkpoint_utc.max(self.offline_watermark);
        if launch_utc < start {
            self.offline_watermark = start;
            return OfflineResult::ClockRollback;
        }
        self.offline_watermark = launch_utc;
        let elapsed = Duration::from_secs(
            (launch_utc - start).min(u64::from(self.config.fatigue_recovery_seconds)),
        );
        self.fatigue.0 = (u128::from(self.fatigue.0)
            + elapsed.as_nanos() * u128::from(SCALE_SECONDS / self.config.fatigue_recovery_seconds))
        .min(u128::from(FULL)) as u64;
        OfflineResult::Applied(elapsed)
    }
}
impl Validate for Survival {
    fn validate(&self) -> Result<(), SaveError> {
        self.config.validate()?;
        self.pipeline.validate()?;
        self.health.validate()?;
        if self.terminal != self.pipeline.death_cause.is_some()
            || self.pipeline.wounds.0 > FULL - self.health.0
        {
            return Err(invalid("inconsistent death source/wound debt"));
        }
        self.hunger.validate()?;
        self.fatigue.validate()?;
        self.health.validate()?;
        if self.terminal != (self.health == Points::ZERO)
            || self.starvation_elapsed >= self.config.starvation_interval
            || (self.hunger != Points::ZERO && !self.starvation_elapsed.is_zero())
        {
            return Err(invalid("inconsistent survival clock/terminal state"));
        }
        Ok(())
    }
}
fn nanos(value: u128) -> Duration {
    Duration::new(
        (value / 1_000_000_000) as u64,
        (value % 1_000_000_000) as u32,
    )
}
