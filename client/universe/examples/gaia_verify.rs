//! Offline real-source/codec measurement. A bounded sample cannot certify a full pack.
use salimon_universe::{
    pack::{self, pipeline},
    partitions::Reader,
};
use std::{fs, io::Cursor, path::Path, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: gaia_verify SOURCE_DIRECTORY NEW_OUTPUT_DIRECTORY".into());
    }
    let source = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    if output.exists() {
        return Err("output already exists".into());
    }
    let mut results = Vec::new();
    for name in ["sample", "golden"] {
        let root = output.join(name);
        let started = Instant::now();
        let manifest = pipeline::build(&source.join(format!("build-{name}.json")), &root)?;
        let build_seconds = started.elapsed().as_secs_f64();
        let manifest_bytes = manifest.encode()?.len();
        let installed_bytes = fs::read_dir(root.join("blobs"))?
            .map(|e| e.and_then(|e| e.metadata()).map(|m| m.len()))
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .sum::<u64>()
            + manifest_bytes as u64;
        let cell = &manifest.cells[0];
        let summary: serde_json::Value =
            serde_json::from_slice(&pack::read_blob(&root, &cell.sidecars["gaia_selection"])?)?;
        let observations: serde_json::Value = serde_json::from_slice(&pack::read_blob(
            &root,
            &cell.sidecars["gaia_observations"],
        )?)?;
        let selected = summary["summary"]["selected_rows"]
            .as_u64()
            .ok_or("selected count")?;
        let mut decoder_seconds = Vec::new();
        let mut record_count = 0;
        for _ in 0..5 {
            let started = Instant::now();
            let mut reader = Reader::open(Cursor::new(pack::read_blob(&root, &cell.catalog)?))?;
            let catalog = reader.read_cell(cell.key)?;
            record_count = catalog.records().len();
            decoder_seconds.push(started.elapsed().as_secs_f64());
        }
        let mut distributions = serde_json::Map::new();
        for field in [
            "parallax",
            "parallax_error",
            "ruwe",
            "ra_error",
            "dec_error",
            "phot_g_mean_mag",
        ] {
            let mut values = Vec::new();
            let mut missing = 0;
            for o in observations
                .as_object()
                .ok_or("observation mapping")?
                .values()
                .flat_map(|v| v.as_array().into_iter().flatten())
            {
                match o["source_fields"][field].as_str().filter(|s| !s.is_empty()) {
                    Some(s) => values.push(s.parse::<f64>()?),
                    None => missing += 1,
                }
            }
            values.sort_by(f64::total_cmp);
            let quantile = |q: f64| {
                values
                    .get(((values.len().saturating_sub(1)) as f64 * q) as usize)
                    .copied()
            };
            distributions.insert(field.into(),serde_json::json!({"present":values.len(),"missing":missing,"min":values.first(),"median":quantile(0.5),"p95":quantile(0.95),"max":values.last()}));
        }
        let sidecar_bytes = cell.sidecars.values().map(|b| b.bytes).sum::<u64>();
        results.push(serde_json::json!({"selection":name,"summary":summary,"selected_stars":selected,"catalog_records":record_count,"manifest_bytes":manifest_bytes,"manifest_sha256":pack::digest(&manifest.encode()?),"catalog_partition_bytes":cell.catalog.bytes,"astrometry_and_selection_sidecars_bytes":sidecar_bytes,"installed_bytes":installed_bytes,"installed_bytes_per_selected_star":installed_bytes as f64/selected as f64,"build_seconds":build_seconds,"decode_seconds_original_five_samples":decoder_seconds,"source_field_distributions":distributions,"full_release_complete":false}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"version":1,"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"input_scope":"Frozen real DR3 random-index sample and explicit golden IDs. No complete-release or hardware budget acceptance.","results":results})
        )?
    );
    Ok(())
}
