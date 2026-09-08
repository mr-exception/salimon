//! Paid active work. Runtime supplies safety and real Core funding, never UI time.
use super::{
    funding::{COMPLETE, Funding, FundingError},
    placement::Blueprint,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    /// None is the legacy #543 quantized progress seam, upgraded on first paid tick.
    pub elapsed_nanos: Option<u64>,
    /// Explicit stop latch. Landing/power restoration never clears it.
    #[serde(default)]
    pub stopped: bool,
    pub damaged: bool,
    pub completed: bool,
    /// All physical contents transferred out; identity remains a durable tombstone.
    #[serde(default)]
    pub destroyed: bool,
}
impl Work {
    pub fn elapsed(&self, funding: &Funding, blueprint: &Blueprint) -> u64 {
        self.elapsed_nanos.unwrap_or(
            u64::from(funding.progress_ppm) * self.duration(blueprint) / u64::from(COMPLETE),
        )
    }
    pub fn duration(&self, blueprint: &Blueprint) -> u64 {
        u64::from(blueprint.duration_seconds) * 1_000_000_000
    }
    pub fn validate(&self, funding: &Funding, blueprint: &Blueprint) -> Result<(), FundingError> {
        let duration = self.duration(blueprint);
        if blueprint.duration_seconds > 300
            || duration == 0
            || self.elapsed_nanos.is_some_and(|n| {
                n > duration
                    || n * u64::from(COMPLETE) / duration != u64::from(funding.progress_ppm)
            })
            || (self.completed && self.stopped)
            || self.completed != (funding.progress_ppm == COMPLETE)
            || (self.elapsed_nanos.is_some_and(|n| n > 0)
                && !funding.funded(&blueprint.requirements))
        {
            return Err(FundingError::Invalid);
        }
        Ok(())
    }
    pub fn remaining(&self, funding: &Funding, blueprint: &Blueprint) -> Duration {
        Duration::from_nanos(
            self.duration(blueprint)
                .saturating_sub(self.elapsed(funding, blueprint)),
        )
    }
    /// The funded interval has already been debited. Complete is published only by
    /// the caller that preflights the physical installation in the same session borrow.
    pub fn advance(
        &mut self,
        funding: &mut Funding,
        blueprint: &Blueprint,
        paid: Duration,
    ) -> Result<(), FundingError> {
        self.validate(funding, blueprint)?;
        if self.stopped
            || self.damaged
            || self.completed
            || !funding.funded(&blueprint.requirements)
        {
            return Err(FundingError::Invalid);
        }
        let n = self.elapsed(funding, blueprint)
            + paid
                .as_nanos()
                .min(self.remaining(funding, blueprint).as_nanos()) as u64;
        funding.set_progress(
            (n * u64::from(COMPLETE) / self.duration(blueprint)) as u32,
            &blueprint.requirements,
        )?;
        self.elapsed_nanos = Some(n);
        self.completed = n == self.duration(blueprint);
        Ok(())
    }
}
