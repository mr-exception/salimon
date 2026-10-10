# Issue #154 — aggregate fragment scalability status

## Outcome

**Blocked; performance acceptance is not complete.** This report consolidates
the evidence and dependency state at main `a7de43a` on 2026-10-10 UTC. It does
not implement an optimization or establish a native frame-time guarantee.

The profiling infrastructure from #155 is merged through #162. GitHub marks
#155 completed, but its [merged report](../issue-155/README.md) explicitly records
incomplete acceptance and says the required native baseline does not yet clear
#157. Issue closure alone is not measurement evidence.

## Dependency and acceptance state

| Task | Observed state | Remaining evidence or work |
| --- | --- | --- |
| #155 profiling | Closed; infrastructure merged | M1 reference matrix, verified settling, sufficient samples, active budgets and loaded interaction routes remain incomplete in the report |
| #156 sleeping/waking | Open | Persistent activation, support-aware wake propagation and measured settled-work reduction |
| #157 spatial candidates | Open | Conservative deterministic candidates, sparse cache, reference coverage and allocation/work comparison |
| #158 resource instancing | Open | Resident geometry, compact transforms, conservative culling and visual/upload/CPU comparison |
| #159 bounded scheduling | Open; depends on #156/#157 | Explicit time accounting, stall/fast-impact tests and measured overload/tail behavior |
| #160 proxies/detail | Open; conditional after primary results | Benchmark-backed keep/defer/change decision; no asset changes justified yet |
| #161 Rapier evaluation | Open; conditional after solver results | Benchmark-backed keep/defer/adopt recommendation; no production migration authorized |

All issue states above were read from GitHub. No child is marked complete by
this change. Follow [repository blocker policy](../../AGENTS.md) before dependent
implementation; retain #154 as open until its complete acceptance matrix passes.

## Existing aggregate evidence

The [48-case CPU summary](../issue-155/cpu-summary.json) and its original report
are historical measurements of the #155 working tree, not fresh measurements
at `a7de43a`. Linux container CPU timings exclude rendering and startup hull
construction. Zero warm-up and very short windows do not establish settled
loads or reliable large-load p95/p99 budgets.

| Dense ship case | Samples | Median physics ms | Median contact ms | Median moving objects |
| --- | ---: | ---: | ---: | ---: |
| 500, requested settled | 3 | 820.10 | 816.18 | 487 |
| 500, initially active | 3 | 832.35 | 826.10 | 500 |
| 1,000, requested settled | 2 | 1859.68 | 1849.41 | 987 |
| 1,000, initially active | 2 | 1946.19 | 1937.26 | 1000 |

Contact processing dominates these sampled CPU updates. Recorded pair visits
are 3,992,000 at 500 and 15,984,000 at 1,000 per update. Requested matrix/projection
capacity is approximately 16.30/57.97 MB respectively; this is not total heap
allocation. Source inspection confirms all-pairs contact passes and per-frame
resource vertex expansion/upload remain in production. CPU evidence cannot
identify GPU bottlenecks or certify native FPS.

## Work needed to unblock and finish

1. Run the [native benchmark command and matrix](../../scripts/FRAGMENT_BENCHMARKS.md)
   on the Apple M1 iMac reference at 1920×1080/AutoVsync/F3 off. Record hardware,
   RAM, OS, refresh, thermal/power conditions and exact revision/binary hash.
   Obtain verified settled and active windows, at least 100 samples per case and
   three repeats; calibrate active budgets and baseline tolerance.
2. Complete #156/#157/#158 against that baseline, preserving stable IDs, material,
   mass, one-object carrying, convex pile accuracy, persistence and f64 frame
   precision. Compare candidate/allocation counters separately from timing.
3. Complete #159 using those results. Account for elapsed time and backlog;
   validate injected stalls and fast impacts without hidden time discard.
4. Resolve #160/#161 with measured decisions. A justified defer is permitted;
   issue metadata does not justify asset changes or a solver migration by itself.
5. Record comparable before/after 0/10/25/50/100/250/500/1,000 results here, with
   median/p95/p99 frame and active-drop budgets, facing/away rendering, and loaded
   support removal, repeated pickup/drop, ejection, ship movement, full landing/
   takeoff and streaming-return validation. Report remaining scalability headroom.

Software Vulkan CI can validate correctness and rendering; it cannot substitute
for reference-machine performance. No small cargo cap, fragment deletion,
visibility-based suspension of active physics or collision simplification is
accepted as a shortcut.

## Validation of this report

- Passed: relative Markdown link/path checks and aggregate table comparison
  against the checked-in CPU summary; `git diff --check`.
- Reviewed: #154–#161 issue metadata, #155 comments and merged report, current
  architecture/maintenance/validation guides, physics and resource upload source.
- Not run: new benchmarks, Rust/Python gameplay tests or native graphical tests;
  this change edits documentation only and makes no new runtime claims.
- No screenshots: no graphical behavior changed or newly validated.

The native reference data and unfinished primary child tasks are the explicit
blockers. This PR relates to #154 and must not close it automatically.
