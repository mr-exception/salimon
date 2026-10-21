# Issue #380 — Persistent survival onboarding

Implements portable monotonic objectives and per-run skip, production success
hooks, compact contextual guidance and native H/T input. Full saves use schema 4;
old runs migrate dismissed. Early consumption/crafting/collection counts without
regranting resources. Restart creates fresh objectives. Discrete checkpoints
include tutorial progress and skip.

Validation: Linux x86_64, Rust 1.99.1. Commands/results are recorded below after
execution. Native M1/DPI/physical walkthrough screenshots are unrun in this cloud
session; existing [#376](https://github.com/mr-exception/salimon/issues/376)
explicitly owns tutorial presentation/input signoff. Source closure does not
claim that native acceptance.

- `cargo test --workspace --locked`: PASS (including five tutorial regressions,
  production monitor interaction, migration, restart and all existing suites).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: PASS.
- `cargo build --workspace --locked`: PASS.
- `cargo fmt --all -- --check` and `git diff --check`: PASS.

The first prompt-size check found a 726px line; shortened guidance now fits the
608px content width of the 640px minimum drawable at scale 2. No screenshot or
M1/native appearance acceptance is inferred from CPU raster tests.
