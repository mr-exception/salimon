//! Local development pack builder. Production publication is deliberately unavailable.
use salimon_universe::pack::pipeline;
use std::path::Path;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: pack_build BUILD_REQUEST.json OUTPUT_DIR (private development)".into());
    }
    let manifest = pipeline::build(Path::new(&args[1]), Path::new(&args[2]))?;
    println!("{}", serde_json::to_string(&manifest)?);
    Ok(())
}
