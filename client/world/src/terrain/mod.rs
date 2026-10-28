//! Versioned body-local terrain contract shared by native rendering and gameplay.
mod query;
mod surface;
mod wire;
pub use query::SurfaceSample;
pub use surface::TerrainMode;
#[cfg(test)]
mod tests;

use crate::{BodyRole, CelestialBody, CelestialBodyId};

pub const GENERATOR_VERSION: u16 = 1;
pub const MAX_LAYERS: usize = 8;
pub const MAX_FEATURES: usize = 32;
/// 32-byte header + eight 17-byte layers + thirty-two 40-byte features.
pub const MAX_DEFINITION_BYTES: usize = 1448;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayerKind {
    Hills,
    Ridges,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layer {
    pub kind: LayerKind,
    pub amplitude_meters: f64,
    pub wavelength_meters: f64,
}
/// Smooth sparse signed elevation override; negative depth describes a pit.
/// Body-local unit direction; support radius is measured as a chord in meters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Feature {
    pub direction: [f64; 3],
    pub radius_meters: f64,
    pub elevation_meters: f64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerrainError {
    UnsupportedVersion,
    InvalidDefinition,
    InvalidDirection,
    InvalidEncoding,
}

/// Validated immutable definition. Center/orbit/material remain caller-owned.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainDefinition {
    canonical: Option<salimon_universe::starting_system::FrozenStart>,
    body: CelestialBodyId,
    version: u16,
    seed: u64,
    radius_meters: f64,
    layers: Vec<Layer>,
    features: Vec<Feature>,
    bounds_meters: [f64; 2],
}
impl TerrainDefinition {
    pub fn new(
        body: CelestialBody,
        version: u16,
        seed: u64,
        layers: Vec<Layer>,
        features: Vec<Feature>,
    ) -> Result<Self, TerrainError> {
        if version != GENERATOR_VERSION {
            return Err(TerrainError::UnsupportedVersion);
        }
        if body.role != BodyRole::Solid
            || body.id == CelestialBodyId::Sun
            || !body.radius_meters.is_finite()
            || !(1.0..=1e9).contains(&body.radius_meters)
            || layers.len() > MAX_LAYERS
            || features.len() > MAX_FEATURES
        {
            return Err(TerrainError::InvalidDefinition);
        }
        let radius = body.radius_meters;
        let valid_scale = |v: f64| v.is_finite() && v >= 1.0 && v <= radius;
        let valid_height = |v: f64| v.is_finite() && v.abs() <= radius * 0.01;
        let mut bounds = [0.0, 0.0];
        for l in &layers {
            // Bound lattice coordinates to an exactly representable integer domain.
            if !valid_scale(l.wavelength_meters)
                || radius / l.wavelength_meters > 1e9
                || !valid_height(l.amplitude_meters)
                || l.amplitude_meters < 0.0
            {
                return Err(TerrainError::InvalidDefinition);
            }
            bounds[0] -= l.amplitude_meters;
            bounds[1] += l.amplitude_meters;
        }
        for f in &features {
            if !valid_scale(f.radius_meters)
                || !valid_height(f.elevation_meters)
                || !f.direction.iter().all(|v| v.is_finite())
                || (query::norm(f.direction) - 1.0).abs() > 1e-12
            {
                return Err(TerrainError::InvalidDefinition);
            }
            bounds[0] += f.elevation_meters.min(0.0);
            bounds[1] += f.elevation_meters.max(0.0);
        }
        // Conservative extrema stay well inside the existing 0.15R landing shell.
        if bounds[0] < -radius * 0.01 || bounds[1] > radius * 0.01 {
            return Err(TerrainError::InvalidDefinition);
        }
        Ok(Self {
            canonical: None,
            body: body.id,
            version,
            seed,
            radius_meters: radius,
            layers,
            features,
            bounds_meters: bounds,
        })
    }
    /// Exact published U11 height evaluator, not a regenerated noise approximation.
    pub fn canonical_start() -> Self {
        let start = crate::canonical::START.clone();
        let radius = start.manifest().physical.radius_meters;
        let height = start.manifest().terrain.ridge_height_meters;
        Self {
            canonical: Some(start),
            body: CelestialBodyId::Vesper,
            version: GENERATOR_VERSION,
            seed: 0,
            radius_meters: radius,
            layers: Vec::new(),
            features: Vec::new(),
            bounds_meters: [0.0, height],
        }
    }
    pub fn body(&self) -> CelestialBodyId {
        self.body
    }
    pub fn radius_meters(&self) -> f64 {
        self.radius_meters
    }
    pub fn elevation_bounds_meters(&self) -> [f64; 2] {
        self.bounds_meters
    }
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }
    pub fn features(&self) -> &[Feature] {
        &self.features
    }

    /// Immutable native representative profiles; see the terrain integration contract.
    pub fn representative(body: CelestialBody) -> Result<Self, TerrainError> {
        let (seed, hills, ridges, pit) = match body.id {
            CelestialBodyId::Earth => (17001, 35.0, 140.0, -40.0),
            CelestialBodyId::Moon => (17002, 8.0, 25.0, -100.0),
            CelestialBodyId::Mars => (17003, 25.0, 220.0, -65.0),
            _ => return Err(TerrainError::InvalidDefinition),
        };
        Self::new(
            body,
            GENERATOR_VERSION,
            seed,
            vec![
                Layer {
                    kind: LayerKind::Hills,
                    amplitude_meters: hills,
                    wavelength_meters: 500.0,
                },
                Layer {
                    kind: LayerKind::Ridges,
                    amplitude_meters: ridges,
                    wavelength_meters: 2000.0,
                },
            ],
            vec![Feature {
                direction: [0.0, 0.0, 1.0],
                radius_meters: 180.0,
                elevation_meters: pit,
            }],
        )
    }
}
