use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub(crate) fn root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "salimon-pack-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}
pub(crate) fn fixture() -> Catalog {
    Catalog::decode(include_bytes!("../../fixtures/schema-v1.json")).unwrap()
}
pub(crate) fn builder(root: &Path) -> Builder {
    Builder::new(
        root,
        "fixture-v1".into(),
        include_bytes!("../../../../catalogs/sources/manifest-v1.json"),
    )
    .unwrap()
}
pub(crate) fn key() -> CellKey {
    CellKey {
        galactic: [0; 3],
        shard: 0,
    }
}
#[test]
fn repeat_incremental_build_preserves_exact_sidecars_and_manifest() {
    let root = root();
    let sidecars = BTreeMap::from([(
        "astrometry".into(),
        br#"{"missing_velocity":null}"#.to_vec(),
    )]);
    let mut b = builder(&root);
    b.add_cell(key(), &fixture(), &sidecars).unwrap();
    let first = b.finish(None, BTreeMap::new()).unwrap();
    let bytes = fs::read(root.join("manifest.json")).unwrap();
    let mut b = builder(&root);
    b.add_cell(key(), &fixture(), &sidecars).unwrap();
    b.finish(Some(&first), BTreeMap::new()).unwrap();
    assert_eq!(bytes, fs::read(root.join("manifest.json")).unwrap());
    assert_eq!(
        read_blob(&root, &first.cells[0].sidecars["astrometry"]).unwrap(),
        sidecars["astrometry"]
    );
    let mut m = first.clone();
    m.production_eligible = true;
    assert!(m.encode().is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn version_two_compresses_exact_sidecar_bytes_and_reader_retains_them() {
    let root = root();
    // Unknown columns, whitespace and nulls must survive byte-for-byte, rather
    // than being parsed/reformatted or dropped to meet a size target.
    let raw = br#" {"source_id":"5853498713190525696", "future_column":null} "#.repeat(1000);
    let mut b = builder(&root);
    b.add_cell(
        key(),
        &fixture(),
        &BTreeMap::from([("astrometry".into(), raw.clone())]),
    )
    .unwrap();
    let first = b.finish(None, BTreeMap::new()).unwrap();
    assert_eq!(first.version, 2);
    let blob = &first.cells[0].sidecars["astrometry"];
    assert_eq!(blob.zlib_raw_bytes, Some(raw.len() as u64));
    assert!(blob.bytes < raw.len() as u64 / 10);
    assert_eq!(read_blob(&root, blob).unwrap(), raw);
    let reader = crate::reader::Reader::open(&root, 4096).unwrap();
    assert_eq!(reader.sidecar(key(), "astrometry").unwrap(), raw);
    let mut legacy = first.clone();
    legacy.version = 1;
    assert!(legacy.encode().is_err());
    let mut b = builder(&root);
    b.add_cell(
        key(),
        &fixture(),
        &BTreeMap::from([("astrometry".into(), raw)]),
    )
    .unwrap();
    let repeat = b.finish(Some(&first), BTreeMap::new()).unwrap();
    assert_eq!(repeat.encode().unwrap(), first.encode().unwrap());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reader_counts_stored_io_when_a_v2_catalog_blob_is_compressed() {
    let root = root();
    let mut b = builder(&root);
    b.add_cell(key(), &fixture(), &BTreeMap::new()).unwrap();
    let mut manifest = b.finish(None, BTreeMap::new()).unwrap();
    let original = read_blob(&root, &manifest.cells[0].catalog).unwrap();
    let compressed = builder(&root).store_sidecar(&original).unwrap();
    assert!(compressed.zlib_raw_bytes.is_some());
    assert!(compressed.bytes < original.len() as u64);
    manifest.cells[0].catalog = compressed.clone();
    fs::write(root.join("manifest.json"), manifest.encode().unwrap()).unwrap();
    let mut reader = crate::reader::Reader::open(&root, 1).unwrap();
    assert_eq!(
        reader.cell(key()).unwrap().encode().unwrap(),
        fixture().encode().unwrap()
    );
    assert_eq!(reader.stats().catalog_blob_reads, 1);
    assert_eq!(reader.stats().catalog_blob_bytes, compressed.bytes);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn compressed_sidecars_reject_incomplete_trailing_bombs_and_false_lengths() {
    let root = root();
    let raw = b"source uncertainty and all original columns\n".repeat(100);
    let mut b = builder(&root);
    b.add_cell(
        key(),
        &fixture(),
        &BTreeMap::from([("astrometry".into(), raw.clone())]),
    )
    .unwrap();
    let first = b.finish(None, BTreeMap::new()).unwrap();
    let blob = &first.cells[0].sidecars["astrometry"];
    for n in [
        0,
        raw.len() as u64 - 1,
        raw.len() as u64 + 1,
        MAX_BLOB_BYTES as u64 + 1,
    ] {
        let mut bad = blob.clone();
        bad.zlib_raw_bytes = Some(n);
        assert!(read_blob(&root, &bad).is_err());
    }
    let stored = fs::read(blob_path(&root, blob).unwrap()).unwrap();
    let b = builder(&root);
    for bad in [
        stored[..stored.len() - 1].to_vec(),
        [stored.clone(), vec![0]].concat(),
        [stored.clone(), stored].concat(),
    ] {
        // Bind the altered bytes so this exercises strict stream validation,
        // beyond the ordinary encoded SHA256 corruption check.
        let descriptor = b.store_encoded(&bad, Some(raw.len() as u64)).unwrap();
        assert!(read_blob(&root, &descriptor).is_err());
    }
    let before = fs::read(root.join("manifest.json")).unwrap();
    fs::write(blob_path(&root, blob).unwrap(), b"corrupt").unwrap();
    assert!(read_blob(&root, blob).is_err());
    let mut b = builder(&root);
    assert!(
        b.add_cell(
            key(),
            &fixture(),
            &BTreeMap::from([("astrometry".into(), raw)])
        )
        .is_err()
    );
    assert_eq!(before, fs::read(root.join("manifest.json")).unwrap());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn legacy_raw_packs_still_open_and_unknown_versions_fail() {
    let root = root();
    let mut b = builder(&root);
    b.add_cell(key(), &fixture(), &BTreeMap::new()).unwrap();
    let mut manifest = b.finish(None, BTreeMap::new()).unwrap();
    manifest.version = 1;
    fs::write(root.join("manifest.json"), manifest.encode().unwrap()).unwrap();
    let mut reader = crate::reader::Reader::open(&root, 4096).unwrap();
    assert_eq!(
        reader.cell(key()).unwrap().encode().unwrap(),
        fixture().encode().unwrap()
    );
    manifest.version = 3;
    assert!(manifest.encode().is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn nearby_manifest_bound_is_versioned_and_incremental_ids_still_reject_collisions() {
    let root = root();
    let mut b = builder(&root);
    b.add_cell(key(), &fixture(), &BTreeMap::new()).unwrap();
    assert!(
        b.add_cell(CellKey { shard: 1, ..key() }, &fixture(), &BTreeMap::new())
            .is_err()
    );
    let mut source = fixture().datasets()[0].clone();
    source.id = fixture().records()[0].id().clone();
    let collision = Catalog::new("fixture-v1".into(), vec![source], vec![]).unwrap();
    assert!(
        b.add_cell(CellKey { shard: 1, ..key() }, &collision, &BTreeMap::new())
            .is_err()
    );
    let mut m = b.finish(None, BTreeMap::new()).unwrap();
    m.version = 1;
    let mut padded = m.encode().unwrap();
    padded.resize(LEGACY_MANIFEST_BYTES + 1, b' ');
    assert!(Manifest::decode(&padded).is_err());
    m.version = 2;
    let mut padded = m.encode().unwrap();
    padded.resize(LEGACY_MANIFEST_BYTES + 1, b' ');
    assert_eq!(Manifest::decode(&padded).unwrap().version, 2);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn frozen_gzip_csv_uses_production_import_and_rejects_bombs_truncation_and_trailing_members() {
    use std::io::Write;
    let root = root();
    fs::create_dir_all(&root).unwrap();
    let original = include_bytes!("../../../../catalogs/sources/fixtures/gaia-synthetic.csv");
    let gzip = |raw: &[u8]| {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(6));
        encoder.write_all(raw).unwrap();
        encoder.finish().unwrap()
    };
    let good = gzip(original);
    let mut sources = crate::sources::SourceManifest::decode(
        include_bytes!("../../../../catalogs/sources/manifest-v1.json"),
        false,
    )
    .unwrap();
    sources.sources.retain(|s| s.id.as_str() == "gaia:dr3");
    let file = "gaia.csv.gz";
    let request = serde_json::json!({"version":1,"revision":"gzip-nearby-v1","source_manifest":"sources.json","cells":[{"key":key(),"observed":{"gaia":{"source":"gaia:dr3","artifact":file}}}]});
    fs::write(
        root.join("build.json"),
        serde_json::to_vec(&request).unwrap(),
    )
    .unwrap();
    let mut publish_source = |bytes: &[u8]| {
        sources.sources[0].artifacts = vec![crate::sources::Artifact {
            path: file.into(),
            sha256: digest(bytes),
            bytes: bytes.len() as u64,
            origin: crate::sources::Origin::ObservedSnapshot,
            encoding: crate::sources::Encoding::CsvGzip,
        }];
        fs::write(root.join(file), bytes).unwrap();
        fs::write(
            root.join("sources.json"),
            serde_json::to_vec(&sources).unwrap(),
        )
        .unwrap();
    };
    publish_source(&good);
    let output = root.join("pack");
    let first = pipeline::build(&root.join("build.json"), &output).unwrap();
    let before = first.encode().unwrap();
    let mut reader = crate::reader::Reader::open(&output, 4096).unwrap();
    assert!(
        reader
            .record(&CatalogId::from_source("gaia_dr3", "1000000000000000001").unwrap())
            .unwrap()
            .is_some()
    );
    for bad in [
        good[..good.len() - 1].to_vec(),
        [good.clone(), good].concat(),
        gzip(&vec![b'x'; 8 * 1024 * 1024 + 1]),
    ] {
        publish_source(&bad);
        assert!(pipeline::build(&root.join("build.json"), &output).is_err());
        assert_eq!(fs::read(output.join("manifest.json")).unwrap(), before);
    }
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn corruption_and_failed_update_preserve_publication() {
    let root = root();
    let mut b = builder(&root);
    b.add_cell(key(), &fixture(), &BTreeMap::new()).unwrap();
    let m = b.finish(None, BTreeMap::new()).unwrap();
    let before = fs::read(root.join("manifest.json")).unwrap();
    let mut b = builder(&root);
    b.add_cell(key(), &fixture(), &BTreeMap::new()).unwrap();
    assert!(b.add_cell(key(), &fixture(), &BTreeMap::new()).is_err());
    assert_eq!(before, fs::read(root.join("manifest.json")).unwrap());
    fs::write(blob_path(&root, &m.cells[0].catalog).unwrap(), b"corrupt").unwrap();
    assert!(read_blob(&root, &m.cells[0].catalog).is_err());
    assert!(
        builder(&root)
            .add_cell(key(), &fixture(), &BTreeMap::new())
            .is_err()
    );
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn silent_identity_churn_and_malicious_manifest_reject() {
    let root = root();
    let mut b = builder(&root);
    b.add_cell(key(), &fixture(), &BTreeMap::new()).unwrap();
    let mut old = b.finish(None, BTreeMap::new()).unwrap();
    old.cells[0]
        .record_ids
        .push(CatalogId::from_source("zzz", "removed").unwrap());
    let mut b = builder(&root);
    b.add_cell(key(), &fixture(), &BTreeMap::new()).unwrap();
    assert!(b.finish(Some(&old), BTreeMap::new()).is_err());
    let mut value = serde_json::to_value(&old).unwrap();
    value["source_manifest"]["sha256"] = "../escape".into();
    assert!(Manifest::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn offline_pipeline_accepts_observed_input_and_preserves_old_pack_on_source_failure() {
    let root = root();
    fs::create_dir_all(root.join("sources")).unwrap();
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalogs/sources");
    let bytes = fs::read(source_root.join("manifest-v1.json")).unwrap();
    let manifest = crate::sources::SourceManifest::decode(&bytes, false).unwrap();
    fs::write(root.join("sources/manifest.json"), &bytes).unwrap();
    for source in manifest.sources {
        for artifact in source.artifacts {
            let dest = root.join("sources").join(&artifact.path);
            fs::create_dir_all(dest.parent().unwrap()).unwrap();
            fs::copy(source_root.join(&artifact.path), dest).unwrap();
        }
    }
    let catalog =
        Catalog::decode(include_bytes!("../../fixtures/storage-observed-v1.json")).unwrap();
    let bundle = serde_json::json!({"catalog":catalog,"ephemerides":{"raw_response":include_str!("../../../../catalogs/sources/fixtures/jpl-earth-j2000.json")},"astrometry":{"missing_velocity":null}});
    fs::write(
        root.join("observed.json"),
        serde_json::to_vec(&bundle).unwrap(),
    )
    .unwrap();
    let request = serde_json::json!({"version":1,"revision":"private-observed-v1","source_manifest":"sources/manifest.json","cells":[{"key":key(),"input":"observed.json"}]});
    let request_file = root.join("build.json");
    fs::write(&request_file, serde_json::to_vec(&request).unwrap()).unwrap();
    let output = root.join("output");
    let first = pipeline::build(&request_file, &output).unwrap();
    let before = fs::read(output.join("manifest.json")).unwrap();
    assert_eq!(first.revision, "private-observed-v1");
    assert!(first.cells[0].sidecars.contains_key("ephemerides"));
    let stored = read_blob(&output, &first.cells[0].sidecars["ephemerides"]).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&stored).unwrap(),
        bundle["ephemerides"]
    );
    pipeline::build(&request_file, &output).unwrap();
    assert_eq!(before, fs::read(output.join("manifest.json")).unwrap());
    let observed = serde_json::json!({"gaia":{"source":"gaia:dr3","artifact":"fixtures/gaia-synthetic.csv"},"exoplanets":{"source":"nasa:exoplanet-ps","artifact":"fixtures/exoplanet-proxima.csv"},"horizons":[{"source":"jpl:horizons-earth-j2000","artifact":"fixtures/jpl-earth-j2000.json","system":"system:solar","parent":"system:solar","body_kind":"planet"}]});
    let raw_request = serde_json::json!({"version":1,"revision":"private-raw-v1","source_manifest":"sources/manifest.json","cells":[{"key":key(),"observed":observed}]});
    let raw_request_file = root.join("raw-build.json");
    fs::write(&raw_request_file, serde_json::to_vec(&raw_request).unwrap()).unwrap();
    let raw = pipeline::build(&raw_request_file, &root.join("raw-output")).unwrap();
    for sidecar in ["gaia_observations", "planet_observations", "ephemerides"] {
        assert!(raw.cells[0].sidecars.contains_key(sidecar));
    }
    let mut reader = partitions::Reader::open(std::io::Cursor::new(
        read_blob(&root.join("raw-output"), &raw.cells[0].catalog).unwrap(),
    ))
    .unwrap();
    assert!(
        reader
            .read_cell(key())
            .unwrap()
            .get(&CatalogId::from_source("jpl", "399").unwrap())
            .is_some()
    );
    let raw_catalog = serde_json::to_string(&bundle["catalog"]).unwrap();
    fs::write(
        root.join("observed.json"),
        format!("{{\"catalog\":{raw_catalog},\"catalog\":{raw_catalog}}}"),
    )
    .unwrap();
    assert!(
        pipeline::build(&request_file, &output)
            .unwrap_err()
            .to_string()
            .contains("duplicate producer field")
    );
    let nested = raw_catalog.replacen(
        "\"schema_version\":1",
        "\"schema_version\":1,\"schema_version\":1",
        1,
    );
    fs::write(
        root.join("observed.json"),
        format!("{{\"catalog\":{nested}}}"),
    )
    .unwrap();
    assert!(pipeline::build(&request_file, &output).is_err());
    fs::write(
        root.join("observed.json"),
        serde_json::to_vec(&bundle).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("sources/fixtures/gaia-synthetic.csv"),
        b"tampered",
    )
    .unwrap();
    assert!(pipeline::build(&request_file, &output).is_err());
    assert_eq!(before, fs::read(output.join("manifest.json")).unwrap());
    let mut bad = request;
    bad["source_manifest"] = "../escape".into();
    fs::write(&request_file, serde_json::to_vec(&bad).unwrap()).unwrap();
    assert!(pipeline::build(&request_file, &output).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn complete_selected_real_release_build_is_repeatable_and_rejects_wrong_target() {
    let snapshot =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalogs/sources/observed-526-v1");
    let output = root();
    let first = pipeline::build(&snapshot.join("build.json"), &output).unwrap();
    assert!(!first.production_eligible);
    assert_eq!(first.cells.len(), 1);
    let before = fs::read(output.join("manifest.json")).unwrap();
    let mut reader = partitions::Reader::open(std::io::Cursor::new(
        read_blob(&output, &first.cells[0].catalog).unwrap(),
    ))
    .unwrap();
    let catalog = reader.read_cell(key()).unwrap();
    let id = |s: &str| CatalogId::try_from(s.to_owned()).unwrap();
    assert!(matches!(
        catalog.get(&id("jpl:10")),
        Some(crate::Record::Star(_))
    ));
    assert!(
        matches!(catalog.get(&id("jpl:399")), Some(crate::Record::Body(b)) if b.parent == id("jpl:10"))
    );
    assert!(
        matches!(catalog.get(&id("jpl:301")), Some(crate::Record::Body(b)) if b.parent == id("jpl:399"))
    );
    let observations: serde_json::Value = serde_json::from_slice(
        &read_blob(&output, &first.cells[0].sidecars["planet_observations"]).unwrap(),
    )
    .unwrap();
    assert_eq!(observations.as_array().unwrap().len(), 6451);
    let ephemerides: serde_json::Value = serde_json::from_slice(
        &read_blob(&output, &first.cells[0].sidecars["ephemerides"]).unwrap(),
    )
    .unwrap();
    assert_eq!(ephemerides.as_array().unwrap().len(), 18);
    pipeline::build(&snapshot.join("build.json"), &output).unwrap();
    assert_eq!(before, fs::read(output.join("manifest.json")).unwrap());
    let cases = [
        serde_json::json!({"source":"jpl:horizons-399-j2000","artifact":"jpl-399.json","system":"system:solar","parent":"system:solar","object_kind":"planet","target":"jpl:299"}),
        serde_json::json!({"source":"jpl:horizons-10-j2000","artifact":"jpl-10.json","system":"system:solar","parent":"jpl:399","object_kind":"star","target":"jpl:10"}),
    ];
    let source_bytes = fs::read(snapshot.join("manifest.json")).unwrap();
    for case in cases {
        let input = serde_json::from_value(serde_json::json!({"horizons":[case]})).unwrap();
        assert!(
            super::observed::import(
                input,
                &snapshot.join("manifest.json"),
                &source_bytes,
                "observed-526-v1"
            )
            .is_err()
        );
    }
    assert_eq!(before, fs::read(output.join("manifest.json")).unwrap());
    fs::remove_dir_all(output).unwrap();
}
