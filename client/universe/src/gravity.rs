//! Deterministic dominant-source celestial gravity with explicit transition state.
use crate::{CatalogId, Error, coordinates::GalacticPosition, invalid};
use salimon_math::{add, length, scale};
use std::collections::{BTreeMap, BTreeSet};
mod index;
pub use index::{QueryWork, SourceIndex};
#[derive(Clone, Debug)]
pub struct Source {
    pub id: CatalogId,
    pub position: GalacticPosition,
    pub mu_m3_per_s2: f64,
    pub radius_meters: f64,
}
impl Source {
    /// Evaluate a body's center through the epoch-checked U02 hierarchy before
    /// gravity subtraction; callers supply known positive mu and physical radius.
    pub fn from_frame(
        tree: &crate::coordinates::FrameTree,
        id: CatalogId,
        epoch: crate::Epoch,
        mu_m3_per_s2: f64,
        radius_meters: f64,
    ) -> Result<Self, Error> {
        let position = tree.galactic_position(
            &crate::coordinates::FrameId::Body(id.clone()),
            crate::coordinates::State {
                meters: [0.0; 3],
                meters_per_second: [0.0; 3],
            },
            epoch,
        )?;
        if !mu_m3_per_s2.is_finite()
            || mu_m3_per_s2 <= 0.0
            || !radius_meters.is_finite()
            || radius_meters < 0.0
        {
            return Err(invalid("gravity source physical properties"));
        }
        Ok(Self {
            id,
            position,
            mu_m3_per_s2,
            radius_meters,
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub contributors: usize,
    pub hysteresis_fraction: f64,
    pub transition_seconds: f64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            contributors: 4,
            hysteresis_fraction: 0.05,
            transition_seconds: 1.0,
        }
    }
}
#[derive(Clone, Debug)]
struct Transition {
    from: Vec<CatalogId>,
    to: Vec<CatalogId>,
    elapsed: f64,
}
#[derive(Clone, Debug)]
pub struct Selector {
    config: Config,
    selected: Vec<CatalogId>,
    transition: Option<Transition>,
}
#[derive(Debug)]
pub struct Acceleration {
    pub meters_per_second_squared: [f64; 3],
    pub weights: Vec<(CatalogId, f64)>,
    pub transitioning: bool,
}
fn fields(
    sources: &[Source],
    position: GalacticPosition,
) -> Result<BTreeMap<CatalogId, ([f64; 3], f64)>, Error> {
    if sources.len() > 65536 {
        return Err(invalid("gravity candidate bound"));
    }
    let mut fields = BTreeMap::new();
    for source in sources {
        if !source.mu_m3_per_s2.is_finite()
            || source.mu_m3_per_s2 <= 0.0
            || !source.radius_meters.is_finite()
            || source.radius_meters < 0.0
        {
            return Err(invalid("gravity source mass/radius"));
        }
        let delta = source.position.relative_meters(position)?;
        let r = delta[0].hypot(delta[1]).hypot(delta[2]);
        if !r.is_finite() || r <= source.radius_meters || r == 0.0 {
            return Err(invalid(
                "gravity source interior/singularity; contact adapter required",
            ));
        }
        let score = source.mu_m3_per_s2 / r / r;
        let acceleration = delta.map(|x| x / r * score);
        if !score.is_finite()
            || !acceleration.into_iter().all(f64::is_finite)
            || fields
                .insert(source.id.clone(), (acceleration, score))
                .is_some()
        {
            return Err(invalid("gravity duplicate/overflow"));
        }
    }
    Ok(fields)
}
impl Selector {
    pub fn new(config: Config) -> Result<Self, Error> {
        if config.contributors == 0
            || config.contributors > 64
            || !config.hysteresis_fraction.is_finite()
            || !(0.0..=0.5).contains(&config.hysteresis_fraction)
            || !config.transition_seconds.is_finite()
            || config.transition_seconds <= 0.0
        {
            return Err(invalid("gravity selector configuration"));
        }
        Ok(Self {
            config,
            selected: vec![],
            transition: None,
        })
    }
    /// Zero dt advances no blend time, but may initialize selection or begin a
    /// handoff. Retain all active sources until the transition completes;
    /// missing active sources fail rather than abruptly removing their force.
    pub fn sample(
        &mut self,
        sources: &[Source],
        position: GalacticPosition,
        dt_seconds: f64,
    ) -> Result<Acceleration, Error> {
        if !dt_seconds.is_finite() || dt_seconds < 0.0 {
            return Err(invalid("gravity transition time"));
        }
        let fields = fields(sources, position)?;
        let active = self
            .transition
            .as_ref()
            .map(|t| t.from.iter().chain(&t.to).collect::<Vec<_>>())
            .unwrap_or_else(|| self.selected.iter().collect());
        if active.iter().any(|id| !fields.contains_key(*id)) {
            return Err(invalid("gravity active source missing from candidates"));
        }
        let mut next = self.clone();
        if next.selected.is_empty() {
            next.selected = rank(&fields, &[], 0.0, next.config.contributors);
        } else if next.transition.is_none() {
            let target = rank(
                &fields,
                &next.selected,
                next.config.hysteresis_fraction,
                next.config.contributors,
            );
            if target.iter().collect::<BTreeSet<_>>() != next.selected.iter().collect() {
                next.transition = Some(Transition {
                    from: next.selected.clone(),
                    to: target,
                    elapsed: 0.0,
                });
            }
        }
        let mut weights = BTreeMap::new();
        if let Some(t) = &mut next.transition {
            t.elapsed = (t.elapsed + dt_seconds).min(next.config.transition_seconds);
            let u = t.elapsed / next.config.transition_seconds;
            // Smoothstep has zero endpoint slope. At most 2*N sources are
            // evaluated during a handoff, exactly N at steady state.
            let alpha = u * u * (3.0 - 2.0 * u);
            for id in &t.from {
                *weights.entry(id.clone()).or_insert(0.0) += 1.0 - alpha;
            }
            for id in &t.to {
                *weights.entry(id.clone()).or_insert(0.0) += alpha;
            }
            if u >= 1.0 {
                next.selected = t.to.clone();
                next.transition = None;
            }
        } else {
            for id in &next.selected {
                weights.insert(id.clone(), 1.0);
            }
        }
        weights.retain(|_, w| *w > 0.0);
        let mut acceleration = [0.0; 3];
        for (id, w) in &weights {
            acceleration = add(acceleration, scale(fields[id].0, *w));
        }
        if !acceleration.into_iter().all(f64::is_finite) {
            return Err(invalid("gravity acceleration overflow"));
        }
        let transitioning = next.transition.is_some();
        *self = next;
        Ok(Acceleration {
            meters_per_second_squared: acceleration,
            weights: weights.into_iter().collect(),
            transitioning,
        })
    }
}
fn rank(
    fields: &BTreeMap<CatalogId, ([f64; 3], f64)>,
    old: &[CatalogId],
    hysteresis: f64,
    count: usize,
) -> Vec<CatalogId> {
    let mut ranked: Vec<_> = fields
        .iter()
        .map(|(id, (_, score))| {
            (
                id,
                score
                    / (if old.contains(id) {
                        1.0
                    } else {
                        1.0 + hysteresis
                    }),
            )
        })
        .collect();
    ranked.sort_by(|(a, sa), (b, sb)| sb.total_cmp(sa).then_with(|| a.cmp(b)));
    ranked
        .into_iter()
        .take(count)
        .map(|(id, _)| id.clone())
        .collect()
}
/// All admitted sources, without selection or smoothing. Suitable comparison
/// baseline for truncation error, not an independent scientific truth model.
pub fn all_sources(sources: &[Source], position: GalacticPosition) -> Result<[f64; 3], Error> {
    let mut out = [0.0; 3];
    for (a, _) in fields(sources, position)?.values() {
        out = add(out, *a);
    }
    if !out.into_iter().all(f64::is_finite) {
        return Err(invalid("reference acceleration overflow"));
    }
    Ok(out)
}
pub fn relative_error(selected: [f64; 3], reference: [f64; 3]) -> Result<f64, Error> {
    if !selected.into_iter().chain(reference).all(f64::is_finite) {
        return Err(invalid("gravity comparison finite"));
    }
    let error = length(salimon_math::sub(selected, reference));
    let norm = length(reference);
    if norm == 0.0 {
        if error == 0.0 {
            Ok(0.0)
        } else {
            Err(invalid("relative error undefined at zero field"))
        }
    } else {
        let value = error / norm;
        if value.is_finite() {
            Ok(value)
        } else {
            Err(invalid("gravity comparison overflow"))
        }
    }
}
#[cfg(test)]
mod tests;
