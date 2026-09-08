//! Real reader/cache/worker workload over the complete pack_bench output.
use salimon_universe::reader::{Prefetch, Reader};
use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::Instant,
};
fn rss() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmRSS:")
                .and_then(|s| s.split_whitespace().next()?.parse().ok())
        })
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        return Err("usage: reader_bench PACK_DIR".into());
    }
    let mut rows = Vec::new();
    for budget in [16384, 65536, 262144] {
        let reader = Arc::new(Mutex::new(Reader::open(Path::new(&args[1]), budget)?));
        let keys: Vec<_> = reader
            .lock()
            .map_err(|_| "reader lock")?
            .manifest()
            .cells
            .iter()
            .map(|c| c.key)
            .collect();
        let before = rss();
        let mut samples = Vec::new();
        let mut residency = 0;
        let mut state = 19_u64;
        let mut memory = Vec::new();
        let prefetch = Prefetch::start(reader.clone());
        for i in 0..20000 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let key = keys[(state >> 32) as usize % keys.len()];
            let start = Instant::now();
            let mut r = reader.lock().map_err(|_| "reader lock")?;
            let catalog = r.cell(key)?;
            if catalog.records().is_empty() {
                return Err("unexpected empty benchmark cell".into());
            }
            residency = residency.max(r.resident_bytes());
            if residency > budget {
                return Err("cache budget exceeded".into());
            }
            drop(r);
            samples.push(start.elapsed().as_secs_f64() * 1e6);
            if i % 100 == 0 {
                let _ = prefetch.request(keys[(i as usize + 1) % keys.len()]);
            }
            if i % 5000 == 0 {
                memory.push(rss());
            }
        }
        drop(prefetch);
        samples.sort_by(f64::total_cmp);
        let r = reader.lock().map_err(|_| "reader lock")?;
        rows.push(serde_json::json!({"cells":keys.len(),"random_seeks":samples.len(),"budget_bytes":budget,"max_resident_encoded_bytes":residency,"latency_microseconds":{"p50":samples[samples.len()/2],"p95":samples[samples.len()*95/100],"max":samples[samples.len()-1]},"catalog_io":r.stats(),"rss_kib_before":before,"rss_kib_quarters":memory,"rss_kib_after":rss(),"prefetch_error":r.last_prefetch_error()}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"rows":rows,"scope":"actual filesystem reader/worker; RSS includes manifest/index, allocator and caller-owned decoded catalogs beyond encoded cache"})
        )?
    );
    Ok(())
}
