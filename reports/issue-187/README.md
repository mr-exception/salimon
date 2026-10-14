# Issue #187 — Physical ship mass

Implemented an event-driven ship mass ledger with a 100,000 kg dry hull and
positive validated object masses. Stable fragment/equipment identity namespaces
prevent double counting; installed/stored/carried are handling states of one
physical identity. Current fixtures define no separate installed equipment masses.

World journals creation/growth, pickup, carried/loose pose mutation and removal.
Runtime drains ordered, coalesced events after physical synchronization, classifies
object centers against authored cabin/nose geometry and updates propulsion mass
without changing velocity or force capacities. Core consumption can call the new
physical removal API; consumed/despawned IDs are never reused. Scene reconciliation
is explicit diagnostic work rather than a per-tick cargo scan.

Regressions cover 500 objects, equipment, repeated handling, invalid masses,
growth/removal/ID allocation, gate/nose/vertical containment, moving ship anchors,
carried boundary exit and re-entry, consumption cleanup and force/mass acceleration.

## Validation

Linux x86_64; Rust 1.99.0.

- PASS: `cargo test -p salimon-ship -p salimon-world -p salimon-character -p salimon-client --locked` (289 tests, one pre-existing ignored profiling test).
- PASS: `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- PASS: `cargo fmt --all -- --check` and `git diff --check`.
- Workspace build and additional unchanged-owner tests are validated on the combined stack in #188.
- Native GPU/M1 smoke: not run; hardware signoff is explicitly outside this issue.

An early compilation overlapped ongoing source edits and saw a stale ship API;
the final frozen-source run above passed.

## Limits

No native Apple M1 signoff (separate native issues). No full interior inertial
physics, container UI or Core intake feature is introduced. Existing physical
pose/frame adapters provide movement events; future equipment/storage owners
must upsert/remove their physical identities using the same ledger.
