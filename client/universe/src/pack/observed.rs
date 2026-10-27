//! Raw selected-source adapters for the standalone offline pack pipeline.
use super::*;
use crate::{
    BodyKind, Dataset, Header, Record, System,
    ingestion::{exoplanets, gaia, horizons::Ephemeris},
    sources::SourceManifest,
};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub gaia: Option<ArtifactInput>,
    pub exoplanets: Option<ArtifactInput>,
    #[serde(default)]
    pub horizons: Vec<HorizonsInput>,
    #[serde(default)]
    pub stellar_cross_matches: Vec<(CatalogId, CatalogId)>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactInput {
    pub source: CatalogId,
    pub artifact: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HorizonsInput {
    pub source: CatalogId,
    pub artifact: String,
    pub system: CatalogId,
    pub parent: CatalogId,
    pub body_kind: BodyKind,
}
fn artifact(
    root: &Path,
    manifest: &SourceManifest,
    id: &CatalogId,
    name: &str,
    reference: &str,
) -> Result<(Vec<u8>, Dataset), Error> {
    let source = manifest
        .sources
        .iter()
        .find(|s| &s.id == id)
        .ok_or_else(|| invalid("observed input source not registered"))?;
    let selected = source
        .artifacts
        .iter()
        .find(|a| a.path == name)
        .ok_or_else(|| invalid("observed input artifact not frozen in source manifest"))?;
    // Raw files are prepartitioned selected batches. Bound collector amplification
    // before U07/U08 allocate maps; never silently truncate a larger selection.
    if selected.bytes > 8 * 1024 * 1024 {
        return Err(invalid(
            "raw source cell exceeds 8 MiB batch bound; partition selected input",
        ));
    }
    let bytes = pipeline::bounded(&pipeline::local(root, name)?, 8 * 1024 * 1024)?;
    if bytes.len() as u64 != selected.bytes || digest(&bytes) != selected.sha256 {
        return Err(invalid(
            "selected source changed between verification and import",
        ));
    }
    Ok((
        bytes,
        Dataset {
            id: source.id.clone(),
            version: source.release.clone(),
            reference: reference.into(),
        },
    ))
}
pub(super) fn import(
    input: Input,
    source_path: &Path,
    source_bytes: &[u8],
    revision: &str,
) -> Result<(Catalog, BTreeMap<String, Vec<u8>>), Error> {
    let manifest = SourceManifest::decode(source_bytes, false)?;
    let root = source_path.parent().ok_or_else(|| invalid("source root"))?;
    let reference = format!("source-manifest-sha256:{}", digest(source_bytes));
    let cross_match = gaia::StellarCrossMatch::new(input.stellar_cross_matches)?;
    let mut catalog = Catalog::new(revision.into(), vec![], vec![])?;
    let mut sidecars = BTreeMap::new();
    let json = |value: &serde_json::Value| {
        serde_json::to_vec(value).map_err(|e| Error::Encoding(e.to_string()))
    };
    if let Some(selected) = input.gaia {
        let (bytes, dataset) = artifact(
            root,
            &manifest,
            &selected.source,
            &selected.artifact,
            &reference,
        )?;
        let imported = gaia::import(
            bytes.as_slice(),
            dataset,
            revision.into(),
            gaia::QualityPolicy::default(),
            &cross_match,
        )?;
        sidecars.insert(
            "gaia_observations".into(),
            json(&serde_json::json!(imported.observations))?,
        );
        sidecars.insert(
            "gaia_selection".into(),
            json(&serde_json::json!({"policy":imported.policy,"summary":imported.summary}))?,
        );
        catalog = imported.catalog;
    }
    if let Some(selected) = input.exoplanets {
        let (bytes, dataset) = artifact(
            root,
            &manifest,
            &selected.source,
            &selected.artifact,
            &reference,
        )?;
        let imported = exoplanets::import(
            bytes.as_slice(),
            dataset,
            revision.into(),
            Some(&catalog),
            &cross_match,
        )?;
        sidecars.insert(
            "planet_observations".into(),
            json(&serde_json::json!(imported.observations))?,
        );
        sidecars.insert(
            "planet_summary".into(),
            json(&serde_json::json!(imported.summary))?,
        );
        catalog = imported.catalog;
    }
    let mut records = catalog.records().to_vec();
    let mut datasets: BTreeMap<_, _> = catalog
        .datasets()
        .iter()
        .map(|d| (d.id.clone(), d.clone()))
        .collect();
    let mut ephemerides = Vec::new();
    for selected in input.horizons {
        let (bytes, dataset) = artifact(
            root,
            &manifest,
            &selected.source,
            &selected.artifact,
            &reference,
        )?;
        let e = Ephemeris::decode(&bytes, &dataset)?;
        if !records.iter().any(|r| r.id() == &selected.system) {
            records.push(Record::System(System {
                header: Header {
                    id: selected.system.clone(),
                    display_name: None,
                    provenance: e.provenance().clone(),
                },
            }));
        }
        records.push(Record::Body(e.body(
            selected.system,
            selected.parent,
            selected.body_kind,
        )));
        ephemerides.push(serde_json::to_value(&e).map_err(|e| Error::Encoding(e.to_string()))?);
        datasets.insert(dataset.id.clone(), dataset);
    }
    if !ephemerides.is_empty() {
        sidecars.insert("ephemerides".into(), json(&serde_json::json!(ephemerides))?);
    }
    if records.is_empty() {
        return Err(invalid("observed cell has no selected records"));
    }
    Ok((
        Catalog::new(revision.into(), datasets.into_values().collect(), records)?,
        sidecars,
    ))
}
