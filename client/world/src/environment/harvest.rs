//! Explicit biological yield authoring and source-bound, non-regrowing sparse deltas.
use super::{EnvironmentError as Error, organism_placement::Population};
use crate::{
    CelestialBodyId,
    resources::{DepositId, FragmentId, NaturalShape, ResourceId},
};
use salimon_universe::synthesis::sha256;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Key {
    pub organism: String,
    pub part: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Method {
    Detach,
    Embedded,
}
#[derive(Clone, Debug)]
pub struct Designation {
    pub key: Key,
    pub method: Method,
    pub material: ResourceId,
}
#[derive(Clone, Debug)]
pub struct Source {
    pub key: Key,
    pub method: Method,
    pub material: ResourceId,
    pub initial_mass_kg: f64,
    pub center: [f64; 3],
    pub shape: NaturalShape,
    pub deposit: DepositId,
}
#[derive(Clone, Debug)]
pub struct Catalog {
    identity: String,
    sources: Vec<Source>,
}
impl Catalog {
    /// Only explicit designations yield material. Presence of life is never a yield grant.
    pub fn publish(
        population: &Population,
        body: CelestialBodyId,
        designations: &[Designation],
    ) -> Result<Self, Error> {
        if designations.len() > 4096 {
            return Err(Error::InvalidDefinition);
        }
        let mut entries = designations.to_vec();
        entries.sort_by(|a, b| a.key.cmp(&b.key));
        if entries.windows(2).any(|v| v[0].key == v[1].key) {
            return Err(Error::InvalidDefinition);
        }
        let mut recipe = format!("biological-yields-1:{}:{body:?}", population.identity());
        let mut sources = Vec::new();
        for entry in entries {
            let object = population
                .objects()
                .iter()
                .find(|o| o.id == entry.key.organism)
                .ok_or(Error::InvalidDefinition)?;
            let physical = object
                .parts
                .iter()
                .find(|p| p.id == entry.key.part)
                .ok_or(Error::InvalidDefinition)?;
            let part = object
                .specimen
                .parts
                .iter()
                .find(|p| p.id == entry.key.part)
                .ok_or(Error::InvalidDefinition)?;
            if entry.key.part == 0
                || (entry.method == Method::Detach
                    && object
                        .specimen
                        .parts
                        .iter()
                        .any(|p| p.id != part.id && p.start == part.end))
            {
                return Err(Error::InvalidDefinition);
            }
            let center =
                std::array::from_fn(|i| (physical.start_meters[i] + physical.end_meters[i]) * 0.5);
            let source_key: [u8; 32] = (0..32)
                .map(|i| {
                    u8::from_str_radix(&object.id[i * 2..i * 2 + 2], 16)
                        .map_err(|_| Error::InvalidDefinition)
                })
                .collect::<Result<Vec<_>, _>>()?
                .try_into()
                .map_err(|_| Error::InvalidDefinition)?;
            let shape = NaturalShape {
                source_key,
                part_id: part.id,
                start: std::array::from_fn(|i| physical.start_meters[i] - center[i]),
                end: std::array::from_fn(|i| physical.end_meters[i] - center[i]),
                radii: [part.radius_start, part.radius_end],
                color: object.specimen.color,
            };
            shape.validate().map_err(|_| Error::InvalidDefinition)?;
            let len = shape.length();
            let [a, b] = shape.radii;
            let volume = 3f64.sqrt() * 0.5 * len * (a * a + a * b + b * b); // six-sided tapered prism
            let mass = volume * entry.material.definition().density_kg_per_m3;
            if !mass.is_finite() || mass <= 0. {
                return Err(Error::InvalidDefinition);
            }
            let hash =
                sha256(format!("yield:{}:{}", entry.key.organism, entry.key.part).as_bytes());
            let local = u64::from_str_radix(&hash[..16], 16)
                .map_err(|_| Error::InvalidDefinition)?
                | (1 << 63);
            if sources.iter().any(|s: &Source| s.deposit.local == local) {
                return Err(Error::InvalidDefinition);
            }
            recipe.push_str(&format!(
                ":{}:{}:{:?}:{}",
                entry.key.organism,
                entry.key.part,
                entry.method,
                entry.material.key()
            ));
            sources.push(Source {
                key: entry.key,
                method: entry.method,
                material: entry.material,
                initial_mass_kg: mass,
                center,
                shape,
                deposit: DepositId { body, local },
            });
        }
        Ok(Self {
            identity: sha256(recipe.as_bytes()),
            sources,
        })
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn sources(&self) -> &[Source] {
        &self.sources
    }
    pub fn get(&self, key: &Key) -> Option<&Source> {
        self.sources.iter().find(|s| s.key == *key)
    }
}
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Delta {
    pub key: Key,
    pub revision: u64,
    pub remaining_kg: f64,
    pub detached: Option<FragmentId>,
}
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Journal {
    identity: String,
    deltas: Vec<Delta>,
}
impl Journal {
    pub fn new(catalog: &Catalog) -> Self {
        Self {
            identity: catalog.identity.clone(),
            deltas: vec![],
        }
    }
    pub fn deltas(&self) -> &[Delta] {
        &self.deltas
    }
    pub fn delta(&self, key: &Key) -> Option<&Delta> {
        self.deltas.iter().find(|d| d.key == *key)
    }
    pub fn revision(&self, key: &Key) -> u64 {
        self.delta(key).map_or(0, |d| d.revision)
    }
    pub fn remaining(&self, source: &Source) -> f64 {
        self.delta(&source.key)
            .map_or(source.initial_mass_kg, |d| d.remaining_kg)
    }
    pub fn validate(&self, catalog: &Catalog) -> Result<(), Error> {
        if self.identity != catalog.identity || self.deltas.len() > catalog.sources.len() {
            return Err(Error::IncompatibleDefinition);
        }
        let mut keys = std::collections::BTreeSet::new();
        let mut ids = std::collections::HashSet::new();
        for d in &self.deltas {
            let s = catalog.get(&d.key).ok_or(Error::InvalidDefinition)?;
            if !keys.insert(&d.key)
                || d.revision == 0
                || !d.remaining_kg.is_finite()
                || d.remaining_kg < 0.
                || d.remaining_kg >= s.initial_mass_kg
                || (s.method == Method::Detach && (d.remaining_kg != 0. || d.detached.is_none()))
                || (s.method == Method::Embedded && d.detached.is_some())
                || d.detached.is_some_and(|id| id.0 == 0 || !ids.insert(id))
            {
                return Err(Error::InvalidDefinition);
            }
        }
        Ok(())
    }
    pub(crate) fn write(&mut self, delta: Delta) {
        if let Some(d) = self.deltas.iter_mut().find(|d| d.key == delta.key) {
            *d = delta;
        } else {
            self.deltas.push(delta);
            self.deltas.sort_by(|a, b| a.key.cmp(&b.key));
        }
    }
}
pub const MAX_CARRY_RADIUS_METERS: f64 = 0.75;
pub const MAX_CARRY_MASS_KG: f64 = 20.;
impl Source {
    pub fn carriable(&self) -> bool {
        self.shape.radius() <= MAX_CARRY_RADIUS_METERS && self.initial_mass_kg <= MAX_CARRY_MASS_KG
    }
}
#[cfg(test)]
mod tests;
