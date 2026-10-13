# Issue #159 — bounded shared gameplay ticks

Implementation is ready for review. Native Apple M1 reference-matrix and visual
acceptance remain **unverified**; this report does not certify full issue acceptance.
The dependency issues #155/#156/#157 are closed; #157's implementation is present
in baseline `421500d6519765db4b780ea84847fa257ab531b5`.

## Changes and overload policy

- All native gameplay domains share 16 ms ticks. Redraw admits at most two ticks,
  checking a 4 ms catch-up budget after the first. A solver call is not preemptible;
  one dense active tick can still exceed the wall budget substantially.
- Observed elapsed time is retained as debt instead of increasing physics delta
  or silently dropping a stall. Sustained overload slows all gameplay relative to
  wall time; bounded recovery drains the debt. Lifecycle resets explicitly clear
  queued debt and history and report the excluded debt. No cargo/time cap or
  fragment deletion is introduced.
- Convex motion targets are reevaluated after each response, including gravity
  speed. A fast deck impact stops imposing its initial timestep once it slows.
  Sleeping islands retain exact poses and skip integration; all collidable bodies
  still share synchronized time and correction-refresh semantics.
- Runtime interpolates loose-object f64 centers and normalized shortest-sign
  quaternions in their simulation frame. Ship-local history follows the current
  ship frame. New/changed/carried/transferred/released objects bypass incompatible
  history; pickup/drop remains authoritative and release clears history immediately.
  Native loose-object presentation can trail by up to one 16 ms tick. Ship and
  camera controllers still render their authoritative poses; full scene smoothing
  is outside this implementation.
- Native benchmark work/time sums both ticks; zero-tick renders count no repeated
  work. JSON adds tick count, retained backlog and cumulative lifecycle-excluded
  debt. CPU-only samples report null native scheduler fields.

Independent active-island stepping was evaluated and not introduced. Swept
motion and positional corrections can merge previous islands during a solve;
initial bounds alone cannot guarantee isolation. Safe merging/cache remapping or
rollback needs a separate design and measurements. An unrelated **awake** body
still shares the minimum current timestep. See the maintained
[physics evaluation](../../client/physics/architecture.md#active-group-evaluation-159)
and [shared-clock/interpolation contract](../../client/runtime/architecture.md#bounded-gameplay-ticks-and-fragment-interpolation-159).

## Validation

Ubuntu 24.04 x86_64 container, AMD EPYC 9V74 virtual CPU (9 visible logical CPUs),
Rust 1.99.0 (`b940084d7`), Python 3.12.14. No native GPU/window/reference Mac available.
Dependencies/toolchain were installed for this session. An initial build directory
had corrupt `naga` metadata; the clean target-directory run below passed.

```sh
CARGO_TARGET_DIR=/tmp/salimon-159-target cargo test --workspace --locked -j 2
CARGO_TARGET_DIR=/tmp/salimon-159-target cargo clippy --workspace --all-targets --locked -j 2 -- -D warnings
CARGO_TARGET_DIR=/tmp/salimon-159-target cargo build --workspace --locked -j 2
cargo fmt --all -- --check
python3 -m unittest discover -s scripts -p 'test_*.py'
git diff --check
```

Passed: **367 Rust tests**, four existing ignored mechanism/cost tests;
**55 Python script tests**, Clippy, build, formatting and whitespace checks.
After presentation-map capacity reuse, both interpolation tests and workspace
Clippy passed again. The native scheduler's injected two-second stall drained
through batches of at most two ticks, retained exact advanced + queued time, and
matched direct shared fixed-tick ship/fragment states throughout. Regression
coverage includes 500/1,000 sleeper populations with a fast deck impact, remote
cargo remaining asleep/unchanged, floor/pile contacts, support removal, retained
projection equivalence, mass/IDs, one-object carrying, streaming, flight and frame
precision. Fixed automation scenarios' CPU contracts passed; graphical execution
is a separate gate.

Local graphical/native staging checks were not run: no installed display/Vulkan
stack. Package installation was blocked by container permissions. GPU/FPS,
visual jitter, native pickup/drop appearance, M1 1920×1080 facing/away and authored
dense 500/1,000 settled/active matrix remain pending. No substitute CPU probe
establishes those requirements. This PR references #159 without closing it until
those criteria are verified or the user explicitly revises acceptance.

## CPU timing probe

Results below use the portable production solver on scattered mixed-mass cubes,
not authored dense cargo. Exact IDs and masses remain intact. Settled bodies
sleep naturally after 40 preparation ticks. One-impact rows change one velocity
to -40 m/s on its deck; active rows drop every body at -40 m/s from 1 m. Each row
uses ten unrecorded warm-up replays and 100 timed independently reset 16 ms replays,
excluding setup/cloning. Revisions were measured sequentially with other local
build/test jobs finished. Shared-host scheduling remains a source of noisy tails.

```sh
# Copy the unchanged tick_cost example into a detached baseline worktree.
CARGO_TARGET_DIR=/tmp/salimon-159-old-correct cargo build --release --locked -p salimon-physics --example tick_cost
CARGO_TARGET_DIR=/tmp/salimon-159-release cargo build --release --locked -p salimon-physics --example tick_cost
/tmp/salimon-159-release/release/examples/tick_cost > after.csv
/tmp/salimon-159-old-correct/release/examples/tick_cost > before.csv
```

The example is checked in at
[`client/physics/examples/tick_cost.rs`](../../client/physics/examples/tick_cost.rs).
Use [the benchmark guide](../../scripts/FRAGMENT_BENCHMARKS.md#tickbacklog-diagnosis-159)
for reproduction and the still-required native matrix.

| Load / state | Before p95 / p99 (ms) | After p95 / p99 (ms) | Substeps before → after |
| --- | ---: | ---: | ---: |
| 500 / settled | 0.094 / 0.096 | 0.093 / 0.144 | 0 → 0 |
| 500 / fast_drop | 123.606 / 131.687 | 31.627 / 36.831 | 22 → 3 |
| 500 / active | 137.722 / 152.954 | 137.401 / 148.051 | 22 → 22 |
| 1000 / settled | 0.254 / 0.430 | 0.220 / 2.248 | 0 → 0 |
| 1000 / fast_drop | 259.183 / 280.750 | 64.619 / 69.396 | 22 → 3 |
| 1000 / active | 277.747 / 285.761 | 347.950 / 376.108 | 22 → 22 |

Raw [before](before.csv) / [after](after.csv) CSVs include medians and counters;
[metadata](measurement.json) records source/executable identity and limits.
The one-impact p95 decreased approximately 74–75% on this CPU probe; substeps and
integrations dropped from 22 to 3. Remote sleepers stayed unchanged. Fully active
work remains expensive and has no demonstrated general improvement; both revisions
integrate 500/1,000 awake bodies over 22 substeps. The fully active 1,000 p95 was
25% higher in the retained after window (348 versus 278 ms); treat this as a
regression signal requiring repeated/native investigation, rather than claiming
efficient high-load active physics. Settled integration remains zero.
The 1,000 settled after p99 includes a host scheduling spike; wall-clock tails
are noisy even when work counters match. No native FPS gain, M1 acceptance,
independent active-island isolation or hard per-frame time guarantee is claimed.

