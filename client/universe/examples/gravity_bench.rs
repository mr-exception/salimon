//! Indexed-production service versus the same complete source set, serial CPU.
use salimon_universe::{
    CatalogId,
    coordinates::GalacticPosition,
    gravity::{Config, Selector, Source, SourceIndex},
};
use std::time::Instant;
fn p(x: f64, y: f64, z: f64) -> GalacticPosition {
    GalacticPosition::new([1_000_000_000; 3], [x, y, z]).expect("bounded benchmark coordinates")
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rows = Vec::new();
    for count in [1024, 32768] {
        let sources: Vec<_> = (0..count)
            .map(|i| {
                let x = (i % 32) as f64 * 1e8;
                let y = ((i / 32) % 32) as f64 * 1e8;
                let z = (i / 1024) as f64 * 1e8;
                Ok(Source {
                    id: CatalogId::from_source("bench", &format!("body-{i:06}"))?,
                    position: p(x, y, z),
                    mu_m3_per_s2: 1e14 + (i % 11) as f64 * 1e13,
                    radius_meters: 1.0,
                })
            })
            .collect::<Result<_, salimon_universe::Error>>()?;
        let start = Instant::now();
        let index = SourceIndex::new(sources.clone())?;
        let build_ms = start.elapsed().as_secs_f64() * 1000.0;
        let mut indexed_us = Vec::new();
        let mut direct_us = Vec::new();
        let mut evaluated = 0;
        let mut nodes = 0;
        for i in 0..512 {
            let position = p(
                (i % 32) as f64 * 1e8 + 1e6,
                ((i * 7) % 32) as f64 * 1e8 + 3e6,
                ((i * 3) % 32) as f64 * 1e8 + 5e6,
            );
            let mut selector = Selector::new(Config::default())?;
            let start = Instant::now();
            let (actual, work) = selector.sample_indexed(&index, position, 0.0)?;
            indexed_us.push(start.elapsed().as_secs_f64() * 1e6);
            evaluated += work.evaluated_sources;
            nodes += work.visited_nodes;
            let start = Instant::now();
            let expected = Selector::new(Config::default())?.sample(&sources, position, 0.0)?;
            direct_us.push(start.elapsed().as_secs_f64() * 1e6);
            if actual.weights != expected.weights
                || actual.meters_per_second_squared != expected.meters_per_second_squared
            {
                return Err("indexed/full reference mismatch".into());
            }
        }
        indexed_us.sort_by(f64::total_cmp);
        direct_us.sort_by(f64::total_cmp);
        rows.push(serde_json::json!({"sources":count,"queries":512,"build_ms":build_ms,"indexed_p50_us":indexed_us[256],"indexed_p95_us":indexed_us[486],"indexed_max_us":indexed_us[511],"full_scan_p50_us":direct_us[256],"evaluated_sources":evaluated,"visited_nodes":nodes,"all_queries_exactly_match_complete_source_selection":true}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"rows":rows,"scope":"serial software CPU algorithm workload; no hardware FPS or global scientific accuracy claim"})
        )?
    );
    Ok(())
}
