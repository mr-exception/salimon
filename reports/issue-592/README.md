# Exact contact stress (#592)

Fifth change in the requested stack. Source changes make the existing six small
math primitives inlinable across crate boundaries and elide redundant one-point
impulse iterations only at an **exact IEEE-bit fixed point**. The sixteen-step
response policy remains: if a full iteration leaves all linear/angular velocities,
normal/tangential impulses and rolling torque bit-identical, every remaining
iteration has identical inputs and repeats that state. Normal, arms, targets,
mass/inertia and geometry are constant in this pure loop; position/environment
processing stays outside it. Multi-point manifolds retain the original loop.
No tolerance, damping, sleep/contact/mass rule, elapsed time or population changes.

`iteration_oracle.rs` retains the original sixteen-iteration ground/pair routines
for exact serialized response and warm-continuation comparisons. Existing spatial
all-pairs, correction-created, retained projection and sleep checks remain.
Physics/math owner guides, runtime diagnostic contract and validation are updated.

## Exact replay and profiling boundary

The serial replay uses disposable archived geometry at
`2ecb271ff4af43ad102367cdea91c8cb08e375ec`, identical to the #559 fixture boundary.
Production gameplay keeps the current Scout layout. Current production stateful
physics receives the archived ID/resource cycle and exact mass/radius bits.
Platform libm-derived adapter radii are recorded distinctly from restored solver
bits. This CPU replay does not certify whole production-adapter cost or frames.

Actual host: macOS 26.2 / Apple M1, Rust 1.89 release. This is a new measured host;
Linux #559 and historical native #455 failures remain separate original evidence.
No builds/render workloads overlap the timed lanes. Completed steps flush to
JSONL before the next call, retaining every timeout/partial/spike. Per-query
clocks are included in contact time; activation/integration have separate clocks.
Clocks/counters and between-step JSONL I/O are declared profiling overhead.

A separate five-second macOS `sample` diagnostic, stopped intentionally after
sampling, attributes the remaining workload to pair/ground impulse loops and
out-of-line component arithmetic. It is excluded from serial timing distributions.
The first inlining-only candidate improves costs but still times out at 120s;
all its original records are retained as `after-inline`. Fixed-point work elision
is the additional change that allows the focused 1,000-mixed moving run to
complete all 100 actual moving intervals within the original 120s bound.

Reproduction: [reproduce.sh](reproduce.sh), [collector](collect.py),
[exact comparison](analyze.py), frozen fixture/prefix/optimization patches and
source/binary hashes beside this report. The diagnostic 32-sample terminal lane
is labeled `diagnostic-prefix`; it cannot qualify as a completed moving pool.
Terminal comparisons preserve float bits including signed zero, not rounded
presentation values. The standard target remains 100 completed samples.

## Preserved failures and limits

Historical #455 native 120s/45s timeouts, 492/500 asleep after 300 steps, startup
frame-tail/cadence failures and cap p95 above guidance remain failures in their
original reports. Natural preparations still use real production steps and the
exact 300-step limit, with no installed sleep flags. Missing settled distributions
remain failed/incomplete. This optimization does not approve a population cap,
60 FPS, Metal frame behavior or a hardware budget. Physical checks remain #557;
actual hardware frame-tail/cadence/stress acceptance remains #563.

Final serial results, exact comparisons, repeated focus trace and all validation
outcomes are recorded below after the complete measurement set. No PR is merged
by this task.

## Final serial results

| Moving pool | Samples before/after | Total p95 ms before/after | Contact p95 ms before/after | Query p95 ms before/after |
|---|---:|---:|---:|---:|
| 500-cap-moving | 100/100 | 5.479/1.767 | 1.316/0.908 | 0.786/0.710 |
| 500-mixed-moving | 100/100 | 49.248/16.513 | 26.336/10.936 | 9.126/8.378 |
| 1000-cap-moving | 100/100 | 36.037/8.313 | 31.942/6.982 | 3.574/3.419 |
| 1000-mixed-moving | 53/100 | 5251.987/1309.552 | 5145.900/1265.173 | 664.579/878.039 |

The baseline 1,000-mixed trace is a **partial 120s timeout with 53 samples**;
its percentile describes only that retained prefix. The inlining-only candidate
also times out, retaining 83 samples. Final exact-fixed-point runs both complete
100 moving samples: 83.188s and 82.949s wall time. Final total p95 is 1309.552ms;
this remains far above a realtime 16.7ms frame interval. This is a bounded CPU
moving-pool completion, not a native frame-budget pass.

[Exact main comparison](comparison.json) preserves archived inventory bits,
per-step counters and every completed old terminal state. The separately labeled
[32-sample terminal prefix](prefix-comparison.json) matches the old solver at
all float bits, including signed zero. The [repeat comparison](repeat-comparison.json)
matches all 100 optimized steps and the complete terminal state between two runs.
No timing field participates in deterministic equivalence.

Both cap controls naturally prepare and retain 100 sleeping samples. The mixed
500 preparation still fails at 492/500 after 300 steps. The mixed 1,000 natural
lane still reaches its 45s timeout; its missing sleeping distribution is
**incomplete**, not a pass. See exact preparation counts in raw rows and
[before](summary-before.json)/[after](summary-after.json). These outcomes do not
change the preserved historical native or balance failures.

Raw logs, all samples, intermediate failure and original profile are in the
[compressed archive](raw-evidence.tar.gz), with [complete inventory](raw-inventory.json).
Executable hashes are inventoried; binaries are reproduced from frozen patches
rather than committed. [Context](context.json) binds source/host/compiler.

## Required checks

Workspace tests/build, warnings-denied Clippy, formatting/diff and per-parent
 documentation gates pass. Physics has 74 passed / two opt-in ignored, including
 the retained original response, varied axes/surfaces/masses, convex fallback and
 warm continuation; shared math tests and all runtime/renderer/domain tests pass.
The full source patches apply in order to an isolated index of the archived
revision and reconstruct every optimized/harness file exactly. Original logs
are in [validation](validation/). Universe/gameplay behavior is unchanged;
math/physics/runtime guides and maintenance/validation mapping are maintained
in this PR. Source rights, art/balance approval and hardware acceptance are not
inferred from solver completion.
