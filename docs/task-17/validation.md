# Task 17 — assisted landing and takeoff timing

[Notion task](https://app.notion.com/p/3e0b456853b981bd867bf6754e2a29f2).
Implementation: `8343190ff5416cc03ce0512d39049cbe64f9d984`.

## Timing and motion

| Sequence | Before | After |
| --- | --- | --- |
| Landing from the outer landing range | 1.500 s | 8.000 s |
| Takeoff from landed to clear of the landing volume | 1.001 s | 6.000 s |

The baseline harness advanced the pre-change ship controller in 1 ms steps on
Mercury, Venus, Earth, Moon, and Mars. Landing began 1 cm inside `1.15R` to avoid
rounding outside the inclusive boundary at the catalog's large world origin.
Takeoff's exact former formula is approximately 1 s; the table includes the
harness's 1 ms observation granularity. Closer landings previously completed
sooner. The new durations apply to every accepted starting distance and body.

Landing now holds position for 2 s while smoothly aligning the hull, approaches
for 4 s, and touches down over the final 2 s. The touchdown segment covers at
most 15 m, or one quarter of the remaining height for a close approach. Takeoff
lifts 15 m in 2 s, then clears the landing range plus the ship collision radius
over 4 s. Smoothstep translation and shortest-arc quaternion interpolation
preserve continuous motion and settle velocity at phase boundaries.

Captured waypoints and elapsed `Duration` prevent accumulated drift and make
assistance independent of update partition. The normal runtime still excludes
suspension/minimize intervals and bounds unreported stalls. Nearby-body radial
telemetry uses the analytic derivative of the active path. Cockpit authority,
closed-door takeoff, flight door locks, and non-cancellable completion remain
protected.

## Automated validation

Validated on 2026-09-20 with Rust 1.89.0, macOS 26.2, Apple M1 iMac (`iMac21,1`):

- `cargo fmt --all -- --check` — passed.
- `cargo build --workspace --locked` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `cargo test --workspace --locked` — all 156 tests passed, including 18 ship
  tests and 26 runtime tests.

Focused regressions cover all five solids, three surface normals, near/far
approach distances, 8 s / 6 s state boundaries, update partitions from 16 ms to
700 ms and oversized deltas, velocity finite differences, quaternion endpoints,
input suppression, door interlocks, and completion without cockpit authority.
Runtime regressions verify that physical proximity monitors keep following the
alignment hold and eased travel after the player leaves the seat. An independent
read-only review found no concrete correctness issues.

Stale local Cargo artifacts were observed during the first validation session;
the six workspace packages were rebuilt and the new test names/counts were
verified in the output before accepting the results.

## Native visual validation

Evidence is recorded with the existing native winit/wgpu renderer at its normal
1920×1080 drawable size on the same M1 iMac. The capture tool returns 960×572
window images. The initial 2026-09-19 session reproduced the blank native surface
also recorded for Task 16. Rendering recovered on the resumed 2026-09-20 session.

A temporary launcher under ignored `target/` copies the current runtime and
adds F4–F8 fixture selection for Mercury, Venus, Earth, Moon, and Mars. Each
fixture places the ship 100 m above the body's +Y surface, tilted by 25.8°,
with zero direct speed and the character seated. It calls the existing public
ship/character APIs and uses the normal runtime L/E routing, state updates,
renderer, and camera. Additional logs observe flight state, cockpit authority,
altitude, and velocity. This does not change normal startup or production input.
High-speed takeover and non-polar orientations are covered by automated tests.

Native results and screenshots are listed below after completing the run.
