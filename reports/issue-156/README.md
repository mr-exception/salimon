# Issue #156 — persistent fragment sleep and contact-island waking

## Current status — 2026-10-12

**Incomplete: native dense 500/1,000 loaded acceptance and the reference matrix
remain pending.** Current implementation commit: `047105c`; staged native binary
SHA-256: `70d36bfd5dc2437603028b469eb0a4da5b300c4103e91db3f6ff0f13c17a1e16`.
Fresh authored dense 500 preparation reached all 500 sleeping after 1,056
production updates (16.896 simulation seconds / 1,733.795 wall seconds). Loaded
25,000 m/s rotating flight, landing and takeoff passed with zero local drift.
The [partial native result](native-500-partial-2026-10-12.json) retains these
checks and the failure: after successful F pickup of support 162, the first
physics update exceeded the harness's five-second response deadline. The full
route remains failed. A fresh 500 retry reproduced complete sleep and rigid
flight checks, but exposed a second five-second deadline in the native server.
The server and Python harness now both allow 120 seconds for valid fixed-step
requests, bounded by the total run deadline. Corrected large-load interaction
reruns remain pending.

The fresh 1,000 route also reached complete sleep: 1,586 production updates
(25.376 simulation seconds / 7,628.898 wall seconds). Fast rotating flight,
landing and takeoff preserved all IDs, 1,250 kg and exactly zero local drift.
Its carrying route failed to acquire a reachable bottom support. The
[partial 1,000 result](native-1000-partial-2026-10-12.json) records that failure;
the ordinary walking/viewpoint search has been revised for a fresh rerun.
No successful complete large-load interaction route is claimed.

The first 500 native timing attempt is excluded: desktop input changed the
camera and picked up a fragment while another game was running. The
[initial view](native-500-facing.png) and
[input-interference view](native-500-input-interference.png) document this
failed attempt, not accepted FPS evidence. Benchmark mode now ignores gameplay
input and rejects mid-collection occlusion or surface configuration changes.
After the user confirmed the iMac was idle, the already naturally prepared
1,000 native timing run began its isolated baseline and six sampling windows.
Its executable is the sampling build below; replacing the staged file does
not change that already-running process. Results remain pending.

The latest staged isolation build passed 351 workspace Rust tests (one existing
GPU test ignored), 50 Python tooling tests, workspace Clippy with warnings
denied, formatting/diff checks and all 17 native scenarios / 1,712 steps.
The [isolation validation manifest](native-validation-2026-10-12-isolation.json)
records its distinct executable hash and source/build limits. The earlier
[point-envelope validation manifest](native-validation-2026-10-12-point.json)
records the production physics build used for the partial loaded routes above.

Remaining acceptance requires naturally settled dense 500/1,000 populations,
loaded flight and support-removal/carrying checks, identical-snapshot physics
baseline comparison, native screenshots, and all 96 reference matrix cases
with three windows and at least 100 retained samples per window. Diagnostic
continuations, partial preparations and timeouts do not satisfy those gates.

The benchmark build adds optional `--minimum-samples 100`, requiring both the
requested wall duration and 100 retained native frame intervals before a window
completes. It records actual sample duration and moving/resting sample counts;
production physics and update timing are unchanged. The
[sampling-build validation manifest](native-validation-2026-10-12-sampling.json)
records its distinct hash, 351 passing Rust tests (one existing ignored GPU test),
50 Python tests, Clippy/format checks and 17 native scenarios / 1,712 steps.
The partial loaded correctness results above use the point-envelope binary;
the corrected reruns will use the isolation build, whose physics is unchanged.

## Intermediate native investigations — 2026-10-12

**Incomplete: the required native dense 500/1,000 settled windows have not passed.**
The earlier committed build (`f5f03c2`) passed all 17 native scenarios / 1,712 steps,
and the native 100-fragment loaded route described below passed. The attempted
native dense 500 preparation failed after 2,400 production updates (38.4 simulation
seconds): only 7/500 fragments slept, 56 met quiet dwell, and 138 still moved.
Preparation took 7,792.544 wall-clock seconds (about 130 minutes). It exited before
readiness, baseline replay or native sampling, so it provides no settled FPS result.
Raw local stderr: `artifacts/fragment-benchmarks/issue156-dense500-friction-native.stderr`.

Subsequent solver changes are committed in `047105c`. Passing small-load or
diagnostic replay checks do not satisfy the large-load acceptance.
The next native 500 attempt also failed: 40/500 asleep after 1,200 updates
(19.2 simulation seconds, 2,273.509 wall seconds). The simultaneous 1,000 attempt
was terminated before readiness after 4,550 wall seconds; its last checkpoint
was 27/1,000 asleep at 850 updates. Neither attempt produced settled sampling
or loaded interaction evidence. Raw results are in
`artifacts/issue156-native-rolling10-500` and
`artifacts/issue156-native-rolling10-1000`.

Read-only preparation checkpoints now retain canonical states for diagnosis.
The runtime adapter continuity regression passed. Moving the convex rest cutoff
after all contact iterations passed 47 physics tests and a fresh native 100
preparation (all sleeping after 202 updates / 3.232 simulation seconds).
The corresponding 500 preparation was stopped after retaining its state at
600 updates: 42/500 asleep, 427 quiet and 23 moving (1,216.276 wall seconds at
that checkpoint). It did not reach readiness or the loaded route. These are
intermediate results, not completion.
The staged end-cutoff build also passed all 347 workspace Rust tests (one existing
GPU test ignored), all 48 Python tests, workspace Clippy, and all 17 native
scenarios / 1,712 steps. Formatting and diff checks passed.
The [dated validation manifest](native-validation-2026-10-12.json) records the
binary SHA-256 and per-scenario results; the large-load gate remains pending.

The next change retains exact SAT projections across substeps/updates when
orientations, shapes and identities are unchanged; rotating pairs and external
changes invalidate them. An exact-state regression matches fresh projection
rebuilds and verifies invalidation. Fully sleeping groups release the matrix.
A first cache trial was stopped at its 450-update retained state (47/500 asleep)
while correcting first-use face rejection and pending reaction handling. It is
excluded from acceptance. The corrected cache matched fresh substep projection
rebuilding exactly for 20 updates of the actual authored initial 500 fixture and
20 updates of the retained 600-update native state. These are diagnostic
equivalence checks, not native acceptance. Local logs:
`artifacts/fragment-benchmarks/cache-probe/actual500-equivalence.log` and
`artifacts/fragment-benchmarks/cache-probe/actual600-equivalence.log`.
A fresh native 500 preparation with the corrected optimization failed after
2,400 updates (38.4 simulation seconds): 31/500 asleep, 472 quiet and 14 moving.
It took 4,044.787 wall seconds and exited before readiness or native sampling.
Raw result: `artifacts/issue156-native-prefilter-500/summary.json`.
The corrected build passed 348 workspace Rust tests (one existing GPU test
ignored), Clippy, formatting/diff checks and all 17 native scenarios / 1,712 steps;
unchanged Python tooling passed its 48 tests. The
[corrected cache validation manifest](native-validation-2026-10-12-cached.json)
records its distinct binary hash. It reproduces the earlier native 50/100-update
motion and sleep counters. The required large settled loads remain unverified.
The retained 600-update state has no deep penetration: full SAT found 614
contacts, no overlap exceeding 1 mm, and a maximum depth of 4.460 micrometres.
Separate velocity-stage experiments are diagnostic only and are not installed
in this production build.
Those experiments did not improve settling and were rejected. Diagnostic
per-body observation failures at the retained 1,100-update native state identify
34 kinetic-displacement, 23 projection, 44 angular-drift and 21 angular-chatter
failures (criteria overlap), plus two speed failures, one short window and one
support failure. Fixed-reference point observation reached 492 quiet bodies but
did not sleep the main island. Growing material-point envelopes subsequently
reached all 500 asleep in a diagnostic continuation (51 updates / 86.940 wall
seconds from the preceding replay state). This is not a fresh native fixture
or native acceptance result.

The production observation rule now tracks actual proxy-vertex trajectories
with the documented 15 mm enclosing-radius / 0.5 s rest budget. It replaces the
axis-dependent quaternion and terminal-velocity displacement estimates. A first
sample receives no prior-time credit. Tests deliberately cover supported motion
inside the envelope and translation/tipping above it, loss of support, first
long updates and quaternion sign equivalence. All 50 physics tests and Clippy
passed. The same build passed 350 workspace Rust tests (one existing GPU test
ignored) and all 17 native scenarios / 1,712 steps. The
[point-envelope validation manifest](native-validation-2026-10-12-point.json)
records the binary and checks. Fresh native 500 and 1,000 loaded acceptance is running;
the issue remains incomplete until the required native gates pass.
The first replay probes in this continuation accidentally linked an older staged
physics library; their results are excluded. Subsequent probes build with explicit
`--target aarch64-apple-darwin` and link that target's matching libraries.
A 32-update trace showed median half-window pose displacement of 2.109 mm.
Further checks separated numerical projection from terminal kinetic motion.
The current observation-window criteria are documented in the physics
[canonical policy](../../client/physics/architecture.md#persistent-contact-island-sleep-156).
They retain genuine slow-slide/turn rejection, require support throughout the
window, and bound projection and residual overlap. Unsuccessful static-deck,
extra-pass and independent-cutoff experiments are excluded from acceptance.

Before the latest warm-starting experiment, the updated solver passed 339 Rust
tests (one existing GPU test ignored), workspace Clippy, formatting and all 17
native scenarios. Current warm-starting safety tests pass, including identity
reordering, removed support, changed mass/pose/velocity/environment and explicit
force invalidation. These checks precede the current 350-test/17-scenario validation above; they
are historical intermediate checks, not acceptance completion.

## Native continuation — 2026-10-11

The continuation runs on the reference iMac21,1: Apple M1 (8 CPU / 8 GPU cores),
16 GB RAM, macOS 26.2 (25C56), Rust 1.89.0, native Metal, 1920×1080 drawable,
scale 1, AutoVsync, diagnostics off. The external display is 1920×1080 at 75 Hz;
the built-in display is 4480×2520. `pmset -g therm` reports no thermal/power warning.
No software GPU or container substitutes for this native evidence.

**Validation is still running; this section does not claim completion.** Large
fixtures now prepare through bounded production 16 ms updates and must reach
complete persistent sleep before measured windows. Three settled windows reuse
that naturally settled population; active repeats start fresh processes. E2E
inspection exposes stable-ID sleep for loaded flight/support checks.

Native work identified and fixes:

- Backport wgpu's [Metal first-frame visibility fix](../../vendor/README.md).
  Repeated native launches otherwise remained blank and returned `Occluded`.
- Select ship cargo in the current frame before final world-pose synchronization;
  the old selection omitted fast-moving cargo and cleared its sleep state.
- Filter swept waking by convex proximity as well as conservative spheres.
- Stabilize nearly coplanar reaction features while retaining exact hull support,
  and require the unchanged fixed 1 mm / quaternion 0.005 dwell envelope even
  with documented chatter velocity guards.
- Respect existing deck support during positional pair separation; deck projection
  previously undid the supported body's share of separation in loaded stacks.
- Deduplicate numerical parallel SAT axes and short-circuit support-graph SAT.
  Native sampling identified projection/face/graph work as the dominant cost.

Current workspace gates: **334 Rust tests passed, one existing ignored GPU
backend test**, 48 Python runner tests, release build, formatting and Clippy.
The current physics owner has 35 passing tests. The preceding native build
passed all 17 standard scenarios / 1,712 steps; the final committed build also passed all 17 scenarios / 1,712 steps.
Native two-camera/repeat/barrier smoke checks passed all eight 0/10 windows;
100 identical-snapshot 10-fragment solver replays retained full sleep, zero
integration and zero pair visits against 1,440 stateless pair visits per replay.
These small-load checks validate tooling, not the required 500/1,000 acceptance.

Additional native investigation repaired loaded-face constraints: a three-body
off-center stack failed to sleep with one pair reaction point. Clipped face/edge
normal constraints fix that regression. A separate twist regression proved that
center-only friction left yaw unaffected except for damping; per-point Coulomb
friction now resists twist while preserving momentum and dissipating energy.
The support graph reuses the final substep's SAT spans for unchanged orientation
and scale, testing final center separation rather than rebuilding those spans.

The 100-fragment authored dense fixture reached complete natural sleep after
972 x 16 ms production updates (15.552 simulation seconds) on the intermediate
build. Fast rotating flight/landing/takeoff passed for 10 naturally sleeping
native fragments. These are progress checks; neither substitutes for a verified
settled dense 500/1,000 native window. Large-load preparation remains the blocker.
[Native 100-fragment loaded correctness](native-100-acceptance.json) now passes:
fast rotating 25,000 m/s flight, landing/takeoff, zero stored-anchor drift,
real F removal of support ID 2 with dependent ID 78 falling, an unrelated
sleeping island, mining lockout, three pickup/drop cycles and unchanged IDs/mass.
Support candidates come from the completed physics contact graph and actual
environmental support, not a nearby-above distance guess. This is fixed-delta
correctness through the native application; it is not an FPS measurement or
500/1,000 loaded acceptance.

No unsuccessful, killed, or incomplete preparation is reported as a passed
benchmark. The final native matrix and loaded-route results must replace this
pending status before #156 can close.

The original implementation report below is historical container evidence from
PR #164 / `b52f2f0`, recorded 2026-10-10; it is not current native acceptance.

## Historical container outcome — 2026-10-10

Reviewable implementation of stable-ID sleep at the portable physics boundary,
composed into production fragment updates. Visible/pickable session entities,
mass, carrying limits, f64 frame conversions and in-memory persistence remain
intact. No cargo cap, deletion, full engine or rendering change.

**Native acceptance evidence remains incomplete.** This environment is not the
Apple M1 reference machine required by #155/#156. Leave #156 open until the
native matrix and verified settled dense 500/1,000 cargo windows are recorded;
the PR relates to the issue without automatic closure. CPU isolation evidence
below establishes work reduction, not native FPS or whole-cabin settling.

## Historical container changes and correctness

- Physics owns `SleepTracker<K>` keyed by caller identities and deterministic
  ordered contact graphs. The [canonical policy](../../client/physics/architecture.md#persistent-contact-island-sleep-156)
  defines support, velocity/angular velocity, dwell-wide pose envelope and time.
  Bounded authored-contact chatter is distinguished from sliding/tipping by a
  fixed 1 mm/quaternion-component 0.005 envelope over 0.5 simulation seconds.
- Only supported quiet islands sleep. Full convex SAT (1 mm contact-gap
  tolerance) builds adjacency; overlapping bounding spheres do not imply support.
  Airborne zero-speed objects keep integrating.
- Sleeping snapshots remain collision participants. Integration/damping and
  sleeping-to-sleeping solving are skipped. Entirely asleep calls allocate no
  contact matrix and perform no substeps/contact passes.
- Previous connected contacts wake on omitted/removed/picked-up support, changed
  mass/scale/hull/pose/surface or velocity impulse. Swept sphere wake precedes
  fast-object integration, and actual contacts wake before response. Floor
  containment changes and explicit `wake_all` invalidate support. Quaternion
  renormalization at <=1e-12 component difference does not repeatedly wake rest.
- Runtime owns cache lifetime alongside the mining session. Selection omission
  invalidates old support; streaming return starts awake, preserving IDs/mass.
  Rigid ship translation/rotation preserves local sleep while all world poses
  still synchronize. A regression uses rotated axes at trillion-metre anchors.
- Benchmark JSON and summaries report real awake/sleeping/integrated counters;
  velocity rest and persistent sleep are distinct observations.
- Tests cover bottom-support removal versus an unrelated island, fast impact,
  geometry/forces/environment/frame changes, streaming invalidation, overlapping
  hull bounds, snapshot rounding, zero delta and profiled/unprofiled parity.
  Six authored material/variant combinations settle from tilted poses and sleep;
  existing production carrying, transfer, resource-loop and streaming routes run
  through the new path.

## CPU isolation results

[Raw CSV](sleep-cost.csv), 100 samples per row, same final settled snapshot or
same replayed active initial state for both paths. The stateless production solver
is the baseline; the sleep path uses naturally simulated 40 x 16 ms dwell updates,
not installed sleep flags. Cube proxies have mixed 0.5/2 kg mass on an unlimited
supported floor with 0.5 m spacing. No authored cabin, renderer or FPS claim.
Cache/snapshot cloning for repeatable replay is outside timed solver calls.

```sh
cargo run --release --locked -p salimon-physics --example sleep_cost
```

This execution used `CARGO_TARGET_DIR=/tmp/issue156-physics` to keep the
independent probe separate from the native executable build. Linux 6.18.44,
Ubuntu 24.04 container, Intel Xeon Platinum 8370C, 9 visible CPUs, Rust 1.99.0,
release default optimization/codegen. Concurrent compilation/tests introduce
container timing noise; no machine-matched native performance gate is claimed.
Source: `d75369948488cc260b202c14e8f190decdf7e27e` plus this PR's implementation;
measurement preceded quaternion-rounding, wake-dwell reset, face-rejection and
activation-timing refinements. The cube snapshots use exact identity rotations,
separate bounds and no waking contacts, so their sleep decisions and zero-work
counters are unchanged; timing remains historical isolation evidence.

| Count | State | Stateless median / p95 ms | Sleep median / p95 ms | Sleeping | Baseline → sleep integrated visits / pair visits |
| ---: | --- | ---: | ---: | ---: | --- |
| 500 | settled | 252.126 / 287.052 | 0.071 / 0.235 | 500 | 1,000 → 0 / 3,992,000 → 0 |
| 500 | active | 102.784 / 124.221 | 108.982 / 128.704 | 0 | 1,000 → 1,000 / 3,992,000 → 3,992,000 |
| 1,000 | settled | 353.920 / 452.950 | 0.105 / 0.288 | 1,000 | 2,000 → 0 / 15,984,000 → 0 |
| 1,000 | active | 359.823 / 457.531 | 384.510 / 503.862 | 0 | 2,000 → 2,000 / 15,984,000 → 15,984,000 |

Settled work reduction is substantial. Active median overhead is about 6.0%
(500) / 6.9% (1,000). Active snapshots have exact pose parity against stateless
response in this probe. These are solver-isolation results, not total runtime
costs, M1/native timings, or comparisons with #155's different CPU machine.

## Production authored matrix

48/48 cases completed, with accounting preserved and no failed cases.
[Summary](cpu-summary.json) contains every population/layout/state; raw 500/1,000
JSON sits beside this report. This is the #155 production fixture matrix, seed
155, release codegen-units=1 build, 16 ms physics deltas, 0 s warmup / 1 s measured
wall-clock windows. Source base is the revision above plus this PR's final code.
Binary SHA-256: `4f5a0eb42cc45999223e5ce405a11af3700ff6c17478b2a23851968f5ba3c099`.

```sh
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release --workspace --locked -j 2
python scripts/benchmark_fragments.py --binary target/release/salimon-client --mode physics --counts 0 10 25 50 100 250 500 1000 --layouts scattered dense surface --states settled active --warmup-seconds 0 --sample-seconds 1 --timeout 120 --machine-label issue156-container-cpu-only --output artifacts/fragment-benchmarks/issue156-final
```

| Count | Layout | Requested state | Samples | Median physics ms | Activation ms | Moving / sleeping |
| ---: | --- | --- | ---: | ---: | ---: | ---: |
| 500 | scattered | settled_requested | 6 | 240.47 | 7.39 | 472 / 0 |
| 500 | scattered | active | 6 | 233.15 | 7.18 | 500 / 0 |
| 500 | dense | settled_requested | 2 | 1047.65 | 97.55 | 487 / 0 |
| 500 | dense | active | 3 | 960.68 | 80.85 | 500 / 0 |
| 500 | surface | settled_requested | 8 | 155.78 | 4.69 | 333 / 0 |
| 500 | surface | active | 8 | 148.54 | 4.94 | 500 / 0 |
| 1000 | scattered | settled_requested | 3 | 663.21 | 19.87 | 972 / 0 |
| 1000 | scattered | active | 3 | 619.80 | 18.77 | 1000 / 0 |
| 1000 | dense | settled_requested | 2 | 2368.55 | 186.46 | 987 / 0 |
| 1000 | dense | active | 2 | 2454.12 | 208.62 | 1000 / 0 |
| 1000 | surface | settled_requested | 3 | 749.61 | 17.92 | 667 / 0 |
| 1000 | surface | active | 3 | 1381.58 | 19.31 | 1000 / 0 |

These zero-warmup large-load samples are **initial moving cases**, including rows
requested as settled. None verifies an all-sleeping dense cargo window. Their
2–8 samples do not establish reliable tails or an active performance budget.
The 100-sample isolated settled comparisons above must not be presented as the
same authored population. The reference native settled matrix remains pending.

Production measurements caught excessive contact-graph overhead during development:
full SAT on every sphere-overlapping pair initially added about 7.9 seconds to a
500 dense update. The final graph first rejects separated faces with the support
gap tolerance, then uses full SAT only for survivors. Final dense 500 median
activation is 81–98 ms; scattered/surface activation is about 5–7 ms. The new
`physics_activation_ms` metric makes this cost visible separately from the
existing contact solver. Active contact processing still dominates total cost.


## Validation

- Passed: `cargo fmt --all -- --check`; `git diff --check`.
- Passed: `cargo clippy --workspace --all-targets --locked -j 2 -- -D warnings`.
- Passed: `cargo test --workspace --locked -j 2`: 317 passed, one existing ignored
  renderer GPU-backend test. Subsequent wake/face-rejection refinements passed
  `cargo test -p salimon-physics --locked -j 2` (23 tests) and
  `cargo test -p salimon-client fragment_physics --locked -j 2` (8 tests),
  plus final Clippy/script gates and all 48 final production benchmark cases.
- Passed: `python -m unittest discover -s scripts -p 'test_*.py'`: 44 tests.
- Passed: `cargo build --workspace --locked -j 2`.
- Release build initially encountered zero-length dependency codegen objects and
  invalid dependency metadata in this local environment. Removed affected Cargo
  dependency artifacts and rebuilt with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`;
  no repository build-profile workaround was introduced. Passed:
  `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release --workspace --locked -j 2`.
- Local graphical run not available: Xvfb/Mesa tooling absent; attempted package
  setup failed on protected apt cache writes. No screenshots/software-Vulkan or
  GPU evidence is claimed; PR CI must exercise the Linux native suite.

## Limits and remaining validation

- Mixed awake/sleeping updates retain the existing quadratic matrix and ordered
  pair traversal; sleeping pairs skip response, but this is not #157's broad phase.
  Contact graph rebuilding adds active work. Large dense moving islands can stay
  awake until every connected member satisfies the dwell policy.
- Swept sphere wake is conservative and may wake nearby noncontact islands. It
  does not add continuous collision detection beyond existing adaptive substeps.
- Interior gravity already abstracts ship acceleration. Rigid ship motion alone
  introduces no changed local force; future inertial/support policies must update
  snapshots or call `wake_all` explicitly.
- Streaming safely invalidates cached contact state rather than persisting an
  absent support. Disk/session restart persistence remains outside current scope.
- Required native Apple M1 iMac 1920x1080 matrix: #155 command/settings, 500 primary
  and 1,000 stress, facing/away, scattered/dense/surface, settled versus active.
  Verify all-sleeping counters with sufficient warmup and >=100 samples, three
  repeats, then loaded support removal, repeated F pickup/drop, complete flight/
  landing/takeoff and streaming return. Native budgets cannot be certified here.
