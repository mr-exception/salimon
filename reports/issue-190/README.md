# Issue #190 — Safe assists, crash recovery and inertial takeoff

Implemented against merged #202 collision/crash states and #203 atomic energy
reservations, stacked on #194's Core status display branch.

- Validated starter eligibility: inclusive 1.15R, total planet-relative speed
  ≤20 m/s including tangential motion, powered cockpit authority, no heading gate.
  `AssistConfig` permits explicit range/speed tuning; active paths capture tuning.
- Crash-grounded L reserves 1.2 GJ at default forward power before a 12-second
  lift/reposition/alignment/approach/touchdown sequence. Outage/insufficient energy
  leaves the ship grounded. Settling cannot trigger assistance. The sequence
  continues uncancelled through bus outage and cockpit exit.
- Recovery uses the existing bounded authored-footprint site search (25 m grid,
  ±500 m per tangent axis, at most 1681 candidates). Relocation occurs above the
  body's maximum terrain plus conservative ship-radius clearance and margin;
  rotation begins only after lift. No supported site rejects before energy debit.
- Landing completes with zero velocity/throttle. Takeoff clearance uses an
  accelerating quadratic path and hands its analytic terminal vector velocity
  into inertial Flying with throttle zero. Updates split exactly at completion
  and advance any remaining interval normally. Door interlocks are preserved.
- Updated existing specs/architecture/invariants and deterministic E2E baseline
  expectations. Landing upgrades remain documentation only.

Validation (Linux x86_64, Rust 1.99.0, Python 3.12):

- `cargo fmt --all -- --check` — passed.
- `cargo test --workspace --locked` — passed; existing opt-in GPU/bench tests ignored.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `cargo build --workspace --locked` — passed.
- `python -m unittest discover -s scripts -p 'test_*.py'` — 55 passed.
- Domain tests cover all solids/normals, exact speed/range thresholds, tangential
  rejection, zero/insufficient energy, no double reservation, uncancellability,
  partitioned recovery and inertial continuation after exact/overshot completion.
- Runtime CPU regressions exercise the real crash/L route and representative
  crater relocation with authored foot/ramp support. Orbit/landed baseline JSON
  actions/assertions/waits also execute through production automation without GPU.

Graphical smoke and Apple M1 visual/performance signoff were not run here; the
host has no configured display/Vulkan ICD. Hardware work stays in native issues.
The existing kinematic assist tuning still traverses large catalog clearance
ranges in a short fixed duration; this is not a fuel/work-based physical autopilot.
Ordinary inertial f64 coordinate updates retain distant-anchor quantization;
partition tests allow a measured bounded 0.1 m after up to 625 coasting additions
at the 1e12 m anchor, while assist endpoints remain within existing 1 mm bounds.
