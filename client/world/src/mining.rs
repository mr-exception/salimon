//! Portable aimed extraction. Runtime supplies proximity candidates and obstruction distance.
use std::time::Duration;

use crate::resource_generation::SurfaceDeposit;
use crate::resources::{DepositId, DepositState, ResourceDeposit, ResourceFragment};
use crate::{BodyRole, CELESTIAL_BODIES, WorldPosition};

pub const MINING_RANGE_METERS: f64 = 4.0;
pub const MINING_RATE_KG_PER_SECOND: f64 = 2.0;

#[derive(Clone, Copy, Debug)]
pub struct MiningRay {
    pub origin: WorldPosition,
    pub direction: [f64; 3],
}

impl MiningRay {
    pub fn new(origin: WorldPosition, target: WorldPosition) -> Option<Self> {
        let direction = target.offset_from(origin);
        let length = dot(direction, direction).sqrt();
        (origin.is_finite() && length.is_finite() && length > 0.0).then(|| Self {
            origin,
            direction: direction.map(|v| v / length),
        })
    }

    /// First forward intersection, including rays beginning within the bound.
    pub fn sphere_distance(self, center: WorldPosition, radius: f64) -> Option<f64> {
        let offset = center.offset_from(self.origin);
        let along = dot(offset, self.direction);
        // Perpendicular-distance form avoids subtracting two planetary-scale squares.
        let perpendicular = std::array::from_fn(|i| offset[i] - along * self.direction[i]);
        let discriminant = radius * radius - dot(perpendicular, perpendicular);
        if discriminant < 0.0 {
            return None;
        }
        let half_chord = discriminant.sqrt();
        if along + half_chord < 0.0 {
            None
        } else {
            Some((along - half_chord).max(0.0))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MiningTarget {
    pub id: DepositId,
    pub distance_meters: f64,
}

pub fn aimed_deposit(
    ray: MiningRay,
    deposits: &[SurfaceDeposit],
    obstruction_distance: Option<f64>,
) -> Option<MiningTarget> {
    let planet_distance = CELESTIAL_BODIES
        .iter()
        .filter(|body| body.role == BodyRole::Solid)
        .filter_map(|body| ray.sphere_distance(body.center, body.radius_meters))
        .min_by(f64::total_cmp);
    let blocked_at = planet_distance
        .into_iter()
        .chain(obstruction_distance)
        .min_by(f64::total_cmp)
        .unwrap_or(f64::INFINITY);
    deposits
        .iter()
        .filter(|entry| entry.deposit.state() != DepositState::Depleted)
        .filter_map(|entry| {
            let distance =
                ray.sphere_distance(entry.deposit.position(), entry.bounds_radius_meters)?;
            (distance <= MINING_RANGE_METERS && distance < blocked_at).then_some(MiningTarget {
                id: entry.deposit.id(),
                distance_meters: distance,
            })
        })
        .min_by(|a, b| {
            a.distance_meters
                .total_cmp(&b.distance_meters)
                .then_with(|| a.id.local.cmp(&b.id.local))
        })
}

/// Returns the actual mass difference, preserving conservation despite floating-point rounding.
pub fn extract(deposit: &mut ResourceDeposit, delta: Duration) -> f64 {
    let before = deposit.remaining_mass_kg();
    let removed =
        (MINING_RATE_KG_PER_SECOND * delta.as_secs_f64()).min(deposit.remaining_mass_kg());
    deposit
        .set_remaining_mass_kg(deposit.remaining_mass_kg() - removed)
        .expect("bounded extraction can only decrease finite remaining mass");
    before - deposit.remaining_mass_kg()
}

/// Session-owned extraction deltas keyed by stable deposit identity, separate from presentation.
#[derive(Default)]
pub struct MiningSession {
    remaining: std::collections::HashMap<DepositId, f64>,
    extracted_mass_kg: f64,
    fragments: crate::resource_fragments::FragmentOutput,
}

impl MiningSession {
    pub fn apply_to(&self, deposits: &mut [SurfaceDeposit]) {
        for entry in deposits {
            if let Some(mass) = self.remaining.get(&entry.deposit.id()) {
                entry
                    .deposit
                    .set_remaining_mass_kg(*mass)
                    .expect("session extraction mass was validated on mutation");
            }
        }
    }

    pub fn extract(&mut self, deposit: &mut ResourceDeposit, delta: Duration) -> f64 {
        let removed = extract(deposit, delta);
        self.fragments.emit(*deposit, removed);
        self.remaining
            .insert(deposit.id(), deposit.remaining_mass_kg());
        self.extracted_mass_kg += removed;
        removed
    }

    /// Diagnostic extraction total, never spendable inventory or carried mass.
    pub const fn extracted_mass_kg(&self) -> f64 {
        self.extracted_mass_kg
    }

    /// Physical session entities, independent of active deposit streaming and tool equipment.
    pub fn fragments(&self) -> &[ResourceFragment] {
        self.fragments.fragments()
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}
