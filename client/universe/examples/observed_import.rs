//! Offline deterministic development fixture build; no production publication.
use salimon_universe::{
    BodyKind, Catalog, CatalogId, Dataset, Header, Record, System,
    ingestion::{
        exoplanets,
        gaia::{self, QualityPolicy, StellarCrossMatch},
        horizons::Ephemeris,
    },
    sources::SourceManifest,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().len() != 1 {
        return Err("observed_import takes no arguments; frozen development fixtures only".into());
    }
    let manifest = SourceManifest::decode(
        include_bytes!("../../../catalogs/sources/manifest-v1.json"),
        false,
    )?;
    let dataset = |key: &str| -> Result<Dataset, Box<dyn std::error::Error>> {
        let source = manifest
            .sources
            .iter()
            .find(|s| s.id.as_str() == key)
            .ok_or("missing frozen source")?;
        Ok(Dataset {
            id: source.id.clone(),
            version: source.release.clone(),
            reference: "catalogs/sources/manifest-v1.json".into(),
        })
    };
    let gaia = gaia::import(
        &include_bytes!("../../../catalogs/sources/fixtures/gaia-synthetic.csv")[..],
        dataset("gaia:dr3")?,
        "development-v1".into(),
        QualityPolicy::default(),
        &StellarCrossMatch::default(),
    )?;
    let planets = exoplanets::import(
        &include_bytes!("../../../catalogs/sources/fixtures/exoplanet-proxima.csv")[..],
        dataset("nasa:exoplanet-ps")?,
        "development-v1".into(),
        Some(&gaia.catalog),
        &StellarCrossMatch::default(),
    )?;
    let ephemeris = Ephemeris::decode(
        include_bytes!("../../../catalogs/sources/fixtures/jpl-earth-j2000.json"),
        &dataset("jpl:horizons-earth-j2000")?,
    )?;
    let mut records = planets.catalog.records().to_vec();
    let solar = CatalogId::from_source("system", "solar")?;
    records.push(Record::System(System {
        header: Header {
            id: solar.clone(),
            display_name: Some("Solar System".into()),
            provenance: ephemeris.provenance().clone(),
        },
    }));
    records.push(Record::Body(ephemeris.body(
        solar.clone(),
        solar,
        BodyKind::Planet,
    )));
    let mut datasets = planets.catalog.datasets().to_vec();
    datasets.push(dataset("jpl:horizons-earth-j2000")?);
    let catalog = Catalog::new("development-v1".into(), datasets, records)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "format":"salimon.universe.ingestion-fixture","schema_version":1,"production_eligible":false,
            "source_manifest":manifest,"catalog":catalog,"gaia_observations":gaia.observations,
            "gaia_quality_policy":gaia.policy,"gaia_summary":gaia.summary,
            "planet_observations":planets.observations,"planet_summary":planets.summary,
            "ephemerides":[ephemeris]
        }))?
    );
    Ok(())
}
