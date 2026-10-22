# Universe binary partition prototype — U04 (#405)

The portable `salimon-universe::partitions` module encodes immutable,
dependency-closed U01 catalogs into independently compressed cells. It is a
bounded prototype, not a production universe builder or a playable-world reader.
U12 #413 owns full-pack construction/publication and U13 #414 owns bounded
runtime caching/query integration. No runtime, scanner or save interpretation changes.

## Wire version 1

All header integers are unsigned little-endian. The 32-byte header is:

| Byte | Value |
| --- | --- |
| 0 | Eight ASCII bytes `SALUCELL` |
| 8 | u32 container version = 1 |
| 12 | u32 UTF-8 JSON index byte length |
| 16 | u64 exact total file byte length |
| 24 | u32 IEEE CRC32 of index bytes |
| 28 | u32 reserved = 0 |

The index is a strict typed JSON object with `revision`, sorted unique `sources`
(U01 Dataset ID/version/reference), and sorted unique `cells`. Each cell contains
`key` (`galactic`: three i64 U02 2^40-meter coordinates; `shard`: u32), `offset`
(u64 relative to payload start), `compressed_bytes`, `raw_bytes`, `crc32` (u32),
and sorted unique `record_ids`. Cell ranges must be contiguous from zero, cover
exactly the payload and agree with the physical file length. Unknown fields,
invalid IDs, source collisions and unsorted/duplicate keys or IDs reject.
Placement/shard selection is supplied by an offline science-aware builder; the
codec does not infer stellar positions from IDs or physical properties.

Each payload is one zlib/DEFLATE stream (writer level 6, pinned pure Rust flate2
backend) containing canonical U01 schema-1 catalog JSON. Cell decoding checks CRC,
limits decompression to declared raw length plus one, checks exact consumed
compressed length, validates the U01 schema and compares revision, IDs and source
metadata with the index. CRCs detect corruption; they are **not authenticity or
cryptographic publication digests**. U06 frozen SHA256 inputs and U12 trusted
manifest hashes remain separate. Unread cells are not certified by opening an index.

`Reader<R: Read + Seek>` opens only the header/index and seeks directly to a
requested cell. `cell_for_record` uses an immutable ID-to-cell index. `sources`,
`cells` and loaded catalog metadata expose shared references only. Scientific
provenance, source IDs, versions, uncertainties and original keys stay with every
record; source identity/revision conflicts reject. A source reference is not reuse
approval. [The production source gate](universe-sources.md) remains #513.

## Precision, bounds and evolution

| Field | Encoding / precision contract |
| --- | --- |
| Cell coordinates / shard / counts / seeds | Exact integer JSON values; no f64 intermediate |
| Catalog mass/radius, uncertainty, validity | Round-trip f64 with Serde float_roundtrip; no quantization, lossless finite U01 values |
| IDs, references, provenance, missing fields | Exact typed U01 values, missing stays missing |
| Future astrometry, ephemerides, full definitions | Not fields of U01; U07/U08/U10 must version their contracts explicitly; no implied position precision from a storage cell key |

Index <=16 MiB, <=4096 cells, raw cell <=16 MiB (U01 bound), whole prototype
<=256 MiB. Arithmetic/bounds are checked before payload allocation. The encoder
accepts an in-memory vector and buffers compressed payloads; it is deliberately
not the GB streaming builder. Reader memory includes the bounded index plus one
compressed/raw cell and validated catalog. Every cell is independently
referentially complete and all record IDs are globally unique: cross-cell parent
references and splitting one system across cells are not supported in v1.
`shard` distinguishes independent groups in a dense spatial cell, not an automatic
split of a system. Large systems/production workloads require deliberate U12/U13
format work. Unknown container or U01 schema versions fail, with no silent migration.

## Reproduce measurements

```sh
cargo test -p salimon-universe --locked
cargo run -p salimon-universe --example partition_bench --release --locked
```

The [observed development fixture](../client/universe/fixtures/storage-observed-v1.json)
uses Earth-399 mass (5.97219e24 kg, +/-0.0006e24 kg) and mean radius (6371.01 km,
+/-0.02 km), from the frozen JPL header, plus Proxima Cen b/host identities from the
frozen NASA PS row. Sources/digests are retained; system grouping is explicitly
fixture fiction. Proxima mass/radius are missing, not guessed. Orbital parameters
remain raw-source science work. This is a small real-record codec sample, not
validated ingestion, full scientific coverage or production reuse approval.

The workload reports total bytes including index/source map, bytes/record,
encode/index-open/reverse-cell-read times and conditional 5/10 GB capacities.
1 cell is the actual four-record fixture; 32/512 cells remap identities and repeat
fields synthetically to expose overhead sensitivity. They do not simulate Gaia
or establish a full-pack estimate. [Report and raw results](../reports/issue-405/README.md)
retain the budget exception: 2–5 GB remains the goal and <=10 GB the initial target;
there is insufficient population/schema/compression evidence to approve either
or a larger budget. U12 must measure representative full inputs and escalate if
10 GB proves impractical. No arbitrary FPS, native M1 or storage target pass.
