# Issue #337 — Consumable raw resources

Adds real world identities organic biomass and medicinal compounds, with positive
finite densities, Earth distribution and ordinary mining/fragment/session behavior.
A separate abundance layer preserves the old minerals' exact generated results.
Runtime reuses authored low-poly deposit meshes with green/purple tints; mined
fragments use the existing shared sphere and mass/density-derived sizing. Core
intake supports both with explicit configurable catalog yields. The proposed
values are tuning, not scientific chemistry or owner-approved balancing.

Validation on Linux x86_64, rustc 1.99.1:

- `cargo test -p salimon-world -p salimon-ship --locked --target-dir artifacts/domain-target`:
  passed (139 tests; 3 pre-existing opt-in benchmarks ignored).
- `cargo test -p salimon-client --locked`: 165 passed, 5 pre-existing opt-in
  benchmarks ignored. Includes distinct deterministic visuals and depleted hiding.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check` and `git diff --check`: passed.

Regression coverage compares old mineral generation with/without new feedstocks,
then exercises deterministic feedstock generation, complete spherical depletion,
physical mass/identity, pickup, in-session restore and one-time removal. Expanded
Core tests cover both new resources over tiny through 10 kg masses.
No generated assets were modified. Native appearance/reference hardware was not
run; Fabricator native visual/playability signoff remains tracked by #348.
Runtime crafting/inventory/energy integration is intentionally later work.
