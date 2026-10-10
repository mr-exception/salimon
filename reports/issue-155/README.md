# Issue #155 — fragment profiling and reproducible load harness

## Outcome

Implementation ready for review: deterministic mixed-material/variant/size
populations, production solver counters/timings, renderer batch observations,
release/native and CPU-only JSON runners, baseline-relative comparison gates,
large-load correctness tests and CI artifacts. This change does not optimize
contacts or rendering and does not change gameplay cargo limits or deletion.

**Issue acceptance remains incomplete.** Native Apple M1 reference measurements,
verified settled 500/1,000 windows, long active-interaction budgets and loaded
support-removal/full flight/landing/streaming-return evidence remain pending.
Do not treat this CPU evidence as meeting the native 60 FPS target or as clearing
#157's required native baseline. The PR deliberately relates to #155 without
closing it automatically.

## Reproduction and evidence

See [the maintained guide](../../scripts/FRAGMENT_BENCHMARKS.md) for full native
matrix/settings, metric definitions, correctness routes and comparison gates.
[CPU summary](cpu-summary.json) contains all 48 cases and median/p95/p99/max,
spikes, state flags and storage/candidate counters. Raw JSON for each 500/1,000
case is beside this report as `physics-<count>-<layout>-<state>-facing.json`.

Measurements: 2026-10-10 UTC; Linux 6.18.44 / Ubuntu 24.04 container, AMD EPYC
9V74 with 9 visible logical CPUs, Rust 1.99.0, optimized workspace release build.
No native GPU/display, no renderer/GPU/FPS measurements. Source base
`6d9b282c1892c6ec15408a8dfabe8bcdbaeb6504`, dirty working-tree implementation of this PR;
exact measured binary SHA-256 `5761e0916da41ddd78c24cac578168f6c9324b52768a1769354004ebbbc069c3` is also in the summary.

```sh
cargo build --release --workspace --locked
python scripts/benchmark_fragments.py --binary target/release/salimon-client --mode physics --counts 0 10 25 50 100 250 500 1000 --layouts scattered dense surface --states settled active --warmup-seconds 0 --sample-seconds 1 --timeout 120 --machine-label container-cpu-only --output artifacts/fragment-benchmarks/final-cpu
```

48/48 cases completed and preserved fragment IDs/source/material/mass. CPU-only
runs use 16ms simulation deltas; the one-second **wall-clock** window can overrun
by an expensive final update. Startup hull construction is excluded. Zero warm-up
makes this exploratory initial-motion evidence, not a settled baseline. The
large-load windows have too few samples for reliable tail statistics; the summary
flags that explicitly. Percentiles are descriptive samples, not stable budgets.

| Count | Layout | Requested initial state | Samples | Median total physics (ms) | Median contacts (ms) | Median moving objects |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| 500 | scattered | settled_requested | 6 | 225.96 | 222.81 | 472 |
| 500 | scattered | active | 6 | 211.21 | 209.07 | 500 |
| 500 | dense | settled_requested | 3 | 820.10 | 816.18 | 487 |
| 500 | dense | active | 3 | 832.35 | 826.10 | 500 |
| 500 | surface | settled_requested | 8 | 151.36 | 140.69 | 333 |
| 500 | surface | active | 9 | 125.08 | 122.45 | 500 |
| 1000 | scattered | settled_requested | 3 | 603.09 | 595.36 | 972 |
| 1000 | scattered | active | 3 | 555.72 | 548.97 | 1000 |
| 1000 | dense | settled_requested | 2 | 1859.68 | 1849.41 | 987 |
| 1000 | dense | active | 2 | 1946.19 | 1937.26 | 1000 |
| 1000 | surface | settled_requested | 4 | 487.08 | 463.70 | 667 |
| 1000 | surface | active | 4 | 445.30 | 438.23 | 1000 |

Contact processing dominates these CPU-only samples. This includes all-pairs
visits, projection storage/builds, contact impulses and repeated ground projection;
it does not isolate each subcomponent's wall time. Integration and adapter
measurements in the JSON provide the remaining attribution. GPU versus CPU
rendering dominance cannot be established here.

- 500: 3,992,000 pair visits per update; 16.30 MB median requested matrix/projection capacity per update, excluding allocator overhead/other scratch.
- 1000: 15,984,000 pair visits per update; 57.97 MB median requested matrix/projection capacity per update, excluding allocator overhead/other scratch.

The pair counts match 16 passes × 2 substeps × N(N−1)/2. Separate storage/candidate
observations support future attribution of allocation removal versus filtering.
These are structural requested-capacity/vector counts, not a total allocator
profile. Convex temporary vertices/axes/ground arms remain unmeasured. Dense piles
can still require quadratic contact work after a broad phase.

## Changes and correctness

- Physics `advance_profiled` shares the unchanged solver, with counters/clocks
  compiled out of ordinary `advance`; exact-pose parity tests protect ordering.
- Runtime adapts solver measurements and observes moving objects. Native benchmark
  clocks retain raw frame spikes while reporting the production simulation clamp.
- Renderer reports resource prepare/enqueue costs, triangles, upload bytes and
  vertex scratch capacity; GPU timestamp readback remains asynchronous.
- Fixtures create every object through normal mass extraction, validate deck
  containment, and preserve six authored variants, partial/full mass, stable
  sources/IDs and session journal restoration. Large stress layers can exceed the
  cabin ceiling because the existing fragment solver has no ceiling contact.
- Fixed-step automation accepts opt-in population setup fields. The 500-load
  scenario uses real cockpit entry/takeoff initiation; its assertions are also
  exercised in CPU protocol tests. A 500-load test uses aimed F pickup/drop and
  one-object equipment lockout; 500/1,000 updates and distant rotated ship
  writeback retain accounting. Existing gameplay/streaming/support tests remain.
- CI records CPU-only 500/1,000 stress samples without FPS thresholds and runs
  the new 500-load scenario in the default graphical suite.

## Validation

Passed:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`: 307 tests; final structural allocation-counter
  refinement additionally verified with `cargo test -p salimon-client benchmark
  --locked`: 7 relevant tests passed.
- `python -m unittest discover -s scripts -p 'test_*.py'`: 40 tests.
- `cargo build --workspace --locked` and final
  `cargo build --release --workspace --locked`.
- The exact 48-case CPU matrix above: no failed cases/accounting changes.
- `git diff --check`.

Graphical attempt failed before renderer initialization:

```sh
WGPU_BACKEND=vulkan VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json xvfb-run -a -s '-screen 0 1920x1080x24' python scripts/benchmark_fragments.py --binary target/release/salimon-client --mode native --counts 0 10 500 1000 --layouts scattered --states settled --cameras facing away --warmup-seconds 0 --sample-seconds 1 --timeout 120 --machine-label container-software-vulkan --output artifacts/fragment-benchmarks/software-vulkan
```

Xvfb cannot establish local/unix listening sockets in this execution environment;
winit reports `Failed to open connection to X server`. No graphical/GPU results
or screenshots are claimed. Native graphical baseline, packaged smoke and actual
M1 reference matrix are not validated locally. CI must exercise graphical setup;
reference hardware evidence must be recorded separately.

## PR CI startup correction

The first PR run (#162, Actions run 38079698725) passed Windows/macOS and
Linux build, quality and CPU stress checks. Linux graphical E2E failed before
the first `fragment-load.json` assertion: the runner compared the native
protocol-1 ready event with all setup options, including `fragment_count` and
`fragment_layout`, which are not fields in that event. Other baseline scenarios
passed in the saved results.

The runner now checks the protocol's three advertised setup fields while still
passing fixture options to the executable. A process-level regression test
requires both population arguments and the existing ready-event format.
`python -m unittest discover -s scripts -p 'test_*.py'` passes all 41 tests;
`git diff --check` passes. Graphical verification remains delegated to CI due to
the local Xvfb limitation documented above.

## Remaining acceptance work

Run the full reference matrix with sufficient verified settling and ≥100 samples
per case, three repeats, and record hardware/RAM/OS/refresh/power conditions.
The guide proposes 500 settled at 60 FPS and provisional CPU/GPU partitions; an
active budget and variability tolerance still require native measurements.
`--baseline` gates p95 only on matching environments/cases with ≥100 samples.
Complete and time loaded support removal, repeated pickup/drop, full takeoff/
flight/landing and streaming-return routes at 500/1,000; the current change
provides initial fixture setup plus bounded checks, not that full evidence.
