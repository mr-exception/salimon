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
