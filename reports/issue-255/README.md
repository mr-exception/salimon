# Issue 255 — M1 cadence calibration and active-load failure

Latest: [active contact-path remediation and fresh matched M1 measurements](2026-10-17-remediation/README.md).
The evidence below records the earlier revision; its raw failures are preserved.

Executed locally on 2026-10-17, Apple M1 iMac21,1 (8 CPU / 8 GPU cores),
16 GiB RAM, macOS 26.2 (25C56), Metal, AutoVsync, 1920×1080 physical drawable
at 2× scale on the built-in Retina display. Secondary 1080p/75 Hz display attached.
AC power; `pmset -g therm` reported no thermal/performance warnings. Rust 1.89.0,
Python 3.14.2 (3.12 unavailable). No build/test job ran during performance windows.
Runtime/assets are source `1da0d9507eb720fc66c10475a25652e39d80555b`; this PR
changes reporting and guidance only. [Build digest](build-info.json),
[environment](environment.txt), [build log](build.log).

## Outcome

Cadence calibration and sufficient active-only sampling are complete. **Active
500-piece performance signoff fails and remains open**, tracked by
[#288](https://github.com/mr-exception/salimon/issues/288). This PR does not close
#255 or claim a strict 60 FPS floor. No physics, sleep, accounting or scheduler
rule was loosened. #254 foreground admission was required for every accepted
window; raw lifecycle records and explicit target-PID focus records are retained.

Three empty 30-second windows retained 1,800/1,801/1,801 completed intervals:
frame p95 **17.241/17.268/17.239 ms**, p99 **17.751/17.694/17.685 ms**.
Medians were 16.678/16.680/16.685 ms. The first window's **41.781 ms** maximum
is retained; other maxima were 18.903/18.703 ms. An empty scene already misses
16.67 ms p95, matching the prior independently focused #199/#229 observations.

The [canonical guide](../../scripts/FRAGMENT_BENCHMARKS.md#measurement-definitions-and-gates)
now explicitly sets reference 60 Hz submitted-present p95 guidance to **17.5 ms**:
one 60 Hz interval plus 5% scheduling allowance, slightly above the repeated
empty p95 envelope. This is an explicit reviewable calibration, not an inferred
scanout measurement. Existing relative 20% gates and provisional 5 ms physics,
4 ms other update, 4 ms CPU render and 12 ms GPU component budgets remain.
Maxima and >50 ms spikes are always reported. Other refresh rates need their
own calibration; active loads do not inherit a passing settled signoff.

## Active-only evidence

Ten fresh 500×0.032 kg `onboard` drops (16 kg total), seed 155, facing camera,
0 s warmup, 3 s measurement, minimum 100 completed intervals per launch. These
legacy small pieces are intentionally distinct from #236's 500×10 kg spheres.
Each population naturally settles; none is externally driven to prolong motion.
Only completed intervals with observed `moving_objects > 0` enter active tails.
Pre-admission work and the first interval without a predecessor are excluded;
this is admitted moving-state coverage, not timing of every initial setup update.

**177 moving intervals** across the ten independent launches provide sufficient
pooled active sample coverage (individual drops contain only 16–20 moving
intervals). This is a pooled drop distribution, not a 100-frame continuously
moving window or independent IID statistical confidence claim. Resting samples
are retained in raw files but excluded from active percentiles.

| Active metric | Median ms | p95 ms | p99 ms | Max ms |
| --- | ---: | ---: | ---: | ---: |
| Presented-call interval | 54.521 | 71.250 | 71.874 | 72.666 |
| Physics including adapter | 53.249 | 70.364 | 71.023 | 71.672 |
| Contact phase | 44.193 | 58.712 | 59.405 | 59.518 |
| CPU rendering | 0.289 | 0.359 | 0.572 | 0.753 |
| GPU latest asynchronous pass | 1.317 | 1.439 | 2.758 | 3.098 |

**107 active intervals exceed 50 ms.** One worst physics interval records 500
moving bodies, 12 substeps / 384 passes, zero pair visits/candidates/contacts,
58.609 ms contact phase and 585.176 ms game backlog. This localizes the measured
cost to physics contact work, not rendering; function-level profiling is still
needed before selecting an optimization. #288 retains this precise reproduction
and requires preserving time, substeps, sleep and conservation contracts.

[Active-only analysis](active-analysis.json) links every raw window and its
full summary. [Analysis script](analyze.py) validates native admission,
accounting, identical configurations/renderers and undecimated sampling before
pooling; fewer than 100 moving intervals explicitly fails sample sufficiency.
Do not use whole-window resting tails to erase this active failure.

## Reproduction

Build with `python3 scripts/build_game.py --profile release --output
artifacts/build/native-255`, then copy that executable to the local macOS app
bundle described in [#254](../issue-254/README.md). Use the existing reviewed
`artifacts/native-input` helper; the wrapper opts into focusing only target PIDs.

```sh
SALIMON_REFERENCE_SIZE=1 RUST_LOG=info python3 reports/issue-229/foreground_matrix.py --helper artifacts/native-input --records artifacts/native-255/active-focus.json -- python3 scripts/benchmark_fragments.py --binary artifacts/SalimonAcceptance.app/Contents/MacOS/salimon-client --machine-label apple-m1-imac-16gb-macos26.2 --counts 500 --layouts onboard --states active --cameras facing --repeats 10 --minimum-samples 100 --warmup-seconds 0 --sample-seconds 3 --timeout 60 --output artifacts/native-255/active
python3 reports/issue-255/analyze.py artifacts/native-255/active/*.stdout --output artifacts/native-255/active-analysis.json
```

For the empty reference use `--counts 0 --states settled --repeats 3
--warmup-seconds 2 --sample-seconds 30 --timeout 180`. For settled cargo use
`--counts 500 --states settled --settle-steps 2400` with those same window options.
Historical same-device #199/#229 evidence is preserved; different window lengths
and old missing-admission schemas are not passed through a matched regression
gate. These measurements establish submitted-present intervals, not physical
scanout. GPU timestamps are asynchronous latest-pass observations. Metal total
GPU allocator residency is unavailable; resource upload/capacity bytes are not
resident device memory.

## Settled cargo and larger-piece preparation

Three settled onboard windows retained 1,800/1,801/1,801 intervals, all 500
asleep throughout; frame p95 **17.140/17.125/17.118 ms**, physics p95
**1.405/1.420/1.435 ms**, CPU-render p95 **0.952/0.941/0.945 ms** and GPU-latest
p95 **1.622/1.625/1.624 ms**. These satisfy the calibrated cadence/component
budgets. A **42.964 ms** maximum in the first window remains visible; no zero-spike
or strict per-frame floor claim follows. [Settled summary and raw windows](settled/).

The previously interrupted larger-piece `scattered` 500 preparation was repeated
with a **120-update / 60-second process bound**. It succeeded through ordinary
production contact/sleep at update **104**, 1.664 simulation seconds and
11.492 preparation wall seconds. The retained [checkpoint](stress-checkpoint.json)
contains all 500 sleeping IDs, poses, masses and radii (alternating 2/0.5 kg
mixed-material sources; current production collision shape is spherical).
The following admitted three-second window retained 180 completed intervals,
all asleep: frame p95 17.152 ms / maximum 28.765 ms, physics p95 1.456 ms.
[Stress result](stress/summary.json), [exact bounded command](stress-attempt.json).
This completes a bounded preparation/settled probe, not an active mixed-piece
or 1,000-piece stress performance signoff.

## Checks and limitations

- Release staging command above: passed; native executable digest preserved.
- Empty (3×30 s), settled onboard (3×30 s), active onboard (10 fresh launches)
  and bounded scattered 500 native commands: completed, admission and accounting
  validated. Active performance **failed**, separately from command success.
- `python3 reports/issue-255/analyze.py reports/issue-255/active/*.gz --output
  artifacts/native-255/reanalyzed.json`: passed, reproduces 177 moving intervals.
- Analysis integrity check: inserting a 1,000 ms **resting** sample leaves
  active p95 unchanged; invalid admission and mismatched workload are rejected.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: see
  [tooling log](python-tests.log).
- `git diff --check`: passed. Relative report links and gzip/JSON archives checked.
- Rust tests/Clippy and visual gameplay routes not repeated for this reporting/
  guide-only PR; runtime and assets are unchanged from the staged source.
  #279 performs the current-art graphical verification separately.

Raw archives and focus records: [empty](empty/), [empty focus](empty-focus.json),
[settled](settled/), [settled focus](settled-focus.json), [active](active/),
[active focus](active-focus.json), [stress](stress/), [stress focus](stress-focus.json).
Active drop-to-drop variability and pre-admission motion limit the coverage;
no on-screen optical/scanout instrumentation, total GPU residency, disk persistence
or exhaustive gameplay performance certification is claimed. #288 blocks final
active performance signoff; #255 remains open.
