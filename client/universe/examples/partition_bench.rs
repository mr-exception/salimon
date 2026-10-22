//! Offline storage sensitivity workload, not a full-catalog extrapolation.
use salimon_universe::{
    Catalog, CatalogId, Record,
    partitions::{self, CellKey, Reader},
};
use std::{io::Cursor, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Catalog::decode(include_bytes!("../fixtures/storage-observed-v1.json"))?;
    let mut rows = Vec::new();
    for count in [1, 32, 512] {
        let mut cells = Vec::new();
        let mut raw_bytes = 0;
        for shard in 0..count {
            let mut records = fixture.records().to_vec();
            let remap = |id: &CatalogId| {
                CatalogId::from_source(
                    "bench",
                    &format!("{}-{shard}", id.as_str().replace(':', "-")),
                )
            };
            if count > 1 {
                for r in &mut records {
                    match r {
                        Record::System(s) => s.header.id = remap(&s.header.id)?,
                        Record::Body(b) => {
                            b.header.id = remap(&b.header.id)?;
                            b.system = remap(&b.system)?;
                            b.parent = remap(&b.parent)?;
                        }
                        _ => unreachable!("frozen body/system fixture"),
                    }
                }
            }
            let c = Catalog::new(
                fixture.revision().into(),
                fixture.datasets().to_vec(),
                records,
            )?;
            raw_bytes += c.encode()?.len();
            cells.push((
                CellKey {
                    galactic: [0; 3],
                    shard,
                },
                c,
            ));
        }
        let start = Instant::now();
        let bytes = partitions::encode(cells)?;
        let encode_ms = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        let mut reader = Reader::open(Cursor::new(&bytes))?;
        let open_ms = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        for shard in (0..count).rev() {
            reader.read_cell(CellKey {
                galactic: [0; 3],
                shard,
            })?;
        }
        let decode_ms = start.elapsed().as_secs_f64() * 1000.0;
        let records = count * fixture.records().len() as u32;
        let bytes_per_record = bytes.len() as f64 / f64::from(records);
        rows.push(serde_json::json!({"cells":count,"records":records,"synthetically_replicated":count>1,"raw_bytes":raw_bytes,"pack_bytes":bytes.len(),"bytes_per_record_including_index":bytes_per_record,"encode_ms":encode_ms,"index_open_ms":open_ms,"read_all_reverse_ms":decode_ms,"records_fitting_5gb_at_sample_density":5e9/bytes_per_record,"records_fitting_10gb_at_sample_density":10e9/bytes_per_record}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"workload":"U04 frozen Earth/Proxima identity and physical fields; duplicated cells are sensitivity only, not a Gaia population or pack-size forecast", "os":std::env::consts::OS,"arch":std::env::consts::ARCH,"rows":rows})
        )?
    );
    Ok(())
}
