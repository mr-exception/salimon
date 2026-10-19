# Issue 364 — physical world snapshot and restore

Implemented typed opaque character/ship save state, sparse mining-world DTOs,
stable fragment allocator/carry/depletion restoration, runtime ship-local anchors
and motion, and physics contact/sleep/dwell/impulse restoration. Core fractional
charge, conversion, steering ramp, assists, doors and bounce paths retain their
source state. Rebuilds fragment indexes, mass ledger, terrain query bindings and
all presentation/query/solver geometry caches without a gameplay tick. Attached
mining releases its already existing physical entity; held input is not restored.

Validation (Linux x86_64): `cargo test -p salimon-client --locked persistence`
passed all 5 dedicated regressions: landed/moving/EVA/carry/blackout/terrain;
500 fragments with subsequent exact motion and mass parity; sleeping contacts;
mid-takeoff continued charge/pose parity; sparse depletion after streaming,
consumed identity high-water marks and malformed/dangling-state rejection.
`cargo clippy --workspace --all-targets --locked -- -D warnings` passed;
`cargo fmt --all -- --check` passed. An initial fixture used an already populated
scenario and failed its population assertion; the corrected fixture starts empty.

Full Fabricator/session atomic publication follows in #365; disk IO/checkpoints
remain #366. The portable checks do not establish graphical/OS-input or M1
performance; native graphical checks were not run because no rendering/input
behavior changes here. See [canonical inventory](../../docs/save-contract.md).
