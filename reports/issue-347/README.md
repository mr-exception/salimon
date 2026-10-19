# Issue #347 — portable Fabricator composition and scenarios

Implemented 2026-10-19. Production inspection now exposes ordered resource bins,
FIFO jobs/progress, stable tray/item identities and the existing 3-food/2-medical
inventory. The scenario DSL's `crafting` action dispatches the native modal's
validated actions; closed-modal requests cannot enqueue or remotely open a
machine. No free-resource/inventory mutations are exposed by the protocol.

## Coverage

- New portable scenario mines spherical biomass, medicinal compounds and water,
  carries and inserts their real identities through the intake adapter, enqueues
  mixed quantities, verifies immediate deduction/nonrefundable cancellation,
  resumes after Core cutoff, blocks behind each tray item, collects once and
  verifies inventory, presentation and ship-mass transitions.
- The declarative `scenarios/fabricator-contract.json` replays through the portable
  production protocol and checks empty slots/queue and closed-modal admission.
- Existing domain/session regressions retain insufficient-resource/overflow/full
  bin rejection, full receiver, no duplicate collection, blocked tray during
  blackout, flight/takeoff funding and mass-ledger coverage.
- M/N dispatch is retired by #346. Selection/consumption via Q and its edge/focus
  integration is supplied by the following #377 stack layer, not duplicated here.
- The existing cached Ubuntu PR job now runs portable workspace unit/session E2E
  tests after its build. Graphical suites and platform/release matrices remain
  periodic; no M1 performance claim is made.

## Validation

Linux x86_64, Rust/Cargo 1.99.1 stable.

- `cargo test -p salimon-client --locked fabrication_scenarios`: 2 passed.
- `cargo test --workspace --locked`: 568 passed, 0 failed, 22 existing opt-in ignored.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo build --workspace --locked`: passed.
- `python -m unittest discover -s scripts -p 'test_*.py'`: 93 passed.
- Initial new test failures (recipe fixture name and JSON integer/float comparison)
  were corrected; the results above are from the corrected revision.
- Native graphical E2E/OS-input/M1 visual and performance: not run, no display or
  Vulkan ICD here. Reference signoff remains #348. Portable tests validate
  deterministic composition, not rendered appearance or hardware performance.
