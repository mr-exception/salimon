# Issue #338 — Portable Fabricator production domain

Introduces `salimon-fabrication`, a workspace library depending only on std and
world raw-material contracts. Validated data records define stable recipe IDs,
ingredients, duration, watts, finished mass/category and unlock prerequisites.
Default-unlocked ration/medkit recipes use the proposed quantities from #336;
advanced categories/unlocks are schema only. No native runtime dependency added.

Private authoritative state owns per-material 20 kg bins, integer-milligram
balances, atomic full-batch ingredient debits, monotonic job IDs, bounded FIFO
jobs, one active operation and one output tray. Cancelling never refunds inputs;
finished tray output survives cancellation. Whole-fragment insertion receipts
explicitly report discarded overflow, even when the bin accepts zero mass.
Kg quantization is once to nearest mg, half upward; unrepresentable and
sub-half-mg fragments fail unchanged at the future adapter boundary.

Power demand exposes watts and the next completion boundary. Explicit funded
Duration steps pause without power and stop at the first tray output; no blocked
or unpowered time is banked. Actual worked time/joules and unused elapsed time
are returned for Core composition. Domain mass contributions separately report
stored resources and tray output; jobs, committed inputs and collected inventory
are intentionally excluded. Crate maintenance/architecture/invariants and root
scope/validation/source navigation are aligned.

## Validation

Linux x86_64; rustc 1.99.1, Cargo 1.99.1:

- `cargo test -p salimon-fabrication --locked --target-dir artifacts/domain-target`:
  initial 21 public contract tests passed. Final workspace run also covers the
  additional private identity-exhaustion regression (22 Fabricator tests total).
- `cargo test --workspace --locked`: **543 passed, 0 failed, 22 ignored**.
  Ignored tests are existing opt-in benchmarks/candidate-processing checks.
- `cargo build --workspace --locked`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check`, `git diff --check`: passed.
- Relative targets in changed Markdown/crate guides: all exist.

Tests cover each material's capacity/overflow, input quantization and failures,
exact repeated insertion/batch ledgers, enqueue atomicity/overflow, capacity and
job-ID exhaustion, FIFO batches, waiting progress, cancellation/tray preservation,
large dt, nanosecond and partition-equivalent progression, partial power funding,
blackout, one-time collection, invalid records, locked schema, typed command parity,
read-only snapshots and every deliberate domain mass transition.

## Integration boundary

This PR delivers the portable domain, not playable fabrication. Physical fragment
removal after an accepted insertion, receiving-slot checks before collection,
Core energy debit and ship ledger synchronization remain the explicit runtime
adapter responsibilities in #339–#347. No physical intake/UI/inventory/HUD,
health/hunger effects, save/load or progression gameplay is added here.
Native visual/reference-machine checks were not run; #348 owns that signoff.
No unresolved portable acceptance criterion remains.
