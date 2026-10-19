# Issue 363 — canonical save contracts

Implemented schema-1 generic typed SaveGame envelope, bounded JSON encoding with
exact float round-trip, explicit unsupported-version/error handling, exact
simulation/backlog clocks and deterministic duplicate-rejecting stable-ID maps.
The exhaustive [state inventory](../../docs/save-contract.md) assigns owners,
cache regeneration, safe restore order and future typed extension registration.
There is no older schema to migrate. Domain adapters are #364/#365; filesystem
checkpoints are #366, survival state is not implemented by this foundation.

Validation (Linux x86_64): `cargo test -p salimon-persistence --offline` passed
(1 test); `cargo clippy -p salimon-persistence --all-targets --locked --offline --
-D warnings` passed; `cargo fmt --all` applied. No GPU/native launch required for
this platform-independent contract. Full composition gates follow in the stack.
