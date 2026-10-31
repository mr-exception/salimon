# Issue #435 — integrated universe and offline verification

Delivered the shared cloud verification for #432, #433, #434 and #441 in the
requested **single PR**. Functional gates and native routes pass. The actual
software-renderer benchmark **misses 60 FPS**; hardware/M1/Metal qualification
remains separate. No merge or release is performed.

Baseline `e68ccb8`; implementation parent `ed1725b` plus the recorded final
changes. [Verification manifest](verification.json) retains exact commands,
durations, original/final source and binary SHA256s, environment and revalidated
main/active-PR context. Captures retain their actual source/binary cohorts rather
than being relabeled as a later revision.

## Functional and source gates

Linux cloud x86_64, Rust/Cargo 1.99.1, locked debug build with the repository's
normal per-crate profile settings.

| Check | Actual result |
| --- | --- |
| Workspace build/test, all-target Clippy `-D warnings`, fmt, docs/base | Passed; **1,283 Rust tests passed, 51 ignored** in the full local run |
| Final corrected-source fixture / launch regressions | **5 / 26 passed**; includes the new fixture regression; final runtime build/Clippy/fmt passed |
| Final Python script tests | **147 passed** |
| Production native matrix | **21 routes, 1,383 steps, 61 captures passed**; 51 distinct original PNGs |
| Native SIGKILL / actual-clock restart | Passed, twice; exact source retention and no replay |
| Normal native OS-input smoke | Passed: OS Enter admission, both F2 transitions, native presentation |
| Frozen source/import/pack/publication checks | Passed; repeated full outputs and starting-world golden byte-identical |

Ignored exports/platform probes are not counted as passes. Lossless
[workspace log](tests.log.gz), [Clippy log](clippy.log.gz),
[final script log](final-scripts.gz), [fixture log](fixture-tests.gz) and
[launch log](launch-tests.gz) preserve their actual revision/context.
The final Rust-only changes after the full local run are the canonical render
fixture/regression, a successful-Continue log and corrected comments; focused
checks cover those changes. Current-head remote CI is reported on the PR.

One-hour production canonical coast versus 250-ms reference steps measured
**3.957114958855 m** position and **0.003570675019 m/s** velocity difference,
within **25 m / 0.05 m/s** budgets. [Raw numerical log](numerical.log.gz).
This is convergence evidence for the declared model/initial orbit, not a universal
trajectory/source uncertainty bound. Active/closed equivalence, migration,
crash/contact, funded scanner cadence/expiration, starvation and atomic late
failure are included in workspace tests.

Source checks ran the maintained importer, observed ingestion, inference and
fiction sanity/build, permanent starting-world examples and selected real JPL/PS
pack builder/verifier. Coverage: **6,451 planets, 4,847 hosts, 18 reviewed targets**.
[Full coverage](coverage-report.json.gz) retains source/manifest hashes, missing
measurements and distributions. Full Gaia acceptance remains #524; redistribution
review remains #659. Architecture review passed its warning-only gate with
**121 candidates and no new cycles**; [output](architecture-report.json.gz).
The local overlap probe lacked PR inventories; separate revalidation found no
other active PR and unchanged main before publication.

## Native evidence and interruption

[Native index](native-evidence.json) maps every exact scenario/step to lossless
state traces, retained original PNG paths, source/binary provenance, actual
backend/context and raw frame distributions. This includes flight/contact,
funded navigation previews, mining, boarding/EVA, cargo, scanner/projector,
player-known chart/history, Vesper night/vacuum, audio settings and resume.
No visual baseline is promoted. Frozen partial-progress captures are byte-identical.

The [restart trace](restart-result.gz) uses a real flying source held at **1.7%**
private progress. SIGKILL leaves source slot bytes unchanged and creates no public
staging save. Two new native processes reopen the same store using the actual
local clock: **1 s then 37 s** of elapsed credit, exactly matching physical epoch
deltas and one durable generation each. The aborted controlled +60 s never
publishes. Initial physical epoch is the explicit deterministic E2E fixture;
restart deltas use actual local time, not a current-date ephemeris accuracy claim.
Current-region readiness took **36.37 / 33.23 s**; process high waters were
**242,012 / 242,800 KiB**. Menu/region time and controlled fixtures remain explicit.

[Normal OS smoke](os-smoke.json) launches without E2E in an unrelated working
directory with isolated saves/settings, waits for verified Continue through
actual PID-scoped X11 Enter, then checks F2 in both directions. Original OS
captures: [startup](os-startup.png), [gameplay](os-gameplay.png). These are virtual
X11 desktop captures, separate from native GPU readbacks and physical device tests.
Audio settings use a declared fresh test file; no physical audio device was
available, so this proves settings/mixer contracts, not speaker playback.

Reproduce from the repository root with a working Vulkan/X11 environment:

```sh
python scripts/verify_universe_expansion.py --binary target/debug/salimon-client --output artifacts/universe-expansion
python scripts/verify_offline_resume.py --binary target/debug/salimon-client --output artifacts/offline-restart
python scripts/benchmark_canonical.py --binary target/debug/salimon-client --output artifacts/canonical-benchmark
```

The first command runs all 21 routes; repeated `--scenario NAME` records explicitly
scoped reruns. Portable/source exact argv and isolated OS-smoke environment are
recorded in the verification manifest.

## Actual normal-play timing and memory

The [benchmark](canonical-benchmark.json) and [raw frame log](benchmark-stderr.gz)
use **normal canonical play, no E2E pacing**, actual OS Continue, five seconds of
warmup, then **125 consecutive foreground frames over 30.23 seconds**. No other
CPU/GPU campaign ran during sampling. Backend: **Mesa llvmpipe LLVM 19.1.7,
software Vulkan CPU adapter**, timestamp queries supported. Drawable **1920×1080**;
virtual X11 desktop **1280×800** (the window extends beyond the desktop). Debug
profile and per-frame logging are instrumentation limits, not optimized hardware
acceptance. Verified Continue took **37.97 s**; sampled process high water was
**238,588 KiB**. Process RSS is not whole-system/GPU VRAM accounting.

| Measurement | Median | p95 |
| --- | ---: | ---: |
| Submitted frame interval | 236.828 ms | 250.000 ms* |
| Render/present wall time | 235.683 ms | 271.986 ms |
| CPU update | 2.436 ms | 3.623 ms |
| CPU render preparation | 10.029 ms | 34.166 ms |
| Software GPU pass timestamp | 212.635 ms | 251.377 ms |

Observed throughput: **4.13 submitted frames/s**. The **16.667-ms p95 / 60 FPS
comparison fails** on this software/debug workload. Acquisition itself passes;
that is distinct from passing the performance target. Simulation backlog reaches
**30.45 s** and is retained. `FrameClock` caps long interval observations at
250 ms (*), so those intervals are lower bounds where capped; uncapped wall time,
raw samples and counted throughput remain available. GPU timestamps measure the
software adapter's pass, not physical GPU counters or display scanout. GPU memory
outside the available allocator/context counters remains unqualified.

## Corrected failures and remaining qualification

Failed attempts remain in the native index with original traces/stderr:

- The terrain render fixture set mesh mode without the newly required motion
  source. It now uses the complete canonical initializer and has a regression.
- Scanner/projector/chart expectations assumed the earlier quiet offline adapter.
  Routes now require exact funded first-day pulse count/latest time, then still
  require contact expiry after actual Core exhaustion while retaining history
  and chart routes. No TTL, cadence, range, energy or numerical budget changed.
- One scenario-edit rerun accidentally targeted a zero-duration checkpoint;
  corrected the selector to the one-day interval and retained the original
  zero-duration selection assertions.
- The audio verifier omitted the explicitly required test settings path. It now
  supplies fresh isolated settings rather than touching personal preferences.

An initial format check found only numerical-log formatting; formatted before the
full passing gate. Coverage/architecture evidence output names initially collided
with stdout metadata; reran the same checks to distinct, validated JSON paths.
No failed attempt is relabeled as a pass.

Physical platform/input/audio and suspend/wake #557, human presentation/audio
#561, hardware budgets #563, source release review #659, balance #660 and
first-hour experience #661 remain separately unverified. The two-body fictional
motion publication does not imply full observed-body playable adoption or
stellar-interior physics; unsupported coasting blocks publication. Long moving
catch-up retains bounded chronological work and progress, without a universal
constant-time or year-long-flight performance guarantee.
