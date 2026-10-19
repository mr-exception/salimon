# Issue 341 — Fabricator mass ledger

Two dedicated ledger identities track raw milligram reserves and physical tray
output. The session keeps the machine private and exposes a command seam that
synchronizes successful enqueue/cancel/collect/insert operations immediately.
Completion and carried removal also synchronize before returning to gameplay.
Zero removes the corresponding ledger entry, preserving exact final dry mass.
Queued/committed inputs and collected consumables deliberately add no mass;
cancellation never refunds materials. Other loose cargo/equipment/Core accounting
remains unchanged. Inventory validation before collection belongs to #345.

Validation: `cargo test -p salimon-client --locked session::fabrication` passed
7 regressions including command/tick transitions, last-unit removal, quantization,
overflow disposal, full reserve preservation, carried identity exclusion and
ship movement. `cargo fmt --all` / `git diff --check` passed. Full workspace
validation is recorded in the final stack report. Linux x86_64, Rust 1.99.1.
Native reference verification remains #348.
