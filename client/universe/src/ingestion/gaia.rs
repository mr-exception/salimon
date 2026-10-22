//! Gaia DR3 astrometry stays in declared source units; distance inference is U09.
use super::csv::{Csv, number};
use crate::{
    Catalog, CatalogId, Dataset, Error, Header, Provenance, Record, Star, System, invalid,
};
use serde::Serialize;
use std::{collections::BTreeMap, io::BufRead};

/// Explicit selection, recorded with the result. Missing RUWE is retained and
/// counted, never interpreted as passing a measured quality threshold.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct QualityPolicy {
    pub max_ruwe: f64,
    pub reject_duplicated_source: bool,
}
impl Default for QualityPolicy {
    fn default() -> Self {
        Self {
            max_ruwe: 1.4,
            reject_duplicated_source: true,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Measurement {
    pub value: f64,
    pub standard_error: Option<f64>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GaiaObservation {
    pub source: CatalogId,
    pub provenance: Provenance,
    /// Julian reference year in Gaia's source convention, not a converted Epoch.
    pub ref_epoch_julian_year: f64,
    pub ra_degrees: Measurement,
    pub dec_degrees: Measurement,
    /// Negative parallaxes remain observed values; no inverse-parallax distance.
    pub parallax_mas: Option<Measurement>,
    /// Gaia pmra already includes cos(dec).
    pub pmra_mas_per_year: Option<Measurement>,
    pub pmdec_mas_per_year: Option<Measurement>,
    pub radial_velocity_km_per_second: Option<Measurement>,
    /// All input columns, including quality fields and unknown future columns.
    pub source_fields: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Summary {
    pub input_rows: u64,
    pub selected_rows: u64,
    pub rejected_quality: u64,
    pub rejected_astrometry: u64,
    pub missing_ruwe: u64,
    pub missing_or_nonpositive_parallax: u64,
    pub incomplete_velocity: u64,
    pub duplicate_rows: u64,
    /// Set by canonical collection; streaming alone cannot know merged counts.
    pub canonical_stars: Option<u64>,
}

/// Reviewed exact-ID associations only. No name or angular proximity heuristics.
/// Ambiguous source assignments fail instead of silently choosing a candidate.
#[derive(Clone, Debug, Default)]
pub struct StellarCrossMatch(BTreeMap<CatalogId, CatalogId>);
impl StellarCrossMatch {
    pub fn new(pairs: impl IntoIterator<Item = (CatalogId, CatalogId)>) -> Result<Self, Error> {
        let mut map = BTreeMap::new();
        for (source, canonical) in pairs {
            if let Some(previous) = map.insert(source, canonical.clone())
                && previous != canonical
            {
                return Err(invalid("ambiguous stellar cross-match"));
            }
        }
        Ok(Self(map))
    }
    pub fn canonical(&self, source: &CatalogId) -> CatalogId {
        self.0.get(source).unwrap_or(source).clone()
    }
}

fn measurement(row: &BTreeMap<String, String>, key: &str) -> Result<Option<Measurement>, Error> {
    let value = number(row, key)?;
    let error = number(row, &format!("{key}_error"))?;
    if error.is_some_and(|v| v < 0.0) || (value.is_none() && error.is_some()) {
        return Err(invalid(format!("invalid/orphan standard error for {key}")));
    }
    Ok(value.map(|value| Measurement {
        value,
        standard_error: error,
    }))
}
fn source_id(text: &str) -> Result<CatalogId, Error> {
    let id: u64 = text
        .parse()
        .map_err(|_| invalid("Gaia source_id must be an exact positive decimal integer"))?;
    if id == 0 || id.to_string() != text {
        return Err(invalid("noncanonical Gaia source_id"));
    }
    CatalogId::from_source("gaia_dr3", text)
}

/// One bounded CSV record at a time. Sink errors stop ingestion; callers must not
/// publish partially collected output on failure. Malformed records fail closed;
/// scientifically unusable coordinates/quality are counted selection rejections.
pub fn stream(
    reader: impl BufRead,
    dataset: &Dataset,
    policy: QualityPolicy,
    mut sink: impl FnMut(GaiaObservation) -> Result<(), Error>,
) -> Result<Summary, Error> {
    if !policy.max_ruwe.is_finite() || policy.max_ruwe <= 0.0 {
        return Err(invalid("positive finite RUWE threshold required"));
    }
    let mut csv = Csv::new(
        reader,
        &[
            "source_id",
            "ref_epoch",
            "ra",
            "dec",
            "parallax",
            "parallax_error",
            "pmra",
            "pmdec",
            "radial_velocity",
        ],
    )?;
    let mut summary = Summary::default();
    while let Some(row) = csv.next()? {
        summary.input_rows += 1;
        let source = source_id(&row["source_id"])?;
        let epoch = number(&row, "ref_epoch")?;
        let ra = measurement(&row, "ra")?;
        let dec = measurement(&row, "dec")?;
        let parallax = measurement(&row, "parallax")?;
        let pmra = measurement(&row, "pmra")?;
        let pmdec = measurement(&row, "pmdec")?;
        let radial_velocity = measurement(&row, "radial_velocity")?;
        let ruwe = number(&row, "ruwe")?;
        if ruwe.is_some_and(|r| r <= 0.0) {
            return Err(invalid("nonpositive RUWE"));
        }
        let duplicate = match row.get("duplicated_source").map(String::as_str) {
            None | Some("" | "false" | "False" | "0") => false,
            Some("true" | "True" | "1") => true,
            _ => return Err(invalid("invalid duplicated_source flag")),
        };
        if ruwe.is_some_and(|r| r > policy.max_ruwe)
            || (duplicate && policy.reject_duplicated_source)
        {
            summary.rejected_quality += 1;
            continue;
        }
        let (Some(epoch), Some(ra), Some(dec)) = (epoch, ra, dec) else {
            summary.rejected_astrometry += 1;
            continue;
        };
        if !(0.0..360.0).contains(&ra.value)
            || !(-90.0..=90.0).contains(&dec.value)
            || !(1.0..10000.0).contains(&epoch)
        {
            summary.rejected_astrometry += 1;
            continue;
        }
        summary.missing_ruwe += u64::from(ruwe.is_none());
        summary.missing_or_nonpositive_parallax +=
            u64::from(parallax.as_ref().is_none_or(|p| p.value <= 0.0));
        summary.incomplete_velocity +=
            u64::from(pmra.is_none() || pmdec.is_none() || radial_velocity.is_none());
        let provenance = Provenance::Observed {
            dataset: dataset.id.clone(),
            record_key: row["source_id"].clone(),
        };
        sink(GaiaObservation {
            source,
            provenance,
            ref_epoch_julian_year: epoch,
            ra_degrees: ra,
            dec_degrees: dec,
            parallax_mas: parallax,
            pmra_mas_per_year: pmra,
            pmdec_mas_per_year: pmdec,
            radial_velocity_km_per_second: radial_velocity,
            source_fields: row,
        })?;
        summary.selected_rows += 1;
    }
    Ok(summary)
}

/// Bounded-batch canonicalization adapter for review/partition builders. Streaming
/// decoding is constant record memory; this collector scales with selected rows.
/// Conflicting duplicates fail; identical repeats are counted. Matched source
/// measurements are retained independently rather than averaged or overwritten.
pub struct ImportedGaia {
    pub catalog: Catalog,
    pub observations: BTreeMap<CatalogId, Vec<GaiaObservation>>,
    pub summary: Summary,
    pub policy: QualityPolicy,
}
pub fn import(
    reader: impl BufRead,
    dataset: Dataset,
    revision: String,
    policy: QualityPolicy,
    cross_match: &StellarCrossMatch,
) -> Result<ImportedGaia, Error> {
    let mut by_source = BTreeMap::new();
    let mut duplicates = 0;
    let mut summary = stream(reader, &dataset, policy, |observation| {
        if let Some(old) = by_source.get(&observation.source) {
            if old != &observation {
                return Err(invalid("conflicting Gaia source rows"));
            }
            duplicates += 1;
        } else {
            by_source.insert(observation.source.clone(), observation);
        }
        Ok(())
    })?;
    let mut observations: BTreeMap<CatalogId, Vec<GaiaObservation>> = BTreeMap::new();
    for observation in by_source.into_values() {
        observations
            .entry(cross_match.canonical(&observation.source))
            .or_default()
            .push(observation);
    }
    let mut records = Vec::new();
    for (id, rows) in &observations {
        let system = system_id(id)?;
        let provenance = rows[0].provenance.clone();
        records.push(Record::System(System {
            header: Header {
                id: system.clone(),
                display_name: None,
                provenance: provenance.clone(),
            },
        }));
        records.push(Record::Star(Star {
            header: Header {
                id: id.clone(),
                display_name: None,
                provenance,
            },
            system,
            mass_kg: None,
            radius_meters: None,
        }));
    }
    summary.duplicate_rows = duplicates;
    summary.canonical_stars = Some(observations.len() as u64);
    let catalog = Catalog::new(revision, vec![dataset], records)?;
    Ok(ImportedGaia {
        catalog,
        observations,
        summary,
        policy,
    })
}

pub(super) fn system_id(star: &CatalogId) -> Result<CatalogId, Error> {
    let (namespace, key) = star
        .as_str()
        .split_once(':')
        .ok_or_else(|| invalid("star ID"))?;
    CatalogId::from_source(&format!("system_{namespace}"), key)
}

#[cfg(test)]
mod tests;
