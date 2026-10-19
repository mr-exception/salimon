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
