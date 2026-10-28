//! Player discovery authority. Catalog access alone never creates a contact.
use crate::{Catalog, CatalogId, Error, Record, coordinates::GalacticPosition, invalid};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const CONTACT_TTL_MILLIS: u64 = 3 * 60 * 60 * 1000;
pub const MAX_CONTACTS: usize = 65_536;

/// Exact catalog evaluation at the successful observation time. Coordinates and
/// velocity use the same galactic inertial axes; positions retain split cells.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub id: CatalogId,
    pub position: GalacticPosition,
    pub velocity_mps: [f64; 3],
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Encounter {
    pub(crate) observation: Observation,
    pub(crate) first_seen_utc_millis: u64,
    pub(crate) last_seen_utc_millis: u64,
    pub(crate) successful_refreshes: u64,
}
impl Encounter {
    pub fn observation(&self) -> &Observation {
        &self.observation
    }
    pub fn first_seen_utc_millis(&self) -> u64 {
        self.first_seen_utc_millis
    }
    pub fn last_seen_utc_millis(&self) -> u64 {
        self.last_seen_utc_millis
    }
    pub fn successful_refreshes(&self) -> u64 {
        self.successful_refreshes
    }
    pub fn fresh(&self, now_utc_millis: u64) -> bool {
        now_utc_millis
            .checked_sub(self.last_seen_utc_millis)
            .is_some_and(|age| age < CONTACT_TTL_MILLIS)
    }
}
/// UTC labels use the host's nondecreasing POSIX millisecond gameplay clock,
/// matching checkpoint UTC. They are not TT/TDB ephemeris conversion or leap
/// second astronomy. The evaluator must supply its own explicit scientific epoch.
#[derive(Clone, Debug, PartialEq)]
pub struct Contacts {
    pub(crate) catalog_revision: String,
    pub(crate) watermark_utc_millis: u64,
    pub(crate) encounters: BTreeMap<CatalogId, Encounter>,
}
impl Contacts {
    pub fn new(catalog_revision: &str) -> Result<Self, Error> {
        if catalog_revision.is_empty() || catalog_revision.len() > 256 {
            return Err(invalid("scanner catalog revision"));
        }
        Ok(Self {
            catalog_revision: catalog_revision.into(),
            watermark_utc_millis: 0,
            encounters: BTreeMap::new(),
        })
    }
    pub fn catalog_revision(&self) -> &str {
        &self.catalog_revision
    }
    pub fn watermark_utc_millis(&self) -> u64 {
        self.watermark_utc_millis
    }
    pub fn history(&self) -> impl Iterator<Item = &Encounter> {
        self.encounters.values()
    }
    pub fn active(&self, now_utc_millis: u64) -> impl Iterator<Item = &Encounter> {
        self.history()
            .filter(move |e| e.fresh(now_utc_millis.max(self.watermark_utc_millis)))
    }
    /// Time passing, misses and power loss do not erase historic discoveries.
    pub fn advance(&mut self, now_utc_millis: u64) -> Result<(), Error> {
        if now_utc_millis < self.watermark_utc_millis {
            return Err(invalid("scanner clock regressed"));
        }
        self.watermark_utc_millis = now_utc_millis;
        Ok(())
    }
    /// The powered scanner is the only contact producer. Batch validation is
    /// atomic: an invalid identity/state never partially refreshes discoveries.
    pub fn refresh(
        &mut self,
        catalog: &Catalog,
        now_utc_millis: u64,
        powered: bool,
        observations: &[Observation],
    ) -> Result<(), Error> {
        if catalog.revision() != self.catalog_revision || now_utc_millis < self.watermark_utc_millis
        {
            return Err(invalid("scanner catalog/clock mismatch"));
        }
        if observations.len() > MAX_CONTACTS {
            return Err(invalid("scanner batch exceeds limit"));
        }
        let mut unique = std::collections::BTreeSet::new();
        for o in observations {
            validate_observation(catalog, o)?;
            if !unique.insert(&o.id) {
                return Err(invalid("duplicate scanner observation"));
            }
            if self
                .encounters
                .get(&o.id)
                .is_some_and(|e| e.successful_refreshes == u64::MAX)
            {
                return Err(invalid("scanner refresh count overflow"));
            }
        }
        if powered
            && self.encounters.len()
                + observations
                    .iter()
                    .filter(|o| !self.encounters.contains_key(&o.id))
                    .count()
                > MAX_CONTACTS
        {
            return Err(invalid("scanner history full"));
        }
        self.watermark_utc_millis = now_utc_millis;
        if !powered {
            return Ok(());
        }
        for o in observations {
            self.encounters
                .entry(o.id.clone())
                .and_modify(|e| {
                    e.observation = o.clone();
                    e.last_seen_utc_millis = now_utc_millis;
                    e.successful_refreshes += 1;
                })
                .or_insert_with(|| Encounter {
                    observation: o.clone(),
                    first_seen_utc_millis: now_utc_millis,
                    last_seen_utc_millis: now_utc_millis,
                    successful_refreshes: 1,
                });
        }
        Ok(())
    }
}
pub(crate) fn validate_observation(catalog: &Catalog, o: &Observation) -> Result<(), Error> {
    if !matches!(catalog.get(&o.id), Some(Record::Star(_) | Record::Body(_)))
        || !o.velocity_mps.into_iter().all(f64::is_finite)
    {
        return Err(invalid("scanner requires identified celestial state"));
    }
    Ok(())
}
#[cfg(test)]
mod tests;
