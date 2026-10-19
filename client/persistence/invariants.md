# Portable save contracts

See [canonical inventory and restore contract](../../docs/save-contract.md).
The envelope owns schema/version/clock validation and encoding, not domain rules,
filesystem IO, scheduling, window/GPU state or offline advancement. Decode checks
version before domain payloads. Errors never publish partially restored state.
Run `cargo test -p salimon-persistence --locked`.
