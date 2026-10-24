//! Portable real-scale inertial trajectory and staged environment composition.
//! Contact/assist policy stays with its owner; this never grants engine authority
//! or spends Core energy.
use crate::Propulsion;
use salimon_math::{add, length, scale};
use salimon_universe::{
    Epoch, Error, TimeScale,
    coordinates::GalacticPosition,
    gravity::{Selector, Source},
};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    pub position: GalacticPosition,
    pub velocity_mps: [f64; 3],
    pub epoch: Epoch,
}
/// Already-funded directional Core work. Existing ship/runtime allocation owns
/// debit and ledger-derived mass; the integrator must never spend it again.
#[derive(Clone, Copy, Debug)]
pub struct Thrust {
    pub direction: [f64; 3],
    pub funded_joules: f64,
    pub propulsion: Propulsion,
}
impl Thrust {
    pub fn coasting(propulsion: Propulsion) -> Self {
        Self {
            direction: [0.0; 3],
            funded_joules: 0.0,
            propulsion,
        }
    }
    fn delta_velocity(self) -> Result<[f64; 3], Error> {
        let norm = length(self.direction);
        if !self.funded_joules.is_finite()
            || self.funded_joules < 0.0
            || !norm.is_finite()
            || (self.funded_joules > 0.0 && (norm - 1.0).abs() > 1e-12)
        {
            return Err(error("trajectory funded thrust/direction"));
        }
        let dv = scale(
            self.direction,
            self.propulsion
                .delta_velocity_meters_per_second(self.funded_joules),
        );
        finite(dv)?;
        Ok(dv)
    }
}
fn error(message: &str) -> Error {
    Error::Invalid(message.into())
}
fn finite(v: [f64; 3]) -> Result<(), Error> {
    if v.into_iter().all(f64::is_finite) {
        Ok(())
    } else {
        Err(error("trajectory nonfinite state"))
    }
}
#[derive(Clone, Debug)]
pub struct Trajectory {
    state: State,
    gravity: Selector,
    max_step_nanos: i64,
}
impl Trajectory {
    pub fn new(state: State, gravity: Selector, max_step_nanos: i64) -> Result<Self, Error> {
        finite(state.velocity_mps)?;
        if state.epoch.scale() == TimeScale::Utc || !(1..=60_000_000_000).contains(&max_step_nanos)
        {
            return Err(error("trajectory requires TT/TDB and positive step <=60 s"));
        }
        Ok(Self {
            state,
            gravity,
            max_step_nanos,
        })
    }
    pub fn state(&self) -> State {
        self.state
    }
    /// Velocity Verlet in split galactic cells, symmetric funded-impulse kicks.
    /// Calls the epoch-aware candidate provider at both endpoints. Caller uses a
    /// fixed step and supplies fresh ledger-derived Propulsion when mass changes.
    /// All internal state is committed only after both evaluations succeed.
    pub fn step(
        &mut self,
        elapsed_nanos: i64,
        thrust: Thrust,
        mut sources: impl FnMut(Epoch, GalacticPosition) -> Result<Vec<Source>, Error>,
    ) -> Result<State, Error> {
        self.advance(elapsed_nanos, thrust, |gravity, epoch, position, dt| {
            Ok(gravity
                .sample(&sources(epoch, position)?, position, dt)?
                .meters_per_second_squared)
        })
    }
    /// Epoch-aware indexed snapshots. Arc ownership lets callers reuse an
    /// immutable snapshot without cloning the spatial tree for every ship.
    pub fn step_indexed(
        &mut self,
        elapsed_nanos: i64,
        thrust: Thrust,
        mut index: impl FnMut(
            Epoch,
            GalacticPosition,
        )
            -> Result<std::sync::Arc<salimon_universe::gravity::SourceIndex>, Error>,
    ) -> Result<State, Error> {
        self.advance(elapsed_nanos, thrust, |gravity, epoch, position, dt| {
            let snapshot = index(epoch, position)?;
            Ok(gravity
                .sample_indexed(snapshot.as_ref(), position, dt)?
                .0
                .meters_per_second_squared)
        })
    }
    /// Strang-split quadratic atmospheric drag around the production gravity
    /// and funded-thrust step. The same operation serves online and closed-game
    /// callers; closed-game composition supplies coasting thrust. Any provider
    /// or numerical failure rolls back position, velocity, epoch and selector.
    pub fn step_with_drag(
        &mut self,
        elapsed_nanos: i64,
        thrust: Thrust,
        aerodynamics: crate::atmosphere::Aerodynamics,
        sources: impl FnMut(Epoch, GalacticPosition) -> Result<Vec<Source>, Error>,
        mut air: impl FnMut(Epoch, GalacticPosition) -> Result<Option<crate::atmosphere::Air>, Error>,
    ) -> Result<State, Error> {
        if elapsed_nanos <= 0 || elapsed_nanos > self.max_step_nanos {
            return Err(error("trajectory step exceeds configured bound"));
        }
        let mut staged = self.clone();
        let half_seconds = elapsed_nanos as f64 * 0.5e-9;
        staged.state.velocity_mps = aerodynamics.velocity_after(
            staged.state.velocity_mps,
            thrust.propulsion,
            air(staged.state.epoch, staged.state.position)?,
            half_seconds,
        )?;
        staged.step(elapsed_nanos, thrust, sources)?;
        staged.state.velocity_mps = aerodynamics.velocity_after(
            staged.state.velocity_mps,
            thrust.propulsion,
            air(staged.state.epoch, staged.state.position)?,
            half_seconds,
        )?;
        *self = staged;
        Ok(self.state)
    }
    /// Continuous moving-body/terrain interception of the exact dense drift
    /// before endpoint gravity can enter a solid. All integration/response is
    /// staged; callers settle only elapsed funded work and honor the returned
    /// propulsion shutdown and terminal/grounded/rebound state.
    #[allow(clippy::too_many_arguments)]
    pub fn step_with_contacts(
        &mut self,
        elapsed_nanos: i64,
        thrust: Thrust,
        aerodynamics: crate::atmosphere::Aerodynamics,
        scene: &crate::contact::Scene,
        ship_radius_meters: f64,
        response: crate::contact::ResponsePolicy,
        mut sources: impl FnMut(Epoch, GalacticPosition) -> Result<Vec<Source>, Error>,
        mut air: impl FnMut(Epoch, GalacticPosition) -> Result<Option<crate::atmosphere::Air>, Error>,
    ) -> Result<crate::contact::Advance, Error> {
        if elapsed_nanos <= 0 || elapsed_nanos > self.max_step_nanos {
            return Err(error("trajectory step exceeds configured bound"));
        }
        let thrust_dv = thrust.delta_velocity()?;
        let mut selector = self.gravity.clone();
        let gravity = selector
            .sample(
                &sources(self.state.epoch, self.state.position)?,
                self.state.position,
                0.0,
            )?
            .meters_per_second_squared;
        let drift = crate::contact::DriftPath {
            start: self.state,
            nanos: elapsed_nanos,
            gravity,
            thrust_dv,
            propulsion: thrust.propulsion,
            aero: aerodynamics,
            air: air(self.state.epoch, self.state.position)?,
        };
        let (hit, query) = scene.sweep_path(&drift, ship_radius_meters)?;
        let used = hit.as_ref().map_or(elapsed_nanos, |h| h.elapsed_nanos);
        let funded = thrust.funded_joules * (used as f64 / elapsed_nanos as f64);
        let mut staged = self.clone();
        if used > 0 {
            staged.step_with_drag(
                used,
                Thrust {
                    funded_joules: funded,
                    ..thrust
                },
                aerodynamics,
                &mut sources,
                &mut air,
            )?;
        }
        let resolution = hit
            .as_ref()
            .map(|h| h.resolve(staged.state, thrust.propulsion, response))
            .transpose()?;
        if let Some(resolution) = &resolution {
            staged.state = resolution.state;
        }
        *self = staged;
        Ok(crate::contact::Advance {
            state: self.state,
            hit,
            resolution,
            elapsed_nanos: used,
            remaining_nanos: elapsed_nanos - used,
            unused_funded_thrust_joules: (thrust.funded_joules - funded).max(0.0),
            query,
        })
    }
    fn advance(
        &mut self,
        elapsed_nanos: i64,
        thrust: Thrust,
        mut acceleration: impl FnMut(
            &mut Selector,
            Epoch,
            GalacticPosition,
            f64,
        ) -> Result<[f64; 3], Error>,
    ) -> Result<State, Error> {
        if elapsed_nanos <= 0 || elapsed_nanos > self.max_step_nanos {
            return Err(error("trajectory step exceeds configured bound"));
        }
        let dt = elapsed_nanos as f64 * 1e-9;
        let dv = thrust.delta_velocity()?;
        let mut gravity = self.gravity.clone();
        let a0 = acceleration(&mut gravity, self.state.epoch, self.state.position, 0.0)?;
        let half = add(
            add(self.state.velocity_mps, scale(a0, dt / 2.0)),
            scale(dv, 0.5),
        );
        finite(half)?;
        let position = self.state.position.translated(scale(half, dt))?;
        let epoch = self.state.epoch.advance_nanos(elapsed_nanos)?;
        let a1 = acceleration(&mut gravity, epoch, position, dt)?;
        let velocity_mps = add(add(half, scale(a1, dt / 2.0)), scale(dv, 0.5));
        finite(velocity_mps)?;
        let state = State {
            position,
            velocity_mps,
            epoch,
        };
        self.state = state;
        self.gravity = gravity;
        Ok(state)
    }
}
#[cfg(test)]
mod tests;
