//! Input DTOs. Catalog publication validates them and exposes shared references only.
use crate::CatalogId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dataset {
    pub id: CatalogId,
    pub version: String,
    /// Published source/manifest reference; ingestion/licensing policy belongs to U06.
    pub reference: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Provenance {
    Observed {
        dataset: CatalogId,
        record_key: String,
    },
    Inferred {
        method: String,
        datasets: Vec<CatalogId>,
    },
    Fiction {
        builder: String,
        revision: String,
        rationale: String,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Uncertainty {
    Unknown,
    /// Published asymmetric bounds in the quantity's SI unit. Optional confidence
    /// is a fraction (0,1]; absence makes no statistical confidence claim.
    Bounds {
        minus: f64,
        plus: f64,
        confidence: Option<f64>,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeScale {
    Utc,
    Tt,
    Tdb,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Validity {
    pub start_julian_day: f64,
    pub end_julian_day: f64,
    pub scale: TimeScale,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Quantity {
    pub value: f64,
    pub uncertainty: Uncertainty,
    pub provenance: Provenance,
    /// Missing validity means unspecified, not valid forever.
    pub validity: Option<Validity>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Header {
    pub id: CatalogId,
    pub display_name: Option<String>,
    pub provenance: Provenance,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct System {
    pub header: Header,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Star {
    pub header: Header,
    pub system: CatalogId,
    pub mass_kg: Option<Quantity>,
    pub radius_meters: Option<Quantity>,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyKind {
    Planet,
    Moon,
    MinorBody,
}
/// Published canonical definition identity. This is a reference contract, not a
/// terrain algorithm/codec or a substitute for the full definition in U10/U11.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    pub sha256: String,
    pub domains: Vec<SeedDomain>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeedDomain {
    pub name: String,
    pub seed: u64,
    pub algorithm_version: u32,
    pub definition_sha256: String,
    pub dependencies: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Body {
    pub header: Header,
    pub system: CatalogId,
    pub parent: CatalogId,
    pub body_kind: BodyKind,
    pub mass_kg: Option<Quantity>,
    pub radius_meters: Option<Quantity>,
    pub definition: Option<Definition>,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PopulationKind {
    Asteroids,
    Comets,
    OtherMinorBodies,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Population {
    pub header: Header,
    pub system: CatalogId,
    pub parent: CatalogId,
    pub population_kind: PopulationKind,
    /// Statistical count estimate, not identities allocated to individual bodies.
    pub estimated_count: u64,
    pub descriptor: String,
    pub definition: Definition,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "record",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Record {
    System(System),
    Star(Star),
    Body(Body),
    Population(Population),
}
impl Record {
    pub fn header(&self) -> &Header {
        match self {
            Self::System(r) => &r.header,
            Self::Star(r) => &r.header,
            Self::Body(r) => &r.header,
            Self::Population(r) => &r.header,
        }
    }
    pub fn id(&self) -> &CatalogId {
        &self.header().id
    }
    pub fn system(&self) -> &CatalogId {
        match self {
            Self::System(r) => &r.header.id,
            Self::Star(r) => &r.system,
            Self::Body(r) => &r.system,
            Self::Population(r) => &r.system,
        }
    }
    pub fn parent(&self) -> Option<&CatalogId> {
        match self {
            Self::Body(r) => Some(&r.parent),
            Self::Population(r) => Some(&r.parent),
            _ => None,
        }
    }
}
