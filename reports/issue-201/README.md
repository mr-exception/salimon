# Issue #201 — persistent inertial flight

The ship owns world-space f64 velocity independent of orientation. A 100,000 kg
starter hull has 4 MN forward / 2 MN reverse capacity. The validated `Propulsion`
value caches total mass and both capacities for event-driven E01 integration;
flight never scans cargo. Signed force/mass acceleration is implemented, with
signed player selection supplied by the next #186 PR.

Motion integrates constant acceleration analytically; zero throttle coasts,
steering changes future force direction, and there is no drag, cap or ship
planetary gravity. Snapshot velocity and scalar speed reflect actual motion.
Captured assists retain their existing paths and finish at rest.

Explicit finite initial velocity is available through `flying_with_velocity`.
EVA and loaded-cargo high-speed fixtures seed inertial drift independently of
throttle, preserving their existing coverage. Regression tests cover coasting
turns, reverse force, mass changes, elapsed-time partitions and high velocities.

## Validation

Linux x86_64, Rust/Cargo 1.99.0. Final command results follow below.

Depends on #185 / PR #204 through the requested integration stack. Endpoint
collision correction remains until #202; this PR does not claim swept collision.
Energy consumption/cargo aggregation remains separate E01 work. Graphical launch
is unavailable in this container (missing desktop packages/package-manager lock).
Apple M1 verification is excluded from this cloud issue.

- Passed: `cargo test -p salimon-ship --locked` (27 tests).
- Passed: `cargo test --workspace --locked` after updating obsolete direct-speed
  monitor/cargo fixtures; all tests/doctests passed (default opt-in ignores remain).
- Passed: `cargo test -p salimon-client --locked coasting_fixtures` (covers the
  subsequently added coasting-orbit fixture alongside both EVA setups).
- Passed: `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- Passed: `cargo fmt --all -- --check`, `git diff --check`.
- Passed: `/usr/bin/python3 -m unittest discover -s scripts -p test_baseline_scenarios.py`
  (baseline/evidence parity and scenario declarations).
- Full Python tooling suite: 52/55 passed on the final local attempt; three
  unchanged subprocess/readiness/capture tests exceeded 100/200 ms deadlines in
  this container. The same suite passed on the #185 idle run. These timing
  failures are recorded, not counted as passing graphical coverage.
