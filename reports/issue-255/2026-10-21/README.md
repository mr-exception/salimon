# #255 — refreshed legacy onboard M1 evidence

Native measurements collected 2026-10-21, stacked after PR #478. This delivery preserves the existing budget proposal and historical failures. Remaining owner budget approval and any unresolved reference-machine gates transfer to [#479](https://github.com/mr-exception/salimon/issues/479). Closing #255 does not grant performance acceptance.

## Scope and provenance

Actual iMac21,1, Apple M1 8 CPU/8 GPU, 16 GiB, macOS 26.2 (25C56), AC power; native Metal, 1920×1080 physical, scale2, AutoVsync, built-in nominal59.995 Hz. Rust1.89.0; Python3.14.2 (3.12 unavailable). Raw focus records verify the exact frontmost target PID in addition to runtime native visibility/presentation admission. All windows were collected strictly serially, without concurrent game/build/compression jobs.

Current binary SHA-256 `1703da38f7109332542a28acde9d896df1eae9af2f18fd20ba69619bf4f48762`, runtime source `c691e6b`, collection checkout `18b28ff` (subsequent commits change reports/follow-up metadata only). Separate `target-production` build; default native benchmark behavior, E2E input isolation is inapplicable. Summary `dirty:true` includes unrelated untracked readiness reports; tracked source was clean for collection. The rejected cache-contaminated `cbba4ab2…` build is never used here. [Binary lineage](binary-provenance.json), [prior source/build verification](../../issue-313/README.md).

This is the legacy `onboard`, seed155, **500×0.032 kg =16kg** fixture, with stable IDs and ordinary gravity/contact/sleep. It is separate from #313's current 10 kg cap/mixed spheres and #455's unresolved stress gates. No motion forcing, sleep installation, physics weakening or budget replacement.

The exact historical binaries are retained independently: before `a7e2bdbec8d32dff611e870daebf2b42465c507c7c1277c737203164f83dae4b` (source `1da0d95`), optimized `98ec9624dcbecf557e1a1e97d93a115c9d19e0610e585674492b0b98cdfd30de` (implementation `0b9ff35`, evidence `77cda71`). Their fresh paired sleeping windows reproduce the historical comparison scope. They are not an old/current-physics comparison: time-based damping and other later domain changes are absent from both old binaries.

## Results

Admission, target-PID focus, undecimated samples, exact workload configuration, renderer, mass and IDs all verify. All seven collection stages complete without rejected cases. [Machine-readable analysis](analysis.json) preserves p95/p99/max, every window and proposal check; [raw data](native-data/) retains resting intervals as well as moving intervals.

| Current native sample | Retained completed intervals | Frame p95 (ms) |
| --- | ---: | ---: |
| Empty, three 30 s windows | 1,800 / 1,801 / 1,801 | 17.079 / 17.097 / 17.105 |
| Naturally sleeping 500 onboard, three 30 s windows | 1,801 / 1,801 / 1,801 | 17.094 / 17.108 / 17.096 |
| Active 500 onboard, ten fresh launches | 150 **moving** intervals | 23.446 |
| Matched empty launch prefixes | 150 intervals | 22.034 |
| Naturally sleeping larger-piece scattered stress, 5 s | 301 | 17.108 |

Only completed intervals with `moving_objects > 0` enter active tails; the first null frame and later resting intervals remain raw and are excluded. All ten active drops and empty launches use no warmup, three-second windows and minimum100 total samples per launch. The 150 moving intervals are a contiguous launch prefix in each loaded case; the empty reference takes exactly the same number of completed launch intervals. No pooling across configurations/states or selecting away spikes.

Active frame p95/p99/max is **23.446 / 30.477 / 33.947 ms**, with **zero intervals >50ms**. Physics+adapter p95 5.932 ms, CPU-render p95 0.422 ms, asynchronous latest-GPU p95 3.562 ms; active backlog maximum38.430 ms. Active/empty-prefix frame-p95 ratio1.0641. Structural allocation counters and raw phase/substep/pass metrics are available; they do not measure total process/GPU residency.

Every numerical check in the **unchanged proposed** #288 budget passes: sample count, 33.333 ms frame p95, 50 ms maximum, 8 ms physics, 4 ms CPU-render, 12 ms GPU, 50 ms backlog, ratio1.2, and zero>50ms intervals. **The proposal remains pending owner review; acceptance_granted=false.** The active frame p95 still exceeds the17.5ms settled guidance and physics p95 exceeds the historical provisional5ms partition. Passing proposed values does not change those historical guides or approve a strict60FPS floor.

Larger mixed-piece scattered preparation settles naturally after **93 updates /1.488 simulated seconds /11.190 wall seconds**, inside the unchanged120-update bound; raw preparation log is [here](native-data/stress/raw/native-500-scattered-settled-facing.stderr). This is the legacy scattered fixture, not #455's1,000 current-mixed timeout. All retained stress/reference intervals observe sleep; no installed sleep flags.

## Exact historical reproduction

Three30s naturally sleeping windows per historical binary retain sufficient samples and matching mass/IDs/config/renderer. Before/optimized frame p95 pairs are17.073/17.061,17.083/17.067,17.069/17.055ms. CPU-render pairs are0.905/0.905,0.910/0.907,0.902/0.907ms; physics pairs1.273/1.253,1.293/1.247,1.278/1.228ms. **All fresh20% relative checks pass.** Raw [before](native-data/historical-before/) and [optimized](native-data/historical-after/) evidence is preserved.

This attempt does not reproduce the earlier same-state CPU-render regression. It does not establish why that older run failed. The [2026-10-17 remediation](../2026-10-17-remediation/README.md) retains its six whole-window relative failures, and [#288](../../issue-288/README.md) retains three additional same-state settled failures (22.0–24.7%). Those historical results remain failures at their recorded conditions; fresh passing measurements neither erase them nor attribute their cause. #479 owns the remaining owner decision and any required further attribution.

## Reproduction and verification

[collect.py](collect.py) and [native_matrix.py](native_matrix.py) record the exact serial sequence; their historical artifact-relative destinations describe this collection. Each lane also retains its full command and foreground-helper records. The canonical invocation is:

```sh
SALIMON_REFERENCE_SIZE=1 RUST_LOG=info python3 reports/issue-229/foreground_matrix.py --helper artifacts/native-20261020/input-313 --records <lane>/focus.json -- python3 scripts/benchmark_fragments.py --binary <verified-app-executable> --machine-label apple-m1-imac-16gb-macos26.2 --counts <0-or-500> --layouts <onboard-or-scattered> --states <active-or-settled> --cameras facing --minimum-samples 100 --warmup-seconds <0-or-2> --sample-seconds <3-or-30-or-5> --repeats <10-or-3-or-1> --timeout <90-or-100-or-120> --output <lane>/raw
python3 reports/issue-255/2026-10-21/verify.py --root reports/issue-255/2026-10-21/native-data --output artifacts/native-20261020/255-archive-reanalysis.json
```

Settled onboard/empty uses`--settle-steps1200`; scattered uses`--settle-steps120`. Helper source is in [#313](../../issue-313/input.swift). Exact CLI spacing/arguments are in each lane's`command.json`, not the prose shorthand. [Collection log](collection.log), [power](power-before.txt), thermal[before](thermal-before.txt)/[after](thermal-after.txt): AC, no recorded thermal/performance warnings. Source/runtime tests and strict lint for this exact staged binary passed in [#313](../../issue-313/README.md); this PR adds evidence/documentation only. No redundant gameplay tests or new screenshots are needed for unchanged rendering.

[verify.py](verify.py) rechecks raw native admission/OS-focus, accounting, configuration/renderer, undecimated moving samples, contiguous matched empty prefixes, fully sleeping reference windows, exact historical pairs and the existing proposal. All checks pass. The[SHA-256 manifest](sha256.json) checks retained archive bytes; gzip is lossless. Original summary filenames point to`.stdout`; their committed form is`.stdout.gz`. Archive reanalysis reproduces all metrics and checks.

After all native measurements, the task's temporary ablation worktree was removed and only the root release`salimon-physics`/`salimon-client` artifacts were cleaned to prevent reuse of the previously contaminated cache. [Cleanup log](cache-cleanup.log), [preserved ablation patch](damping-ablation.patch). Dedicated valid production/ablation staged binaries/evidence remain intact. Unrelated terrain-baseline worktree and untracked readiness reports are preserved.

Remaining: #479 explicitly owns budget approval and any additional attribution required for prior failures. No physics/sleep/accounting/budget value was loosened, no optical scanout or per-frame60FPS floor is claimed, and #455 current-sphere overload remains independent. Source #255 closes through its PR for refreshed evidence and tracked scope transfer.
