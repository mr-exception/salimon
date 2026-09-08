//! Reproducible pack storage/throughput workload; expanded rows are synthetic.
use salimon_universe::{
    Catalog, CatalogId, Record,
    pack::{Builder, digest},
    partitions::CellKey,
};
use std::{collections::BTreeMap, fs, path::PathBuf, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        return Err("usage: pack_bench OUTPUT_DIR".into());
    }
    let output = PathBuf::from(&args[1]);
    if output.exists() {
        return Err("benchmark output already exists".into());
    }
    let fixture = Catalog::decode(include_bytes!("../fixtures/storage-observed-v1.json"))?;
    let mut rows = Vec::new();
    for count in [1, 128, 1024] {
        let root = output.join(count.to_string());
        let mut builder = Builder::new(
            &root,
            fixture.revision().into(),
            include_bytes!("../../../catalogs/sources/manifest-v1.json"),
        )?;
        let start = Instant::now();
        for shard in 0..count {
            let mut records = fixture.records().to_vec();
            let remap = |id: &CatalogId| {
                CatalogId::from_source(
                    "bench",
                    &format!("{}-{shard}", id.as_str().replace(':', "-")),
                )
            };
            for r in &mut records {
                match r {
                    Record::System(s) => s.header.id = remap(&s.header.id)?,
                    Record::Body(b) => {
                        b.header.id = remap(&b.header.id)?;
                        b.system = remap(&b.system)?;
                        b.parent = remap(&b.parent)?;
                    }
                    _ => return Err("unsupported storage fixture kind".into()),
                }
            }
            let catalog = Catalog::new(
                fixture.revision().into(),
                fixture.datasets().to_vec(),
                records,
            )?;
            builder.add_cell(
                CellKey {
                    galactic: [i64::from(shard), 0, 0],
                    shard: 0,
                },
                &catalog,
                &BTreeMap::new(),
            )?;
        }
        let manifest = builder.finish(None, BTreeMap::new())?;
        let elapsed = start.elapsed().as_secs_f64();
        let encoded = manifest.encode()?;
        let bytes = encoded.len() as u64
            + fs::read_dir(root.join("blobs"))?
                .map(|p| p.and_then(|p| p.metadata()).map(|m| m.len()))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .sum::<u64>();
        rows.push(serde_json::json!({"cells":count,"records":count as usize*fixture.records().len(),"source":"frozen Earth/Proxima fields; identities remapped synthetically","installed_bytes":bytes,"build_seconds":elapsed,"manifest_sha256":digest(&encoded)}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"rows":rows,"full_milky_way_budget":"not extrapolated from replicated fixtures; real full-release resource acceptance remains #524/#526"})
        )?
    );
    Ok(())
}
