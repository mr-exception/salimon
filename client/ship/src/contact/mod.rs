//! Continuous real-scale moving-body and streamed-triangle contact.
use crate::{surface::error, trajectory::State};
use salimon_universe::{CatalogId, Epoch, Error, coordinates::GalacticPosition};
use std::collections::BTreeSet;
mod geometry;
mod path;
mod response;
use geometry::Closest;
pub use geometry::Collider;
pub use path::CubicPath;
pub(crate) use path::{Bounds, DriftPath, MotionPath};
pub use response::{Consequence, Resolution, ResponsePolicy};
const DISTANCE_TOLERANCE_METERS: f64 = 0.001;
const MAX_DISTANCE_SAMPLES: usize = 16384;

#[derive(Clone, Copy, Debug, Default)]
pub struct QueryStats {
    pub broadphase_candidates: usize,
    pub narrowphase_candidates: usize,
    pub distance_samples: usize,
}
#[derive(Clone, Debug)]
pub struct Scene {
    colliders: Vec<Collider>,
}
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Hit {
    pub body: CatalogId,
    pub primitive: u64,
    pub elapsed_nanos: i64,
    pub epoch: Epoch,
    pub position: GalacticPosition,
    pub normal: [f64; 3],
    pub surface_velocity_mps: [f64; 3],
    pub spatial_uncertainty_meters: f64,
}
impl Scene {
    pub fn new(mut colliders: Vec<Collider>) -> Result<Self, Error> {
        if colliders.len() > 65536 {
            return Err(error("contact scene collider bound"));
        }
        let mut keys = BTreeSet::new();
        for c in &colliders {
            if !keys.insert((c.body.clone(), c.primitive)) {
                return Err(error("duplicate contact identity"));
            }
        }
        colliders.sort_by(|a, b| (&a.body, a.primitive).cmp(&(&b.body, b.primitive)));
        Ok(Self { colliders })
    }
    pub fn sweep(
        &self,
        ship: &CubicPath,
        ship_radius_meters: f64,
    ) -> Result<(Option<Hit>, QueryStats), Error> {
        self.sweep_path(ship, ship_radius_meters)
    }
    pub(crate) fn sweep_path(
        &self,
        ship: &dyn MotionPath,
        radius: f64,
    ) -> Result<(Option<Hit>, QueryStats), Error> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err(error("ship contact radius"));
        }
        let origin = ship.position(0)?;
        let bounds = ship.bounds(origin)?;
        let mut best: Option<Hit> = None;
        let mut stats = QueryStats::default();
        for collider in &self.colliders {
            let path = collider.path();
            if path.epoch() != ship.epoch() || path.duration_nanos() != ship.duration_nanos() {
                return Err(error("contact scene/path epoch interval mismatch"));
            }
            stats.broadphase_candidates += 1;
            if !bounds.overlaps(collider.bounds(origin)?, radius) {
                continue;
            }
            stats.narrowphase_candidates += 1;
            let speed = collider.relative_speed_bound(ship)?;
            // At uncapped astronomical speeds one epoch nanosecond can exceed
            // 1 mm. Expose that conservative uncertainty, never hide it as exact.
            let tolerance = DISTANCE_TOLERANCE_METERS.max(speed * 1e-9);
            let end = best
                .as_ref()
                .map_or(ship.duration_nanos(), |h| h.elapsed_nanos);
            let frame = path.state_at(0)?.velocity_mps;
            let mut sample_at = |nanos: i64| -> Result<(GalacticPosition, Closest, f64), Error> {
                stats.distance_samples += 1;
                if stats.distance_samples > MAX_DISTANCE_SAMPLES {
                    return Err(error("contact distance work limit; refine/split scene"));
                }
                let position = ship.position(nanos)?;
                let sample = collider.closest(position, nanos)?;
                let gap = sample.distance - radius;
                if !gap.is_finite() {
                    return Err(error("contact gap overflow"));
                }
                Ok((position, sample, gap))
            };
            let (_, _, fa) = sample_at(0)?;
            let fb = if end == 0 { fa } else { sample_at(end)?.2 };
            let mut intervals = vec![(0, end, fa, fb)];
            while let Some((a, b, fa, fb)) = intervals.pop() {
                let width = (b - a) as f64 * 1e-9;
                // Endpoint distance plus Lipschitz speed and a tighter moving
                // interval AABB/curvature certificate both prove clear spans.
                if fa.min(fb) - speed * width * 0.5 > tolerance {
                    continue;
                }
                let ship_bounds = ship.interval_bounds(origin, a, b, frame)?;
                if collider.interval_clearance_bound(ship_bounds, origin, a, b, frame, radius)?
                    > tolerance
                {
                    continue;
                }
                if fa <= tolerance || b - a <= 1 {
                    let (position, sample, gap) = sample_at(a)?;
                    let hit = Hit {
                        body: collider.body.clone(),
                        primitive: collider.primitive,
                        elapsed_nanos: a,
                        epoch: ship.epoch().advance_nanos(a)?,
                        position,
                        normal: sample.normal,
                        surface_velocity_mps: sample.velocity,
                        spatial_uncertainty_meters: tolerance.max(gap),
                    };
                    if best
                        .as_ref()
                        .is_none_or(|old| hit.elapsed_nanos < old.elapsed_nanos)
                    {
                        best = Some(hit);
                    }
                    break;
                }
                let mid = a + (b - a) / 2;
                let fm = sample_at(mid)?.2;
                // LIFO order searches earliest intervals first, including when
                // both distant endpoints are clear but the interior collides.
                intervals.push((mid, b, fm, fb));
                intervals.push((a, mid, fa, fm));
            }
        }
        Ok((best, stats))
    }
}
#[derive(Clone, Debug)]
pub struct Advance {
    pub state: State,
    pub hit: Option<Hit>,
    pub resolution: Option<Resolution>,
    pub elapsed_nanos: i64,
    pub remaining_nanos: i64,
    pub unused_funded_thrust_joules: f64,
    pub query: QueryStats,
}
#[cfg(test)]
mod tests;
