//! Offline CLI: candidate file required; nonzero on errors or tolerance failures.
use salimon_universe::reference::{MAX_REFERENCE_BYTES, Suite, decode_samples};
use std::{fs::File, io::Read};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 {
        return Err("usage: reference_compare <versioned-candidate-samples.json>".into());
    }
    let suite = Suite::decode(include_bytes!("../fixtures/references-v1.json"))?;
    let mut bytes = Vec::new();
    File::open(&args[0])?
        .take(MAX_REFERENCE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    let result = suite.compare(&decode_samples(&bytes)?)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    if !result.passed {
        return Err("reference tolerance exceeded".into());
    }
    Ok(())
}
