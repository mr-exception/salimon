use super::*;
use std::io::Cursor;
fn fixture() -> Catalog {
    Catalog::decode(include_bytes!("../../fixtures/schema-v1.json")).unwrap()
}
fn key(shard: u32) -> CellKey {
    CellKey {
        galactic: [12, -3, 0],
        shard,
    }
}
fn pack() -> Vec<u8> {
    encode(vec![(key(0), fixture())]).unwrap()
}
fn rewrite_index(bytes: &[u8], change: impl FnOnce(&mut serde_json::Value)) -> Vec<u8> {
    let len = u32_at(bytes, 12) as usize;
    let mut value = serde_json::from_slice(&bytes[32..32 + len]).unwrap();
    change(&mut value);
    let index = serde_json::to_vec(&value).unwrap();
    let mut out = bytes[..32].to_vec();
    out[12..16].copy_from_slice(&(index.len() as u32).to_le_bytes());
    out[16..24]
        .copy_from_slice(&((32 + index.len() + bytes.len() - 32 - len) as u64).to_le_bytes());
    out[24..28].copy_from_slice(&crc32fast::hash(&index).to_le_bytes());
    out.extend(index);
    out.extend_from_slice(&bytes[32 + len..]);
    out
}
#[test]
fn exact_roundtrip_sources_lookup_and_determinism() {
    let bytes = pack();
    assert_eq!(bytes, pack());
    assert_eq!(&bytes[..12], b"SALUCELL\x01\0\0\0");
    let mut reader = Reader::open(Cursor::new(bytes)).unwrap();
    assert_eq!(reader.revision(), "fixture-v1");
    assert_eq!(reader.sources(), fixture().datasets());
    assert_eq!(
        reader.cell_for_record(&CatalogId::from_source("fixture", "star").unwrap()),
        Some(key(0))
    );
    assert_eq!(reader.read_cell(key(0)).unwrap(), fixture());
    assert!(reader.read_cell(key(1)).is_err());
}
#[test]
fn lossless_f64_missing_data_and_provenance() {
    let c = fixture();
    let mut records = c.records().to_vec();
    if let crate::Record::Star(star) = &mut records[0] {
        star.mass_kg.as_mut().unwrap().value = f64::from_bits(0x4350_0000_0000_0001);
        star.radius_meters = None;
    }
    let c = Catalog::new(c.revision().into(), c.datasets().to_vec(), records).unwrap();
    let mut reader = Reader::open(Cursor::new(encode(vec![(key(0), c.clone())]).unwrap())).unwrap();
    assert_eq!(reader.read_cell(key(0)).unwrap(), c);
}
#[test]
fn independent_cells_do_not_read_corrupt_neighbors() {
    let a = fixture();
    let mut records = a.records().to_vec();
    for record in &mut records {
        match record {
            crate::Record::System(r) => {
                r.header.id = CatalogId::from_source("fixture", "system-b").unwrap()
            }
            crate::Record::Star(r) => {
                r.header.id = CatalogId::from_source("fixture", "star-b").unwrap();
                r.system = CatalogId::from_source("fixture", "system-b").unwrap();
            }
            _ => unreachable!(),
        }
    }
    let b = Catalog::new(a.revision().into(), a.datasets().to_vec(), records).unwrap();
    let bytes = encode(vec![(key(1), b), (key(0), a.clone())]).unwrap();
    let mut bad = bytes.clone();
    *bad.last_mut().unwrap() ^= 1;
    let mut reader = Reader::open(Cursor::new(bad)).unwrap();
    assert_eq!(reader.read_cell(key(0)).unwrap(), a);
    assert!(reader.read_cell(key(1)).is_err());
    assert_eq!(Reader::open(Cursor::new(bytes)).unwrap().cells().len(), 2);
}
#[test]
fn reject_bad_headers_truncation_crc_and_trailing_bytes() {
    let original = pack();
    for at in [0, 8, 12, 16, 24, 28, 35] {
        let mut bytes = original.clone();
        bytes[at] ^= 1;
        assert!(Reader::open(Cursor::new(bytes)).is_err(), "byte {at}");
    }
    for len in [0, 31, 32, original.len() - 1] {
        assert!(Reader::open(Cursor::new(&original[..len])).is_err());
    }
    let mut extra = original;
    extra.push(0);
    assert!(Reader::open(Cursor::new(extra)).is_err());
}
#[test]
fn reject_rechecksummed_hostile_index() {
    let bytes = pack();
    for (field, value) in [
        ("offset", u64::MAX),
        ("compressed_bytes", u32::MAX as u64),
        ("raw_bytes", (MAX_SCHEMA_JSON_BYTES + 1) as u64),
        ("raw_bytes", 0),
    ] {
        let bad = rewrite_index(&bytes, |v| v["cells"][0][field] = value.into());
        assert!(Reader::open(Cursor::new(bad)).is_err(), "{field} {value}");
    }
    let bad = rewrite_index(&bytes, |v| {
        let c = v["cells"][0].clone();
        v["cells"].as_array_mut().unwrap().push(c);
    });
    assert!(Reader::open(Cursor::new(bad)).is_err());
    let bad = rewrite_index(&bytes, |v| {
        v["cells"][0]["record_ids"] = serde_json::json!(["fixture:system", "fixture:star"])
    });
    assert!(Reader::open(Cursor::new(bad)).is_err());
    let bad = rewrite_index(&bytes, |v| v["unexpected"] = true.into());
    assert!(Reader::open(Cursor::new(bad)).is_err());
}
#[test]
fn reject_payload_index_mismatch_and_expansion_past_limit() {
    for change in ["revision", "ids", "length", "source"] {
        let bad = rewrite_index(&pack(), |v| match change {
            "revision" => v["revision"] = "different".into(),
            "ids" => v["cells"][0]["record_ids"] = serde_json::json!(["fixture:other"]),
            "source" => v["sources"][0]["version"] = "different".into(),
            _ => v["cells"][0]["raw_bytes"] = 1.into(),
        });
        let mut reader = Reader::open(Cursor::new(bad)).unwrap();
        assert!(reader.read_cell(key(0)).is_err(), "{change}");
    }
}
#[test]
fn reject_duplicate_conflicting_or_incomplete_cells() {
    let c = fixture();
    assert!(encode(vec![]).is_err());
    assert!(encode(vec![(key(0), c.clone()), (key(0), c.clone())]).is_err());
    assert!(encode(vec![(key(0), c.clone()), (key(1), c)]).is_err());
}

#[test]
fn independent_python_zlib_v1_fixture_remains_readable() {
    let bytes = include_bytes!("../../fixtures/partition-v1.bin");
    let mut reader = Reader::open(Cursor::new(bytes)).unwrap();
    assert_eq!(reader.read_cell(key(0)).unwrap(), fixture());
}

#[test]
fn rechecksummed_extra_stream_bytes_are_rejected() {
    let mut bytes = pack();
    let len = u32_at(&bytes, 12) as usize;
    let mut compressed = bytes[32 + len..].to_vec();
    compressed.push(0);
    bytes.push(0);
    let total = bytes.len() as u64;
    bytes[16..24].copy_from_slice(&total.to_le_bytes());
    let bad = rewrite_index(&bytes, |v| {
        v["cells"][0]["compressed_bytes"] = (compressed.len() as u32).into();
        v["cells"][0]["crc32"] = crc32fast::hash(&compressed).into();
    });
    let mut reader = Reader::open(Cursor::new(bad)).unwrap();
    assert!(reader.read_cell(key(0)).is_err());
}

#[test]
fn rechecksummed_truncated_zlib_trailer_is_rejected() {
    let mut bytes = pack();
    let len = u32_at(&bytes, 12) as usize;
    bytes.pop();
    let total = bytes.len() as u64;
    bytes[16..24].copy_from_slice(&total.to_le_bytes());
    let compressed = bytes[32 + len..].to_vec();
    let bad = rewrite_index(&bytes, |v| {
        v["cells"][0]["compressed_bytes"] = (compressed.len() as u32).into();
        v["cells"][0]["crc32"] = crc32fast::hash(&compressed).into();
    });
    let mut reader = Reader::open(Cursor::new(bad)).unwrap();
    assert!(reader.read_cell(key(0)).is_err());
}
