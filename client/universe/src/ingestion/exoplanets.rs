//! NASA Exoplanet Archive PS default-row ingestion. Missing orbital elements,
//! upper/lower limits and minimum masses never become complete measured orbits.
use super::{
    csv::{Csv, number},
    gaia::{StellarCrossMatch, system_id},
};
use crate::{
    Body, BodyKind, Catalog, CatalogId, Dataset, Error, Header, Provenance, Quantity, Record, Star,
    System, Uncertainty, invalid,
};
use serde::Serialize;
use std::{collections::BTreeMap, io::BufRead};

const DAY_SECONDS: f64 = 86400.0;
const PARSEC_METERS: f64 = 3.085677581491367e16;
const AU_METERS: f64 = 149_597_870_700.0;
// Explicit adopted conversion constants; no inference of unmeasured quantities.
const JUPITER_MASS_KG: f64 = 1.89813e27;
const JUPITER_RADIUS_METERS: f64 = 71_492_000.0;
const SOLAR_MASS_KG: f64 = 1.98847e30;
const SOLAR_RADIUS_METERS: f64 = 695_700_000.0;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Limit {
    Unspecified,
    Measurement,
    Upper,
    Lower,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ReportedQuantity {
    pub quantity: Quantity,
    pub limit: Limit,
    pub error_minus_si: Option<f64>,
    pub error_plus_si: Option<f64>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlanetObservation {
    pub planet: CatalogId,
    pub host: CatalogId,
    pub provenance: Provenance,
    pub literature_reference: String,
    pub host_distance_meters: Option<ReportedQuantity>,
    pub host_mass_kg: Option<ReportedQuantity>,
    pub host_radius_meters: Option<ReportedQuantity>,
    pub planet_mass_kg: Option<ReportedQuantity>,
    pub minimum_mass_kg: Option<ReportedQuantity>,
    pub planet_radius_meters: Option<ReportedQuantity>,
    pub orbital_period_seconds: Option<ReportedQuantity>,
    pub semi_major_axis_meters: Option<ReportedQuantity>,
    pub eccentricity: Option<ReportedQuantity>,
    pub inclination_radians: Option<ReportedQuantity>,
    pub argument_of_periastron_radians: Option<ReportedQuantity>,
    /// Archive days are preserved verbatim; no scale/frame assumption or Epoch conversion.
    pub periastron_time_source_days: Option<ReportedQuantity>,
    pub source_fields: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct Summary {
    pub input_rows: u64,
    pub planets: u64,
    pub hosts: u64,
    pub duplicate_rows: u64,
    pub hosts_using_gaia_ids: u64,
    pub hosts_using_archive_ids: u64,
}
pub struct ImportedPlanets {
    pub catalog: Catalog,
    pub observations: Vec<PlanetObservation>,
    pub summary: Summary,
}
/// Hex-encode the Archive's exact table key (not a normalized display alias).
/// UTF-8 bytes are reversible; whitespace/case/punctuation cannot cause collisions.
fn archive_id(namespace: &str, key: &str) -> Result<CatalogId, Error> {
    if key.trim().is_empty() {
        return Err(invalid("empty Archive table key"));
    }
    let encoded: String = key.as_bytes().iter().map(|b| format!("{b:02x}")).collect();
    CatalogId::from_source(namespace, &encoded)
}
fn host_id(
    row: &BTreeMap<String, String>,
    matches: &StellarCrossMatch,
) -> Result<(CatalogId, bool), Error> {
    if let Some(raw) = row.get("gaia_dr3_id").filter(|s| !s.is_empty()) {
        let key = raw
            .strip_prefix("Gaia DR3 ")
            .ok_or_else(|| invalid("Archive Gaia DR3 ID prefix"))?;
        let parsed: u64 = key
            .parse()
            .map_err(|_| invalid("Archive exact Gaia DR3 integer"))?;
        if parsed == 0 || parsed.to_string() != key {
            return Err(invalid("Archive noncanonical Gaia DR3 ID"));
        }
        return Ok((
            matches.canonical(&CatalogId::from_source("gaia_dr3", key)?),
            true,
        ));
    }
    Ok((
        matches.canonical(&archive_id("nea_host", &row["hostname"])?),
        false,
    ))
}
fn reported(
    row: &BTreeMap<String, String>,
    key: &str,
    factor: f64,
    provenance: &Provenance,
    positive: bool,
) -> Result<Option<ReportedQuantity>, Error> {
    let value = number(row, key)?;
    let plus = number(row, &format!("{key}err1"))?;
    let minus = number(row, &format!("{key}err2"))?;
    let limit = match number(row, &format!("{key}lim"))? {
        None => Limit::Unspecified,
        Some(0.0) => Limit::Measurement,
        Some(1.0) => Limit::Upper,
        Some(-1.0) => Limit::Lower,
        _ => return Err(invalid(format!("Archive invalid limit for {key}"))),
    };
    if plus.is_some_and(|v| v < 0.0) || minus.is_some_and(|v| v > 0.0) {
        return Err(invalid(format!("Archive signed errors for {key}")));
    }
    let Some(value) = value else {
        if plus.is_some() || minus.is_some() || matches!(limit, Limit::Upper | Limit::Lower) {
            return Err(invalid(format!("Archive orphan metadata for {key}")));
        }
        return Ok(None);
    };
    let value = value * factor;
    if !value.is_finite() || (positive && value <= 0.0) {
        return Err(invalid(format!("Archive invalid SI value for {key}")));
    }
    let error_minus_si = minus.map(|v| -v * factor);
    let error_plus_si = plus.map(|v| v * factor);
    if error_minus_si
        .into_iter()
        .chain(error_plus_si)
        .any(|e| !e.is_finite())
    {
        return Err(invalid("Archive error SI overflow"));
    }
    let uncertainty = match (minus, plus) {
        (Some(minus), Some(plus)) => {
            let minus = -minus * factor;
            let plus = plus * factor;
            if !minus.is_finite() || !plus.is_finite() {
                return Err(invalid("Archive error SI overflow"));
            }
            // Preserve published bounds even when they cross physical zero.
            // Canonical positive quantities below are promoted only when valid.
            Uncertainty::Bounds {
                minus,
                plus,
                confidence: None,
            }
        }
        // A one-sided source error stays in raw fields; no invented other side.
        _ => Uncertainty::Unknown,
    };
    Ok(Some(ReportedQuantity {
        quantity: Quantity {
            value,
            uncertainty,
            provenance: provenance.clone(),
            validity: None,
        },
        limit,
        error_minus_si,
        error_plus_si,
    }))
}
fn canonical_quantity(q: &Option<ReportedQuantity>) -> Option<Quantity> {
    q.as_ref()
        .filter(|q| {
            q.limit == Limit::Measurement
                && q.error_minus_si
                    .is_none_or(|minus| minus < q.quantity.value)
                && match &q.quantity.uncertainty {
                    Uncertainty::Bounds { minus, .. } => *minus < q.quantity.value,
                    Uncertainty::Unknown => true,
                }
        })
        .map(|q| q.quantity.clone())
}

/// Reconciles against an optional immutable Gaia/catalog batch. Caller-reviewed
/// aliases resolve non-Gaia hosts; names alone never merge distinct records.
/// Conflicting default rows fail before the combined catalog is published.
pub fn import(
    reader: impl BufRead,
    dataset: Dataset,
    revision: String,
    base: Option<&Catalog>,
    matches: &StellarCrossMatch,
) -> Result<ImportedPlanets, Error> {
    let mut csv = Csv::new(reader, &["pl_name", "hostname", "pl_refname"])?;
    let mut summary = Summary::default();
    let mut observations: BTreeMap<CatalogId, PlanetObservation> = BTreeMap::new();
    let mut records: BTreeMap<CatalogId, Record> = base
        .map(|c| {
            c.records()
                .iter()
                .map(|r| (r.id().clone(), r.clone()))
                .collect()
        })
        .unwrap_or_default();
    let mut datasets = base.map(|c| c.datasets().to_vec()).unwrap_or_default();
    if let Some(old) = datasets.iter().find(|d| d.id == dataset.id) {
        if old != &dataset {
            return Err(invalid("Archive dataset conflicts with base"));
        }
    } else {
        datasets.push(dataset.clone());
    }
    let mut hosts = BTreeMap::new();
    // A hostname-to-ID disagreement is ambiguous, not an invitation to merge.
    let mut named_hosts = BTreeMap::new();
    for row in std::iter::from_fn(|| csv.next().transpose()) {
        let row = row?;
        summary.input_rows += 1;
        if row.get("default_flag").is_some_and(|v| v != "1") {
            return Err(invalid("Archive requires default_flag=1 PS rows"));
        }
        let planet = matches.canonical(&archive_id("nea_planet", &row["pl_name"])?);
        if row["pl_refname"].trim().is_empty() {
            return Err(invalid("Archive literature reference required"));
        }
        if row["hostname"].trim().is_empty() {
            return Err(invalid("Archive hostname required"));
        }
        let (host, gaia) = host_id(&row, matches)?;
        if let Some(previous) = named_hosts.insert(row["hostname"].clone(), host.clone())
            && previous != host
        {
            return Err(invalid("Archive ambiguous host identity"));
        }
        hosts
            .entry(host.clone())
            .and_modify(|v| *v |= gaia)
            .or_insert(gaia);
        let provenance = Provenance::Observed {
            dataset: dataset.id.clone(),
            record_key: row["pl_name"].clone(),
        };
        let eccentricity = reported(&row, "pl_orbeccen", 1.0, &provenance, false)?;
        if eccentricity
            .as_ref()
            .is_some_and(|q| !(0.0..1.0).contains(&q.quantity.value))
        {
            return Err(invalid("Archive requires bound-orbit eccentricity [0,1)"));
        }
        let inclination = reported(
            &row,
            "pl_orbincl",
            std::f64::consts::PI / 180.0,
            &provenance,
            false,
        )?;
        if inclination
            .as_ref()
            .is_some_and(|q| !(0.0..=std::f64::consts::PI).contains(&q.quantity.value))
        {
            return Err(invalid("Archive inclination range"));
        }
        let observation = PlanetObservation {
            planet: planet.clone(),
            host: host.clone(),
            provenance: provenance.clone(),
            literature_reference: row["pl_refname"].clone(),
            host_distance_meters: reported(&row, "sy_dist", PARSEC_METERS, &provenance, true)?,
            host_mass_kg: reported(&row, "st_mass", SOLAR_MASS_KG, &provenance, true)?,
            host_radius_meters: reported(&row, "st_rad", SOLAR_RADIUS_METERS, &provenance, true)?,
            planet_mass_kg: reported(&row, "pl_massj", JUPITER_MASS_KG, &provenance, true)?,
            minimum_mass_kg: reported(&row, "pl_msinij", JUPITER_MASS_KG, &provenance, true)?,
            planet_radius_meters: reported(
                &row,
                "pl_radj",
                JUPITER_RADIUS_METERS,
                &provenance,
                true,
            )?,
            orbital_period_seconds: reported(&row, "pl_orbper", DAY_SECONDS, &provenance, true)?,
            semi_major_axis_meters: reported(&row, "pl_orbsmax", AU_METERS, &provenance, true)?,
            eccentricity,
            inclination_radians: inclination,
            argument_of_periastron_radians: reported(
                &row,
                "pl_orblper",
                std::f64::consts::PI / 180.0,
                &provenance,
                false,
            )?,
            periastron_time_source_days: reported(&row, "pl_orbtper", 1.0, &provenance, false)?,
            source_fields: row,
        };
        if let Some(old) = observations.get(&planet) {
            if old != &observation {
                return Err(invalid("Archive conflicting planet default rows/matches"));
            }
            summary.duplicate_rows += 1;
            continue;
        }
        observations.insert(planet, observation);
    }
    // Stable planet order chooses only header provenance for new hosts. Host
    // physical measurements stay per-source sidecars, never arbitrary composites.
    for (planet, observation) in &observations {
        let host = observation.host.clone();
        let provenance = observation.provenance.clone();
        let system = match records.get(&host) {
            Some(Record::Star(s)) => s.system.clone(),
            Some(_) => return Err(invalid("Archive matched host is not a star")),
            None => {
                let system = system_id(&host)?;
                let record = Record::System(System {
                    header: Header {
                        id: system.clone(),
                        display_name: None,
                        provenance: provenance.clone(),
                    },
                });
                if records.insert(system.clone(), record).is_some() {
                    return Err(invalid("Archive new system collides with existing record"));
                }
                records.insert(
                    host.clone(),
                    Record::Star(Star {
                        header: Header {
                            id: host.clone(),
                            display_name: Some(observation.source_fields["hostname"].clone()),
                            provenance: provenance.clone(),
                        },
                        system: system.clone(),
                        mass_kg: None,
                        radius_meters: None,
                    }),
                );
                system
            }
        };
        let body = Record::Body(Body {
            header: Header {
                id: planet.clone(),
                display_name: Some(observation.source_fields["pl_name"].clone()),
                provenance,
            },
            system,
            parent: host,
            body_kind: BodyKind::Planet,
            mass_kg: canonical_quantity(&observation.planet_mass_kg),
            radius_meters: canonical_quantity(&observation.planet_radius_meters),
            definition: None,
        });
        if records.insert(planet.clone(), body).is_some() {
            return Err(invalid(
                "Archive planet ID collides with existing catalog record",
            ));
        }
    }
    summary.planets = observations.len() as u64;
    summary.hosts = hosts.len() as u64;
    summary.hosts_using_gaia_ids = hosts.values().filter(|gaia| **gaia).count() as u64;
    summary.hosts_using_archive_ids = summary.hosts - summary.hosts_using_gaia_ids;
    let catalog = Catalog::new(revision, datasets, records.into_values().collect())?;
    Ok(ImportedPlanets {
        catalog,
        observations: observations.into_values().collect(),
        summary,
    })
}
#[cfg(test)]
mod tests;
