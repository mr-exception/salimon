# Fragment load measurements

Build a release executable with `python scripts/build_game.py --profile release
--output artifacts/build/release`. Use the `.exe` suffix on Windows. Scripts use
Python 3.10+ and the standard library. The primary reference is the Apple M1 iMac,
1920×1080, normal gameplay camera, AutoVsync, F3 off. Record RAM, OS, monitor refresh,
power/thermal conditions and compiler in the run notes. This reference is a target
configuration; this issue's container measurements do not certify it.

## Native wall-clock matrix

On the native reference desktop:

```sh
python scripts/benchmark_fragments.py --binary artifacts/build/release/salimon-client --machine-label apple-m1-imac-reference --output artifacts/fragment-benchmarks/native
```

Default matrix: 0/10/25/50/100/250/**500**/**1,000**, scattered ship/dense ship/
planet surface, initially active/requested settled, facing/away camera; seed 155,
10s warm-up and 30s measured wall time per case. The primary load is 500; 1,000 is
stress. `--counts`, `--layouts`, `--states`, `--cameras`, `--seed`, `--warmup-seconds`,
`--sample-seconds` and per-case `--timeout` select repeatable subsets. Default
per-case timeout is 300s, including startup and an expensive final update; any
failure/timeout is retained and makes the runner exit nonzero. No partial case is
reported as a successful measurement. Reports store binary SHA-256, source base
revision/dirty flag, CPU/OS label, adapter/backend/type, resolution and settings.

The active fixture starts above support with linear/angular motion. It naturally
settles; it is not a perpetual externally driven stress loop. Requested settled
loads start at support, but layered piles and faceted hulls can still move. Check
`active_observed`, `settled_observed` and per-sample `moving_objects`; increase
warm-up until settled, and capture short active windows with explicit 0s warm-up
when investigating initial drops. Never describe a requested state as verified
when its counters disagree. Dense 1,000 loads are deliberately overfull: current
physics constrains horizontal footprint/floor but has no fragment ceiling solver.

## CPU wall-clock diagnosis and CI

Without a native display:

```sh
python scripts/benchmark_fragments.py --binary artifacts/build/release/salimon-client --mode physics --machine-label cpu-only --counts 500 1000 --layouts scattered dense surface --states settled active --warmup-seconds 0 --sample-seconds 1 --output artifacts/fragment-benchmarks/cpu
```

This times actual fragment adapters/convex physics at **16ms simulation deltas**,
without rendering or real-time pacing. It reports CPU cost, not FPS or native
frame latency. One-second windows are exploratory, frequently with too few
samples for reliable tails. Physics-only mode runs one camera setting because
it performs no GPU work. CI uses this bounded command for 500/1,000 scattered
loads and preserves JSON/logs, without absolute FPS assertions. Rust fixture tests
protect all population sizes, conservation/stale-source restoration, profiler
pose parity, 500/1,000 production updates and distant rotated frame writeback;
500-fragment F pickup/drop/lockout and the native takeoff scenario share the
production interaction route.

## Fixed-step correctness and integration routes

`python scripts/salimon-test run scenarios/fragment-load.json --binary
artifacts/build/release/salimon-client` verifies a 500-load setup, one-step
accounting, real cockpit entry and takeoff initiation. It runs in the normal
Linux graphical baseline and is also exercised as a CPU protocol regression.
There is no test-only teleport operation after initialization.

Scenario `setup` can add `fragment_count: 0..1000` and `fragment_layout:
scattered|dense|surface`. Layout without count is rejected. Surface fixtures
require `landed-earth`; `fragment-pile` cannot also request a population.
For longer loaded flight/landing/takeoff or streaming-return walkthroughs, copy
an existing authoritative scenario, add these initial population fields and
update accounting expectations to include fixture mass. Keep production input
and fixed-step actions. Existing support-removal/streaming/flight scenarios remain
in the standard suites; full 500/1,000 loaded walkthrough timings are still
required on the native machine. One 16ms update is already expensive at those
loads; ordinary automation deadlines can prevent long batches. Record failures
rather than skipping updates or weakening interaction gates.

## Measurement definitions and gates

- Pair visits, radius candidates, narrow-phase visits and contacts count solver
  work, including repetitions over passes/substeps. They are not unique pairs.
- Adapter time includes snapshot selection, cloning and pose writeback; solver
  time splits integration/environment contacts from matrix/pair/ground work.
- Matrix Vec allocation counts and requested projection/matrix capacity bytes
  are structural samples, not total heap allocations; ground-arm/axis/vertex
  scratch and allocator overhead are excluded. Resource scratch bytes similarly
  describe only transformed vertex capacity. Use a native allocator profiler to
  complement these counters before claiming total allocation reduction.
- Renderer CPU time excludes surface acquisition/presentation; raw frame intervals
  include their effects. Resource prepare/upload time isolates transformation/
  buffer growth from CPU encoding/enqueue, not GPU transfer latency. Bytes and
  triangles cover only the resource batch (including deposits).
- GPU durations are asynchronously completed **latest** pass values, potentially
  repeated/stale; pending/unsupported remain explicit. No blocking timing readback.
- `objects_simulated` is the selected snapshot population (legacy field name).
  `awake_objects`/`sleeping_objects` report persistent activation after the update;
  `integrated_objects` counts actual object-substep visits. `sleeping_observed`
  requires the entire selected population asleep throughout the sample window.
  `settled_observed` remains the distinct velocity-rest observation. Fully sleeping
  solver counters are zero; snapshot/cache/writeback and rendering remain measured.
  `physics_activation_ms` isolates cache validation and post-solve support/dwell
  work (in-solver sweep wake is part of total solver cost).
  The stateless pre-#156 baseline contains null sleep counters; do not synthesize them.
- Samples include unclamped wall-frame intervals, simulation deltas, median/p95/
  p99/max and >16.67ms/>50ms spike counts. Bounded deterministic decimation reports
  retained samples/observed updates/stride. Spike counts then describe retained
  samples, not every original update. Tails with <100 samples are flagged unreliable.

Proposed native 500 settled target: 60 FPS, frame p95 ≤16.67ms with actual settled
state verified. A practical first **provisional**, unvalidated partition is ≤5ms
physics+adapter, ≤4ms other CPU update/preparation, ≤4ms CPU rendering and ≤12ms
GPU pass time (CPU/GPU overlap; these are not summed as frame time). For active
loads record budgets from the native baseline before setting a gate; prioritize
p95/max/spikes and missed interaction deadlines. 1,000 has no 60 FPS requirement.

Repeat each baseline case three times on the same machine/settings with ≥100
samples. `--baseline path/to/summary.json --tolerance 0.2` rejects incompatible
machine/case/renderer metadata and insufficient samples, and fails p95 regressions
above 20% for frame, physics and CPU-render time (baseline ≥0.1ms). Treat that
initial tolerance as provisional; calibrate variability from the repeats. Shared
CI checks deterministic accounting/order/counters, not hardware timing thresholds.
Before/after optimization reports should compare pair/candidate counts separately
from storage/allocation counters so allocation removal and filtering remain
attributable. A dense pile can still require quadratic contact work.
