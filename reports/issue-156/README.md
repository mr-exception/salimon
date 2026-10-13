# Issue #156 — persistent fragment sleep and contact-island waking

## Accepted scope — 2026-10-13

The user approved the verified **dense 500-fragment** test as sufficient for
this iteration and requested a review PR. Issue #156's acceptance now uses the
complete native 500 correctness route and six dense settled 500 timing windows,
plus the retained sleep/wake/material/frame/streaming regressions. This scope
change is ready for review; #156 remains open until the PR is merged.

- Full native 500 route passed: natural sleep, fast rotating flight,
  landing/takeoff, zero local drift, real support removal/dependent falling,
  unrelated sleeping islands, three pickup/drop cycles, stable IDs and 625 kg.
- Six settled 500 windows passed with 1,801–1,802 intervals each. Facing median
  FPS was 59.67–59.74; away was 59.72–59.77. These are approximately 60 FPS
  medians, not a strict frame-time floor.
- The same naturally settled snapshot measured median solver time
  2,518.251 ms stateless versus 0.198813 ms sleeping, with zero sleeping
  integration/pair visits. This replay excludes cloning and is separate from
  native frame timing.
- Active dense contact work remains expensive. The full 1,000 interaction route
  and 67 missing reference windows (including other 500 cases) are deferred to
  [#165](https://github.com/mr-exception/salimon/issues/165).
  [#157](https://github.com/mr-exception/salimon/issues/157) owns candidate and
  allocation optimization.

This scope update changes documentation and issue acceptance only; it adds no
cargo cap, deletes no fragments and changes no runtime behavior. The previously
recorded 353 Rust tests, 55 Python tests, Clippy/format checks and 17 native
scenarios remain the source validation evidence. No native runs or code tests
were repeated for this documentation-only update. Relative report/evidence
links and the retained result summaries were checked; historical measurements
and failed attempts below are preserved unchanged. Historical statements that
the original acceptance is incomplete refer to the superseded full-matrix
requirement, not the revised 500-fragment acceptance.

## Historical continuation status — prior to scope reduction

**Focused 50-fragment continuation completed.** The user requested a smaller
check rather than restarting expensive large-load work. The native 50 fixture
settled naturally after 37 updates (0.592 simulation seconds / 0.262 preparation
wall seconds). Its flight/landing/takeoff route passed in 1.011 s, preserving all
50 IDs/materials, 64 kg, complete sleep and exactly zero local drift through
25,000 m/s rotating flight. The first `--carrying` attempt completed those checks
but failed the loaded-support precondition: all 50 fragments directly touch the
environment and there are zero bottom/dependent candidates. No 50-fragment
loaded support-removal or pickup/drop pass is claimed. Two existing small-pile
support-removal regressions and one fast-impact/unrelated-island regression
passed separately. [Focused summary](native-focused-50-2026-10-13.json),
[full checkpoint archive](native-focused-50-2026-10-13.json.gz),
[support tests](focused-50-support-tests-2026-10-13.log), and
[impact test](focused-50-impact-test-2026-10-13.log) retain the evidence.

Exact checks (unchanged staged executable and source):

```sh
python3 scripts/fragment_sleep_acceptance.py --binary artifacts/build/native-matrix/salimon-client --counts 50 --layout dense --settle-steps 2400 --timeout 180 --request-timeout 120 --carrying --output artifacts/issue156-native-focused-50
python3 scripts/fragment_sleep_acceptance.py --binary artifacts/build/native-matrix/salimon-client --counts 50 --layout dense --settle-steps 2400 --timeout 180 --request-timeout 120 --output artifacts/issue156-native-focused-50-flight
cargo test -p salimon-physics --locked support_removal
cargo test -p salimon-physics --locked fast_impact_wakes_whole_island_without_waking_unrelated_support
```

This reduced check is complete; the original issue acceptance below remains
incomplete. No new performance measurement or large-load run was started.

**Stopped at the user's request.** Resume instructions and the exact remaining
matrix are in the root [handoff](../../ISSUE-156-HANDOFF.md). All task processes
have been stopped. The latest 1,000 preparation ended with SIGTERM before
readiness; its [stopped result](native-1000-preparation-stopped-2026-10-13.json)
provides no additional acceptance evidence. The final high-load gate retry also
lost its native children before collection. Neither attempt is a passed run.

**Incomplete: the complete native 1,000 interaction route and the remaining
reference matrix remain pending.** The complete 500 interaction route and dense settled 500/1,000 performance windows
have passed, as recorded below. Current production physics is `047105c`; the
current staged surface/measurement executable SHA-256 is
`95e662d5aeea7cec6a8913ba59adbfe51f84d70eaf5f3cfeb6315bf106cc2d68`.
The first point-envelope interaction attempt used executable SHA-256
`70d36bfd5dc2437603028b469eb0a4da5b300c4103e91db3f6ff0f13c17a1e16`.
Fresh authored dense 500 preparation reached all 500 sleeping after 1,056
production updates (16.896 simulation seconds / 1,733.795 wall seconds). Loaded
25,000 m/s rotating flight, landing and takeoff passed with zero local drift.
The [partial native result](native-500-partial-2026-10-12.json) retains these
checks and the failure: after successful F pickup of support 162, the first
physics update exceeded the harness's five-second response deadline. The full
route remains failed. A fresh 500 retry reproduced complete sleep and rigid
flight checks, but exposed a second five-second deadline in the native server.
The server and Python harness now both allow 120 seconds for valid fixed-step
requests, bounded by the total run deadline. Subsequent corrected large-load
interaction outcomes are recorded below.

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
not change that already-running process. That run completed successfully:
all six windows retained 2,243–2,251 frame intervals and all 1,000 fragments
remained asleep, with accounting preserved. Facing median intervals were
13.289–13.337 ms (74.98–75.25 FPS); away medians were 13.248–13.256 ms
(75.44–75.48 FPS). Across windows, p95 intervals were 16.253–16.696 ms;
the largest interval was 42.116 ms. This does not prove a per-frame 60 FPS floor.

One hundred identical naturally settled snapshot comparisons measured median
stateless solver time 7,069.035 ms versus sleeping solver time 0.460667 ms.
Sleeping integration, pair visits and matrix allocations were all zero;
stateless medians were 2,000 integrated-object visits, 31,968,000 pair visits
and 2,002 matrix allocations. Cloning was excluded from both timers. These
solver replay timings are distinct from native frame timings. The
[six-window summary](native-1000-dense-settled-2026-10-12-summary.json) and
[compressed complete raw samples](native-1000-dense-settled-2026-10-12.json.gz)
retain counters, distributions and accounting. This earlier executable did not
record DPI scale; drawable resolution was 1920×1080, native Apple M1 Metal,
AutoVsync. This passes only the dense settled 1,000 case, not the whole matrix.
Measured median native physics time was 1.143–1.184 ms, total CPU update
1.760–1.828 ms, CPU rendering 2.788–3.826 ms, and latest asynchronous GPU
timestamps 0.793–1.349 ms. CPU resource preparation/upload remained measurable
after sleeping physics was skipped. The roughly 75 FPS cadence is consistent
with the 75 Hz display and AutoVsync; these results do not establish uncapped
throughput or an active-interaction budget.

The corrected harness subsequently passed the full native dense 100 route:
support 10 removal made dependent 86 fall; all 98 objects outside that original
component remained asleep; three real pickup/drop cycles passed. It used the
visibility build `c0a262da0e4ce092d6f99743c40e4400e1335c7ffca161aa8505d02bc76c72af`.
Local full evidence: `artifacts/issue156-native-corrected-100/summary.json`.
The [retained corrected route](native-100-corrected-2026-10-12.json) records
its checks, support/dependent IDs, unrelated islands and checkpoint hashes.
The [benchmark screenshot](native-benchmark-input-isolation.png) shows the
10-fragment camera and pickup prompt. F, W, F2 and Escape were sent through
native UI controls, but the window was paused by startup occlusion: the unchanged
image alone does not prove input isolation. It was explicitly closed and supplies
no accepted timing result. Later bundled and bare short-window attempts were
rejected when occlusion arrived during sampling; their empty stdout is excluded.
Initial occlusion before retained frame intervals now restarts the full warm-up;
the focused regression passed. Original focus/cursor behavior is retained while
gameplay events are ignored. The corrected 500/1,000 interaction attempts used
executable SHA-256
`afb221ebebecc1652b6f9ee6329c0bf7289bb2a82b581d5a0c73c5cce1cc9cb1`;
the older-harness 1,000 retry later completed with the failed repeat-pickup
outcome recorded below.
The [visibility validation manifest](native-validation-2026-10-12-visibility.json)
distinguishes current build/lint/format checks and 13 focused benchmark tests from
the earlier complete workspace/native suite. Python tooling passed all 50 tests
again after the final helper changes.
The final [surface validation manifest](native-validation-2026-10-12-surface.json)
records executable `95e662d5aeea7cec6a8913ba59adbfe51f84d70eaf5f3cfeb6315bf106cc2d68`,
353 passing Rust tests (one existing ignored GPU test), 50 Python tests, Clippy,
formatting and 17 native scenarios / 1,712 steps ultimately passed. The first
native sweep passed 15 cases and failed carrying/resource-loop assertions;
both failed cases passed unchanged-binary reruns. Their cause was not isolated,
and the manifest retains the initial failures and retries.

After the user confirmed the benchmark could stay visible, a two-camera native
10-fragment tooling smoke completed with 601 intervals per window, all ten
objects sleeping and accounting preserved. Native F/W/F2/Escape input was sent
during that live run; the intended camera switch and
[live away view](native-benchmark-input-isolation-live.png) remained correct.
Actual metadata: Apple M1 Metal, 1920×1080 drawable, DPI scale 2, AutoVsync.
Concurrent large-scene CPU preparations exclude this smoke from performance
claims and the required three-repeat matrix. It verifies the final native
measurement guards without presenting its FPS as reference evidence.

### Primary 500 native results and remaining interaction gap

The fresh 500 fixture settled naturally after 1,056 updates (16.896 simulation
seconds / 2,310.719 wall seconds). The 1,000 preparation was suspended with
SIGSTOP during the isolated 500 baseline and six native windows, then resumed;
its preparation wall duration includes that pause, which grants no simulated
rest time. All 500 fragments remained asleep and IDs/material/mass were
preserved in every window, with 1,801–1,802 retained intervals per window.
Facing median intervals were 16.739–16.758 ms (59.67–59.74 FPS), away
16.731–16.744 ms (59.72–59.77 FPS). Across windows, p95 was 18.263–18.599 ms,
p99 19.502–19.716 ms and the maximum 20.551 ms. This is approximately the
proposed 60 FPS target, not a strict 60 FPS floor. Actual renderer metadata was
Apple M1 Metal, 1920×1080, DPI 2, AutoVsync. Its cadence differs from the earlier
1,000 dataset; FPS is not treated as a same-display comparison between loads.

One hundred identical-snapshot solver comparisons measured median stateless
time 2,518.251 ms versus sleeping time 0.198813 ms. Sleeping integration, pair
visits and matrix allocations were zero, versus stateless medians of 1,000,
7,984,000 and 1,002 respectively. Native physics medians were 0.437–0.443 ms,
CPU rendering 3.488–3.521 ms and asynchronous GPU timestamps 2.155–2.310 ms.
The [six-window summary](native-500-dense-settled-2026-10-12-summary.json) and
[complete compressed raw samples](native-500-dense-settled-2026-10-12.json.gz)
retain the measurements, counters and accounting. Dense settled 500 and 1,000
now supply all twelve required primary/stress windows. Subsequent low-load
matrix results and the remaining case count are recorded below.

The fresh 500 interaction route separately reached all sleeping at 1,056 updates
(2,313.753 preparation wall seconds). Loaded flight/landing/takeoff preserved
exactly zero local drift and all 625 kg. F pickup removed support 162; dependent
238 fell over 1 cm, and all 34 objects outside the original component remained
asleep. The [support-removal partial](native-500-support-partial-2026-10-12.json)
retains those successful checks and the subsequent failed release. The blind
forward/left walking search reached the narrow nose/outer hull, where the
production placement validator rejected the carried footprint.
The complete carrying route remains failed.

The harness now computes ordinary look inputs toward ship-local waypoints from
actual inspection data, walks around the Core, tries both strafe directions,
and faces a floor gap before every release. It never installs player or
fragment poses or sleep flags. Five analytic look/axis/precision regressions
bring Python tooling to 55 passing tests. A first 100-fragment probe released
successfully but failed a repeat release; correcting the held-offset view reset
then passed the [full native 100 route](native-100-floor-navigation-2026-10-12.json)
with all three cycles. A fresh 500 interaction route then used the final surface
build with this corrected harness; its completed outcome follows.

That 500 route subsequently **passed**: all 500 settled naturally after 1,056
updates, loaded fast rotating flight/landing/takeoff preserved all IDs, 625 kg
and zero local drift, real F removal of support 162 made dependent 238 fall,
all 34 fragments outside the original component stayed asleep, and three
pickup/drop cycles preserved identity and accounting. Preparation wall time
4,384.476 s and total route time 4,420.072 s include the earlier measurement
pauses; simulation settling remained 16.896 s. The
[complete retained route summary](native-500-floor-navigation-2026-10-13.json)
and [full checkpoint archive](native-500-floor-navigation-2026-10-13.json.gz)
retain the actual state evidence and hashes.

The older-harness 1,000 retry reached natural sleep at 1,586 updates and passed
rigid flight/landing/takeoff, real F removal of support 515, falling dependent
743 and all 11 unrelated fragments remaining asleep. A repeat pickup selected
another identity, so the **complete route failed**. Its
[partial summary](native-1000-support-partial-2026-10-13.json) and
[full checkpoint archive](native-1000-support-partial-2026-10-13.json.gz)
preserve that failure. A fresh 1,000 route used the floor-navigation harness
that passed the complete 500 route, but its preparation was terminated before
readiness. It remains unverified at 1,000.

The latest staged surface build passed 353 workspace Rust tests (one existing
GPU test ignored), 55 Python tooling tests, workspace Clippy with warnings
denied and formatting/diff checks. All 17 native scenarios / 1,712 steps passed
after rerunning two first-sweep assertion failures. The
[surface validation manifest](native-validation-2026-10-12-surface.json) records
its distinct executable hash, failed first attempts, retries and build limits.
The earlier [isolation validation manifest](native-validation-2026-10-12-isolation.json)
retains that build's separate provenance. The earlier
[point-envelope validation manifest](native-validation-2026-10-12-point.json)
records the production physics build used for the partial loaded routes above.

The additional [settled low-load summary](native-matrix-settled-low-2026-10-12-summary.json)
and [compressed complete raw data](native-matrix-settled-low-2026-10-12.json.gz)
record 108 passing windows: counts 0/10/25/50/100/250, all three layouts,
both cameras and three repeats. Every window retained at least 100 frame
intervals, preserved accounting and retained the complete population asleep.
The final surface executable was used, with 1 s warmup, 5 s requested sampling,
natural preparation bounded by 2,400 production updates, and isolated sequential
measurement after each preparation batch. The two large interaction preparation
processes were suspended during collection; that pause grants no simulation
time and remains included in their total wall-clock deadlines.

The [active low-load summary](native-matrix-active-low-2026-10-12-summary.json)
and [compressed complete raw data](native-matrix-active-low-2026-10-12.json.gz)
record 90 passing fresh-process windows: counts 0/10/25/50/100, all layouts,
both cameras, three repeats, zero warmup and 5 s requested sampling. Accounting
and at least 100 native frame intervals passed for every window. These fixtures
naturally settle during collection: all 72 nonempty windows were mixed, retaining
30–98 moving samples each. Moving/resting counts in the summary constrain
interpretation of their full-window distributions; fewer than 100 moving samples
do not establish reliable active-only tails. The suspended interaction preparations resumed
after these isolated windows completed.

The [active 250 partial summary](native-matrix-active-250-partial-2026-10-13-summary.json)
and [raw archive](native-matrix-active-250-partial-2026-10-13.json.gz) retain
11 completed isolated windows from before the scheduler pause: three repeats
for each scattered view and dense facing, plus two dense-away repeats. The
later resumed windows are excluded because of occlusion and concurrent gaming.
Dense moving contact work is expensive: the first two facing repeats took
119.383 and 214.170 s to retain 100 frame intervals, with p95 frame intervals
5,245.295 and 4,862.117 ms dominated by physics. Their near-60-FPS medians after
settling do not describe initial-motion responsiveness.

Together the retained datasets cover 73 complete distinct cases / 219 windows,
plus two completed windows of one still-incomplete case (221 valid windows
total). Remaining acceptance requires the complete 1,000 carrying route and
23 reference matrix cases, one already having two repeats. Dense 500/1,000 natural
sleep, loaded rigid flight and identical-snapshot baseline comparisons are now
retained above. The complete matrix still requires all 96 cases
with three windows and at least 100 retained samples per window. Diagnostic
continuations, partial preparations and timeouts do not satisfy those gates.

The [first local high-load start-gate failure](native-matrix-start-gate-failure-2026-10-13.json)
records end-of-input after its
four fixtures had naturally prepared, before any native measurement window.
That scheduling failure supplies no performance result. Interactive stdin was
then verified with a real `MATRIX_START` round-trip and the retry uses a PTY,
with a 86,400 s per-case orchestration deadline to retain prepared fixtures while
waiting for an idle display. The ordinary simulation clock and sleep criteria
are unchanged. The active 250 group also remains incomplete: the resumed
windows were occluded, and League of Legends was again using substantial CPU/GPU.
Even its resumed window that did not report occlusion is excluded from reference
claims because of that concurrent game. Further performance collection requires
an idle desktop. The user subsequently stopped the task, so no preparation or
measurement remains running.

### Exact completed native invocations

These commands used the local app-bundle copy of the staged executable with
SHA-256 `95e662d5aeea7cec6a8913ba59adbfe51f84d70eaf5f3cfeb6315bf106cc2d68`.
The bundle permits native UI inspection; the executable bytes match staging.
Other preparing Salimon processes were suspended during measured windows.

```sh
python3 scripts/benchmark_fragments.py --binary artifacts/SalimonAcceptance.app/Contents/MacOS/salimon-client --machine-label apple-m1-imac-reference --mode native --seed 155 --counts 0 10 25 50 100 250 --layouts scattered dense surface --states settled --cameras facing away --repeats 3 --warmup-seconds 1 --sample-seconds 5 --minimum-samples 100 --settle-steps 2400 --prepare-workers 4 --timeout 14400 --output artifacts/fragment-benchmarks/issue156-matrix-settled-low-bundled
python3 scripts/benchmark_fragments.py --binary artifacts/SalimonAcceptance.app/Contents/MacOS/salimon-client --machine-label apple-m1-imac-reference --mode native --seed 155 --counts 0 10 25 50 100 --layouts scattered dense surface --states active --cameras facing away --repeats 3 --warmup-seconds 0 --sample-seconds 5 --minimum-samples 100 --prepare-workers 1 --timeout 14400 --output artifacts/fragment-benchmarks/issue156-matrix-active-low-bundled
python3 scripts/fragment_sleep_acceptance.py --binary artifacts/SalimonAcceptance.app/Contents/MacOS/salimon-client --counts 500 --layout dense --settle-steps 2400 --timeout 14400 --request-timeout 120 --carrying --output artifacts/issue156-native-floor-navigation-500
```

Results: 108 settled windows passed; 90 active low-load windows passed; complete
500 loaded correctness passed. The active 250 invocation used the same active
options with `--counts 250`; only its eleven explicitly retained isolated
windows count, not the complete invocation. Pending high-load measurement and
1,000 correctness retries are not completed commands.

The benchmark build adds optional `--minimum-samples 100`, requiring both the
requested wall duration and 100 retained native frame intervals before a window
completes. It records actual sample duration and moving/resting sample counts;
production physics and update timing are unchanged. The
[sampling-build validation manifest](native-validation-2026-10-12-sampling.json)
records its distinct hash, 351 passing Rust tests (one existing ignored GPU test),
50 Python tests, Clippy/format checks and 17 native scenarios / 1,712 steps.
The original partial loaded results use the point-envelope binary. Subsequent
sampling, isolation and surface builds retain that physics implementation;
each dataset above identifies its own executable and harness revision.

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
