# Issue #156 — persistent fragment sleep and contact-island waking

## Outcome

Reviewable implementation of stable-ID sleep at the portable physics boundary,
composed into production fragment updates. Visible/pickable session entities,
mass, carrying limits, f64 frame conversions and in-memory persistence remain
intact. No cargo cap, deletion, full engine or rendering change.

**Native acceptance evidence remains incomplete.** This environment is not the
Apple M1 reference machine required by #155/#156. Leave #156 open until the
native matrix and verified settled dense 500/1,000 cargo windows are recorded;
the PR relates to the issue without automatic closure. CPU isolation evidence
below establishes work reduction, not native FPS or whole-cabin settling.

## Changes and correctness

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
