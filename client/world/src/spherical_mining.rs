//! Portable continuous-mining lifecycle; platform input, poses and physics stay outside.
use std::time::Duration;

use crate::resources::{DepositId, RawMaterial, ResourceDeposit, ResourceError};

pub const DEFAULT_SPHERE_CAP_KG: f64 = 10.0;
pub const SPHERE_TICK: Duration = Duration::from_millis(16);
/// Geometric display/support bounds only; never round away material mass.
pub const MIN_SPHERE_RADIUS_METERS: f64 = 0.005;
pub const MAX_SPHERE_RADIUS_METERS: f64 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MiningConfig {
    cap_kg: f64,
    rate_kg_per_second: f64,
}

impl Default for MiningConfig {
    fn default() -> Self {
        Self {
            cap_kg: DEFAULT_SPHERE_CAP_KG,
            rate_kg_per_second: crate::mining::MINING_RATE_KG_PER_SECOND,
        }
    }
}

impl MiningConfig {
    /// Bounded tuning prevents overflow and unrepresentably small per-tick growth.
    pub fn new(cap_kg: f64, rate_kg_per_second: f64) -> Result<Self, ResourceError> {
        if !cap_kg.is_finite()
            || !(1e-6..=10_000.0).contains(&cap_kg)
            || !rate_kg_per_second.is_finite()
            || !(1e-6..=10_000.0).contains(&rate_kg_per_second)
        {
            return Err(ResourceError::InvalidMass);
        }
        Ok(Self {
            cap_kg,
            rate_kg_per_second,
        })
    }
    pub const fn cap_kg(self) -> f64 {
        self.cap_kg
    }
    pub const fn rate_kg_per_second(self) -> f64 {
        self.rate_kg_per_second
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrowingSphere {
    pub source: DepositId,
    pub material: RawMaterial,
}

impl GrowingSphere {
    pub fn radius_meters(self) -> f64 {
        crate::resource_size::fragment_contact_radius_meters(self.material)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReleaseReason {
    InputReleased,
    AimLost,
    TargetChanged,
    Capacity,
    Depleted,
    Interrupted,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReleasedSphere {
    pub sphere: GrowingSphere,
    pub reason: ReleaseReason,
}

/// One bounded tick produces at most one release. No hidden mass or banked work.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MiningStep {
    pub added_kg: f64,
    pub released: Option<ReleasedSphere>,
}

#[derive(Debug, Default)]
pub struct SphericalMining {
    attached: Option<GrowingSphere>,
}

impl SphericalMining {
    pub const fn attached(&self) -> Option<GrowingSphere> {
        self.attached
    }

    /// Use on unequip, focus/lifecycle loss or explicit pause. A zero-duration
    /// simulation tick alone neither grows nor releases valid held work.
    pub fn interrupt(&mut self) -> MiningStep {
        self.release(ReleaseReason::Interrupted)
    }

    fn release(&mut self, reason: ReleaseReason) -> MiningStep {
        MiningStep {
            added_kg: 0.0,
            released: self
                .attached
                .take()
                .map(|sphere| ReleasedSphere { sphere, reason }),
        }
    }

    /// Caller restores the stable-ID deposit journal first and supplies one
    /// valid aimed deposit. Large deltas admit at most one 16 ms tick; runtime
    /// drains its fixed-step backlog instead of banking tool work here.
    pub fn tick(
        &mut self,
        held: bool,
        target: Option<&mut ResourceDeposit>,
        delta: Duration,
        config: MiningConfig,
    ) -> MiningStep {
        if !held {
            return self.release(ReleaseReason::InputReleased);
        }
        let Some(deposit) = target else {
            return self.release(ReleaseReason::AimLost);
        };
        if self.attached.is_some_and(|sphere| {
            sphere.source != deposit.id()
                || sphere.material.resource() != deposit.material().resource()
        }) {
            // Release wins: switching targets cannot mutate either sphere this tick.
            return self.release(ReleaseReason::TargetChanged);
        }
        if deposit.remaining_mass_kg() == 0.0 {
            return self.release(ReleaseReason::Depleted);
        }
        let previous = self.attached.map_or(0.0, |s| s.material.mass_kg());
        if previous >= config.cap_kg {
            return self.release(ReleaseReason::Capacity);
        }
        let limit = (config.rate_kg_per_second * delta.min(SPHERE_TICK).as_secs_f64())
            .min(config.cap_kg - previous)
            .min(deposit.remaining_mass_kg());
        if limit <= 0.0 {
            return MiningStep::default();
        }
        let before = deposit.remaining_mass_kg();
        let mut remaining = before - limit;
        // Never debit more than the cap/rate because subtraction rounded upward.
        if before - remaining > limit {
            remaining = remaining.next_up().min(before);
        }
        let removed = before - remaining;
        if removed <= 0.0 {
            return MiningStep::default();
        }
        let Ok(material) = RawMaterial::new(deposit.material().resource(), previous + removed)
        else {
            return MiningStep::default();
        };
        deposit
            .set_remaining_mass_kg(remaining)
            .expect("bounded conservative debit");
        self.attached = Some(GrowingSphere {
            source: deposit.id(),
            material,
        });
        // A rounding-only cap gap is released, never debited or made into dust.
        let reason = if remaining == 0.0 {
            Some(ReleaseReason::Depleted)
        } else if config.cap_kg - material.mass_kg() <= 1e-12 {
            Some(ReleaseReason::Capacity)
        } else {
            None
        };
        let mut result = reason.map_or_else(MiningStep::default, |reason| self.release(reason));
        result.added_kg = removed;
        result
    }
}
