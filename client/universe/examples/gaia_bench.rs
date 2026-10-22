//! Synthetic ingestion/storage sensitivity; never a production-pack forecast.
use flate2::{Compression, write::ZlibEncoder};
use salimon_universe::{
    CatalogId, Dataset,
    ingestion::gaia::{self, QualityPolicy, StellarCrossMatch},
};
use std::{io::Write, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = include_str!("../../../catalogs/sources/fixtures/gaia-synthetic.csv");
    let dataset = Dataset {
        id: CatalogId::from_source("dataset", "gaia-dr3")?,
        version: "synthetic-v1".into(),
        reference: "catalogs/sources/manifest-v1.json".into(),
    };
    let mut results = Vec::new();
    for count in [2_u64, 10_000] {
        let mut csv = String::from(fixture.lines().next().ok_or("fixture header")?);
        csv.push('\n');
        for row in 0..count {
            csv.push_str(&format!(
                "{},2016.0,{},{},100.0,0.5,10.0,-20.0,\n",
                1_000_000_000_000_000_001_u64 + row,
                row % 360,
                (row % 180) as i64 - 90
            ));
        }
        let start = Instant::now();
        let imported = gaia::import(
            csv.as_bytes(),
            dataset.clone(),
            "bench-v1".into(),
            QualityPolicy::default(),
            &StellarCrossMatch::default(),
        )?;
        let ingest_seconds = start.elapsed().as_secs_f64();
        let payload = serde_json::to_vec(
            &serde_json::json!({"catalog":imported.catalog,"observations":imported.observations,"selection":imported.policy,"summary":imported.summary}),
        )?;
        let start = Instant::now();
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(6));
        encoder.write_all(&payload)?;
        let compressed = encoder.finish()?;
        let seconds = start.elapsed().as_secs_f64();
        let bytes_per_star = compressed.len() as f64 / count as f64;
        results.push(serde_json::json!({"synthetic_stars":count,"input_bytes":csv.len(),"ingest_seconds":ingest_seconds,"stars_per_second":count as f64 / ingest_seconds,"json_bytes":payload.len(),"zlib_bytes":compressed.len(),"compressed_bytes_per_star":bytes_per_star,"compress_seconds":seconds,"conditional_1_8_billion_star_gb":bytes_per_star * 1.8,"conditional_5gb_star_capacity":5e9/bytes_per_star,"conditional_10gb_star_capacity":10e9/bytes_per_star}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"workload":"synthetic source rows + canonical records + all astrometry/provenance; JSON/zlib6 sensitivity, not U04 wire or production pack forecast; excludes full quality-column diversity, partition indexes and planets", "os":std::env::consts::OS,"arch":std::env::consts::ARCH,"rows":results})
        )?
    );
    Ok(())
}
