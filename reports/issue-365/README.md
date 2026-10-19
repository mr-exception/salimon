# Issue 365 — complete Fabricator/session round-trip

Implemented `FabricationSaveState` and concrete `SessionState` inside the
canonical SaveGame envelope. Captures recipe definitions/IDs/unlocks, complete
ordered reserves, committed FIFO jobs/counts/progress, next IDs, tray, exact
three food/two medical slots, five equipment slots/selection, all physical state,
gameplay seed/camera and exact elapsed/admitted-backlog clocks.

Restore validates everything into temporary owners, rebuilds source-derived
mass and Core recipe demand, then publishes once. It never steps crafting,
charges power, refunds committed ingredients or collects output. Invalid restore
leaves the live session unchanged. Output identities cannot occupy both tray and
inventory. Held controls/modal/feedback/history/query caches reset; native
focus/capture and automation policy remain host-owned. Propulsion's cached mass
is omitted from wire state. Current valid interactions are recomputed spatially.

Validation on Linux x86_64, Rust 1.99.1:

- `cargo test --workspace --locked`: **576 passed, 0 failed, 22 existing opt-in
  ignored** (includes compile-fail documentation tests).
- `cargo build --workspace --locked`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check`, `git diff --check`: passed.
- Dedicated scenarios cover mining/carrying/intake → mixed FIFO crafting → save
  mid-craft → load → finish → save occupied tray → collect exactly once → reload
  both inventory categories → food/medicine consumption. Byte-exact normalized
  re-snapshots, exact clock/energy/mass continuation and empty-equipment selection
  are checked. Blackout freezes paid progress across restore, power restart
  resumes, malformed progress and tray/inventory duplicate identity reject
  atomically, and held movement/crafting modal never replay.
- The #364 physical scenarios remain passing, including moving flight/EVA,
  500-fragment continuation, sleeping contacts, takeoff and streamed depletion.

No graphical/OS-input/M1 checks were run: this is the portable state boundary,
not graphical/native signoff. Python/model suites were not rerun because no
Python, asset or model processing changed. Existing ignored tests remain opt-in
GPU/profile/hardware gates. Durable files, checkpoint scheduling/fallback and
native startup/exit/manual-save integration remain #366. Survival/effects are
not invented here; their later owner must add required versioned typed state.

See [canonical contract and inventory](../../docs/save-contract.md). The existing
#377 wheel PR must be rebased onto this stack and its consume tests adapted to
its exact-slot API; wheel input/modal state must remain excluded from saves.
