# U04 — binary partitions, codecs and indexes (#405)

Delivered a bounded portable storage prototype on 2026-10-22, based on main
`8ada131`. [Contract](../../docs/universe-storage.md),
[architecture](../../client/universe/architecture.md#binary-partitions--u04-405),
[invariants](../../client/universe/invariants.md).

The versioned 32-byte header protects a strict immutable source/record/cell index.
Dependency-closed U01 cells have independent zlib compression and CRC32 checks;
Read/Seek lookup reads only requested payloads. All U01 numerical values,
missing values, original keys and provenance remain lossless. Unsupported schemas,
bad ranges/order, corrupt bytes, expansion past bounds, extra/truncated streams and
payload/index disagreements reject. CRC32 is not publication authentication.
A separately Python/zlib-encoded v1 fixture checks cross-implementation compatibility.

## Measurements and budget exception

[Raw Linux release measurements](partition-bench.json) use the checked-in observed
Earth/Proxima development fixture. Earth mass/radius and their published header
uncertainties come from frozen Horizons; Proxima identities come from frozen NASA
PS. Grouping is fixture fiction, absent Proxima quantities stay absent and source
hash/reuse references survive. This is no scientific importer or reuse approval.

| Workload | Raw U01 bytes | Container bytes | Bytes/record including index |
| --- | --- | --- | --- |
| Four real/structural records, one cell | 2,174 | 1,647 | 411.75 |
| 32 synthetically replicated cells / 128 records | 71,792 | 33,150 | 258.98 |
| 512 synthetically replicated cells / 2,048 records | 1,153,168 | 524,263 | 255.99 |

The 512-cell diagnostic encoded in 18.91 ms, opened its index in 0.50 ms and read
all cells in reverse in 6.46 ms on this shared Linux x86_64 executor. These are
single diagnostic runs, not performance guarantees. At that **sample density**,
5/10 decimal GB holds about 19.5/39.1 million records; it is not a population or
full-pack forecast. Larger counts would exceed those targets under that assumption.
The 2–5 GB goal and initial <=10 GB target remain unproven; no larger budget is
approved. U12 #413 must measure full representative inputs/schema/compression and
escalate if 10 GB is impractical. No native M1/FPS evidence is claimed.

## Validation

Environment: Linux x86_64, rustc 1.99.1 (2026-10-13), Python 3.12.14; cached native
build prerequisites via `/workspace/.salimon-tools/env.sh`.

- `cargo test -p salimon-universe --locked`: 25 tests pass (including the final
  rechecksummed truncated-stream regression).
- `cargo run -p salimon-universe --example partition_bench --release --locked`:
  pass; raw output above.
- `cargo build --workspace --locked`: pass.
- `cargo test --workspace --locked`: 805 pass, 0 fail, 28 existing opt-in checks ignored
  (native/GPU/performance checks retain their opt-in prerequisites).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: pass;
  final partition-only Clippy repeated after the last regression: pass.
- `cargo fmt --all -- --check`, `git diff --check`: pass.
- `python3 scripts/check_docs.py`: 0 failures. Source-impact check against
  `origin/main` after commit: pass (0 failures).

## Scope and documentation mapping

`partitions.rs` / `partitions/tests.rs` own container behavior; `universe-storage.md`
is the wire/precision/bounds contract. Crate README/AI/architecture/invariants,
root architecture/maintenance routing, validation and approved-design status are
updated together. Existing U01 JSON bytes/semantics remain unchanged. Added pinned
pure Rust flate2/CRC32 dependencies already present in the workspace lock graph.

The 256 MiB in-memory prototype, 16 MiB cell/index and 4096-cell bounds are explicit.
No cross-cell parents, dense-system splitting, full-definition content ingestion,
GB streaming construction, runtime caching, world replacement or pack publication.
Those retain U07/U08/U10/U12/U13 ownership; production source use retains #513.
No native/GPU launch is needed for this isolated portable storage change.
