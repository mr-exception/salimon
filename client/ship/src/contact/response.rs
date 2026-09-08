//! Body-relative impact consequences; gameplay owns fatality thresholds.
use super::Hit;
use crate::{
    Propulsion,
    surface::{error, finite},
    trajectory::State,
};
use salimon_math::{add, dot, length, scale, sub};
use salimon_universe::Error;

#[derive(Clone, Copy, Debug)]
pub struct ResponsePolicy {
    restitution: f64,
    maximum_rebound_mps: f64,
    minimum_rebound_mps: f64,
    fatal_relative_speed_mps: Option<f64>,
}
impl Default for ResponsePolicy {
    fn default() -> Self {
        Self {
            restitution: 0.35,
            maximum_rebound_mps: 8.0,
            minimum_rebound_mps: 0.1,
            fatal_relative_speed_mps: None,
        }
    }
}
impl ResponsePolicy {
    /// Fatal thresholds belong to the gameplay/health owner. None preserves
    /// nonfatal legacy ship contact; no new damage/balance value is guessed.
    pub fn new(
        restitution: f64,
        maximum_rebound_mps: f64,
        minimum_rebound_mps: f64,
        fatal_relative_speed_mps: Option<f64>,
    ) -> Result<Self, Error> {
        if !restitution.is_finite()
            || !(0.0..=1.0).contains(&restitution)
            || [maximum_rebound_mps, minimum_rebound_mps]
                .into_iter()
                .any(|v| !v.is_finite() || v < 0.0)
            || minimum_rebound_mps > maximum_rebound_mps
            || fatal_relative_speed_mps.is_some_and(|v| !v.is_finite() || v <= 0.0)
        {
            return Err(error("contact response policy"));
        }
        Ok(Self {
            restitution,
            maximum_rebound_mps,
            minimum_rebound_mps,
            fatal_relative_speed_mps,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Consequence {
    GroundedAfterCrash,
    Rebounding,
    Fatal,
}
#[derive(Clone, Debug)]
pub struct Resolution {
    pub state: State,
    pub consequence: Consequence,
    pub relative_impact_speed_mps: f64,
    pub dissipated_joules: f64,
    pub disable_propulsion: bool,
}
impl Hit {
    pub fn resolve(
        &self,
        state: State,
        propulsion: Propulsion,
        policy: ResponsePolicy,
    ) -> Result<Resolution, Error> {
        finite(self.normal)?;
        finite(self.surface_velocity_mps)?;
        if (length(self.normal) - 1.0).abs() > 1e-12
            || !self.spatial_uncertainty_meters.is_finite()
            || self.spatial_uncertainty_meters <= 0.0
            || self.elapsed_nanos < 0
        {
            return Err(error("invalid contact witness normal/tolerance/time"));
        }
        if state.epoch != self.epoch
            || length(state.position.relative_meters(self.position)?)
                > self.spatial_uncertainty_meters + 0.001
        {
            return Err(error("stale contact state/witness"));
        }
        finite(state.velocity_mps)?;
        let relative = sub(state.velocity_mps, self.surface_velocity_mps);
        finite(relative)?;
        let speed = length(relative);
        let rebound = (-dot(relative, self.normal) * policy.restitution)
            .clamp(0.0, policy.maximum_rebound_mps);
        let consequence = if policy
            .fatal_relative_speed_mps
            .is_some_and(|threshold| speed >= threshold)
        {
            Consequence::Fatal
        } else if rebound == 0.0 || rebound < policy.minimum_rebound_mps {
            Consequence::GroundedAfterCrash
        } else {
            Consequence::Rebounding
        };
        let after = if consequence == Consequence::Rebounding {
            scale(self.normal, rebound)
        } else {
            [0.0; 3]
        };
        let velocity = add(self.surface_velocity_mps, after);
        finite(velocity)?;
        let dissipated_joules =
            0.5 * propulsion.total_mass_kg() * (speed * speed - dot(after, after)).max(0.0);
        if !dissipated_joules.is_finite() {
            return Err(error("impact energy overflow"));
        }
        // Millimeter separation bias lets a subsequent rebound step depart the
        // same surface without repeatedly resolving t=0. Actual witness is retained.
        let position = if consequence == Consequence::Rebounding {
            state
                .position
                .translated(scale(self.normal, 2.0 * self.spatial_uncertainty_meters))?
        } else {
            state.position
        };
        Ok(Resolution {
            state: State {
                position,
                velocity_mps: velocity,
                ..state
            },
            consequence,
            relative_impact_speed_mps: speed,
            dissipated_joules,
            disable_propulsion: true,
        })
    }
}
