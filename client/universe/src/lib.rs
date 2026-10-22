//! U01 immutable schema foundation. Not connected to playable world/navigation.
pub mod coordinates;
mod epoch;
pub mod inference;
pub mod ingestion;
pub mod partitions;
pub mod reference;
pub mod sources;
pub use epoch::Epoch;
mod identity;
mod records;
mod validation;
pub use identity::CatalogId;
pub use records::*;
use serde::{Deserialize, Serialize};
use std::fmt;

pub const SCHEMA_VERSION: u32 = 1;
pub const FORMAT: &str = "salimon.universe.catalog";
pub const MAX_SCHEMA_JSON_BYTES: usize = 16 * 1024 * 1024;
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Invalid(String),
    Encoding(String),
    UnsupportedSchema(u32),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "universe catalog: {self:?}")
    }
}
impl std::error::Error for Error {}
pub(crate) fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

/// Validated immutable catalog revision. Standalone input records may be edited;
/// publication owns them and provides no mutable access to identity/provenance.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Catalog {
    format: String,
    schema_version: u32,
    revision: String,
    datasets: Vec<Dataset>,
    records: Vec<Record>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    format: String,
    schema_version: u32,
    revision: String,
    datasets: Vec<Dataset>,
    records: Vec<Record>,
}
impl Catalog {
    pub fn new(
        revision: String,
        mut datasets: Vec<Dataset>,
        mut records: Vec<Record>,
    ) -> Result<Self, Error> {
        datasets.sort_by(|a, b| a.id.cmp(&b.id));
        records.sort_by(|a, b| a.id().cmp(b.id()));
        let c = Self {
            format: FORMAT.into(),
            schema_version: SCHEMA_VERSION,
            revision,
            datasets,
            records,
        };
        c.validate()?;
        Ok(c)
    }
    pub fn revision(&self) -> &str {
        &self.revision
    }
    pub fn records(&self) -> &[Record] {
        &self.records
    }
    pub fn datasets(&self) -> &[Dataset] {
        &self.datasets
    }
    pub fn get(&self, id: &CatalogId) -> Option<&Record> {
        self.records
            .binary_search_by(|r| r.id().cmp(id))
            .ok()
            .map(|i| &self.records[i])
    }
    /// Bounded review/fixture interchange only. Binary partitions/streaming U04
    /// must not use this whole-document JSON API for a multi-GB production pack.
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        let b = serde_json::to_vec(self).map_err(|e| Error::Encoding(e.to_string()))?;
        if b.len() > MAX_SCHEMA_JSON_BYTES {
            return Err(invalid("schema JSON exceeds fixture size limit"));
        }
        Ok(b)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_SCHEMA_JSON_BYTES {
            return Err(invalid("schema JSON exceeds fixture size limit"));
        }
        #[derive(Deserialize)]
        struct Version {
            schema_version: u32,
        }
        let v: Version =
            serde_json::from_slice(bytes).map_err(|e| Error::Encoding(e.to_string()))?;
        if v.schema_version != SCHEMA_VERSION {
            return Err(Error::UnsupportedSchema(v.schema_version));
        }
        let input: Input =
            serde_json::from_slice(bytes).map_err(|e| Error::Encoding(e.to_string()))?;
        if input.format != FORMAT || input.schema_version != SCHEMA_VERSION {
            return Err(invalid("catalog format/schema"));
        }
        Self::new(input.revision, input.datasets, input.records)
    }
}
#[cfg(test)]
mod tests;
