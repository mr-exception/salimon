use super::*;
use crate::storage::{CheckpointStore, SaveManager};
use salimon_universe::TimeScale;
use std::{cell::RefCell, rc::Rc};
fn id(key: &str) -> CatalogId {
    CatalogId::from_source("test", key).unwrap()
}
fn fixture() -> (Catalog, CatalogBinding, UniverseSave) {
    let catalog =
        Catalog::decode(include_bytes!("../../../universe/fixtures/schema-v1.json")).unwrap();
    let binding = CatalogBinding {
        revision: catalog.revision().into(),
        schema_version: CATALOG_SCHEMA,
        pack_sha256: "a".repeat(64),
    };
    let epoch = Epoch::calendar([2026, 10, 21], [0; 4], TimeScale::Tt).unwrap();
    let save = UniverseSave {
        format: FORMAT.into(),
        schema_version: VERSION,
        catalog: binding.clone(),
        epoch,
        ship: ShipState {
            position: Position::Galaxy {
                position: GalacticPosition::new([1_000_000; 3], [0.125, -0.25, 0.5]).unwrap(),
            },
            velocity_mps: [1.0, 2.0, 3.0],
            orientation_xyzw: [0.0, 0.0, 0.0, 1.0],
        },
        definitions: vec![],
        discovery_history: vec![Discovery {
            object: catalog.records()[0].id().clone(),
            first_seen: epoch,
            last_seen: epoch,
            observation_count: 1,
        }],
        player_deltas: vec![DeltaSection {
            owner: id("inventory"),
            schema_version: 1,
            payload: b"[]".to_vec(),
        }],
    };
    (catalog, binding, save)
}
fn delta(d: &DeltaSection) -> Result<(), SaveError> {
    if d.owner != id("inventory") || d.schema_version != 1 {
        return Err(SaveError::UnsupportedVersion(d.schema_version));
    }
    if d.payload != b"[]" {
        return Err(invalid("invalid inventory delta"));
    }
    Ok(())
}
#[test]
fn universe_wire_roundtrips_without_catalog_data_or_lossy_galactic_meters() {
    let (c, b, s) = fixture();
    let bytes = s.encode(&c, &b, delta).unwrap();
    assert_eq!(UniverseSave::decode(&bytes, &c, &b, delta).unwrap(), s);
    let text = String::from_utf8(bytes).unwrap();
    assert!(!text.contains("datasets"));
    assert!(text.contains("1000000"));
    assert!(text.contains("0.125"));
}
#[test]
fn unknown_schemas_catalog_digest_and_domain_delta_are_never_accepted() {
    let (c, b, s) = fixture();
    let mut wrong = s.clone();
    wrong.catalog.pack_sha256 = "b".repeat(64);
    assert!(matches!(
        wrong.encode(&c, &b, delta),
        Err(SaveError::IncompatibleCatalog(_))
    ));
    wrong = s.clone();
    wrong.schema_version = 2;
    assert_eq!(
        wrong.encode(&c, &b, delta).unwrap_err(),
        SaveError::UnsupportedVersion(2)
    );
    wrong = s.clone();
    wrong.player_deltas[0].schema_version = 2;
    assert_eq!(
        wrong.encode(&c, &b, delta).unwrap_err(),
        SaveError::UnsupportedVersion(2)
    );
    let mut value = serde_json::to_value(&s).unwrap();
    value["unknown"] = true.into();
    assert!(UniverseSave::decode(&serde_json::to_vec(&value).unwrap(), &c, &b, delta).is_err());
    assert!(UniverseSave::decode(b"{\"schema_version\":2}", &c, &b, delta).is_err());
    assert!(UniverseSave::decode(&vec![b' '; MAX_SAVE_BYTES + 1], &c, &b, delta).is_err());
}
#[test]
fn invalid_pose_history_definition_and_duplicate_delta_reject() {
    let (c, b, s) = fixture();
    let mutate: Vec<fn(&mut UniverseSave)> = vec![
        |s| s.ship.velocity_mps[0] = f64::INFINITY,
        |s| s.ship.orientation_xyzw = [0.0; 4],
        |s| {
            s.ship.position = Position::Relative {
                frame: FrameId::Local(id("not-persisted")),
                meters: [0.0; 3],
            }
        },
        |s| s.discovery_history[0].object = id("unknown"),
        |s| s.discovery_history[0].last_seen = s.epoch.advance_nanos(1).unwrap(),
        |s| s.discovery_history.push(s.discovery_history[0].clone()),
        |s| {
            s.definitions.push(DefinitionBinding {
                body: id("wrong"),
                version: 1,
                sha256: "a".repeat(64),
            })
        },
        |s| s.player_deltas.push(s.player_deltas[0].clone()),
    ];
    for m in mutate {
        let mut broken = s.clone();
        m(&mut broken);
        assert!(broken.encode(&c, &b, delta).is_err());
    }
}
#[derive(Default)]
struct Disk {
    slots: [Option<Vec<u8>>; 2],
    torn: bool,
}
#[derive(Clone)]
struct Store(Rc<RefCell<Disk>>);
impl CheckpointStore for Store {
    fn read(&self, slot: usize) -> Result<Option<Vec<u8>>, SaveError> {
        Ok(self.0.borrow().slots[slot].clone())
    }
    fn publish(&mut self, slot: usize, bytes: &[u8]) -> Result<(), SaveError> {
        let mut d = self.0.borrow_mut();
        d.slots[slot] = Some(if d.torn {
            bytes[..12].to_vec()
        } else {
            bytes.to_vec()
        });
        if d.torn {
            Err(invalid("injected interruption"))
        } else {
            Ok(())
        }
    }
}
#[test]
fn interrupted_universe_publication_falls_back_to_verified_generation() {
    let (c, b, s) = fixture();
    let bytes = s.encode(&c, &b, delta).unwrap();
    let disk = Rc::new(RefCell::new(Disk::default()));
    let mut manager = SaveManager::new(Store(disk.clone()));
    let validate = |v: &[u8]| UniverseSave::decode(v, &c, &b, delta).map(|_| ());
    manager.checkpoint(&bytes, validate).unwrap();
    disk.borrow_mut().torn = true;
    assert!(manager.checkpoint(&bytes, validate).is_err());
    let mut restart = SaveManager::new(Store(disk.clone()));
    let recovered = restart.recover(validate).unwrap();
    assert!(recovered.recovered);
    assert_eq!(recovered.bytes.unwrap(), bytes);
    let mut wrong = b.clone();
    wrong.pack_sha256 = "b".repeat(64);
    assert!(matches!(
        restart.recover(|v| UniverseSave::decode(v, &c, &wrong, delta).map(|_| ())),
        Err(SaveError::IncompatibleCatalog(_))
    ));
}
