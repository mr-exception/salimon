# Issue 288 — active-budget proposal and remaining acceptance checks

2026-10-17, Apple M1 iMac21,1 / 16 GiB / macOS 26.2 (25C56), Metal,
1920×1080 physical drawable, 2× scale, AutoVsync, built-in 60 Hz display.

## Outcome and review decision

The owner requested a separately justified active budget for PR review on
2026-10-17. This change supplies that proposal and a reproducible evaluation.
**The thresholds are proposed, not approved, and performance signoff remains
open.** Neither passing proposed numbers nor merging evidence tooling silently
waives the retained CPU-render relative failures. #255/#288 must remain open
until the owner explicitly reviews the budget and the remaining failures.

The function-level investigation, optimization and ten fresh native drops were
already implemented in merged [PR #292](https://github.com/mr-exception/salimon/pull/292),
implementation `0b9ff35`, evidence `77cda71`. Its
[report](../issue-255/2026-10-17-remediation/README.md) owns profiling, unchanged
physics contracts, focused exact-pose/fresh-grid regressions and all raw drops.
This PR changes no runtime code, assets, solver passes, adaptive substeps,
contacts, sleep, IDs/mass, foreground admission or elapsed-time accounting.
#254, the admission prerequisite, is closed and its checks remain required.

## Proposed active budget

The canonical values for this proposal are in
[proposed-budget.json](proposed-budget.json). Scope: ten fresh native
500×0.032 kg `onboard` drops, seed 155, facing camera, no warmup, three-second
windows extended to at least 100 completed intervals per launch. Only completed
intervals with `moving_objects > 0` enter active tails; the pool must contain at
least 100 such intervals. Each individual drop is shorter than 100 moving
intervals, so these are repeated launch distributions, not a continuous-motion
or independent-sample confidence claim.

| Proposed check | Limit | Retained final result |
| --- | ---: | ---: |
| Active frame p95 | 33.333 ms (two 60 Hz periods) | 24.416 ms |
| Active frame maximum | 50 ms (three periods) | 34.135 ms |
| Active intervals above 50 ms | 0 | 0 / 175 |
| Physics + adapter p95 | 8 ms | 5.741 ms |
| CPU rendering p95 | 4 ms | 0.604 ms |
| GPU latest-pass p95 | 12 ms | 2.663 ms |
| Active game backlog maximum | 50 ms | 48.005 ms |
| Active frame p95 / matched empty launch-prefix p95 | ≤1.20 | 1.022 |

Rationale:

- Allow one missed refresh at active p95, and at most two missed refreshes in
  any retained moving interval. These explicit 30 Hz/20 Hz transient bounds
  intentionally differ from the 60 Hz settled guidance; they do not establish
  an optical FPS floor. They are a product tradeoff for the owner to accept,
  rather than rounding a measured percentile up to obtain a pass.
- Allocate at most 8 ms of the nominal 16.667 ms CPU period to active physics,
  leaving 8.667 ms for other update/preparation/render work. Existing 4 ms
  other-CPU, 4 ms CPU-render and 12 ms overlapping GPU allowances remain
  provisional. Marginal percentiles cannot be added to prove frame latency;
  the independent frame checks above still apply. The 5 ms provisional
  physics partition remains recorded for settled/reference comparison; this
  proposal explicitly asks to allow more during the short active drop.
- Bound active simulation lag to three display periods without discarding time.
  The retained completed-window backlog is also reported: p95 **51.665 ms**,
  p99 **64.951 ms**, max **73.451 ms**, including nonmoving frames before sleep.
  These exceed the proposed *moving-only* backlog allowance outside its scope;
  no general backlog or interaction-deadline acceptance is claimed. Last-sample
  backlog is 1.168–15.076 ms across the ten drops.
- Require loaded active p95 within the existing 20% tolerance of matched empty
  launches **as well as** the absolute limit. The empty prefixes have p95
  23.897 ms, already above settled guidance; the extra relative check prevents
  the two-period allowance from hiding a large loaded-only cost.

The historical baseline fails this proposal (active frame p95 75.004 ms,
max 75.583 ms, physics p95 73.984 ms, 122 intervals over 50 ms and backlog max
711.928 ms). Final active p99/max remain **30.847/34.135 ms** for frames and
**6.770/6.877 ms** for physics. The proposal therefore bounds the observed
remediation while retaining a meaningful failure on the original bottleneck.

## Relative-check investigation

The original whole-window 20% checker has **six CPU-render failures**.
They are retained unchanged in
[the original comparison](../issue-255/2026-10-17-remediation/comparison.json)
and this report's [analysis](analysis.json). No exception or threshold change
is applied to those comparisons. Renderer code/assets are byte-identical across
the implementation change, but that alone does not explain timing differences.

Each original baseline window contains roughly 16–19 moving intervals,
32 awake-but-nonmoving intervals and 75–79 fully sleeping intervals. Final
windows contain 16–19 moving, 32 awake-but-nonmoving and 129–132 fully sleeping
intervals. Thus equal wall-time windows measure different proportions of these
states after physics became faster. This is a demonstrated comparison
confounder, not proof of a CPU scheduling cause or a waiver of failed checks.

Fresh paired settled runs use the original and optimized **identical retained
executables**, three 30-second windows each, two-second warmup, native admission
and target-PID focus, 500 naturally sleeping pieces throughout. They independently
check the same 20% frame/physics/CPU-render gates on matching states:

| Three-window p95, ms | Before | Optimized |
| --- | --- | --- |
| Frame | 17.173 / 17.198 / 17.231 | 17.147 / 17.154 / 17.151 |
| Physics + adapter | 1.166 / 1.193 / 1.150 | 1.259 / 1.225 / 1.292 |
| CPU render | 0.885 / 0.902 / 0.884 | 1.097 / 1.100 / 1.102 |

All six windows pass the 17.5 ms settled frame guidance. All three paired frame
and physics relative checks pass. **All three CPU-render relative checks fail**,
by 24.0%, 22.0% and 24.7% respectively. Each window retains 1,801–1,802 completed
intervals. These same-state failures show that the original difference in state
proportions is **insufficient to explain the regression**. The cause remains
unprofiled; no CPU-frequency, cache, scheduler or renderer cause is asserted.
They do not replace or delete historical active-window failures. Although
CPU-render remains below the provisional absolute 4 ms partition, that is not a
substitute for the independent relative gate. Further attribution is required
before unconditional performance signoff.

## Evidence and reproduction

[verify.py](verify.py) first runs the #255 raw verifier: ten drops per stage,
matching configurations/renderer, IDs/mass, foreground admission, target-PID
focus, active-only sample sufficiency and matched empty prefixes. It then
evaluates the proposal and reconstructs fresh settled summaries from raw
intervals before comparing them. Its output always says
`acceptance_granted: false`; numerical results cannot approve a product decision.

```sh
python3 reports/issue-288/verify.py --output artifacts/issue-288/analysis.json
python3 -m unittest discover -s scripts -p 'test_*.py'
git diff --check
```

The new regression tests guard against granting acceptance from passing numbers,
masking historical relative failures, accepting sparse active pools, omitting
spikes/individual over-budget metrics, or ignoring the matched empty limit.

Executable lineage remains the #255 baseline `1da0d95` (SHA-256
`a7e2bdbec8d32dff611e870daebf2b42465c507c7c1277c737203164f83dae4b`)
and optimized implementation `0b9ff35` (SHA-256
`98ec9624dcbecf557e1a1e97d93a115c9d19e0610e585674492b0b98cdfd30de`).
The checkout for this follow-up is based on merged main `aa4fde4`; runner
revision/dirty metadata records that checkout, not a rebuild of either binary.
See [original environment](../issue-255/2026-10-17-remediation/environment.json)
for display, hardware and compiler details. Fresh commands/durations and focus
records are retained in [native-attempts.json](native-attempts.json),
[before focus](settled-before-focus.json), [after focus](settled-after-focus.json),
[before raw windows](settled-before/) and [after raw windows](settled-after/).
The [raw SHA-256 manifest](raw-sha256.json) covers new successful and rejected
payloads; the verifier also checks every original #255 archive digest.
Successful `.stdout` JSON is retained losslessly as `.stdout.json.gz`; rejected
stdout is empty and retained unchanged. [Fresh environment checks](environment.json)
record AC power and no reported thermal/performance warning after sampling.
No competing build/test job
runs during performance sampling. Python is 3.14.2; recommended 3.12 is unavailable.

Two initial baseline attempts lost focus within seconds and correctly failed.
Their empty stdout, stderr, failed summaries and target-PID focus records are
retained separately as rejected evidence; neither contributes a frame statistic.
After the owner confirmed the desktop was available, fresh measurements were
started. No screenshot is required for this nonvisual change.

## Validation and limitations

- Raw-evidence verification: **passed**; [analysis.json](analysis.json)
  reproduces the proposal checks, active-only p95/p99/max/spikes/backlog,
  original six relative failures and three fresh relative failures. All 175
  moving intervals pass the proposed limits. `acceptance_granted` remains false.
- Native collection: **passed**, six admitted 30-second windows. All 500 IDs
  and 16 kg conserved; full sleeping state, matching settings and target-PID
  focus verified. Two initial attempts **rejected** for focus loss and excluded.
- Native performance relative acceptance: **failed**, CPU-render p95 in all
  three new matched settled comparisons, in addition to the six historical
  whole-window failures. Settled frame guidance and proposed active numerical
  limits pass; owner approval is pending.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: **passed**, 86 tests,
  including four new evidence-evaluation regressions; [log](python-tests.log).
- `python3 reports/issue-288/verify.py --output artifacts/issue-288/analysis.json`:
  **passed**; reproduces the committed analysis exactly and checks 47 raw
  payload digests (43 previous, four new including two rejected empty payloads).
- Relative Markdown links and `git diff --check`: **passed**.

The earlier workspace Rust tests, Clippy, formatting and release checks passed
at `0b9ff35`/`77cda71`; this report does not claim to rerun them. This change
only adds proposal/analysis tooling and documentation, so no new Rust/asset
build or gameplay/visual smoke is required. Native measurements use production
Metal but measure submitted-present calls, not physical scanout. GPU latest
values are asynchronous observations. Larger-piece/1,000-piece, dense contact,
continuous motion, #236's 10 kg spheres, input deadlines and general #255
performance signoff remain outside this proposal. The earlier bounded
larger-piece preparation is retained in the #255 remediation report.
