//! Local development pack builder. Production publication is deliberately unavailable.
use salimon_universe::{Catalog, pack::Builder, partitions::CellKey};
use std::{collections::BTreeMap, path::Path};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err(
            "usage: pack_build SOURCE_MANIFEST OUTPUT_DIR (private development only)".into(),
        );
    }
    let source = std::fs::read(&args[1])?;
    let catalog = Catalog::decode(include_bytes!("../fixtures/schema-v1.json"))?;
    let mut builder = Builder::new(Path::new(&args[2]), catalog.revision().into(), &source)?;
    builder.add_cell(
        CellKey {
            galactic: [0; 3],
            shard: 0,
        },
        &catalog,
        &BTreeMap::new(),
    )?;
    let manifest = builder.finish(None, BTreeMap::new())?;
    println!("{}", serde_json::to_string(&manifest)?);
    Ok(())
}
