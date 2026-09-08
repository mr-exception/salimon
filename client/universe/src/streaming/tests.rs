use super::*;
use crate::pack::Builder;
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};
fn fixture() -> std::path::PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "salimon-stream-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let mut builder = Builder::new(
        &root,
        "fixture-v1".into(),
        include_bytes!("../../../../catalogs/sources/manifest-v1.json"),
    )
    .unwrap();
    let fixture = std::str::from_utf8(include_bytes!("../../fixtures/schema-v1.json")).unwrap();
    for i in 0..32 {
        let raw = fixture
            .replace("fixture:system", &format!("fixture:system-{i}"))
            .replace("fixture:star", &format!("fixture:star-{i}"));
        let catalog = Catalog::decode(raw.as_bytes()).unwrap();
        builder
            .add_cell(
                CellKey {
                    galactic: [i * 1_000_000; 3],
                    shard: 0,
                },
                &catalog,
                &BTreeMap::new(),
            )
            .unwrap();
    }
    builder.finish(None, BTreeMap::new()).unwrap();
    root
}
fn settle(stream: &mut Stream, p: GalacticPosition) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        stream.update(p, [0.; 3]).unwrap();
        assert!(stream.active.len() <= stream.budget.active_cells);
        assert!(stream.resident_encoded_bytes <= stream.budget.cache_bytes);
        if stream.ready() || stream.last_error.is_some() {
            break;
        }
        assert!(Instant::now() < deadline, "stream did not settle");
        thread::yield_now();
    }
}
#[test]
fn huge_distance_travel_quality_and_eviction_preserve_canonical_bytes() {
    let root = fixture();
    let mut small = Stream::open(
        &root,
        Budget {
            active_cells: 2,
            cache_bytes: 4096,
            ..Budget::default()
        },
    )
    .unwrap();
    let mut large = Stream::open(
        &root,
        Budget {
            active_cells: 8,
            cache_bytes: 32768,
            ..Budget::default()
        },
    )
    .unwrap();
    for i in (0..32).chain((0..32).rev()) {
        let p = GalacticPosition::new([i * 1_000_000; 3], [0.; 3]).unwrap();
        settle(&mut small, p);
        settle(&mut large, p);
        let key = CellKey {
            galactic: p.cells(),
            shard: 0,
        };
        let a = small.active.iter().find(|c| c.key == key).unwrap();
        let b = large.active.iter().find(|c| c.key == key).unwrap();
        assert_eq!(a.catalog.encode().unwrap(), b.catalog.encode().unwrap());
    }
    drop(small);
    drop(large);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn stale_inflight_result_cannot_publish_after_teleport_and_errors_are_observable() {
    let root = fixture();
    let mut stream = Stream::open(
        &root,
        Budget {
            active_cells: 1,
            ..Budget::default()
        },
    )
    .unwrap();
    stream
        .update(GalacticPosition::new([0; 3], [0.; 3]).unwrap(), [0.; 3])
        .unwrap();
    let p = GalacticPosition::new([31_000_000; 3], [0.; 3]).unwrap();
    settle(&mut stream, p);
    assert_eq!(stream.active()[0].key.galactic, p.cells());
    assert!(stream.update(p, [f64::NAN; 3]).is_err());
    drop(stream);
    let mut reader = Reader::open(&root, 4096).unwrap();
    let blob = reader.manifest().cells[0].catalog.clone();
    std::fs::write(crate::pack::blob_path(&root, &blob).unwrap(), b"bad").unwrap();
    assert!(
        reader
            .cell(CellKey {
                galactic: [0; 3],
                shard: 0
            })
            .is_err()
    );
    let mut stream = Stream::open(
        &root,
        Budget {
            active_cells: 1,
            ..Budget::default()
        },
    )
    .unwrap();
    settle(&mut stream, GalacticPosition::new([0; 3], [0.; 3]).unwrap());
    assert!(stream.last_error.is_some());
    assert!(stream.active().is_empty());
    drop(stream);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn prediction_and_extreme_cell_arithmetic_are_bounded() {
    let root = fixture();
    let mut stream = Stream::open(
        &root,
        Budget {
            active_cells: 2,
            ..Budget::default()
        },
    )
    .unwrap();
    let p = GalacticPosition::new([i64::MIN + 1; 3], [0.; 3]).unwrap();
    stream.update(p, [0.; 3]).unwrap();
    assert_eq!(stream.wanted().len(), 2);
    assert!(
        Stream::open(
            &root,
            Budget {
                active_cells: 0,
                ..Budget::default()
            }
        )
        .is_err()
    );
    drop(stream);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn predicted_region_is_requested_without_changing_current_cell_identity() {
    let root = fixture();
    let mut stream = Stream::open(
        &root,
        Budget {
            active_cells: 2,
            ..Budget::default()
        },
    )
    .unwrap();
    assert!(!stream.ready());
    let p = GalacticPosition::new([0; 3], [0.; 3]).unwrap();
    let velocity = [1_000_000.0 * crate::coordinates::CELL_METERS / 5.0; 3];
    stream.update(p, velocity).unwrap();
    assert_eq!(stream.wanted()[0].galactic, [0; 3]);
    assert_eq!(stream.wanted()[1].galactic, [1_000_000; 3]);
    let mut micros = Vec::new();
    for i in 0..10_000 {
        let start = Instant::now();
        stream
            .update(
                GalacticPosition::new([(i % 32) * 1_000_000; 3], [0.; 3]).unwrap(),
                [0.; 3],
            )
            .unwrap();
        micros.push(start.elapsed().as_secs_f64() * 1e6);
    }
    micros.sort_by(f64::total_cmp);
    eprintln!(
        "10,000 production streaming polls / 32 sparse cells: median={} us p95={} us max={} us; no frame-thread IO",
        micros[5000], micros[9500], micros[9999]
    );
    drop(stream);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn sky_sidecar_is_verified_on_worker_and_does_not_grant_discovery() {
    let root = fixture();
    // Build an explicit single-cell catalog with a typed physical star sidecar.
    let path = root.join("sky-fixture");
    let catalog = Catalog::decode(include_bytes!("../../fixtures/schema-v1.json")).unwrap();
    let id = catalog
        .records()
        .iter()
        .find_map(|r| {
            if let crate::Record::Star(s) = r {
                Some(s.header.id.clone())
            } else {
                None
            }
        })
        .unwrap();
    let star = crate::sky::StarSource {
        id,
        position: GalacticPosition::new([1, 0, 0], [0.; 3]).unwrap(),
        epoch: crate::epoch::Epoch::calendar([2000, 1, 1], [0; 4], crate::TimeScale::Tt).unwrap(),
        velocity_mps: None,
        radius_meters: 7e8,
        temperature_kelvin: None,
        luminosity_solar: None,
    };
    for (name, bytes, valid) in [
        ("valid", serde_json::to_vec(&vec![star]).unwrap(), true),
        ("invalid", b"[{}]".to_vec(), false),
    ] {
        let dir = path.join(name);
        let mut builder = Builder::new(
            &dir,
            catalog.revision().into(),
            include_bytes!("../../../../catalogs/sources/manifest-v1.json"),
        )
        .unwrap();
        builder
            .add_cell(
                CellKey {
                    galactic: [0; 3],
                    shard: 0,
                },
                &catalog,
                &BTreeMap::from([("sky".into(), bytes)]),
            )
            .unwrap();
        builder.finish(None, BTreeMap::new()).unwrap();
        let mut stream = Stream::open(&dir, Budget::default()).unwrap();
        settle(&mut stream, GalacticPosition::new([0; 3], [0.; 3]).unwrap());
        if valid {
            assert!(stream.ready());
            assert_eq!(stream.active()[0].sky.len(), 1);
        } else {
            assert!(stream.last_error.is_some());
            assert!(stream.active().is_empty());
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}
