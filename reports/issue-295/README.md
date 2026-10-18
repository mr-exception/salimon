# Issue #295 — local takeoff and stationary powered handoff

2026-10-18. Incremental on #308 / PR #322. Linux x86_64, Rust 1.99.1.
Native Apple M1 execution is **not run**, and remains in #312.

## Root cause and reproduction

The old second stage targeted `body radius + 0.15R + 16 m` in four seconds,
using a quadratic path that ended at its highest speed. That analytic velocity
was copied to Flying, with throttle zero. Captured assistance intentionally
ignores manual thrust, and zero-throttle flight conserves this excessive
momentum. Force insufficiency and cargo mass were not the cause.

The same observational test was inserted into the pre-fix assist owner from
`a6e0b6db` (PR #322 source tree; equivalent local commit `9c5d9fa`) for the [before trace](takeoff-before.txt), then
run after the fix for the [after trace](takeoff-after.txt). It exercises production
landing/takeoff controllers on all five solid bodies, at dry mass and 110,000 kg
(1,000 × 10 kg fragments), before activation, each second of assistance, handoff,
and one second afterward. It records actual pose, altitude, vector velocity,
throttle, mass and stored energy. A +100 throttle request during assistance is
rejected in both traces, as required.

| Body | Before handoff speed (m/s) | After handoff speed (m/s) |
| --- | ---: | ---: |
| Mercury | 179,999.446 | 0 |
| Venus | 412,499.446 | 0 |
| Earth | 449,999.446 | 0 |
| Moon | 119,999.446 | 0 |
| Mars | 262,499.446 | 0 |

## Chosen behavior and budgets

- Keep six-second uncancellable assistance and landed orientation. Rise 10 m in
  two seconds, then another 40 m in four seconds, using eased radial segments.
  Local clearance is independent of landing eligibility range and planet radius.
- Stage peak speeds are 7.5 and 15 m/s; each stage's acceleration magnitude is
  bounded by 15 m/s². Position and velocity join continuously at rest at two and
  six seconds. At completion the ship has risen 50 m from its captured origin
  (52.107544 m nominal altitude in these flat fixtures), velocity/throttle zero.
- Free flight remains stationary with zero throttle; powered manual thrust is
  available immediately at handoff. A 1% command in the maintained landed-Earth
  route now proves acceleration after handoff. A landing prompt may remain
  available because the ship remains inside the unchanged landing-eligibility
  shell. The cockpit says `Assisted takeoff - flight controls locked` while captured.
- Keep 4 MN forward / 2 MN reverse, 100/50 MW, and Core 60 GJ capacity / 45 GJ
  initial charge. Dry acceleration is 40/20 m/s²; 110,000 kg is 36.36/18.18;
  200,000 kg is 20/10. Force increases are not justified by this scripted-path bug.
- Keep the accepted conservative kinematic energy model: reserve 600 MJ up
  front, plus 110 kW ordinary idle load (600.66 MJ over six seconds with ample
  bus charge). Existing reservation rejection, blackout completion, refuel and
  conversion behavior are unchanged. The assist is not a mass-dependent work
  solver; force/mass governs subsequent manual flight.

The deliberate invariant change replaces clearing the planetary landing volume
with clearing the local surface. Existing canonical ship/gameplay guides and
both baseline/evidence scenarios reflect that change; native requirements stay
separate.

## Validation

- **Passed:** `cargo test -p salimon-ship --locked` — 66 passed, one opt-in
  trace ignored. Tests cover all solid bodies, arbitrary normals, partitioned
  and large deltas, continuous derivative/handoff, uncancellable controls,
  100,000/110,000/1,000,000 kg assist cases, energy conservation/blackout,
  and dry/loaded powered acceleration and reverse braking after handoff.
- **Passed:** `cargo test -p salimon-ship --locked takeoff_trace -- --ignored --nocapture`
  before and after, with the documented pre-fix owner swap for the before trace.
- **Passed:** `cargo test --workspace --locked` — 501 passed; 19 opt-in tests ignored; no failures. This includes replay of landed-Earth,
  crash/recovery, cargo, blackout/refuel, cockpit telemetry, walking and EVA routes,
  plus final cooperative terrain/cache regressions from the preceding PRs.
- **Passed:** `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo build --workspace --locked`, `cargo fmt --all -- --check`, `git diff --check`.
- **Passed:** `python -m unittest discover -s scripts -p 'test_*.py'` — 92 tests;
  maintained baseline/evidence parity and visual corpus workflow remain valid.
- **Resolved initial failures:** legacy tests assumed the old terminal speed,
  old 11.25 m/s lift telemetry, or stale evidence-copy assertions. Replaced
  implementation-mirroring speed expectations with local speed/acceleration/
  handoff contracts; aligned telemetry and paired evidence scenarios.
- **Not run:** native Metal/Apple M1 input, flight/EVA visual smoke and hardware
  performance. [Native #312](https://github.com/mr-exception/salimon/issues/312)
  remains pending and is not closed by this portable delivery.
