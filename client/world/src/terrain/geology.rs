//! Immutable geological interpretation of the authoritative terrain, never a second heightfield.
use super::{SurfaceSample, TerrainDefinition, TerrainError, query};
use crate::CelestialBody;
use salimon_universe::synthesis::sha256;

pub const GEOLOGY_VERSION: u16 = 1;
const MAX_BYTES: usize = super::MAX_DEFINITION_BYTES + 32;

/// Explicit authoring choice, not a habitability or measured-composition claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum GeologyProfile {
    DryRock = 0,
    WeatheredRock = 1,
    Ice = 2,
    HomogeneousRock = 3,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeologicalProcess {
    Impacts,
    Volcanism,
    TectonicDeformation,
    Deposition,
    Weathering,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Substrate {
    Silicate,
    Basalt,
    Regolith,
    WaterIce,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Landform {
    Plain,
    Highland,
    ImpactBasin,
    Fractured,
    IcyExpanse,
}
/// Ordered oldest-to-youngest conceptual stages, not a geophysical simulation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HistoryStage {
    pub process: GeologicalProcess,
    pub relative_extent: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct GeologySample {
    /// Body/revision/recipe-bound province identity; no tile, LOD, camera or time.
    pub province_id: String,
    pub neighboring_province_id: String,
    /// Normalized weight of the dominant province; all eight contribute to continuous fields.
    pub province_weight: f64,
    pub landform: Landform,
    pub substrate: Substrate,
    pub underlying_substrate: Substrate,
    pub weathering: f64,
    pub exposure: f64,
    pub layer_depth_meters: f64,
    pub history: Vec<HistoryStage>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct GeologicalSurface {
    pub surface: SurfaceSample,
    pub geology: GeologySample,
}
/// Owns an exact terrain snapshot. Revisions cannot silently rebind geology to new ground.
#[derive(Clone, Debug, PartialEq)]
pub struct GeologyDefinition {
    terrain: TerrainDefinition,
    seed: u64,
    profile: GeologyProfile,
    identity: String,
}
impl GeologyDefinition {
    pub fn new(
        terrain: TerrainDefinition,
        version: u16,
        seed: u64,
        profile: GeologyProfile,
    ) -> Result<Self, TerrainError> {
        if version != GEOLOGY_VERSION {
            return Err(TerrainError::UnsupportedVersion);
        }
        let mut definition = Self {
            terrain,
            seed,
            profile,
            identity: String::new(),
        };
        definition.identity = sha256(&definition.to_bytes());
        Ok(definition)
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn terrain(&self) -> &TerrainDefinition {
        &self.terrain
    }
    pub fn profile(&self) -> GeologyProfile {
        self.profile
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = b"SLMGEO01".to_vec();
        bytes.extend(GEOLOGY_VERSION.to_le_bytes());
        bytes.extend(self.seed.to_le_bytes());
        bytes.push(self.profile as u8);
        bytes.extend(self.terrain.to_bytes());
        bytes
    }
    /// Unknown versions, malformed upstream terrain and trailing bytes fail closed.
    pub fn from_bytes(body: CelestialBody, bytes: &[u8]) -> Result<Self, TerrainError> {
        if bytes.len() < 19 || bytes.len() > MAX_BYTES || &bytes[..8] != b"SLMGEO01" {
            return Err(TerrainError::InvalidEncoding);
        }
        let version = u16::from_le_bytes([bytes[8], bytes[9]]);
        let mut word = [0; 8];
        word.copy_from_slice(&bytes[10..18]);
        let profile = match bytes[18] {
            0 => GeologyProfile::DryRock,
            1 => GeologyProfile::WeatheredRock,
            2 => GeologyProfile::Ice,
            3 => GeologyProfile::HomogeneousRock,
            _ => return Err(TerrainError::InvalidDefinition),
        };
        Self::new(
            TerrainDefinition::from_bytes(body, &bytes[19..])?,
            version,
            u64::from_le_bytes(word),
            profile,
        )
    }
    pub fn compatible_with(&self, identity: &str, terrain: &TerrainDefinition) -> bool {
        self.identity == identity && self.terrain.to_bytes() == terrain.to_bytes()
    }
    /// Body-fixed unit direction, using exactly the collision/landing terrain sample.
    pub fn sample(&self, direction: [f64; 3]) -> Result<GeologicalSurface, TerrainError> {
        let surface = self.terrain.sample(direction)?;
        let (first, second, weight) = self.provinces(direction);
        let mut values = [0.0; 5];
        let mut total = 0.0;
        let count = if self.profile == GeologyProfile::HomogeneousRock {
            1
        } else {
            8
        };
        for id in 0..count {
            let distance = query::norm(std::array::from_fn(|i| direction[i] - self.center(id)[i]));
            let influence = if count == 1 {
                1.0
            } else {
                1.0 / (0.04 + distance * distance).powi(2)
            };
            total += influence;
            for (value, province) in values.iter_mut().zip(self.province(id)) {
                *value += influence * province;
            }
        }
        let values = values.map(|x| x / total);
        let bounds = self.terrain.elevation_bounds_meters();
        let relief = bounds[1] - bounds[0];
        let elevation = if relief > 0.0 {
            ((surface.elevation_meters - bounds[0]) / relief).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let basin = self.terrain.features().iter().any(|f| {
            f.elevation_meters < 0.0
                && query::norm(std::array::from_fn(|i| direction[i] - f.direction[i]))
                    * self.terrain.radius_meters()
                    < f.radius_meters * 0.7
        });
        let landform = if self.profile == GeologyProfile::Ice {
            Landform::IcyExpanse
        } else if basin {
            Landform::ImpactBasin
        } else if self.profile == GeologyProfile::HomogeneousRock {
            Landform::Plain
        } else if surface.slope > 0.35 {
            Landform::Fractured
        } else if elevation > 0.65 {
            Landform::Highland
        } else {
            Landform::Plain
        };
        let weathering = values[0];
        let exposure = (surface.slope / (1.0 + surface.slope)).clamp(0.0, 1.0);
        let substrate = match self.profile {
            GeologyProfile::Ice => Substrate::WaterIce,
            GeologyProfile::HomogeneousRock => Substrate::Silicate,
            _ if exposure > 0.3 => Substrate::Basalt,
            _ => Substrate::Regolith,
        };
        let mut history = vec![HistoryStage {
            process: GeologicalProcess::Impacts,
            relative_extent: values[1],
        }];
        if self.profile != GeologyProfile::Ice && self.profile != GeologyProfile::HomogeneousRock {
            history.push(HistoryStage {
                process: GeologicalProcess::Volcanism,
                relative_extent: values[2],
            });
        }
        if self.profile == GeologyProfile::WeatheredRock {
            history.push(HistoryStage {
                process: GeologicalProcess::TectonicDeformation,
                relative_extent: values[3],
            });
        }
        history.push(HistoryStage {
            process: GeologicalProcess::Deposition,
            relative_extent: 1.0 - exposure,
        });
        if self.profile == GeologyProfile::WeatheredRock {
            history.push(HistoryStage {
                process: GeologicalProcess::Weathering,
                relative_extent: weathering,
            });
        }
        Ok(GeologicalSurface {
            surface,
            geology: GeologySample {
                province_id: format!("{}:{first}", self.identity),
                neighboring_province_id: format!("{}:{second}", self.identity),
                province_weight: weight,
                landform,
                substrate,
                underlying_substrate: if self.profile == GeologyProfile::Ice {
                    Substrate::WaterIce
                } else {
                    Substrate::Silicate
                },
                weathering,
                exposure,
                layer_depth_meters: values[4] * (1.0 - exposure),
                history,
            },
        })
    }
    // Sparse spherical provinces avoid cube-face seams and per-tile RNG. All centers
    // blend continuously, including triple junctions; query order never enters sampling.
    fn provinces(&self, d: [f64; 3]) -> (u8, u8, f64) {
        if self.profile == GeologyProfile::HomogeneousRock {
            return (0, 0, 1.0);
        }
        let mut nearest = [(f64::INFINITY, 0_u8); 2];
        for id in 0..8 {
            let center = self.center(id);
            let distance = query::norm(std::array::from_fn(|i| d[i] - center[i]));
            if distance < nearest[0].0 {
                nearest[1] = nearest[0];
                nearest[0] = (distance, id);
            } else if distance < nearest[1].0 {
                nearest[1] = (distance, id);
            }
        }
        let total: f64 = (0..8)
            .map(|id| {
                let c = self.center(id);
                let distance = query::norm(std::array::from_fn(|i| d[i] - c[i]));
                1.0 / (0.04 + distance * distance).powi(2)
            })
            .sum();
        let weight = 1.0 / (0.04 + nearest[0].0 * nearest[0].0).powi(2) / total;
        (nearest[0].1, nearest[1].1, weight)
    }
    fn center(&self, id: u8) -> [f64; 3] {
        let p = std::array::from_fn(|i| {
            let sign = if id & (1 << i) == 0 { -1.0 } else { 1.0 };
            sign * (0.6 + self.random(id, i as u8) * 0.4)
        });
        let length = query::norm(p);
        p.map(|x| x / length)
    }
    fn province(&self, id: u8) -> [f64; 5] {
        [
            if self.profile == GeologyProfile::WeatheredRock {
                0.3 + self.random(id, 3) * 0.5
            } else {
                0.0
            },
            self.random(id, 4),
            self.random(id, 5),
            self.random(id, 6),
            0.25 + self.random(id, 7) * 3.75,
        ]
    }
    fn random(&self, id: u8, domain: u8) -> f64 {
        let mut x = self.seed ^ (u64::from(id) << 32) ^ u64::from(domain);
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
        ((x ^ (x >> 31)) >> 11) as f64 / (1_u64 << 53) as f64
    }
}
impl TerrainDefinition {
    /// Additive interpretation; existing terrain bytes, heights and resource seeds stay frozen.
    pub fn geology(&self) -> GeologyDefinition {
        let profile = match self.body() {
            crate::CelestialBodyId::Earth => GeologyProfile::WeatheredRock,
            crate::CelestialBodyId::Vesper => GeologyProfile::HomogeneousRock,
            _ => GeologyProfile::DryRock,
        };
        GeologyDefinition::new(self.clone(), GEOLOGY_VERSION, self.seed, profile)
            .expect("supported built-in geology recipe")
    }
}
#[cfg(test)]
mod tests;
