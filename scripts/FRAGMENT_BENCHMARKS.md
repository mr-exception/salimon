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

Native collection now waits for actual window focus, known visibility, a non-minimized
window and no reported occlusion. Warmup starts only after admission; losing readiness
before retained intervals restarts warmup, and losing it during collection fails.
Suspension, surface loss and surface configuration changes also invalidate collection.
Raw `presentation` evidence records the target PID and monotonic lifecycle events.
The summary/comparison runner rejects missing or incomplete evidence, including old
unverified baselines; recollect those on the same device. CPU-only runs are exempt.
These intervals measure successful submitted present calls, not physical scanout or
a certified visible-display FPS floor. No refresh-rate threshold admits readiness.

On macOS, use the identical executable inside a local app bundle and explicitly
bring its PID's window forward before warmup. OS focus/input helpers remain local
opt-in actions (see [desktop prerequisites](PACKAGED_SMOKE.md)); the benchmark runner
does not change desktop focus. An unfocused executable times out rather than producing
accepted results. Keep the window unobscured throughout all repeat windows.

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


## Manual CPU wall-clock diagnosis

Without a native display:

```sh
python scripts/benchmark_fragments.py --binary artifacts/build/release/salimon-client --mode physics --machine-label cpu-only --counts 500 1000 --layouts scattered dense surface --states settled active --warmup-seconds 0 --sample-seconds 1 --output artifacts/fragment-benchmarks/cpu
```

This advances the production `GameplaySession` at **16ms simulation deltas**,
including ship/character, mining, fragments, carrying and mass synchronization.
Solver/adapter measurements remain separate within each full update. Historical
pre-#309 CPU totals timed only fragment stepping and are not directly comparable
to these full-session update totals. Reports identify this scope as
`update_scope: gameplay_session_v1`. The CLI `--mode physics` name remains for
compatibility. Sampling runs without rendering or real-time pacing. It reports CPU cost, not FPS or native
frame latency. One-second windows are exploratory, frequently with too few
samples for reliable tails. Physics-only mode runs one camera setting because
it performs no GPU work. Run this bounded command for 500/1,000 scattered
loads and preserve JSON/logs, without absolute FPS assertions. Rust fixture tests
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

Reference M1 60 Hz settled cadence guidance: submitted-present frame p95 ≤17.5ms,
with actual settled state and foreground admission verified. This explicitly
replaces the provisional ≤16.67ms p95 guidance: three 30-second empty windows on
2026-10-17 measured p95 17.239–17.268ms, despite median cadence near 16.68ms.
17.5ms is one 60 Hz interval plus a 5% scheduling allowance; it is a bounded
reference-machine cadence budget, not a strict visible 60 FPS floor or a portable
threshold for other refresh rates. Preserve p99/max and spike counts, including
the empty-reference 41.781ms maximum. See [#255 evidence and unresolved active
signoff](../reports/issue-255/README.md). The existing 20% same-device relative
regression gate remains independently applicable; this calibration does not
turn active overload into a pass. A practical first **provisional** partition is ≤5ms
physics+adapter, ≤4ms other CPU update/preparation, ≤4ms CPU rendering and ≤12ms
GPU pass time (CPU/GPU overlap; these are not summed as frame time). For active
loads record budgets from the native baseline before setting a gate; prioritize
p95/max/spikes and missed interaction deadlines. 1,000 has no 60 FPS requirement.

### Proposed active launch budget (#288; pending owner review)

The [#288 proposal](../reports/issue-288/README.md) separates ten fresh,
naturally settling 500×0.032 kg onboard drops from the long settled reference.
Its [machine-readable limits](../reports/issue-288/proposed-budget.json) and
[raw-evidence evaluation](../reports/issue-288/analysis.json) are review material,
not approved replacement guidance. The owner requested this proposal on
2026-10-17; approval of the thresholds remains a PR review decision.
The proposal preserves the settled guidance above and independently reports
the existing whole-window 20% relative checks, including their failures.
Passing the proposed numbers does not grant #255/#288 signoff or establish
continuous motion, larger-piece, dense-contact or #236 sphere acceptance.
The [2026-10-21 refresh](../reports/issue-255/2026-10-21/README.md) passes all
numerical proposal checks and fresh historical relative comparisons; prior failed
windows remain recorded. [#479 reanalysis and decision package](../reports/issue-479/README.md) reverify
the unchanged archive/proposal without repeating completed collection.
[#563](https://github.com/mr-exception/salimon/issues/563) owns the remaining
explicit owner budget/limitation decision. Until recorded, the proposal remains
pending; closing delivery issues does not grant approval.

### Same-device relative regression checks

Repeat each baseline case three times on the same machine/settings with ≥100
samples. `--baseline path/to/summary.json --tolerance 0.2` rejects incompatible
machine/case/renderer metadata and insufficient samples, and fails p95 regressions
above 20% for frame, physics and CPU-render time (baseline ≥0.1ms). Treat that
initial tolerance as provisional; calibrate variability from the repeats. Manual
regression checks verify deterministic accounting/order/counters, not hardware timing thresholds.
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

For #255/#288 empty-contact attribution, run
`cargo test --release -p salimon-physics separated_sphere_contact_cost --locked -- --ignored --nocapture`.
The probe uses 500/1,000 separated analytic spheres, ten warmups and 100 fresh
16 ms replays. It reports solver/contact p95 alongside isolated repeated spatial
queries, all-awake swept activation queries and exact grid refresh. Setup is
excluded; the old repeated-query pattern remains an attribution control. This
does not establish authored cargo or native FPS acceptance. See the
[matched M1 evidence and remaining signoff](../reports/issue-255/2026-10-17-remediation/README.md).

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

## Solver replacement decision (#161)

The [keep/defer evaluation](../reports/issue-161/README.md) retains the custom
solver for accepted settled 500 cargo and defers the Rapier comparison. It
contains historical evidence, an untested candidate behavior matrix and explicit
reopening gates; no head-to-head speedup or active budget is established.
A reopened comparison must reuse this guide's authored fixtures, reference
configuration, naturally verified sleep and 100/500/1,000 populations, with
separate settled/active samples and full adapter/writeback timing. Measure memory
and platform impact separately from structural pair-capacity counters. Adoption
and production dependencies require a subsequent reviewable implementation.

## Flight, mass and cabin power CPU profiles (#196)

Build release, then run:

```sh
python scripts/profile_flight.py --binary target/release/salimon-client --samples 20 --output artifacts/flight-profile.json --baseline reports/issue-196/baseline.json
```

The opt-in `--flight-profile` CLI runs fresh deterministic 0/500 populations for
idle event-driven mass reads, boundary exit/re-entry, moving powered ship/cargo,
moving zero-G cargo, alternating power transitions and dense falling contacts.
Every row checks stable IDs and physically onboard mass outside its timed window.
The contained fixture is explicitly `legacy-onboard-032kg-v1`: 500 iron
fragments, 0.032 kg each (16 kg total),
not 500 large pieces piled outside the hull. Dense rows place a supported single layer of authored hulls at 12 cm spacing;
ordinary rows spread across the aft deck. Multilayer pile stress remains the
optional `--stacked` variant (ten by ten by five, non-overlapping 12 cm
spacing), and the existing `--benchmark --layout dense` workload. The bounded
manual deck suite does not claim their performance acceptance. The recorded stacked
run fails its p95 ceiling; pass `--stacked` without the deck `--baseline` to
reproduce that stress result. Different fixture baselines cannot be compared. Each measured update is 16 ms; no
warmup hides initial collision work. Per-sample moving counts reveal settling.

Reports retain raw CPU update times and p50/p95/p99/max distributions, binary
SHA-256 and source/machine metadata. `mass-idle` isolates the event drain/ledger;
`mass-crossing` includes one validated round-trip per sample. Movement/contact
rows time the production `advance_game` composition. These are CPU work samples,
not presented frames; GPU time/FPS and Apple M1 acceptance are not claimed.

The Python runner rejects incomplete/duplicate/nonfinite rows, offboard cargo
and summaries that disagree with raw samples. Limits are explicit in
`profile_flight.py`: p95 must satisfy both the absolute ceiling and 4× baseline
plus a workload-specific noise allowance. Missing workload rows fail. Compare
like builds/sample counts/machines when interpreting changes; the manual runner uses permissive
portable regression guards, not a 60 FPS target. Update a baseline deliberately
with report evidence; do not regenerate it automatically on failure.

## Contained flight-wave load (#199)

`--layouts onboard --counts 0 500` selects the same contained 0.032 kg iron
pieces as the flight/Core regression and CPU profile (16 kg for 500). It supports
active drops and natural settled preparation through the production solver.
The existing scattered/dense fixtures retain their larger mixed-material pieces
and multilayer stress geometry; keep their measurements separate. Onboard count
is capped at 500 by the shared fixture. All native clocks, sleep checks, renderer
metadata, raw samples and failure handling remain unchanged.

## Spherical mining matrix (#235)

### Native 10 kg workloads (#236)

`--layouts sphere-deck sphere-pile --counts 10 100 500` selects real 10 kg
iron spheres emitted through capped production spherical extraction. The deck
uses the #235 supported 21 cm grid; the pile uses its first 100 columns and up
to five layers. Both are capped at 500. They are distinct from 0.032 kg `onboard`
and mixed-mass legacy layouts. Active starts add 30 cm of height and 0.4 m/s
upward velocity, then run ordinary motion/contact; no continuous forcing is used.
Requested settled cases require explicit `--settle-steps` and verified natural
sleep to establish resting acceptance. IDs and 5,000 kg total at 500 are preserved.
The same names are accepted by opt-in E2E `fragment_layout` setup for GPU captures.
Native foreground admission and all raw-sample/budget limitations above apply.

### Portable CPU matrix

`python scripts/benchmark_spheres.py --run --output artifacts/sphere-benchmarks`
executes the opt-in **release** CPU tests and validates a complete 10/100/500
matrix (100 samples per case). Run without `--run` to validate/summarize retained
`cpu.json` and `renderer.csv`. Missing/duplicate/nonfinite evidence fails.
Weekly verification runs this matrix and retains its output in the normal raw
benchmark artifact; PR checks remain source-build only.

Runtime uses actual 10 kg extraction, sphere snapshots, ship placement and the
production physics adapter: active initial drops, naturally sleeping single-deck
contents, and a 100-column/five-layer 500-piece pile. Setup/cloning/mining are not
timed. Rest must arise from ordinary contact/sleep, never installed flags. IDs,
mass and zero SAT projections are checked throughout. Active samples can settle
and expose actual moving/sleeping observations; they are not a perpetual stress
workload. Renderer preparation uses the same fixed universe-scale camera/poses
and material masses for old six-shard and shared-sphere batches, with/without a
maximum-size growth preview. No production code is replaced during measurement.

Structural regression gates follow the measured implementation: one sphere batch,
48 encoded bytes per submitted instance, 80 triangles per sphere, zero sleeping
integration/pair work and unchanged identity/mass. These are deterministic gates;
this shared cloud environment does not justify a portable millisecond threshold.
Timings retain all 100 samples and mean/p95/max, with no synthesized FPS. Encoded
bytes/batch counts are CPU submission observations, not GPU transfer/draw timing.
Capacity excludes allocator overhead and is not process/GPU resident memory.

The dated report separates local test-profile measurements from recommended
release reruns. Old geometry comparison covers renderer preparation only; its
inflated hulls are not a matched settled 500-sphere physics baseline. Native
presented FPS/1% lows, GPU frame/pass time and total memory require a graphical
reference run under #236. Existing wall-clock native tools remain available;
CPU tests cannot supply those metrics.


## Current sphere fixture and CPU workload contracts (#303)

Runtime carrying/contact tests construct released spheres through the default
production 16 ms, 2 kg/s, 10 kg-cap lifecycle using the checked test-only
`sphere_fixtures` builder. It verifies growth ID stability, source, one new object,
actual mass (canonical 1e-9 kg budget) and an independently calculated density
radius (1e-12 m budget) before behavior checks. Tiny positive residuals retain
physical mass even when support radius clamps to 5 mm. A separate regression
checks repeated cap releases and one-object carrying. Core conversion tests
also use current spherical extraction, including residual and 10 kg inputs.

The older 10/100/500 #235 matrix and its fixtures are unchanged. The following
**separate**, opt-in release CPU workload adds 500/1,000-object populations:

```sh
mkdir -p artifacts/current-spheres-303
SALIMON_CURRENT_SPHERE_CPU_REPORT="$PWD/artifacts/current-spheres-303/cpu.json" cargo test --release -p salimon-client --locked current_sphere_workloads_cpu -- --ignored --nocapture
```

`current-spheres-303-v1` has `cap-10kg-iron-v1` and `mixed-residuals-v1` profiles.
The latter cycles 1e-9, 0.000003, 0.032, 0.25, 2 and 10 kg, across all three
materials. `supported-deck-layers-active-v1` uses supported 30 cm deck cells,
conservative largest-radius layer spacing, a 30 cm initial lift and 0.4 m/s
upward velocity. No forcing or sleep flags are installed. Setup is outside
measurements. Every row retains actual total mass, count, individual IDs,
material/mass/radius and initial local positions; every sample checks complete
active selection, stable identity/material and onboard accounting.

There are 100 consecutive 16 ms samples per case, including adapter and mass-sync
wall time, activation/contact work and observed moving/sleeping/substep counts.
These are fresh-drop CPU samples; they can settle and are not native frame/FPS,
steady-state or active-only performance acceptance. No timing threshold is
introduced. Keep their raw distributions and environment/source metadata when
comparing the same profile, count, layout, build and machine. Do not compare them
against the 16 kg historical flight baseline or overwrite old evidence.

Legacy extraction remains deliberate in the #195 2 kg refuel fixture, 0.032 kg
onboard flight populations, #47 six-piece mixed transfer fixture, and the explicit
unsealed-tail-growth regression. Their counts/masses remain fixed; they do not
certify cap-sized cargo. New flight profile output includes the fixture version;
older reports are identified by their recorded source revisions, never rewritten.

## Portable retained-session attribution (#305–#306)

`fragment_update_costs` is a separate corrected current-sphere workload, not the
historical 16 kg cargo fixture or the authored convex/native matrix. It builds
500/1,000 active identities with independently 0/10,000 retained inactive
0.032 kg spheres. Active exterior populations use 10 kg current spheres across
all materials; mixed rows cycle 10/0.032/0.000003 kg. Exterior centers use a
40 cm tangent grid at Earth's surface. Ship/zero-g rows use 0.032 kg spheres on
validated 8 cm ship-deck cells; these isolate adapter/frame costs and do not
stand in for 10 kg crowded cargo. Every population records actual IDs, masses,
radii and poses. Inactive identities stay beyond 1 km, outside the exact 125 m gate.

Modes: verified asleep (80 production preparation ticks), active at 1 m lift,
mixed sleepers/repeatedly driven half population, moving ship-local frame
(5 cm/sample), and unpowered/undamped cabin. Active/mixed samples explicitly
reapply velocities to retain scheduling pressure; they are driven attribution
fixtures, not unforced settling trajectories. Each row retains 20 raw 16 ms
samples: allocating read-only selection, isolated stateful solver replay,
production adapter/physics total, frame synchronization, mass/containment work
and complete capture/advance/synchronization time. Replay setup/cloning and
postcondition checks stay outside the total-update timer. Replay checks velocity
and angular equality plus 0.2 mm absolute-position tolerance at the large anchor.

For a same-machine sequential comparison (requires Cargo and optionally Linux
heaptrack; output path must not exist):

```sh
python3 scripts/fragment_update_attribution.py --before issue-304 --after issue-306 --output artifacts/fragment-attribution --heaptrack
cargo test --release -p salimon-world --locked fragment_index_cost -- --ignored --nocapture
```

The comparison runner copies only the identical test module into isolated older
worktrees. Use the #304 commit as baseline so damping is already corrected and
only indexing/scratch/mutation behavior differs. Reports retain exact refs,
fixture SHA-256, compiler/platform, full populations and raw samples as gzip JSON.
Before #306, `mass_visits` is `"()"` (not instrumented); afterwards it reports the
ledger visit count. Timings always include actual production mass synchronization.
The standalone world lookup test retains 20 samples of historical linear versus
indexed lookup on identical 500/1,000 active + 0/10,000/50,000 inactive 10 kg
spheres; index capacity must not grow during queries.

Heaptrack measures a separate entire five-sample 500-object test process for
each of asleep/active/mixed/moving-ship/zero-g, including identical creation,
applicable 80-tick preparation, snapshot replay and test harness. Retain its allocation stacks/totals; they are **not** per-tick counters.
Six per-body solver arrays and adapter vector capacity reuse have separate
regressions. Remaining activation/contact allocations are not hidden. CPU timing
and allocation observations never establish native M1 visuals or FPS (#313).

`SALIMON_FRAGMENT_COST_SAMPLES=N` optionally shortens the per-case CPU sample
count (default 20); record it and use the same value for both refs. Short samples
are descriptive attribution, not a tail-latency or performance-budget gate.
The independent `fragment_journal_cost` integration test records writeback and
journal-drain CPU time separately on checked 0.032 kg current spheres. Use
`SALIMON_EXPECT_QUIET_JOURNAL=1` only for the optimized ref's assertion; it never
changes production behavior. The historical baseline must still emit one event
per requested write. Heaptrack trace extraction discovers `.gz`/`.zst` output
rather than assuming the runner's compression format.

`--allocations-only --heaptrack` collects only those five per-mode process
profiles for both refs. Separate per-ref Cargo target directories prevent an
older worktree's artifacts being reused by a checkout with older file mtimes.
When running the standalone journal integration probe across refs, also use
separate `CARGO_TARGET_DIR` paths (or clean the world package between revisions).

### Native current-sphere profiles (#313)

`--layouts current-cap current-mixed --counts 500 1000` exposes the
`current-spheres-303-v1` population to the native benchmark and opt-in E2E setup.
`current-cap` is 10 kg iron; `current-mixed` cycles the existing residual masses
and three materials. Both use the canonical 30 cm supported deck cells and
conservative water-ice-radius layer spacing shared with `sphere_workloads.rs`.
Active starts retain the 30 cm lift and 0.4 m/s upward velocity; settled starts
omit the lift/velocity and still require natural preparation/sleep verification.
The legacy `sphere-deck`, `sphere-pile` and `onboard` layouts are unchanged.

Raw accounting now also records each fragment's actual `radius_meters`; IDs,
material and mass remain authoritative. The 5 mm minimum contact radius is not
an invented minimum mass. Native foreground admission, completed-frame sample
counts, active-only filtering and timeout/failure rules above remain required.
A 1,000-object residual workload can be expensive; a timed-out run has no passing
frame/FPS result. Never substitute a legacy 500-piece run for these populations.

### Controlled-build provenance (#313)

Use a distinct `CARGO_TARGET_DIR` for production and each source-variant
worktree. A shared target reused the temporary damping ablation library during
#313 staging; that binary's evidence was rejected and recollected from an
isolated production target. Record actual binary/source hashes and target paths,
and run release physics invariants before native sampling. A checkout revision
alone does not establish which physics artifact was linked.

### Independent inactive-history measurement (#455)

Run the unchanged #305–#306 sleeping fixtures separately from the expensive
active matrix on the reference machine:

```sh
SALIMON_FRAGMENT_COST_INACTIVE_ONLY=1 SALIMON_FRAGMENT_COST_REPORT="$PWD/artifacts/inactive-history.json" cargo test --release -p salimon-client --locked fragment_update_costs -- --ignored --nocapture
```

This selects 500/1,000 nearby objects with 0/10,000 distant retained objects,
20 samples per row by default. It verifies natural sleep, identities, mass and
solver replay parity and records selection/history, adapter, frame-sync and
mass costs. `SALIMON_FRAGMENT_COST_SAMPLES` controls repeats; do not combine
with `SALIMON_FRAGMENT_COST_QUICK`. The fixture/schema stays version 1; additive
`scope` identifies this subset. This CPU timing lane does not replace the
current-cap/current-mixed native matrix. See the [#455 report](../reports/issue-455/README.md).
