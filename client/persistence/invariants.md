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
