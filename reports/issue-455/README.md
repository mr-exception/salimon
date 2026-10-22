# #455 — Current-sphere profiling and exhausted-row reuse

## Outcome

Delivered a bounded spatial-query cache improvement, reference-M1 before/after
measurements, three diagnostic stack windows and the previously unrun retained
inactive-history attribution. **Mixed stress/performance acceptance remains
failed/incomplete**, tracked by [#559](https://github.com/mr-exception/salimon/issues/559).
Closing #455 records this delivery and transfer, not a 60 FPS or stress-budget pass.

`solve_contacts` now reuses rows with **no later-ID candidate**, including rows
containing only earlier neighbors. Those pairs belong to earlier rows, so the
later row cannot resolve another pair. Every positional correction still
invalidates the absence cache; integration starts a new solve. SAT, impulses,
16 passes, adaptive steps, damping, sleep, mass/radius and elapsed time are unchanged.
An earlier-only row gaining a later contact now has an exact-state/contact-count
all-pairs regression, alongside existing dense/mixed/fast/correction oracles.

The opt-in `fragment_update_costs` test can select its unchanged sleeping
500/1,000 nearby × 0/10,000 distant-history fixtures independently, avoiding the
unrelated expensive active matrix. Its additive `scope` field identifies the
selected subset. No normal-play option or new population cap was introduced.

## Machine and provenance

2026-10-22: iMac21,1 / Apple M1 / 16 GiB / macOS 26.2 (25C56), AC, native Metal,
physical 1920×1080, scale 2, AutoVsync, built-in nominal 59.995 Hz. Rust 1.89.0,
Python 3.14.2. Native timing lanes were serial, without concurrent builds or
compression; all spikes/backlog remain raw. The diagnostic sampler and later
CPU attribution ran separately from native timing collection.

Before is the final #453 production binary `88745ae1…`, based on merged main
`ccbc2fda`. After is the same source plus this cache correction; test-only
measurement selection does not enter the application. [Exact binary/source
hashes](evidence/provenance.json), [staging metadata](evidence/build-info.json),
[source patch](evidence/source.patch) and [build log](evidence/build.log) retain
lineage. The existing production-only Cargo target was reused sequentially;
no damping/shader ablation entered it. Preserved staged before/after binaries
were distinct files. Foreground helper records validate the exact target PID;
raw runtime admission, Metal/drawable context, full IDs/masses/radii and every
completed interval are retained in [native-data](evidence/native-data/).

## Native results

Only completed intervals with `moving_objects > 0` enter active pools. Initial
null frame intervals and subsequent quiet-awake/sleeping states remain raw and
separate. Cap comparison uses five fresh launches per count per binary, seed155,
no warmup, at least 100 completed moving intervals per pool.

| Workload | Before / after moving intervals | Before / after frame p95 ms | Before / after contact p95 ms |
| --- | ---: | ---: | ---: |
| 500 current-cap | 120 / 113 | 19.761 / 24.806 | 1.396 / 1.475 |
| 1,000 current-cap | 118 / 123 | 38.736 / 37.082 | 33.106 / 30.882 |
| 500 current-mixed | 100 / 100 | 119.584 / 120.865 | 82.155 / 81.808 |
| 1,000 current-mixed | incomplete / incomplete | 120s timeout / 120s timeout | no accepted distribution |

The 1,000-cap contact p95 is about 6.7% lower in this comparison. This is **not a
general frame improvement**: 500-cap startup frame p95 increases about 25.5%,
exceeding a 20% relative comparison; the 500-mixed result is effectively unchanged.
Neither the frame-tail cause nor universal benefit is established. No failure
was selected away or relabelled as a pass. Whole-frame physics p95 before/after
is 5.675/5.982 ms (500 cap), 37.338/35.928 ms (1,000 cap), and
118.818/120.114 ms (500 mixed); adapter p95 is much smaller. The exact raw metrics
are in the [before cap](evidence/before-cap-analysis.json),
[after cap](evidence/after-cap-analysis.json),
[initial mixed](evidence/baseline-analysis.json) and
[after mixed](evidence/after-mixed-analysis.json) analyses.

Mixed scheduling reaches substep p95 40 versus cap p95 4; 500-mixed contact p95
is ~82 ms while GPU p95 is ~2 ms. Three separate 3s macOS stack samples of the
exact 1,000-mixed CPU fixture repeatedly show `Grid::query` under
`solve_contacts`. [Stacks and bounded diagnostic outcome](evidence/diagnostic/)
corroborate expensive spatial/contact work; they do not isolate every cause,
approve a mass cutoff, or supply a native frame distribution. The diagnostic
was intentionally stopped after its three windows; its incomplete output is retained.

Natural-sleep preparation uses a declared 300-step bound, never installed sleep
flags. After cap 500/1,000 sleep and retain 301/301 completed intervals, but frame
p95 18.977/19.916 ms exceeds 17.5 ms guidance. Matching before **facing** controls
also exceed that guidance (18.319/18.727 ms); additional before away rows remain
separately keyed. This does not attribute the current cadence failure to the fix.
500 mixed fails preparation at 492/500 asleep, 4 still moving after 300 steps;
1,000 mixed times out at45s. [Before](evidence/before-sleeping-analysis.json),
[after](evidence/after-sleeping-analysis.json). Earlier #313 failures remain
unchanged in their historical report.

## Inactive retained-history attribution

Three standalone release runs retain 100 samples per row and verify sleep,
identities/mass and production-versus-replay state. Across those runs:

| Nearby / distant retained objects | Selection p95 range ms | Complete measured update p95 range ms |
| --- | ---: | ---: |
| 500 / 0 | 0.057–0.061 | 0.228–0.245 |
| 500 / 10,000 | 0.115–0.139 | 0.344–0.526 |
| 1,000 / 0 | 0.108–0.120 | 0.471–0.543 |
| 1,000 / 10,000 | 0.173–0.183 | 0.613–0.675 |

The adapter is not independent of retained history. These bounded CPU costs do
not explain the ~120 ms mixed contact tail or establish native FPS. Raw
[inactive0](evidence/inactive-history-0.json.gz),
[inactive1](evidence/inactive-history-1.json.gz),
[inactive2](evidence/inactive-history-2.json.gz) preserve population inventories,
selection, solver replay, adapter, frame-sync, mass and total measurements. The
initial 20-sample exploration is retained separately as diagnostic; the table
uses the later standalone 100-sample repetitions. Structural allocation counters
are retained; full allocator/process/GPU-residency profiling was not available
from these counters and is not claimed.

## Checks and reproduction

- Workspace: **850 passed, 0 failed, 29 ignored**, including the new all-pairs
  correction regression. [Log](evidence/workspace-tests.log).
- Release physics tests passed; final production release build and all-target
  workspace Clippy pass. [Physics](evidence/physics-tests.log),
  [Clippy](evidence/clippy.log).
- Three opt-in inactive-history runs pass, 4 rows ×100 samples each.
- Python tooling, workspace build, formatting, documentation/base-impact and
  whitespace results are recorded in [checks](evidence/checks.json).

Canonical commands and interpretation are in the
[benchmark guide](../../scripts/FRAGMENT_BENCHMARKS.md#independent-inactive-history-measurement-455).
[collect.py](collect.py) and the recorded per-lane command JSON describe this
serial collection's historical artifact paths; use fresh output directories and
verified staged binaries for another collection. [profile.py](profile.py) records
the bounded diagnostic separately. Reanalyse any committed lane, including gzip:

```sh
python3 reports/issue-455/analyze.py --root reports/issue-455/evidence/native-data/after-cap --output artifacts/455-after-cap.json
```

[SHA-256 inventory](evidence/sha256.json) covers promoted evidence; large raw
stdout/JSON is losslessly gzipped and original paths remain historical. The
[physics architecture](../../client/physics/architecture.md#deterministic-spatial-candidates-157)
now describes exhausted-row ownership; runtime and the benchmark guide document
the test-only selection. No asset, renderer or Universe behavior changes.
#559 owns retained mixed stress, frame-tail/cadence failures, further attribution
and any explicitly reviewed limits. Legacy #479 budgets remain independent.
