# Portable save contracts and durable checkpoints

See [canonical save contract](../../docs/save-contract.md). The envelope owns
schema/version/clock validation and bounded encoding; domains own payload rules.
`storage.rs` owns portable two-generation verification/recovery and the native
filesystem adapter. Publication cannot destroy the newest verified generation.
CRC32 includes generation/length/payload. Unknown schema rejects before overwrite.
Native directories have an exclusive crash-safe OS lease; stale temporary files
are ignored. Runtime owns scheduling and session capture, never this crate.
No window/GPU state or offline simulation is saved. Run
`cargo test -p salimon-persistence --locked` (fault injection, corruption, schema,
size, atomic publication, filesystem restart and lease checks).


#371 composes schema-2 survival source, explicit v1 migration, verified launch-only
fatigue credit, terminal freeze and archive/restart publication. See the canonical
[save/lifecycle contract](../../docs/save-contract.md). Restart must publish an
archive and both fresh fallback generations before exposing new gameplay; failures
remain failures. Maintain lifecycle, offline rollback, publication-failure and
restart fault-injection tests with changes to this boundary.


#372 advances captures to schema 3 with explicit v1/v2 migration in runtime.
Terminal publication verifies both fallback slots before archiving. Report versions
use separate immutable namespaces (legacy `journeys/`, current `journeys/v2/`);
allocator high-water scanning covers both. Runtime supplies the version and opaque
bytes; storage does not interpret gameplay JSON. See the canonical
[journey contract](../../docs/save-contract.md#journey-statistics-372).

#380 advances captures to schema 4 for per-run tutorial progress/skip. Runtime
explicitly migrates versions 1–3 dismissed; missing schema-4 source rejects.

#493 advances captures to schema 5 for explicit engine on/off and the fictional
impulse coefficient. Runtime migrates v1-v4 force records without a simulation
tick; see the current canonical save contract.


#467 advances captures to schema 6, requiring versioned local Archive identities,
read progress and selection. Runtime initializes old v1–v5 Archives unread;
current payload omissions/unknown identities reject transactionally. See
[Archive save contract](../../docs/save-contract.md#scout-archive-schema-6-467).

## Universe new-run envelope (#406)

`universe.rs` owns the independent version-1 future universe save contract, using
`salimon-universe` IDs/frames/epochs. It does not replace the compressed-world runtime envelope (now schema 7).
Catalog/definition incompatibility is fatal during recovery; corruption alone can
fall back. Domain delta validators are mandatory. See the canonical
[universe save contract](../../docs/save-contract.md#future-universe-new-run-contract--u05-406)
and `src/universe/tests.rs` for wire, identity and interrupted-publication coverage.

## Runtime schema 7 (#468)

New compressed-world captures require runtime-owned Recovery config/module/paid
progress; explicit v1-v6 migration initializes empty hardware/work and preserves
prior recipe sources while adding the blueprint. Current universe envelope is
independent and unchanged. See [save contract](../../docs/save-contract.md#recovery-module-schema-7-468).
