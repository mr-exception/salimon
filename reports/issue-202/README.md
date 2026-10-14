# Issue #202 — swept celestial contact and crash grounding

Free-flight movement sweeps all catalog solids in body-local f64 coordinates and
selects the earliest contact; the Sun remains visual-only. Flat bodies use stable
analytic line/sphere intervals. Version-1 terrain uses its conservative extrema
as broad bounds and height-gradient bounds for chronological interval refinement,
including hills between clear endpoints. Resolution is 1 mm, with 4,096 uncertain
intervals per body; exhaustion conservatively contacts the earliest unresolved
span rather than allowing tunneling. This may produce early contact in a
pathological unresolved sweep.

Contact preserves the existing conservative 16 m ship proxy and sampled
local-normal clearance. Selected throttle/held input and steering clear. Normal
impact velocity rebounds with restitution 0.35, capped at 8 m/s; tangential motion
is absorbed. A captured local settling response returns to the anchor, reducing
each subsequent launch speed. At most four rebounds occur before stable
`GroundedAfterCrash` at rest. This does not add free-flight planetary gravity,
damage, exact triangle-level hull collision, or full cabin inertial physics.

`Bouncing` and `GroundedAfterCrash` are distinct from `Landed`; ordinary L assists
and throttle changes are rejected. Contextual messages and runtime surface-frame,
velocity/telemetry and automation mapping preserve these states. Powered recovery
landing is separate follow-up work. Native Apple M1 smoke is issue #198.

## Validation

Linux x86_64; Rust/Cargo 1.99.0.

- Passed: `cargo test -p salimon-ship --locked` (39 tests).
- Passed: `cargo test --workspace --locked` (all tests/doctests, including 121
  runtime tests and the crash-state command/mapping regression; default opt-in
  benchmark/GPU ignores remain).
- Passed: `cargo test --workspace --locked controller::collision` after refining
  contact-time velocity interpolation (six collision regressions).
- Passed: `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- Passed: `cargo build --workspace --locked`.
- Passed: `cargo fmt --all -- --check`, `git diff --check`.
- Passed: `/usr/bin/python3 -m unittest discover -s scripts -p 'test_*.py'`
  (all 55 tests).

Regressions cover high-speed full crossings of every solid body, low-speed
contact, rebound partitioning and finite settling, representative terrain
clearance, hill tunneling with clear endpoints, clear passes, visual Sun exclusion
and inability to trigger ordinary assists after crash.

This final incremental PR uses #201/#186 via the requested integration stack;
merge the predecessor PRs first and retarget it to main before merging. Local
GUI launch was not run (missing desktop packages/package-manager lock). Native
M1 signoff remains outside this cloud issue and is not claimed from CPU tests.
