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

For baseline collection, pass `--minimum-samples 100`: each window must meet both
its requested duration and retained sample count. Native mode counts completed
frame intervals; CPU mode counts updates. Expensive updates can extend actual
sampling beyond `--sample-seconds`; `sample_wall_seconds` records that duration,
and the process deadline still bounds the run. `moving_samples` / `resting_samples`
identify windows that naturally settle during collection. Such mixed windows do
not establish reliable active-only tails from their later resting frames.

The active fixture starts above support with linear/angular motion. It naturally
settles; it is not a perpetual externally driven stress loop. Requested settled
loads start at support, but layered piles and faceted hulls can still move. Check
`active_observed`, `settled_observed` and per-sample `moving_objects`; increase
warm-up until settled, and capture short active windows with explicit 0s warm-up
when investigating initial drops. Never describe a requested state as verified
when its counters disagree. Dense 1,000 loads are deliberately overfull: current
physics constrains horizontal footprint/floor but has no fragment ceiling solver.

`--settle-steps N` optionally prepares **settled cases only** with up to N actual
production 16 ms updates before opening the native window. It exits on complete
persistent sleep, prints actual steps/simulation/wall time to the case stderr,
and fails rather than measuring an unsettled fixture when the limit is reached.
The native warmup/sample clocks then run normally; the matrix runner also rejects
any prepared case that wakes during measurement. This avoids expensive initial
falls feeding larger wall-clock deltas back into an already-settled-load probe.
Active rows retain their original initial motion and do not use preparation.
Preparation and compiler runs must finish before collecting performance samples;
increase per-case `--timeout` to include preparation. Record `settle_steps_limit`
with the other configuration fields when comparing baselines.

Set `SALIMON_FIXTURE_CHECKPOINT` to a local JSON file to retain the last production
preparation state on success or failure. The read-only diagnostic export runs at
initialization, every 50 updates and completion, and replaces that file with the
actual step, stable IDs, poses, velocities, sizes, masses, hulls and sleep flags.
Use it for diagnosis; it is not a native sampling or acceptance result.

`--repeats 3` collects three independent warmup/sample windows from one naturally
settled population. Each window resets clocks, retained samples and decimation,
checks the original IDs/mass, and appears separately in the summary with a
`repeat` index. The raw output uses schema 2 with schema-1 `runs`; one repeat
retains schema 1. Active rows instead launch three fresh processes to repeat
their initial moving state. Incomplete windows fail as a whole case.

Native settled `--camera both` measures all facing repeats followed by all away
repeats from the same naturally prepared population. Each camera gets its own
warmup/sample windows. Active windows use fresh processes, so `both` is rejected
for active and CPU-only launches.

The matrix helper's `--prepare-workers 1..4` can prepare several native fixtures
concurrently, then measure them **one at a time after all preparations finish**.
Other prepared processes wait on stdin before creating a native window or
starting clocks. Direct `--defer-start true` emits `SALIMON_BENCHMARK_PREPARED`
and requires the exact stdin line `SALIMON_BENCHMARK_START`; default false keeps
ordinary CLI behavior. Deadlines include preparation, the barrier and recording.
This saves duplicated preparation without concurrent physics work contaminating
native sample windows. `--capture-binary` may select the identical SHA-256 binary
inside a local app bundle for the dense 500 capture surface.

Optional matrix `--solver-baseline-samples 1..1000` replays the naturally settled
dense 500/1000 snapshots through both stateless and cached solvers before native
sampling. Direct CLI `--baseline-samples` also accepts smaller prepared fixtures
for smoke checks. Each replay resets cloned input state outside its timer; the
cached path must preserve exact poses and report all objects asleep. Raw
`solver_baseline` includes authored snapshots, wall times, integration/pair/matrix
counters and sleeping counts. This isolates solver work on identical geometry;
it is a CPU comparison on the host, and does not provide a baseline native FPS.


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
Optional `fragment_settle_steps: 0..37500` requires a population and performs the
same bounded production preparation before E2E readiness. Increase the overall
scenario deadline for preparation; ordinary input deadlines are unchanged.
Inspection exposes `world.sleeping_fragment_count` and each fragment's `sleeping`
flag from the last completed update, excluding the carried object. These allow
loaded routes to assert real activation rather than infer it from screenshots.
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
- The legacy `matrix_allocations` field now counts sparse pair-table growth (#157);
  requested projection storage includes pair-entry/geometry/SAT capacity growth.
  Historical pre-#157 values counted outer/row matrix vectors. Both representations
  are structural samples, not total heap allocations; hash control bytes,
  spatial grid/transformed-hull buffers, ground-arm/axis/vertex
  scratch, contact-response multiplier vectors and allocator overhead are excluded.
  Geometry-record and SAT candidate-index capacities are included. Retained
  pair-table/projection capacity is not counted again in a later call; these counters
  do not measure resident memory. Resource scratch bytes now describe retained
  instance staging plus last-uploaded byte capacities (#158); historical
  pre-instancing measurements described transformed vertex capacity. These values
  are not directly comparable. Use a native allocator profiler to
  complement these counters before claiming total allocation reduction.
- Renderer CPU time excludes surface acquisition/presentation; raw frame intervals
  include their effects. Resource prepare/upload time isolates transformation/
  buffer growth from CPU encoding/enqueue, not GPU transfer latency. Bytes and
  triangles cover only the resource batch (including deposits).
- GPU durations are asynchronously completed **latest** pass values, potentially
  repeated/stale; pending/unsupported remain explicit. No blocking timing readback.
- `objects_simulated` is the selected snapshot population (legacy field name).
  `awake_objects`/`sleeping_objects` report persistent activation after the update;
  `quiet_objects` counts bodies that passed dwell, including bodies still awake
  because another member of their supported contact island is unstable;
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


Native benchmark gameplay input is disabled, normal cursor capture is retained, and
window titles identify the exact case/repeat. Keep the window visible and its
resolution/DPI unchanged throughout collection. Interrupted/occluded/resized
collection fails. Initial occlusion before retained intervals restarts the full
warm-up. Renderer metadata records the actual DPI scale; screenshots
must show the matching benchmark title and be taken from a live process.

## Native loaded sleep acceptance (#156)

`scripts/fragment_sleep_acceptance.py` uses the normal protocol-1 controller
routes after bounded natural preparation. It verifies fast rotating flight,
landing/takeoff, stable anchors, IDs/material/mass and full sleep. `--carrying`
also requires actual F removal of a single-support bottom body, dependent fall,
an unrelated sleeping island, mining lockout and three pickup/drop cycles. It
walks around the authored Core and tries both lateral viewpoints through the
controller to reach cargo. Release navigation faces clear floor while accounting
for the held object's side offset, including after each repeated pickup;
no poses or sleep flags are installed. Fall checks identify the selected root's
previous dependents, and unrelated sleep is checked outside its old component.
Failures remain failures in its JSON summary and process exit status.
Loaded requests use a 120-second limit (`--request-timeout`), also capped by the
whole-run `--timeout`. This allows expensive real contact rebuilding after support
removal; it changes neither the production timestep nor the acceptance assertions.
The request limit is recorded in each result. Valid native protocol fixed-step
requests also have a 120-second server limit; extending the Python deadline
alone cannot extend an older binary's five-second single-step limit.

```sh
python3 scripts/fragment_sleep_acceptance.py --binary artifacts/build/release/salimon-client --counts 500 1000 --layout dense --settle-steps 2400 --timeout 14400 --carrying --output artifacts/fragment-sleep-acceptance
```

This opt-in route does not replace the standard native suite or benchmark matrix.
Inspect protocol logs and exact counters before promoting a run as evidence.

## Resident resource preparation comparison (#158)

Run the opt-in CPU preparation comparison:

```sh
cargo test --release -p salimon-renderer resource_preparation_benchmark --locked -- --ignored --nocapture
```

It compares the preserved test-only pre-#158 CPU vertex expansion with production
instance staging/culling for 500/1,000 mixed fragment variants, facing/away and
fixed/active transforms (50 warm-up + 200 samples). Active transforms vary center
and quaternion deterministically; this microbenchmark does not run physics and
its fixed transforms do not establish a verified settled gameplay window. It
prints preparation median/p95 and old/max-new upload bytes. Unchanged production
batches actually upload only the 64-byte camera uniform. It excludes enqueue/GPU,
other runtime preparation and vsync, so it cannot establish FPS or native budgets.
Use the native matrix above for actual settled/active gameplay performance.

For indexed instancing/buffer growth/cache/draw validation on a GPU backend:

```sh
cargo test -p salimon-renderer headless_resource --locked -- --ignored
```

This also compares all 18 variants against the CPU-expanded offscreen raster.
`SALIMON_RESOURCE_EVIDENCE_DIR` optionally writes PNGs; use an absolute path.
This explicitly requires a GPU adapter; Linux lavapipe is correctness coverage,
not the Apple M1 native reference. CPU tests also compare all 18 indexed variants
and quaternion transforms at distant origins and keep rotated frustum crossings.

## Spatial candidate attribution (#157)

```sh
cargo test --release -p salimon-physics candidate_filtering_cost --locked -- --ignored --nocapture
cargo run --release --locked -p salimon-physics --example sleep_cost
```

The first compares sparse all-pairs (test-only candidate oracle) with spatial
filtering on identical separated 500/1,000 cube snapshots, settled on a floor
versus active in air. It uses 20 replay samples; this isolates filtering after
matrix removal, not authored cargo or native FPS. The second uses 100 samples
and actual sleep dwell; its CSV also reports requested pair-storage growth.
Compare the same example at a pre-#157 revision for matrix/storage attribution.
Native authored matrix, dense piles and active interactions still require the
reference commands above. The spatial grid does not imply linear dense-pile cost.


## Tick/backlog diagnosis (#159)

Native samples report `game_ticks` (0–2), `game_backlog_ms` (retained elapsed
simulation debt) and cumulative `lifecycle_excluded_ms` (debt explicitly cleared
at lifecycle resets). Interpret FPS alongside debt: an overloaded shared-clock
simulation can stay behind wall time without doing an unbounded catch-up update.
Simulation delta is the sum of completed 16 ms ticks; solver times/work include
both calls when two ticks ran. Zero-tick presented frames retain activation
observations but report zero new solver work. CPU-only samples have null scheduler
fields and still use their explicit fixed delta. Fixed-time correctness is unchanged.
For ordinary native launches, `RUST_LOG=salimon_client::frame=debug cargo run --release
--locked` includes tick count/backlog in the periodic frame log.

`cargo run --release --locked -p salimon-physics --example tick_cost` isolates
500/1,000 scattered mixed-mass cube snapshots: naturally settled, one fast deck
impact among sleepers, and all active drops. Each row uses 10 unrecorded replays
then 100 independently reset timed replays at 16 ms; clone/setup is outside the
timer. It reports median/p95/p99 plus work and sleep counters. Run revisions
sequentially without competing validation jobs. This is a CPU mechanism probe,
not authored dense/native reference-matrix acceptance or an FPS comparison.

## Resource geometry evaluation (#160)

```sh
cargo test --release -p salimon-renderer resource_geometry_inventory --locked -- --ignored --nocapture
SALIMON_GEOMETRY_EVIDENCE=/absolute/path/to/geometry.json cargo test --release -p salimon-client resource_geometry_evaluation --locked -- --ignored --nocapture
cargo test --release -p salimon-renderer resource_geometry_gpu_cost --locked -- --ignored --nocapture
```

The inventory reports triangle corners, resident position/material vertices and
triangles for all 18 authored variants. The runtime probe builds the production
convex envelopes, reports unique stored positions, supporting faces, face axes,
edge axes and raw same-variant SAT bounds. Deposits are audited for completeness;
only the six fragment envelopes participate in loose-object collision.

The CPU probe compares authored shapes with diagnostic eight-vertex boxes made
from their local authored extents. These boxes are neither exported assets nor
approved gameplay proxies. A 1,024-direction support scan records a sampled
lower bound on their largest outward discrepancy per unit side. Multiply by the
actual fragment side for world metres; the reported 0.4 m side is a control scale.
Convex envelopes already bridge concavities, so this comparison measures extra
error relative to today's collision envelope, not fidelity inside concavities.

Each CPU row replays a 16 ms production solver call excluding cloning. Settled
and active floor rows use ten warm-up and 100 timed replays; expensive active
pair rows use two warm-up and 20 timed replays with unreliable tails flagged.
Six shapes first settle naturally, then their
poses are tiled onto an unbounded floor and observed through production sleep
dwell. Settled rows require all 500/1,000 bodies asleep and exact pose retention.
Active floor rows perturb every body; active pair rows additionally place
isolated two-body stacks at supporting extents for SAT/manifold cost. Box/authored controls naturally settle to
different poses, so this measures geometry workload, not exact-pose parity or
dense ship cargo acceptance. IDs/order/mass are held constant. Run probes and
repeats sequentially without concurrent builds.

The GPU probe times only the resource pass at 1920×1080 with asynchronous
production timestamp machinery. This offscreen test drains each submission to
attribute every sample; native presentation retains its nonblocking readback.
It uses 500/1,000 mixed fragments at a `1e12` m origin, fixed or changing
quaternions, facing/away, ten warm-up and 100 retained samples. Clear/depth work
is included; physics, ship, planets, UI, uploads and presentation are excluded.
Static transforms do not prove gameplay sleep. Unsupported timestamps are
explicitly reported. Software Vulkan results cannot establish hardware GPU
budgets or justify a native LOD choice.

See [the decision and evidence](../reports/issue-160/README.md). Reconsider
authored proxies only with an accepted contact error budget and measured native
benefit; reconsider projected-size LOD/hysteresis only with resource-specific GPU
evidence. Any selected shapes/LODs must follow the existing `models/` source,
export and validation workflow, while visibility must never suspend physics.
