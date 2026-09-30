//! Validated, renderer-independent inputs for deterministic planetary generation.
use crate::CelestialBodyId;
use crate::resources::ResourceId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DistributionError {
    InvalidWeight,
    InvalidSpacing,
    InvalidMassRange,
    DuplicateResource,
    InvalidBiome,
    DuplicateBiome,
}

impl std::fmt::Display for DistributionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}
impl std::error::Error for DistributionError {}

/// Relative abundance is a positive selection weight, not a percentage.
/// Spacing is the generator's nominal surface spacing in meters; mass bounds
/// describe initial deposit kilograms. Actual sampling belongs to generation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResourceDistribution {
    resource: ResourceId,
    weight: f64,
    spacing_meters: f64,
    mass_range_kg: [f64; 2],
}

impl ResourceDistribution {
    pub fn new(
        resource: ResourceId,
        weight: f64,
        spacing_meters: f64,
        mass_range_kg: [f64; 2],
    ) -> Result<Self, DistributionError> {
        if !weight.is_finite() || weight <= 0.0 {
            return Err(DistributionError::InvalidWeight);
        }
        if !spacing_meters.is_finite() || spacing_meters <= 0.0 {
            return Err(DistributionError::InvalidSpacing);
        }
        if !mass_range_kg
            .iter()
            .all(|mass| mass.is_finite() && *mass > 0.0)
            || mass_range_kg[0] > mass_range_kg[1]
        {
            return Err(DistributionError::InvalidMassRange);
        }
        Ok(Self {
            resource,
            weight,
            spacing_meters,
            mass_range_kg,
        })
    }
    pub const fn resource(self) -> ResourceId {
        self.resource
    }
    pub const fn weight(self) -> f64 {
        self.weight
    }
    pub const fn spacing_meters(self) -> f64 {
        self.spacing_meters
    }
    pub const fn mass_range_kg(self) -> [f64; 2] {
        self.mass_range_kg
    }
}

/// A named biome replaces the complete body-wide list, including with an empty
/// list. Unknown/absent biomes use the body-wide list. No biome terrain is added.
#[derive(Clone, Debug, PartialEq)]
pub struct BiomeResourceOverride {
    biome: String,
    resources: Vec<ResourceDistribution>,
}
impl BiomeResourceOverride {
    pub fn new(
        biome: String,
        resources: Vec<ResourceDistribution>,
    ) -> Result<Self, DistributionError> {
        if biome.is_empty() || biome.trim() != biome {
            return Err(DistributionError::InvalidBiome);
        }
        validate_resources(&resources)?;
        Ok(Self { biome, resources })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BodyResourceDistribution {
    body: CelestialBodyId,
    seed: u64,
    resources: Vec<ResourceDistribution>,
    biomes: Vec<BiomeResourceOverride>,
}

/// Canonical material-key order and explicit seeds form the contract consumed
/// by #42. No process-random hash, wall clock, or catalog index participates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResourceGenerationInput {
    pub distribution: ResourceDistribution,
    pub seed: u64,
}

impl BodyResourceDistribution {
    pub fn new(
        body: CelestialBodyId,
        seed: u64,
        resources: Vec<ResourceDistribution>,
        biomes: Vec<BiomeResourceOverride>,
    ) -> Result<Self, DistributionError> {
        validate_resources(&resources)?;
        for (index, biome) in biomes.iter().enumerate() {
            if biomes[..index]
                .iter()
                .any(|other| other.biome == biome.biome)
            {
                return Err(DistributionError::DuplicateBiome);
            }
        }
        Ok(Self {
            body,
            seed,
            resources,
            biomes,
        })
    }
    pub const fn body(&self) -> CelestialBodyId {
        self.body
    }
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    pub fn generation_inputs(
        &self,
        world_seed: u64,
        biome: Option<&str>,
    ) -> Vec<ResourceGenerationInput> {
        let selected = biome.and_then(|key| self.biomes.iter().find(|entry| entry.biome == key));
        let resources = selected.map_or(&self.resources, |entry| &entry.resources);
        let body_key = match self.body {
            CelestialBodyId::Sun => "sun",
            CelestialBodyId::Mercury => "mercury",
            CelestialBodyId::Venus => "venus",
            CelestialBodyId::Earth => "earth",
            CelestialBodyId::Moon => "moon",
            CelestialBodyId::Mars => "mars",
        };
        let mut seed = hash_bytes(0xcbf29ce484222325, &world_seed.to_le_bytes());
        seed = hash_bytes(seed, &self.seed.to_le_bytes());
        seed = hash_bytes(seed, body_key.as_bytes());
        // Domain delimiter distinguishes base configuration from any biome key.
        seed = hash_bytes(seed, &[u8::from(selected.is_some())]);
        if let Some(entry) = selected {
            seed = hash_bytes(seed, entry.biome.as_bytes());
        }
        let mut inputs: Vec<_> = resources
            .iter()
            .map(|entry| ResourceGenerationInput {
                distribution: *entry,
                seed: hash_bytes(seed, entry.resource.key().as_bytes()),
            })
            .collect();
        inputs.sort_by_key(|input| input.distribution.resource.key());
        inputs
    }
}

fn hash_bytes(mut seed: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        seed = (seed ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
    }
    seed
}

fn validate_resources(resources: &[ResourceDistribution]) -> Result<(), DistributionError> {
    for (index, entry) in resources.iter().enumerate() {
        if resources[..index]
            .iter()
            .any(|other| other.resource == entry.resource)
        {
            return Err(DistributionError::DuplicateResource);
        }
    }
    Ok(())
}

/// Initial gameplay approximations for every canonical solid body. Sun has no
/// collectible surface. Kept separate from geometry/rendering and body constructors.
pub fn default_resource_distribution(body: CelestialBodyId) -> BodyResourceDistribution {
    use ResourceId::{IronOre, SilicateRock, WaterIce};
    type ProfileEntry = (ResourceId, f64, f64, [f64; 2]);
    let (seed, entries): (u64, &[ProfileEntry]) = match body {
        CelestialBodyId::Sun => (0, &[]),
        CelestialBodyId::Mercury => (
            1,
            &[
                (IronOre, 3.0, 40.0, [20.0, 80.0]),
                (SilicateRock, 2.0, 30.0, [10.0, 60.0]),
            ],
        ),
        CelestialBodyId::Venus => (
            2,
            &[
                (IronOre, 1.0, 60.0, [15.0, 70.0]),
                (SilicateRock, 4.0, 25.0, [10.0, 60.0]),
            ],
        ),
        CelestialBodyId::Earth => (
            3,
            &[
                (IronOre, 2.0, 40.0, [20.0, 80.0]),
                (SilicateRock, 4.0, 25.0, [10.0, 60.0]),
                (WaterIce, 1.0, 70.0, [5.0, 30.0]),
            ],
        ),
        CelestialBodyId::Moon => (
            4,
            &[
                (IronOre, 1.0, 50.0, [15.0, 70.0]),
                (SilicateRock, 4.0, 25.0, [10.0, 60.0]),
                (WaterIce, 1.0, 80.0, [5.0, 30.0]),
            ],
        ),
        CelestialBodyId::Mars => (
            5,
            &[
                (IronOre, 3.0, 35.0, [20.0, 80.0]),
                (SilicateRock, 3.0, 30.0, [10.0, 60.0]),
                (WaterIce, 2.0, 60.0, [5.0, 40.0]),
            ],
        ),
    };
    let resources = entries
        .iter()
        .map(|&(resource, weight, spacing, mass)| {
            ResourceDistribution::new(resource, weight, spacing, mass)
                .expect("valid default distribution")
        })
        .collect();
    BodyResourceDistribution::new(body, seed, resources, vec![]).expect("unique default resources")
}
