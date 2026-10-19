# Issue 345 — Typed consumable inventory and consumption

Completed the portable receiver foundation introduced for #344's atomic tray
transaction. `ConsumableInventory` defines exactly three food and two medical
one-item slots with named canonical counts. Matching collection fills the first
vacancy; full/unsupported/duplicate resident inputs preserve state. Consumption
removes the first occupied matching slot without shifting, stacking, selecting or
mixing categories. Read-only slots retain stable item identities for future HUD
and versioned persistence adapters; no save/load or disk integration exists.

Gameplay M consumes medical, N consumes food. Portable press/release latches
consume once per edge, independent per category. Native policy rejects repeats,
synthetic events, unfocused/uncaptured/occluded presses; modal UI captures these
keys and lifecycle/view clearing resets latches. N retains camera behavior only
in precision tour. Empty categories show feedback. Gear, world carrying, flight,
ship mass and vitals are unaffected. There is no health/hunger/full-vitals gate;
items are intentionally consumed without gameplay benefit in this phase.
Automation uses the same route through `consume_food` and `consume_medkit` keys.
Dedicated visible consumable HUD remains the separate #346 task.

Validation on Linux x86_64, Rust 1.99.1:

- `cargo test --workspace --locked`: **566 passed, 0 failed, 22 existing opt-in
  tests ignored**. Includes all prior UI/tray/queue/mass/power/geometry regressions.
- `cargo test -p salimon-fabrication --locked`: passed, including slot capacity,
  category validation, atomic rejection, vacancy ordering, refill/first consumption,
  independent categories and depletion tests.
- `cargo test -p salimon-client --locked consumable`: 2 focused tests passed;
  production M/N routing, repeated edges, no camera interference, unchanged gear,
  carried identity/ship state, empty/focus/modal/tour gates.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo build --workspace --locked`: passed.
- `cargo fmt --all -- --check`, `git diff --check`: passed.

Native visual/input signoff was not run (no display/Vulkan driver). Actual HUD,
OS playthrough and reference-machine performance remain #346/#347/#348, with
reference native signoff explicitly tracked by [#348](https://github.com/mr-exception/salimon/issues/348).
Portable tests do not assert native GPU fidelity or full accessibility support.
