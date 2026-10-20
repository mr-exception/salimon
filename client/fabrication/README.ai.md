# Fabrication maintenance guide

Read the active issue, [architecture](architecture.md), [invariants](invariants.md)
and [approved gameplay contracts](../../docs/spaceship-building-and-survival.md#accepted-fabricator-contracts-2026-10-19-336).

This portable crate owns recipes, per-material storage, committed reservation,
FIFO progress, one output tray and typed consumable slots. Only depend on std and world resource values.
Runtime owns physical fragment removal, Core funding, inventory transfer/input
and ship mass journals. Never add renderer/native handles, IO, health, survival,
research acquisition, upgrade effects or persistence here without a new issue.

Source routing: `recipes.rs` validates definitions, `mass.rs` owns integer units,
`machine.rs` owns commands/state/progress and snapshots; `inventory.rs` owns
category slots/first-item consumption; `lib.rs` is the facade.
Tests belong beside private invariants or in `tests/fabricator.rs` for public
transactions. Keep errors atomic and keep snapshots observational.

Run `cargo test -p salimon-fabrication --locked`, formatting and Clippy; use the
root [validation matrix](../../docs/validation.md) for cross-crate checks.
Maintain the root task report and affected guides with behavior changes.

#370 adds the exact `ConsumableSlot`/`item`/`consume_slot` facade already used by
#377's wheel draft. Prefer exact identity selection for new consumers. The runtime
survival adapter owns effects and transaction preparation; keep this crate free
of health/hunger and input/focus policy.

#379 adds the explicit `Fabricator::starter_with_supplies` constructor, deriving
initial outputs/reserves from starter recipes and reserving their identities.
It is never a restore repair or recurring inventory refill. See the
[canonical supply contract](../../docs/save-contract.md#starting-supplies-379).

#378 extends the existing catalogs with four food definitions and captured food
modifiers; see the owning architecture for provisional tuning and regressions.
Keep advanced food recipes locked and preserve exact-slot consumption; no new
progression acquisition or native Q-wheel input is introduced.
