# Issue 255 / 288 — active contact-path remediation

Local follow-up on 2026-10-17. The parent report preserves the earlier failure;
this directory records a fresh matched baseline, an intermediate candidate and
the final candidate. No earlier raw evidence is rewritten.

## Ownership and change

`client/physics` removes three measured sources of unnecessary work:

- Remember empty spatial query rows across ordered contact passes, invalidating
  all remembered rows after any positional correction. Each current-pose solve
  starts fresh, including after integration.
- Refresh every exact AABB, retaining grid membership when its cell range stays
  unchanged. Population-length/cell-width changes rebuild membership; changed
  ranges are removed/reinserted. Obsolete cells are pruned at pose boundaries.
- Omit swept activation queries when all participants are already awake.
  Mixed populations retain swept waking and connected-support propagation.

Sixteen contact passes, SAT, impulse/friction response, adaptive substeps, sleep
dwell, mass/IDs and elapsed-time/backlog accounting are unchanged. Empty-row
scratch is O(n), without retaining dense candidate lists. The architecture guide
documents cache boundaries. New regressions compare corrected/moving poses
with the all-pairs oracle and retained membership with a fresh grid across
origin, radius, oversize, order and population changes. Existing sleep/support,
fast-impact and exact-pose regressions remain the behavioral oracles.

## Profiling

`separated_sphere_contact_cost` is an ignored release CPU attribution probe with
500/1,000 separated small spheres, 10 warmups and 100 independently reset 16 ms
solver replays. It separately times the old repeated-query pattern, all-awake
activation queries and grid refresh. Setup/cloning is outside the timer. This
is function-path attribution, not an authored cargo/native FPS test.

The original 500 probe measured contact p95 18.666 ms and repeated-query replay
18.157 ms. After empty-row reuse, a follow-up measured contact p95 5.042 ms,
all-awake swept-query replay 7.895 ms and grid-reset replay 2.903 ms (eight
substeps / 256 passes). These measurements motivated the remaining two changes.
CPU replay variability is visible in the retained logs; native comparisons below
are the reference performance evidence. The attempted macOS `sample` attachment
returned an empty call graph and is not used as profiling evidence.

## Native results

The redundant contact work is removed. **Active performance signoff remains
open**: this PR does not change a budget or close #255/#288. It provides a
reviewable optimization and the remaining budget decision in
[Project Q&A](../../../docs/project-qa.md#what-budget-should-complete-active-500-piece-m1-signoff-255).

Ten fresh 500×0.032 kg `onboard` launches per stage, seed 155, facing camera,
zero warmup, three-second windows extended to at least 100 completed intervals
per launch. All IDs/material/source/mass agree exactly across every stage;
16 kg is conserved. All raw intervals, spikes and admission/focus records are
retained. Only moving intervals enter this table:

| Metric | Baseline | Empty-row reuse only | Final |
| --- | ---: | ---: | ---: |
| Moving intervals | 170 | 175 | 175 |
| Frame median / p95 / p99 / max, ms | 58.802 / 75.004 / 75.544 / 75.583 | 17.992 / 26.002 / 35.221 / 37.463 | 16.654 / 24.416 / 30.847 / 34.135 |
| Physics p95 / p99 / max, ms | 73.984 / 74.507 / 74.518 | 20.110 / 20.777 / 21.198 | 5.741 / 6.770 / 6.877 |
| Contact p95, ms | 61.462 | 7.555 | 3.663 |
| Game backlog p95 / max, ms | 655.979 / 711.928 | 54.228 / 67.188 | 34.323 / 48.005 |
| Moving intervals >50 ms | 122 | 0 | 0 |

Final physics p95 improves **92.2%**, contact p95 **94.0%** and frame p95
**67.4%**. These are admitted pooled drop distributions, not IID confidence
claims or continuously moving 100-frame windows. Before/after wall-time
scheduling changes the number of updates/samples; exact-pose oracle tests,
rather than raw frame ordinal equality, establish physics equivalence.

The final frame p95 still exceeds the existing **17.5 ms settled** cadence
guidance; physics p95 exceeds the **provisional 5 ms** partition. Neither is
loosened. Final active GPU-latest p95/max is 2.663/4.427 ms; CPU-render p95/max
is 0.604/0.856 ms, within their absolute provisional partitions.

Ten additional zero-piece launches use the same no-warmup configuration. Pair
each with a final drop by repeat number and retain exactly its first N completed
intervals, where N is that drop's moving-prefix length (16–19). This yields 175
matched empty-prefix intervals: median **16.666 ms**, p95 **23.897 ms**, p99
**31.960 ms**, max **38.834 ms**, zero >50 ms. Final loaded p95 is 2.2% above
that empty prefix. This identifies a launch-period cadence limitation even
without physics; it does not prove a specific OS/renderer cause or approve a
replacement active budget. Full three-second empty windows remain raw.

The existing 20% same-device checker passes frame/physics comparisons but
**fails CPU-render p95 in six of ten whole-window comparisons** (baseline
0.667–0.757 ms; affected candidate 0.878–1.088 ms). Those whole windows mix
moving, awake/resting and sleeping samples differently after faster simulation.
Renderer code/assets are unchanged, but no cause or waiver is inferred: the
failed relative checks are retained in [comparison.json](comparison.json).

Three final 30-second windows each, after two seconds warmup:

| Reference | Frame p95 by window, ms | Largest raw frame, ms | State |
| --- | --- | ---: | --- |
| Empty | 17.139 / 17.135 / 17.141 | 26.870 | 0 pieces |
| Onboard 500 | 17.157 / 17.183 / 17.150 | 26.770 | All 500 asleep throughout |

These pass the existing settled cadence guidance. Settled physics p95 is
1.187/1.234/1.205 ms; CPU-render p95 is 1.109/1.131/1.119 ms, within their
absolute provisional partitions. Window sizes are 1,801–1,802 completed
intervals; maxima remain in the raw evidence. This does not waive the separate
short-window relative failures above.

The larger mixed-piece `scattered` 500 preparation repeats the explicit
**120-update / 60-second process bound**. It naturally reaches all-asleep at
update **104**, 1.664 simulation seconds and **10.659 preparation wall seconds**.
The admitted three-second window retains 179 completed intervals, all 500 asleep,
frame p95 **17.196 ms**, maximum **29.409 ms**, physics p95 **1.347 ms**.
[Exact commands, process durations and statuses](native-attempts.json) and
[preparation log](stress/native-500-scattered-settled-facing.stderr) retain the
bound and result. This is bounded preparation/settled coverage, not an active
mixed-piece or 1,000-piece stress signoff.

## Environment and provenance

Apple M1 iMac21,1, 8 CPU / 8 GPU cores, 16 GiB, macOS 26.2 (25C56), Metal,
AutoVsync, 1920×1080 physical drawable at 2× on the built-in display. A secondary
1080p/75 Hz display remains attached. AC power, no reported thermal/performance
warning. Rust 1.89.0, Python 3.14.2 (3.12 unavailable). No competing build/test
job ran during retained native performance windows. Foreground/visibility
admission and target-PID OS focus were required for each launch.

The baseline is the unchanged runtime/assets at `1da0d95`, binary SHA-256
`a7e2bdbec8d32dff611e870daebf2b42465c507c7c1277c737203164f83dae4b`.
Main `ba7a116` only adds reporting/guidance since that runtime source. Candidate
runtime is commit `0b9ff3572e5188761c6b2a81fef5e7470fbf4cf0`, binary SHA-256
`98ec9624dcbecf557e1a1e97d93a115c9d19e0610e585674492b0b98cdfd30de`.
The intermediate candidate digest is
`0c57bd617d558167aafee870665f02f918ebaa752841126b09b901b270b326f7`;
it has only empty-row reuse. Runner `revision`/`dirty` fields describe the
checkout, not a rebuild; these digests and the implementation commit establish
the executable provenance.

## Reproduction and checks

From the repository root, stage the release and copy its identical executable
into the local app bundle described by [#254](../../issue-254/README.md):

```sh
python3 scripts/build_game.py --profile release --output artifacts/native-255-remediation/build-final
SALIMON_REFERENCE_SIZE=1 RUST_LOG=info python3 reports/issue-229/foreground_matrix.py --helper artifacts/native-input --records artifacts/native-255-remediation/final-focus.json -- python3 scripts/benchmark_fragments.py --binary artifacts/Salimon255Candidate.app/Contents/MacOS/salimon-client --machine-label apple-m1-imac-16gb-macos26.2 --counts 500 --layouts onboard --states active --cameras facing --repeats 10 --minimum-samples 100 --warmup-seconds 0 --sample-seconds 3 --timeout 60 --output artifacts/native-255-remediation/final
python3 reports/issue-255/analyze.py artifacts/native-255-remediation/final/*.stdout --output artifacts/native-255-remediation/final-analysis.json
python3 reports/issue-255/2026-10-17-remediation/verify.py --output artifacts/native-255-remediation/verified-report.json
cargo test --release -p salimon-physics separated_sphere_contact_cost --locked -- --ignored --nocapture
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
python3 -m unittest discover -s scripts -p 'test_*.py'
git diff --check
```

- `cargo test --workspace --locked`: passed; all ordinary tests/doc tests pass.
  Opt-in benchmark/GPU tests remain ignored by the ordinary gate.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- Release staging: passed, final binary digest above.
- Release attribution probe: passed; final 500 p95 is 3.485 ms total,
  2.866 ms contact, 0.004833 ms all-awake activation replay and 0.113 ms
  grid-reset replay. It retains eight substeps / 256 ordered passes.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: 82 tests passed
  with Python 3.14.2; the recommended Python 3.12 is unavailable.
- The ten baseline, ten intermediate, ten final and ten empty-launch processes
  completed; raw admission, target-PID focus, renderer/configuration, undecimated
  sampling and identical 500-piece accounting verified.
- Native performance acceptance: **not passed** for the reasons above; six
  CPU-render relative comparisons fail. Collection/analysis success is separate.
- One long empty-reference attempt was **rejected** after losing focus at
  approximately seven seconds. Its logs/focus/failed summary are retained under
  `empty-stable-rejected`; it contributes no accepted frame evidence. A fresh
  admitted retry is recorded separately.
- Final evidence integrity/link checks and `git diff --check`: passed; see
  [integrity log](integrity.log). All 43 gzip payloads parse and match their
  SHA-256 manifest; reanalysis exactly reproduces the comparison. Invalid
  admission, mass mismatch and missing OS focus are rejected; an inserted
  1,000 ms resting spike leaves active frame tails unchanged.
- No new screenshots or asset/visual routes: renderer appearance/assets are
  unchanged. The native measurement processes exercise the production Metal
  renderer; this is not a new visual acceptance claim.

Evidence: [baseline](before/), [empty-row candidate](after/), [final candidate](final/),
[matched empty launches](empty-launch/), [long empty references](empty-stable/),
[settled cargo](settled/), [bounded larger-piece run](stress/), and
[rejected reference attempt](empty-stable-rejected/). Original summary metadata
names uncompressed `.stdout` files; their lossless retained form is
`.stdout.json.gz`. [Raw archive SHA-256 manifest](raw-sha256.json) and
[verification script](verify.py) make admission/focus/accounting, active-only
tails, matched empty prefixes and failed relative checks reproducible.

Logs: [workspace tests](workspace-tests.log), [Clippy](clippy.log),
[Python tooling](python-tests.log), [release build](build.log),
[build digest](build-info.json), [environment](environment.json),
[post-run thermal status](thermal-after.txt), and CPU attribution
[before](query-before.log), [empty-row only](query-after.log),
[second-path attribution](query-intermediate.log), [final](query-final.log).

Submitted-present intervals measure completed calls, not optical scanout. Active
tails pool admitted `moving_objects > 0` intervals from ten fresh naturally
settling drops; resting intervals remain raw but are excluded from active tails.
Pre-admission updates are outside coverage. GPU timestamps are asynchronous
latest-pass observations. The 0.032 kg onboard fixtures remain distinct from
#236's 10 kg sphere workloads. No strict per-frame FPS floor, 1,000-piece native
acceptance, total GPU residency or general dense-pile performance is claimed.
