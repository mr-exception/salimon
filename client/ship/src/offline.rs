//! Resumable gravity-only closed-game propagation. No wall clock, engine or IO.
//!
//! The region provider must supply complete collision geometry and epoch-checked
//! ephemerides. Missing data is an error, never empty space or a frozen body.
use crate::{
    Propulsion,
    atmosphere::{Aerodynamics, Air},
    contact::{Advance, Consequence, Hit, ResponsePolicy, Scene},
    trajectory::{State, Thrust, Trajectory},
};
use salimon_universe::{CatalogId, Epoch, Error, coordinates::GalacticPosition, gravity::Source};
use std::time::Duration;

pub trait Environment {
    fn sources(&self, epoch: Epoch, position: GalacticPosition) -> Result<Vec<Source>, Error>;
    fn air(&self, epoch: Epoch, position: GalacticPosition) -> Result<Option<Air>, Error>;
    /// Geometry covers the entire proposed interval, including moving terrain.
    fn scene(&self, start: State, nanos: i64) -> Result<Scene, Error>;
    /// Follow an actual supported body-fixed contact. Never pin a moving body
    /// to its last inertial position. The provider verifies continuing support.
    fn supported(&self, state: State, body: &CatalogId, nanos: i64) -> Result<State, Error>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Coasting,
    Grounded(CatalogId),
    Fatal,
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub step_nanos: i64,
    pub ship_radius_meters: f64,
    pub propulsion: Propulsion,
    pub aerodynamics: Aerodynamics,
    pub response: ResponsePolicy,
}

#[derive(Clone, Debug)]
pub struct Progress {
    pub completed: Duration,
    pub total: Duration,
    pub steps: u64,
    pub impacts: u64,
    /// Largest actual collision witness uncertainty, not a guessed zero error.
    pub maximum_contact_uncertainty_meters: f64,
}
impl Progress {
    pub fn complete(&self) -> bool {
        self.completed == self.total
    }
    pub fn fraction(&self) -> f64 {
        if self.complete() {
            1.0
        } else {
            // Rounding a long interval must not display completion while its
            // final nanoseconds still await verification.
            (self.completed.as_secs_f64() / self.total.as_secs_f64()).min(1.0 - f64::EPSILON)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Boundary {
    pub after: Duration,
    pub state: State,
    pub mode: Mode,
    pub hit: Option<Hit>,
}

/// A clone is a private transaction candidate, including every timed-system
/// source it changes. Implementations must not perform IO or mutate shared
/// state from this callback; durable publication belongs to the host.
pub trait Consumer: Clone {
    fn apply(&mut self, boundary: Boundary) -> Result<(), Error>;
}
impl Consumer for Vec<Boundary> {
    fn apply(&mut self, boundary: Boundary) -> Result<(), Error> {
        self.push(boundary);
        Ok(())
    }
}

/// Private candidate state; only a completed job can produce a verified result.
/// A budget bounds work per poll, never truncates the requested elapsed time.
#[derive(Clone, Debug)]
pub struct Propagation {
    trajectory: Trajectory,
    mode: Mode,
    config: Config,
    progress: Progress,
}
impl Propagation {
    pub fn new(
        state: State,
        mode: Mode,
        selector: salimon_universe::gravity::Selector,
        elapsed: Duration,
        config: Config,
    ) -> Result<Self, Error> {
        if !config.ship_radius_meters.is_finite() || config.ship_radius_meters <= 0.0 {
            return Err(error("offline ship radius"));
        }
        // Validate the final calendar up front, without narrowing a potentially
        // multi-century Duration to one overflowing i64 nanosecond interval.
        let mut epoch = state.epoch;
        let mut seconds = elapsed.as_secs();
        while seconds > 0 {
            let chunk = seconds.min(86_400_000);
            epoch = epoch.advance_nanos((chunk * 1_000_000_000) as i64)?;
            seconds -= chunk;
        }
        epoch.advance_nanos(i64::from(elapsed.subsec_nanos()))?;
        Ok(Self {
            trajectory: Trajectory::new(state, selector, config.step_nanos)?,
            mode,
            config,
            progress: Progress {
                completed: Duration::ZERO,
                total: elapsed,
                steps: 0,
                impacts: 0,
                maximum_contact_uncertainty_meters: 0.0,
            },
        })
    }
    pub fn progress(&self) -> &Progress {
        &self.progress
    }
    pub fn verified(&self) -> Result<(State, &Mode), Error> {
        if !self.progress.complete() {
            return Err(error("offline propagation is not complete"));
        }
        Ok((self.trajectory.state(), &self.mode))
    }
    /// Commit the candidate trajectory and its timed-system consumer together.
    /// Failure rolls both back, including successful earlier boundaries in this
    /// poll. The host still owns durable checkpoint publication.
    pub fn poll(
        &mut self,
        environment: &impl Environment,
        budget: usize,
        consumer: &mut impl Consumer,
    ) -> Result<(), Error> {
        if budget == 0 {
            return Err(error("offline poll requires positive work budget"));
        }
        let mut staged = self.clone();
        let mut staged_consumer = consumer.clone();
        for _ in 0..budget {
            if staged.progress.complete() {
                break;
            }
            let remaining = staged.progress.total - staged.progress.completed;
            let nanos = remaining.as_nanos().min(staged.config.step_nanos as u128) as i64;
            let start = staged.trajectory.state();
            let mut hit = None;
            let used = match &staged.mode {
                Mode::Fatal => {
                    // Terminal chronology is settled by its owner; no simulated
                    // motion or timed work is invented after death.
                    staged.progress.completed = staged.progress.total;
                    break;
                }
                Mode::Grounded(body) => {
                    let state = environment.supported(start, body, nanos)?;
                    if state.epoch != start.epoch.advance_nanos(nanos)? {
                        return Err(error("offline supported-state epoch mismatch"));
                    }
                    staged.trajectory.replace_offline_state(state)?;
                    nanos
                }
                Mode::Coasting => {
                    let scene = environment.scene(start, nanos)?;
                    let advance: Advance = staged.trajectory.step_with_contacts(
                        nanos,
                        Thrust::coasting(staged.config.propulsion),
                        staged.config.aerodynamics,
                        &scene,
                        staged.config.ship_radius_meters,
                        staged.config.response,
                        |at, p| environment.sources(at, p),
                        |at, p| environment.air(at, p),
                    )?;
                    if let Some(resolution) = &advance.resolution {
                        let contact = advance
                            .hit
                            .as_ref()
                            .ok_or_else(|| error("offline missing impact witness"))?;
                        staged.mode = match resolution.consequence {
                            Consequence::Rebounding => Mode::Coasting,
                            Consequence::GroundedAfterCrash => Mode::Grounded(contact.body.clone()),
                            Consequence::Fatal => Mode::Fatal,
                        };
                        staged.progress.impacts += 1;
                        staged.progress.maximum_contact_uncertainty_meters = staged
                            .progress
                            .maximum_contact_uncertainty_meters
                            .max(contact.spatial_uncertainty_meters);
                    }
                    hit = advance.hit;
                    if advance.elapsed_nanos == 0
                        && staged.mode == Mode::Coasting
                        && staged.trajectory.state() == start
                    {
                        return Err(error("offline zero-time rebound; refine contact geometry"));
                    }
                    advance.elapsed_nanos
                }
            };
            staged.progress.completed += Duration::from_nanos(used as u64);
            staged.progress.steps += 1;
            staged_consumer.apply(Boundary {
                after: staged.progress.completed,
                state: staged.trajectory.state(),
                mode: staged.mode.clone(),
                hit,
            })?;
        }
        *self = staged;
        *consumer = staged_consumer;
        Ok(())
    }
}
fn error(message: &str) -> Error {
    Error::Invalid(message.into())
}

#[cfg(test)]
mod tests;
