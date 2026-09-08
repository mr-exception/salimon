//! Immutable potential versus existence; no organisms, regrowth or proven alien-life claims.
use super::{
    EnvironmentError as Error,
    climate::{Chemistry, ClimateDefinition},
    random,
};
use crate::{CelestialBody, CelestialBodyId};
use salimon_universe::{CatalogId, Epoch, TimeScale, synthesis::sha256};
use std::f64::consts::TAU;
pub const ECOLOGY_VERSION: u16 = 1;
const MAX_PUBLICATIONS: usize = 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BodyNature {
    ObservedNoLifeClaim = 0,
    Fictional = 1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BiosphereDecision {
    Unknown = 0,
    Sterile = 1,
    MicrobialOnly = 2,
    Developed = 3,
}
/// Immutable publication bound to one catalog body, geology and climate revision.
#[derive(Clone, Debug, PartialEq)]
pub struct BiosphereDefinition {
    catalog_body: CatalogId,
    nature: BodyNature,
    decision: BiosphereDecision,
    seed: u64,
    climate: ClimateDefinition,
    identity: String,
}
/// Continuous community weights, not an Earth-biome promise or physical vegetation instances.
#[derive(Clone, Debug, PartialEq)]
pub struct EcologySample {
    pub definition_id: String,
    pub catalog_body: CatalogId,
    pub nature: BodyNature,
    pub decision: BiosphereDecision,
    pub potential: f64,
    pub macro_cover_fraction: f64,
    /// Stable selected realm; continuous weights below preserve ecological transitions.
    pub realm_id: String,
    pub family_grammar_id: Option<String>,
    pub regional_weights: [f64; 8],
    /// Dry exposed, sheltered substrate, cold substrate; presentation grammar remains #619.
    pub community_weights: [f64; 3],
}
#[derive(Clone, Debug, PartialEq)]
pub struct EcologySite {
    pub id: String,
    pub realm_id: String,
    pub direction: [f64; 3],
    pub body_local_position_meters: [f64; 3],
    pub potential: f64,
}
impl BiosphereDefinition {
    /// The decision is publication state, never a visit-time random draw.
    pub fn new(
        catalog_body: CatalogId,
        nature: BodyNature,
        decision: BiosphereDecision,
        climate: ClimateDefinition,
        version: u16,
        seed: u64,
    ) -> Result<Self, Error> {
        if version != ECOLOGY_VERSION {
            return Err(Error::UnsupportedVersion);
        }
        if (nature == BodyNature::ObservedNoLifeClaim && decision != BiosphereDecision::Unknown)
            || (climate.geology().terrain().body() == CelestialBodyId::Vesper
                && decision != BiosphereDecision::Sterile)
            || (matches!(
                decision,
                BiosphereDecision::MicrobialOnly | BiosphereDecision::Developed
            ) && suitability(&climate) < 0.05)
        {
            return Err(Error::InvalidDefinition);
        }
        let mut d = Self {
            catalog_body,
            nature,
            decision,
            seed,
            climate,
            identity: String::new(),
        };
        d.identity = sha256(&d.to_bytes());
        Ok(d)
    }
    /// Rare fictional biospheres. Observations remain unknown, and Vesper remains sterile.
    /// Suitability enables a decision; it never guarantees existence.
    pub fn generate(
        catalog_body: CatalogId,
        nature: BodyNature,
        climate: ClimateDefinition,
        seed: u64,
    ) -> Result<Self, Error> {
        let chance = random(seed, 0xec01);
        let decision = if climate.geology().terrain().body() == CelestialBodyId::Vesper {
            BiosphereDecision::Sterile
        } else if nature == BodyNature::ObservedNoLifeClaim {
            BiosphereDecision::Unknown
        } else if suitability(&climate) < 0.05 || chance >= 0.02 {
            BiosphereDecision::Sterile
        } else if chance < 0.004 {
            BiosphereDecision::Developed
        } else {
            BiosphereDecision::MicrobialOnly
        };
        Self::new(
            catalog_body,
            nature,
            decision,
            climate,
            ECOLOGY_VERSION,
            seed,
        )
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn catalog_body(&self) -> &CatalogId {
        &self.catalog_body
    }
    pub fn nature(&self) -> BodyNature {
        self.nature
    }
    pub fn decision(&self) -> BiosphereDecision {
        self.decision
    }
    pub fn climate(&self) -> &ClimateDefinition {
        &self.climate
    }
    pub fn compatible_with(
        &self,
        identity: &str,
        catalog_body: &CatalogId,
        climate: &ClimateDefinition,
    ) -> bool {
        self.identity == identity && self.catalog_body == *catalog_body && self.climate == *climate
    }
    pub fn family_grammar_id(&self) -> Option<String> {
        if matches!(
            self.decision,
            BiosphereDecision::MicrobialOnly | BiosphereDecision::Developed
        ) {
            let mut b = b"SLMECO-FAMILY-01".to_vec();
            b.extend(self.seed.to_le_bytes());
            b.extend(self.catalog_body.as_str().as_bytes());
            Some(sha256(&b))
        } else {
            None
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = b"SLMECO01".to_vec();
        b.extend(ECOLOGY_VERSION.to_le_bytes());
        b.extend(self.seed.to_le_bytes());
        b.extend([
            self.nature as u8,
            self.decision as u8,
            self.catalog_body.as_str().len() as u8,
        ]);
        b.extend(self.catalog_body.as_str().as_bytes());
        b.extend(self.climate.to_bytes());
        b
    }
    pub fn from_bytes(body: CelestialBody, b: &[u8]) -> Result<Self, Error> {
        if b.len() < 21
            || b.len() > crate::terrain::MAX_DEFINITION_BYTES + 400
            || &b[..8] != b"SLMECO01"
        {
            return Err(Error::InvalidEncoding);
        }
        let version = u16::from_le_bytes([b[8], b[9]]);
        let seed = u64::from_le_bytes(b[10..18].try_into().map_err(|_| Error::InvalidEncoding)?);
        let nature = match b[18] {
            0 => BodyNature::ObservedNoLifeClaim,
            1 => BodyNature::Fictional,
            _ => return Err(Error::InvalidEncoding),
        };
        let decision = match b[19] {
            0 => BiosphereDecision::Unknown,
            1 => BiosphereDecision::Sterile,
            2 => BiosphereDecision::MicrobialOnly,
            3 => BiosphereDecision::Developed,
            _ => return Err(Error::InvalidEncoding),
        };
        let end = 21 + usize::from(b[20]);
        let id = std::str::from_utf8(b.get(21..end).ok_or(Error::InvalidEncoding)?)
            .map_err(|_| Error::InvalidEncoding)?;
        let id = CatalogId::try_from(id.to_owned()).map_err(|_| Error::InvalidEncoding)?;
        Self::new(
            id,
            nature,
            decision,
            ClimateDefinition::from_bytes(body, b.get(end..).ok_or(Error::InvalidEncoding)?)?,
            version,
            seed,
        )
    }
    fn center(&self, slot: u8) -> [f64; 3] {
        let z = 2.0 * random(self.seed, 0xec10 + u64::from(slot) * 2) - 1.0;
        let angle = TAU * random(self.seed, 0xec11 + u64::from(slot) * 2);
        let r = (1.0 - z * z).max(0.0).sqrt();
        [r * angle.cos(), z, r * angle.sin()]
    }
    /// Immutable baseline at a fixed declared climate reference epoch, not today's weather.
    /// Seasons may later animate appearance, never rewrite biosphere/individual identities.
    pub fn sample(&self, direction: [f64; 3]) -> Result<EcologySample, Error> {
        let reference = Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], TimeScale::Tt)
            .expect("valid fixed TT epoch");
        let weather = self.climate.sample(direction, reference)?;
        let geology = self.climate.geology().sample(direction)?;
        let potential = (suitability(&self.climate)
            * thermal(weather.climate_temperature_kelvin)
            * (0.3 + 0.7 * geology.geology.weathering)
            * (0.5 + 0.5 * (1.0 - direction[1].clamp(-1.0, 1.0).powi(2)).sqrt())
            / (1.0 + geology.surface.slope))
            .clamp(0.0, 1.0);
        let mut weights = [0.0; 8];
        for (slot, w) in weights.iter_mut().enumerate() {
            let center = self.center(slot as u8);
            let distance: f64 = (0..3).map(|i| (direction[i] - center[i]).powi(2)).sum();
            *w = 1.0 / (0.08 + distance).powi(2);
        }
        let total: f64 = weights.iter().sum();
        for w in &mut weights {
            *w /= total;
        }
        let realm = (0..8)
            .max_by(|a, b| weights[*a].total_cmp(&weights[*b]).then_with(|| b.cmp(a)))
            .expect("eight fixed realms");
        let regional: f64 = weights
            .iter()
            .enumerate()
            .map(|(i, w)| w * (0.2 + 0.8 * random(self.seed, 0xec40 + i as u64)))
            .sum();
        let cold = ((290.0 - weather.climate_temperature_kelvin) / 110.0).clamp(0.0, 1.0);
        let shelter = (1.0 - cold) * geology.geology.weathering;
        let communities = [
            (1.0 - cold) * (1.0 - geology.geology.weathering),
            shelter,
            cold,
        ];
        Ok(EcologySample {
            definition_id: self.identity.clone(),
            catalog_body: self.catalog_body.clone(),
            nature: self.nature,
            decision: self.decision,
            potential,
            macro_cover_fraction: if self.decision == BiosphereDecision::Developed {
                potential * regional
            } else {
                0.0
            },
            realm_id: format!("{}:realm:{realm}", self.identity),
            family_grammar_id: self.family_grammar_id(),
            regional_weights: weights,
            community_weights: communities,
        })
    }
    /// Sparse fixed ecological patch anchors for downstream grammar/placement (#619/#620).
    /// They are not guaranteed landmarks, colliders, harvestable organisms or map markers.
    pub fn sites(&self) -> Result<Vec<EcologySite>, Error> {
        if self.decision != BiosphereDecision::Developed {
            return Ok(Vec::new());
        }
        let mut sites = Vec::new();
        for slot in 0..32_u8 {
            let z = 2.0 * random(self.seed, 0xec80 + u64::from(slot) * 2) - 1.0;
            let angle = TAU * random(self.seed, 0xec81 + u64::from(slot) * 2);
            let r = (1.0 - z * z).max(0.0).sqrt();
            let direction = [r * angle.cos(), z, r * angle.sin()];
            let sample = self.sample(direction)?;
            if sample.macro_cover_fraction > 0.02
                && random(self.seed, 0xed00 + u64::from(slot)) < sample.macro_cover_fraction
            {
                let surface = self.climate.geology().terrain().sample(direction)?;
                sites.push(EcologySite {
                    id: format!("{}:patch:{slot}", self.identity),
                    realm_id: sample.realm_id,
                    direction,
                    body_local_position_meters: surface.body_local_position_meters,
                    potential: sample.potential,
                });
            }
        }
        Ok(sites)
    }
}
fn thermal(k: f64) -> f64 {
    ((k - 180.0) / 70.0).clamp(0.0, 1.0) * ((500.0 - k) / 150.0).clamp(0.0, 1.0)
}
fn suitability(climate: &ClimateDefinition) -> f64 {
    let p = climate.inputs();
    if p.chemistry == Chemistry::Vacuum {
        return 0.0;
    }
    let equilibrium = (p.stellar_flux_w_m2 * (1.0 - p.albedo) / (4.0 * 5.670374419e-8))
        .sqrt()
        .sqrt()
        + p.greenhouse_kelvin;
    let chemistry = match p.chemistry {
        Chemistry::Vacuum => 0.0,
        Chemistry::Nitrogen => 1.0,
        Chemistry::CarbonDioxide => 0.65,
        Chemistry::Hydrogen => 0.45,
    };
    (thermal(equilibrium)
        * chemistry
        * p.condensable_fraction
        * (p.pressure_pascal / 10000.0).min(1.0)
        * (1e7 / p.pressure_pascal).min(1.0)
        * (p.stellar_flux_w_m2 / 300.0).min(1.0))
    .clamp(0.0, 1.0)
}
/// Sorted bounded publication inventory. Save references must match exact body and sidecar IDs.
#[derive(Clone, Debug, PartialEq)]
pub struct BiosphereCatalog {
    definitions: Vec<BiosphereDefinition>,
}
impl BiosphereCatalog {
    pub fn new(mut definitions: Vec<BiosphereDefinition>) -> Result<Self, Error> {
        if definitions.len() > MAX_PUBLICATIONS {
            return Err(Error::InvalidDefinition);
        }
        definitions.sort_by(|a, b| a.catalog_body.cmp(&b.catalog_body));
        if definitions
            .windows(2)
            .any(|w| w[0].catalog_body == w[1].catalog_body)
        {
            return Err(Error::InvalidDefinition);
        }
        Ok(Self { definitions })
    }
    pub fn definitions(&self) -> &[BiosphereDefinition] {
        &self.definitions
    }
    pub fn get(&self, body: &CatalogId) -> Option<&BiosphereDefinition> {
        self.definitions
            .binary_search_by(|d| d.catalog_body.cmp(body))
            .ok()
            .map(|i| &self.definitions[i])
    }
    pub fn resolve(&self, body: &CatalogId, identity: &str) -> Result<&BiosphereDefinition, Error> {
        self.get(body)
            .filter(|d| d.identity == identity)
            .ok_or(Error::IncompatibleDefinition)
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = b"SLMECAT1".to_vec();
        b.extend((self.definitions.len() as u16).to_le_bytes());
        for d in &self.definitions {
            let bytes = d.to_bytes();
            b.extend((bytes.len() as u32).to_le_bytes());
            b.extend(bytes);
        }
        b
    }
    /// Caller resolves known physical body metadata, never guessed radii or fallback definitions.
    pub fn from_bytes(
        b: &[u8],
        mut body_for: impl FnMut(&CatalogId) -> Option<CelestialBody>,
    ) -> Result<Self, Error> {
        if b.len() < 10
            || b.len() > MAX_PUBLICATIONS * (crate::terrain::MAX_DEFINITION_BYTES + 404) + 10
            || &b[..8] != b"SLMECAT1"
        {
            return Err(Error::InvalidEncoding);
        }
        let count = usize::from(u16::from_le_bytes([b[8], b[9]]));
        if count > MAX_PUBLICATIONS {
            return Err(Error::InvalidEncoding);
        }
        let mut cursor = 10;
        let mut definitions = Vec::with_capacity(count);
        for _ in 0..count {
            let length = b.get(cursor..cursor + 4).ok_or(Error::InvalidEncoding)?;
            let length =
                u32::from_le_bytes(length.try_into().map_err(|_| Error::InvalidEncoding)?) as usize;
            cursor += 4;
            if length > crate::terrain::MAX_DEFINITION_BYTES + 400 {
                return Err(Error::InvalidEncoding);
            }
            let bytes = b
                .get(cursor..cursor + length)
                .ok_or(Error::InvalidEncoding)?;
            cursor += length;
            if bytes.len() < 21 {
                return Err(Error::InvalidEncoding);
            }
            let id_bytes = bytes
                .get(21..21 + usize::from(bytes[20]))
                .ok_or(Error::InvalidEncoding)?;
            let id = CatalogId::try_from(
                std::str::from_utf8(id_bytes)
                    .map_err(|_| Error::InvalidEncoding)?
                    .to_owned(),
            )
            .map_err(|_| Error::InvalidEncoding)?;
            let body = body_for(&id).ok_or(Error::IncompatibleDefinition)?;
            definitions.push(BiosphereDefinition::from_bytes(body, bytes)?);
        }
        if cursor != b.len() {
            return Err(Error::InvalidEncoding);
        }
        let catalog = Self::new(definitions)?;
        // A single canonical ordering prevents semantically equal inventories having different bytes.
        if catalog.to_bytes() != b {
            return Err(Error::InvalidEncoding);
        }
        Ok(catalog)
    }
}
#[cfg(test)]
mod tests;
