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
