# #230 — portable spherical-mining contracts

Added a validated `MiningConfig`, typed attached/released material snapshots and
release reasons, and portable deterministic bounded lifecycle. Defaults: 10 kg,
2 kg/s, one 16 ms tick, no energy. Target switch/release precedence, aim loss,
interruption, cap auto-release/continued hold and final depletion preserve mass.
Radius derives from existing effective density with geometric-only 5 mm–2 m
clamps. Gameplay/resource/architecture docs record authoritative behavior and
separate the old 2 kg fixture APIs from the accepted new tool lifecycle.

## Validation

Linux, Rust stable 1.99.0, no new dependencies.

- `cargo test -p salimon-world --locked`: passed all world unit/integration/doc
  tests, including three new lifecycle regressions.
- `cargo clippy -p salimon-world --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all`: applied; formatting check passes.

Regression tests prove 23.5 kg => two capped releases and a final 3.5 kg piece,
conservation after every tick, tiny positive depletion, zero time, input-up
precedence, target switching, aim loss/reacquisition, idempotent interruption,
invalid config, configurable caps, density-derived radius for all resources and
bounded huge-duration work.

## Sequencing and limits

This PR supplies portable contracts. #231 owns physical session IDs and runtime
input/attachment wiring; later mining-sphere issues own visual/contact migration.
Existing generation/storage, legacy fixture extraction, one-object carry and
Core feeding stay intact. No disk save/backend or native M1 testing is introduced.
