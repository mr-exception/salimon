# Issue #123 — Focused ship modules

## Outcome

Refactored `salimon-ship` into cohesive internal modules while retaining the
crate-root public API, dependencies, gameplay rules and all 21 existing ship
regressions. No consumer changes or new crates/frameworks were needed.

## Ownership

- `src/lib.rs`: public reexports and existing constants (39 lines).
- `state.rs`: public pose, axes, flight/door/steering and snapshot contracts.
- `controller/mod.rs`: private authoritative fields, constructors, update order
  and snapshots; default-pose regression.
- `controller/flight.rs`: direct speed/motion/velocity, thruster authority and
  collision correction; associated regressions.
- `controller/steering.rs`: gated steering input, ramps and rotations; regressions.
- `controller/assist.rs`: captured paths, landing range, assist timing/derivatives
  and takeoff interlocks; all-body, partition, authority and timing regressions.
- `controller/door.rs`: landed/open-space airlock permissions and hinge animation;
  threshold, locked interaction and animation regressions.
- `controller/cockpit.rs`: cockpit authority and typed/contextual messages.
- `controller/telemetry.rs`: bounded Core fixture and nearby-body observations;
  telemetry regressions.
- `orientation.rs`: domain-local normalization and quaternion policy;
  composition/handedness, axes and shortest-arc regressions.

Update order remains door animation, 100 ms-clamped steering, then direct flight
or full-delta assist dispatch. Extraction preserves arithmetic and function/test
bodies; `advance` now delegates its original door/motion blocks. Public DTO
fields/variants and regression names were compared against the base source.
README.ai, architecture, invariants, maintenance map, shared math inventory and
coding conventions now route to the new owners. Stale airlock invariants were
corrected to describe the already-implemented open-space policy.

## Validation

Environment: Linux x86_64; Rust 1.99.0 stable (b940084d7, 2026-09-28).
Base revision: `fc35db22d9ba9f68aa417eaf5f67430600b913e6`.

| Check | Result |
| --- | --- |
| `cargo test -p salimon-ship --locked` | Passed: all 21 existing regressions |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | Passed: 260 tests; doc tests also passed |
| `cargo build --workspace --locked` | Passed |
| `git diff --check` | Passed |
| Affected documentation relative-link existence check | Passed |
| Public DTO and moved-function/test source comparison | Passed; dispatch delegation is the deliberate structural change |

## Limitations

Native graphical scenarios, platform builds on macOS/Windows, release staging
and model/Python tooling suites were not run locally. This changes portable Rust
source organization and documentation only; assets, rendering, inputs, tooling
and gameplay behavior are unchanged. The PR's normal cross-platform workflow
provides additional build and graphical coverage. No gameplay blocker remains.
