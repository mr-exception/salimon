//! Cosmetic exposure only. Condition, mass, capability and repair belong to physical owners.
use salimon_persistence::{SaveError, Validate, invalid};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Duration};

const LIMIT: u64 = 100 * 365 * 86400 * 1_000_000_000;
pub const FABRICATOR: u64 = 10;
pub const GALLEY: u64 = 11;
pub const CABIN_FLOOR: u64 = 12;
pub const CARGO_FLOOR: u64 = 13;
pub const PILOT_SEAT: u64 = 14;
pub const TOOL: u64 = 15;
// Disjoint namespaces retain exact physical IDs, including after detachment.
pub fn part(id: u64) -> u64 {
    (1 << 62) | id
}
pub fn project(id: u64) -> u64 {
    (2 << 62) | id
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exposure {
    born_nanos: u64,
    use_nanos: u64,
}
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceAge {
    elapsed_nanos: u64,
    surfaces: BTreeMap<u64, Exposure>,
}
impl SurfaceAge {
    pub fn ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.surfaces.keys().copied()
    }
    pub fn register(&mut self, id: u64) {
        self.surfaces.entry(id).or_insert(Exposure {
            born_nanos: self.elapsed_nanos,
            use_nanos: 0,
        });
    }
    /// Installing a replacement uses a new physical ID. Existing records never reset.
    pub fn advance(&mut self, delta: Duration) {
        self.elapsed_nanos = self.elapsed_nanos.saturating_add(nanos(delta)).min(LIMIT);
    }
    pub fn contact(&mut self, id: u64, duration: Duration) {
        self.register(id);
        let e = self.surfaces.get_mut(&id).expect("registered exposure");
        e.use_nanos = e.use_nanos.saturating_add(nanos(duration)).min(LIMIT);
    }
    /// Quantized mechanism travel: reversal/obstruction counts actual travel only.
    pub fn travel(&mut self, id: u64, from: f64, to: f64) {
        let quantize = |v: f64| (v.clamp(0., 1.) * 1_000_000.).round() as u64;
        self.contact(
            id,
            Duration::from_micros(quantize(from).abs_diff(quantize(to))),
        );
    }
    /// Slow passive exposure plus use. Maximum remains a moderate maintained finish.
    pub fn intensity(&self, id: u64) -> f32 {
        let Some(e) = self.surfaces.get(&id) else {
            return 0.;
        };
        let passive = (self.elapsed_nanos - e.born_nanos) as f64 / (120. * 86400. * 1e9);
        let usage = e.use_nanos as f64 / (1200. * 1e9);
        (0.12 * passive.min(1.) + 0.32 * usage.min(1.)) as f32
    }
}
fn nanos(d: Duration) -> u64 {
    d.as_nanos().min(u128::from(LIMIT)) as u64
}
impl Validate for SurfaceAge {
    fn validate(&self) -> Result<(), SaveError> {
        if self.elapsed_nanos > LIMIT
            || self.surfaces.len() > 16384
            || self
                .surfaces
                .iter()
                .any(|(id, e)| *id == 0 || e.born_nanos > self.elapsed_nanos || e.use_nanos > LIMIT)
        {
            return Err(invalid("surface exposure identity/time/budget"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slow_idle_use_partition_replacement_and_exact_roundtrip() {
        let mut idle = SurfaceAge::default();
        idle.register(1001);
        idle.advance(Duration::from_secs(86400));
        assert!(idle.intensity(1001) < 0.002);
        let mut used = idle.clone();
        for _ in 0..600 {
            used.travel(1001, 0., 1.);
            used.travel(1001, 1., 0.);
        }
        assert!(used.intensity(1001) > 0.32);
        let mut whole = idle.clone();
        whole.contact(1001, Duration::from_secs(1200));
        assert_eq!(used, whole);
        used.register(1002);
        assert_eq!(used.intensity(1002), 0.);
        assert!(used.intensity(1001) > 0.32);
        let copy: SurfaceAge = serde_json::from_slice(&serde_json::to_vec(&used).unwrap()).unwrap();
        assert_eq!(used, copy);
        copy.validate().unwrap();
        used.advance(Duration::MAX);
        used.contact(1001, Duration::MAX);
        assert!(used.intensity(1001) <= 0.44);
        used.validate().unwrap();
    }
    #[test]
    fn corrupt_exposure_rejects() {
        let mut s = SurfaceAge::default();
        s.register(1);
        s.surfaces.get_mut(&1).unwrap().born_nanos = 1;
        assert!(s.validate().is_err());
    }
}
