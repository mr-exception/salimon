# #313 — current spheres on Apple M1

Native verification collected 2026-10-20 and completed 2026-10-21. Delivery: [PR #456](https://github.com/mr-exception/salimon/pull/456), stacked on #454. Current fixtures, functional regressions and corrected native distributions are delivered. **Performance/stress acceptance is incomplete**, transferred to [#455](https://github.com/mr-exception/salimon/issues/455); closure of #313 does not approve a budget or turn failed/unrun gates into passes.

## Changes and provenance

Runtime `benchmark_fixture.rs` now shares canonical `current-cap` and `current-mixed` population construction with the CPU sphere matrix. The native CLI supports 500/1,000 current spheres and reports actual radius, mass and IDs. Legacy onboard layouts are unchanged. Cap spheres are 10 kg iron (radius 0.0880267056 m); mixed spheres cycle six masses from 1e-9 to 10 kg across iron/silicate/ice. No forced motion, sleep flags or weakened physics are introduced. Two pre-existing iterator lint failures were repaired without changing behavior.

Deterministic E2E runner children explicitly opt into desktop-input isolation: native keyboard/pointer events cannot alter scripted aim/holds, and focus/resize resets preserve fixed-step backlog. Inspection exposes this mode. Direct `Game` probes retain native input by default; normal play and benchmarks ignore this E2E-only setting. This fixes demonstrated external-input interference, and scenario passes do **not** certify actual OS delivery or live interpolation.

The first staged candidate accidentally reused the constant-damping diagnostic library through a shared Cargo cache. **All evidence attributed to `cbba4ab2…` is rejected**, retained in [rejected-cache](rejected-cache/STATUS.md). Actual production source was never reverted. Fresh separate production/ablation target directories, release physics regressions, source-file hashes and binary hashes establish the replacement provenance:

- Native timing binary `572e70b514a2a7deb98caaf0b6f15e8b29abc2d7c34bef5966e88b204d06c74c`; runtime code `fc634bf`, collection checkout `20d4b4f`. [Full provenance](verified-provenance.json), [release physics tests](verified-physics-tests.log).
- Functional replay binary `1703da38f7109332542a28acde9d896df1eae9af2f18fd20ba69619bf4f48762`, source `c691e6b`, adds only the E2E isolation change after the timing collection. [Build](isolated-build-info.json), [functional evidence](isolated-functional/).
- Controlled pre-#304 two-coefficient ablation `a2546a54…`, separate `target-ablation`, changes only the linear/angular retention expressions against the lint-fixed current source. [Provenance](ablation-provenance.json), [raw native window](ablation-isolated/). This diagnostic is not a historical whole-game baseline or permission to restore incorrect damping.

## Environment and commands

Actual iMac21,1, Apple M1 8 CPU/8 GPU, 16 GiB, macOS 26.2 (25C56), AC power, no reported thermal warnings. Native timings: Metal, 1920×1080 physical pixels, scale 2, AutoVsync, built-in nominal 59.995 Hz. Foreground admission combines renderer presentation metadata with the macOS helper's verified frontmost target PID. Runs are serial, without concurrent builds/compression/game jobs. Rust 1.89.0; Python 3.14.2 (3.12 unavailable). Functional captures: 1280×800, Metal, scale 2, fixed 16 ms steps; not live FPS evidence.

Reproduction uses `python3 scripts/benchmark_fragments.py --binary <staged-app-executable> --counts 500 1000 --layouts current-cap current-mixed --states active --cameras facing --minimum-samples 100 --warmup-seconds 0 --sample-seconds 3 --repeats <fresh-launch-count>` through `reports/issue-229/foreground_matrix.py`. Exact commands, focus records, failures and raw distributions are under [verified](verified/), summarized by [analysis.json](verified/analysis.json) and [analyzer](analyze_verified.py). Settled rows use `--states settled --settle-steps 1200 --warmup-seconds 2 --sample-seconds 10`.

Functional reproduction: `RUST_LOG=info python3 scripts/salimon-test run <scenario> --binary <isolated-build> --screenshot-command '["native"]'`; full resource-collection and terrain evidence groups plus report-local cap/carry, mixed, cap500/cap1000 and door-reversal routes. Results and PNGs are in [isolated-functional/full](isolated-functional/full/).

## Corrected results

| Workload | Completed moving intervals | Active frame p95 / max (ms) | Physics + adapter p95 (ms) |
| --- | ---: | ---: | ---: |
| 500 cap, 5,000 kg | 187 across 8 admitted launches | 26.185 / 152.350 | 6.169 |
| 1,000 cap, 10,000 kg | 117 across 5 admitted launches | 40.870 / 42.570 | 39.510 |
| 500 mixed, 1,019.406252084 kg | 100 | 133.331 / 185.907 | 132.665 |
| 1,000 mixed | incomplete, 60 s timeout | no accepted distribution | no accepted distribution |

The 500-cap spike and backlog are retained, without speculative attribution. Quiet but still-awake intervals are **not sleeping**: the 1,000-cap nonmoving frame p95 is 134.857 ms. Separate naturally sleeping cap windows each retain 601 intervals; frame p95 is 17.175 / 16.987 ms for 500/1,000. Structural matrix-allocation/projection counters are available in raw records; these are not whole-process/GPU memory allocation measurements.

The fresh separate-target constant-damping 500-mixed ablation retains 100 completed moving intervals, frame p95 111.945 ms and physics p95 111.007 ms. Both variants are expensive. This bounded comparison does not isolate all substep/contact causes. Release elapsed-damping and distant-fast-residual regressions pass; native full large inactive-history attribution remains unrun. The original CPU matrix was stopped at its recorded 903 s bound during the 1,000-mixed pair-contact case; no complete matrix report or native acceptance is claimed.

The isolated native functional replay passes all 25 scenarios: actual 10 kg pickup/drop/re-pickup/final Core conversion, mixed spheres, 500/1,000 cap capture, seven resource routes, thirteen terrain routes and 71-step door reversal. The Core route removes the final object without mass-ledger panic and restores base ship mass. Maintained routes cover cold terrain/resource traversal, extraction/session state, carrying, piles and bounded frame transitions. See [#312](../issue-312/README.md) for actual native flight/input/cargo checks and [#299](../issue-299/README.md) for dedicated Core access/outage checks; their original `a25fb874…` build was unaffected by cache contamination. No disk persistence or whole-session large-history stress signoff is inferred.

One valid pre-isolation carry replay failed because real desktop input changed scripted camera/movement. Its failed raw result remains under [verified/functional](verified/functional/); the isolated replay passes and exposes `native_input_isolated=true`. Earlier focus-failed, overlapping and invalid-cache attempts remain retained and excluded from accepted pools.

## Checks and remaining scope

- Shared fixture/actual mass/radius/ID tests pass. Full workspace before the E2E isolation change: 682 passed, 22 ignored. After isolation: client 261 passed, 5 ignored; strict workspace/all-target Clippy passed; 94 Python tooling tests passed; formatting passed. [Client](313-isolation-client-tests.log), [Clippy](313-isolation-clippy.log), [Python](313-isolation-script-tests.log).
- Separate-target release physics: 71 passed, 2 ignored, including elapsed damping and distant fast/small population regression. Ignored CPU benchmarks are opt-in, not native acceptance.
- Active stress tails/timeout, large inactive-history adapter attribution and any required remediation remain **failed/incomplete/unrun** in #455. #255 retains separate legacy onboard budget/signoff requirements. No budget was relaxed, no strict 60 FPS floor or physical scanout result is claimed.
- Hash manifests: [verified](verified-sha256.json), [isolated functional/ablation](isolated-sha256.json). Large JSON/log files are losslessly gzipped; recorded original artifact paths are historical paths, not additional committed files.
