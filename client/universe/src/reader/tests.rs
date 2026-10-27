use super::*;
use crate::pack::Builder;
#[test]
fn randomized_seek_eviction_stays_bounded() {
    let (root, _) = setup();
    let fixture = std::str::from_utf8(include_bytes!("../../fixtures/schema-v1.json")).unwrap();
    let mut builder = Builder::new(
        &root,
        "fixture-v1".into(),
        include_bytes!("../../../../catalogs/sources/manifest-v1.json"),
    )
    .unwrap();
    for i in 0..32 {
        let raw = fixture
            .replace("fixture:system", &format!("fixture:system-{i}"))
            .replace("fixture:star", &format!("fixture:star-{i}"));
        let catalog = Catalog::decode(raw.as_bytes()).unwrap();
        builder
            .add_cell(
                CellKey {
                    galactic: [i; 3],
                    shard: 0,
                },
                &catalog,
                &BTreeMap::new(),
            )
            .unwrap();
    }
    builder.finish(None, BTreeMap::new()).unwrap();
    let mut reader = Reader::open(&root, 4096).unwrap();
    let mut state = 19_u64;
    for _ in 0..1000 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let i = ((state >> 32) % 32) as i64;
        let id = CatalogId::from_source("fixture", &format!("star-{i}")).unwrap();
        assert!(reader.record(&id).unwrap().is_some());
        assert!(reader.resident_bytes() <= 4096);
    }
    fs::remove_dir_all(root).unwrap();
}
fn setup() -> (PathBuf, CellKey) {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "salimon-reader-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let catalog = Catalog::decode(include_bytes!("../../fixtures/schema-v1.json")).unwrap();
    let key = CellKey {
        galactic: [0; 3],
        shard: 0,
    };
    let mut b = Builder::new(
        &root,
        catalog.revision().into(),
        include_bytes!("../../../../catalogs/sources/manifest-v1.json"),
    )
    .unwrap();
    b.add_cell(key, &catalog, &BTreeMap::new()).unwrap();
    b.finish(None, BTreeMap::new()).unwrap();
    (root, key)
}
#[test]
fn queries_and_tiny_budget_never_cache_oversized_cells() {
    let (root, key) = setup();
    let mut reader = Reader::open(&root, 1).unwrap();
    let id = CatalogId::from_source("fixture", "system").unwrap();
    for _ in 0..100 {
        assert!(reader.record(&id).unwrap().is_some());
        assert_eq!(reader.resident_bytes(), 0);
    }
    assert_eq!(reader.region([-1; 3], [1; 3]).unwrap(), vec![key]);
    assert!(reader.region([1; 3], [-1; 3]).is_err());
    assert!(
        reader
            .record(&CatalogId::from_source("none", "missing").unwrap())
            .unwrap()
            .is_none()
    );
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn concurrent_prefetch_uses_same_verified_cache_and_retains_errors() {
    let (root, key) = setup();
    let reader = Arc::new(Mutex::new(Reader::open(&root, 1024 * 1024).unwrap()));
    let prefetch = Prefetch::start(reader.clone());
    assert!(prefetch.request(key));
    assert!(prefetch.request(CellKey { shard: 99, ..key }));
    drop(prefetch);
    let mut reader = reader.lock().unwrap();
    assert!(reader.resident_bytes() > 0);
    assert!(reader.resident_bytes() <= 1024 * 1024);
    assert!(reader.last_prefetch_error().is_some());
    assert_eq!(reader.cell(key).unwrap().revision(), "fixture-v1");
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn corrupt_or_missing_cells_never_enter_cache() {
    let (root, key) = setup();
    let mut reader = Reader::open(&root, 1024 * 1024).unwrap();
    let path = pack::blob_path(&root, &reader.manifest.cells[0].catalog).unwrap();
    fs::write(&path, b"bad").unwrap();
    assert!(reader.cell(key).is_err());
    assert_eq!(reader.resident_bytes(), 0);
    fs::remove_file(path).unwrap();
    assert!(reader.cell(key).is_err());
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn cache_hits_avoid_catalog_io_and_parallel_navigation_is_bounded() {
    let (root, key) = setup();
    let reader = Arc::new(Mutex::new(Reader::open(&root, 16384).unwrap()));
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let reader = reader.clone();
            std::thread::spawn(move || {
                for _ in 0..100 {
                    let mut r = reader.lock().unwrap();
                    assert!(!r.cell(key).unwrap().records().is_empty());
                    assert!(r.resident_bytes() <= 16384);
                }
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    let r = reader.lock().unwrap();
    assert_eq!(r.stats().catalog_blob_reads, 1);
    assert_eq!(r.stats().cache_hits, 799);
    fs::remove_dir_all(root).unwrap();
}
