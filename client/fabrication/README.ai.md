# Fabrication maintenance guide

Read the active issue, [architecture](architecture.md), [invariants](invariants.md)
and [approved gameplay contracts](../../docs/spaceship-building-and-survival.md#accepted-fabricator-contracts-2026-10-19-336).

This portable crate owns recipes, per-material storage, committed reservation,
FIFO progress and one output tray. Only depend on std and world resource values.
Runtime owns physical fragment removal, Core funding, inventory transfer/input
and ship mass journals. Never add renderer/native handles, IO, health, survival,
research acquisition, upgrade effects or persistence here without a new issue.

Source routing: `recipes.rs` validates definitions, `mass.rs` owns integer units,
`machine.rs` owns commands/state/progress and snapshots; `lib.rs` is the facade.
Tests belong beside private invariants or in `tests/fabricator.rs` for public
transactions. Keep errors atomic and keep snapshots observational.

Run `cargo test -p salimon-fabrication --locked`, formatting and Clippy; use the
root [validation matrix](../../docs/validation.md) for cross-crate checks.
Maintain the root task report and affected guides with behavior changes.
