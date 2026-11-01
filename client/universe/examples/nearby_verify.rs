//! Build and exercise the real nearby pack through production filesystem queries.
use salimon_universe::{
    CatalogId, Record,
    pack::{self, pipeline},
    reader::Reader,
};
use std::{collections::BTreeMap, fs, path::Path, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: nearby_verify SOURCE_DIRECTORY NEW_OUTPUT_DIRECTORY".into());
    }
    let source = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    if output.exists() {
        return Err("output already exists".into());
    }
    let selection: serde_json::Value =
        serde_json::from_slice(&fs::read(source.join("selection.json"))?)?;
    let started = Instant::now();
    let manifest = pipeline::build(&source.join("build.json"), output)?;
    let build_seconds = started.elapsed().as_secs_f64();
    let encoded = manifest.encode()?;
    let installed_bytes = fs::read_dir(output.join("blobs"))?
        .map(|e| e.and_then(|e| e.metadata()).map(|m| m.len()))
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .sum::<u64>()
        + encoded.len() as u64;
    if installed_bytes > 1_000_000_000 {
        return Err("nearby installed pack exceeds owner-selected 1 GB cap".into());
    }
    let mut reader = Reader::open(output, 64 * 1024)?;
    let mut counts = BTreeMap::<String, u64>::new();
    let mut original_decode_seconds = Vec::new();
    let mut sidecar_digests = Vec::new();
    for cell in &manifest.cells {
        let started = Instant::now();
        let catalog = reader.cell(cell.key)?;
        original_decode_seconds.push(started.elapsed().as_secs_f64());
        for record in catalog.records() {
            let kind = match record {
                Record::System(_) => "systems",
                Record::Star(_) => "stars",
                Record::Body(_) => "bodies",
                Record::Population(_) => "populations",
            };
            *counts.entry(kind.into()).or_default() += 1;
        }
        for (name, blob) in &cell.sidecars {
            let raw = reader.sidecar(cell.key, name)?;
            let value: serde_json::Value = serde_json::from_slice(&raw)?;
            if name == "gaia_selection" {
                for field in [
                    "input_rows",
                    "selected_rows",
                    "rejected_quality",
                    "rejected_astrometry",
                    "missing_ruwe",
                    "missing_or_nonpositive_parallax",
                    "incomplete_velocity",
                    "duplicate_rows",
                ] {
                    *counts.entry(format!("gaia_{field}")).or_default() += value["summary"][field]
                        .as_u64()
                        .ok_or("Gaia summary count")?;
                }
            } else if name == "planet_summary" {
                *counts.entry("ps_planets".into()).or_default() +=
                    value["planets"].as_u64().ok_or("planet summary")?;
            } else if name == "ephemerides" {
                *counts.entry("solar_system_targets".into()).or_default() +=
                    value.as_array().ok_or("ephemerides")?.len() as u64;
            }
            sidecar_digests.push(serde_json::json!({"key":cell.key,"name":name,"encoded_bytes":blob.bytes,"decoded_bytes":raw.len(),"decoded_sha256":pack::digest(&raw)}));
        }
        if reader.resident_bytes() > 64 * 1024 {
            return Err("catalog cache exceeded budget".into());
        }
    }
    if counts.get("gaia_input_rows").copied() != selection["expected_rows"].as_u64()
        || counts.get("ps_planets").copied() != selection["companions"]["selected_planets"].as_u64()
        || counts.get("solar_system_targets").copied()
            != selection["companions"]["solar_system_targets"].as_u64()
    {
        return Err("production counts differ from frozen regional source selection".into());
    }
    // Exercise exact-ID lookup, including planet-bearing independent goldens and
    // canonical Solar System parent identities, after reading all real cells.
    let mut lookups = Vec::new();
    for id in [
        "gaia_dr3:5853498713190525696",
        "gaia_dr3:4472832130942575872",
        "jpl:10",
        "jpl:399",
        "jpl:301",
    ] {
        let id = CatalogId::try_from(id.to_owned())?;
        let started = Instant::now();
        if reader.record(&id)?.is_none() {
            return Err(format!("missing nearby golden: {}", id.as_str()).into());
        }
        lookups.push(serde_json::json!({"id":id,"seconds":started.elapsed().as_secs_f64()}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version":1,"os":std::env::consts::OS,"arch":std::env::consts::ARCH,
            "installed_bytes":installed_bytes,"cap_bytes":1_000_000_000,"cells":manifest.cells.len(),
            "manifest_bytes":encoded.len(),"manifest_sha256":pack::digest(&encoded),
            "source_manifest_sha256":pack::digest(&fs::read(source.join("manifest.json"))?),
            "build_seconds":build_seconds,"counts":counts,"decode_seconds_each_original_cell":original_decode_seconds,
            "sidecar_digests":sidecar_digests,"golden_lookups":lookups,
            "reader_stats":reader.stats(),"reader_resident_bytes":reader.resident_bytes(),
            "selected_query_complete":true,"full_release_complete":false,
            "physical_coverage":"Source parallax/PS nominal-distance selections. Storage shards are not physical galactic coordinates; no ordinary-play world adoption or full Milky Way acceptance."
        }))?
    );
    Ok(())
}
