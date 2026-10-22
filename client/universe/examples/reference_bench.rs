//! Repeated portable CPU diagnostics; never a game FPS or model-accuracy claim.
#[path = "support/reference_inputs.rs"]
mod reference_inputs;
use salimon_universe as universe;
use salimon_universe::reference::Suite;
use std::{hint::black_box, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|s| s == "--emit-samples") && args.len() == 1 {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({"format":"salimon.universe.samples","schema_version":1,"samples":reference_inputs::samples()?})
            )?
        );
        return Ok(());
    }
    if !args.is_empty() {
        return Err("usage: reference_bench [--emit-samples]".into());
    }
    let suite = Suite::decode(include_bytes!("../fixtures/references-v1.json"))?;
    let actual = reference_inputs::samples()?;
    let initial = suite.compare(&actual)?;
    if !initial.passed {
        return Err("reference preflight failed".into());
    }
    let iterations = 1000;
    let mut rows = Vec::new();
    for workload in [
        "production_u02_and_frozen_source_adapter",
        "strict_comparator",
    ] {
        let mut elapsed = Vec::new();
        for _ in 0..7 {
            let start = Instant::now();
            for _ in 0..iterations {
                if workload == "strict_comparator" {
                    black_box(suite.compare(black_box(&actual))?);
                } else {
                    black_box(reference_inputs::samples()?);
                }
            }
            elapsed.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        let mut sorted = elapsed.clone();
        sorted.sort_by(f64::total_cmp);
        rows.push(serde_json::json!({"workload":workload,"iterations_per_sample":iterations,"elapsed_ms":elapsed,"min_ms":sorted[0],"median_ms":sorted[3],"max_ms":sorted[6]}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"profile":if cfg!(debug_assertions){"debug"}else{"release"},"reference_preflight":initial,"workloads":rows,"limitations":"shared-executor CPU diagnostics incl allocation/parse overhead; no ephemeris solver, frame rate or native M1 signoff"})
        )?
    );
    Ok(())
}
