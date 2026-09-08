//! U05 independent new-run wire contract. Never reinterprets compressed-world saves.
use crate::{MAX_SAVE_BYTES, SaveError, invalid};
use salimon_universe::{
    Catalog, CatalogId, Epoch, Record, SCHEMA_VERSION as CATALOG_SCHEMA,
    coordinates::{FrameId, GalacticPosition},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const FORMAT: &str = "salimon.universe.save";
pub const VERSION: u32 = 1;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogBinding {
    pub revision: String,
    pub schema_version: u32,
    /// Exact verified published pack digest supplied by the future pack loader.
    pub pack_sha256: String,
}
impl CatalogBinding {
    pub fn validate(&self, catalog: &Catalog) -> Result<(), SaveError> {
        if self.revision != catalog.revision() || self.schema_version != CATALOG_SCHEMA {
            return Err(SaveError::IncompatibleCatalog(
                "revision or schema mismatch".into(),
            ));
        }
        digest(&self.pack_sha256)
    }
}
fn digest(s: &str) -> Result<(), SaveError> {
    if s.len() == 64
        && s.bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        Ok(())
    } else {
        Err(invalid("expected lowercase SHA256 digest"))
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Position {
    Galaxy { position: GalacticPosition },
    Relative { frame: FrameId, meters: [f64; 3] },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShipState {
    pub position: Position,
    /// Expressed in the saved position's frame; rotation is xyzw child->frame.
    pub velocity_mps: [f64; 3],
    pub orientation_xyzw: [f64; 4],
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefinitionBinding {
    pub body: CatalogId,
    pub version: u32,
    pub sha256: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Discovery {
    pub object: CatalogId,
    pub first_seen: Epoch,
    pub last_seen: Epoch,
    /// History is not a live contact. U30 derives live eligibility from pulses.
    pub observation_count: u64,
}
/// Explicitly versioned domain-owned mutable source. The restore caller must
/// validate every section through its owner, including all nested identities.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeltaSection {
    pub owner: CatalogId,
    pub schema_version: u32,
    pub payload: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UniverseSave {
    pub format: String,
    pub schema_version: u32,
    pub catalog: CatalogBinding,
    pub epoch: Epoch,
    pub ship: ShipState,
    pub definitions: Vec<DefinitionBinding>,
    pub discovery_history: Vec<Discovery>,
    pub player_deltas: Vec<DeltaSection>,
}
fn finite(v: &[f64]) -> Result<(), SaveError> {
    if v.iter().all(|x| x.is_finite()) {
        Ok(())
    } else {
        Err(invalid("nonfinite universe pose"))
    }
}
impl UniverseSave {
    pub fn validate(
        &self,
        catalog: &Catalog,
        expected: &CatalogBinding,
        validate_delta: impl Fn(&DeltaSection) -> Result<(), SaveError>,
    ) -> Result<(), SaveError> {
        if self.schema_version != VERSION {
            return Err(SaveError::UnsupportedVersion(self.schema_version));
        }
        if self.format != FORMAT {
            return Err(invalid("unrecognized universe save format"));
        }
        expected.validate(catalog)?;
        self.catalog.validate(catalog)?;
        if &self.catalog != expected {
            return Err(SaveError::IncompatibleCatalog(
                "pack identity mismatch".into(),
            ));
        }
        finite(&self.ship.velocity_mps)?;
        finite(&self.ship.orientation_xyzw)?;
        if (self
            .ship
            .orientation_xyzw
            .iter()
            .map(|x| x * x)
            .sum::<f64>()
            - 1.0)
            .abs()
            > 1e-10
        {
            return Err(invalid("ship orientation must be unit quaternion"));
        }
        if let Position::Relative { frame, meters } = &self.ship.position {
            finite(meters)?;
            let valid = match frame {
                FrameId::System(id) => matches!(catalog.get(id), Some(Record::System(_))),
                FrameId::Body(id) => matches!(catalog.get(id), Some(Record::Body(_))),
                FrameId::Local(_) => false, // no persisted local-frame definition authority yet
            };
            if !valid {
                return Err(invalid("unknown or unsaved ship reference frame"));
            }
        }
        let mut defs = BTreeSet::new();
        for d in &self.definitions {
            digest(&d.sha256)?;
            if !defs.insert(&d.body) {
                return Err(invalid("duplicate world definition"));
            }
            let definition = match catalog.get(&d.body) {
                Some(Record::Body(b)) => b.definition.as_ref(),
                _ => None,
            };
            if !definition.is_some_and(|v| v.version == d.version && v.sha256 == d.sha256) {
                return Err(SaveError::IncompatibleCatalog(
                    "world definition mismatch".into(),
                ));
            }
        }
        if let Position::Relative {
            frame: FrameId::Body(id),
            ..
        } = &self.ship.position
            && matches!(catalog.get(id), Some(Record::Body(b)) if b.definition.is_some())
            && !defs.contains(id)
        {
            return Err(invalid(
                "saved body frame requires canonical definition binding",
            ));
        }
        let mut ids = BTreeSet::new();
        for h in &self.discovery_history {
            if !ids.insert(&h.object)
                || catalog.get(&h.object).is_none()
                || h.observation_count == 0
            {
                return Err(invalid("discovery identity/count"));
            }
            if h.first_seen.scale() != self.epoch.scale()
                || h.last_seen.scale() != self.epoch.scale()
            {
                return Err(invalid("discovery time scale"));
            }
            // Calendar wire ordering is exact; elapsed UTC arithmetic needs a
            // leap adapter, but chronological ordering does not.
            if h.first_seen
                .compare(h.last_seen)
                .map_err(|e| invalid(e.to_string()))?
                .is_gt()
                || h.last_seen
                    .compare(self.epoch)
                    .map_err(|e| invalid(e.to_string()))?
                    .is_gt()
            {
                return Err(invalid("discovery chronology"));
            }
        }
        let mut owners = BTreeSet::new();
        for d in &self.player_deltas {
            if !owners.insert(&d.owner)
                || d.schema_version == 0
                || d.payload.len() > MAX_SAVE_BYTES / 2
            {
                return Err(invalid("duplicate/invalid domain delta"));
            }
            validate_delta(d)?;
        }
        Ok(())
    }
    pub fn encode(
        &self,
        catalog: &Catalog,
        expected: &CatalogBinding,
        validate_delta: impl Fn(&DeltaSection) -> Result<(), SaveError>,
    ) -> Result<Vec<u8>, SaveError> {
        self.validate(catalog, expected, validate_delta)?;
        let b = serde_json::to_vec(self).map_err(|e| SaveError::Encoding(e.to_string()))?;
        if b.len() > MAX_SAVE_BYTES {
            return Err(invalid("universe save exceeds size limit"));
        }
        Ok(b)
    }
    pub fn decode(
        b: &[u8],
        catalog: &Catalog,
        expected: &CatalogBinding,
        validate_delta: impl Fn(&DeltaSection) -> Result<(), SaveError>,
    ) -> Result<Self, SaveError> {
        if b.len() > MAX_SAVE_BYTES {
            return Err(invalid("universe save exceeds size limit"));
        }
        #[derive(Deserialize)]
        struct Version {
            schema_version: u32,
        }
        let v: Version =
            serde_json::from_slice(b).map_err(|e| SaveError::Encoding(e.to_string()))?;
        if v.schema_version != VERSION {
            return Err(SaveError::UnsupportedVersion(v.schema_version));
        }
        let s: Self = serde_json::from_slice(b).map_err(|e| SaveError::Encoding(e.to_string()))?;
        s.validate(catalog, expected, validate_delta)?;
        Ok(s)
    }
}
#[cfg(test)]
mod tests;
