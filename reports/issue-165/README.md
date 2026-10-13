# Issue #165 — deferred native stress validation disposition

## Outcome — 2026-10-13

**Skip the remaining Apple M1 verification at the user's explicit request and
close #165 when the review PR merges.** The user requested work on #165, a PR
for review, and instructed: “if it needs M1 verification, skip it and mention
to close in pr description”. The entire remaining scope is reference-machine
validation, so this is a documented acceptance waiver, not a benchmark pass.

No runtime, solver, harness, dependencies or fixtures change. Existing IDs,
mass, one-object carrying, contact geometry, coordinate precision and streaming
behavior retain their current contracts. No cargo cap or automatic deletion is
introduced. Collision candidate/allocation optimization remains owned by #157.

## Historical evidence and waived checks

The [#156 report](../issue-156/README.md) and
[historical handoff](../../ISSUE-156-HANDOFF.md#exact-remaining-matrix) retain the
original source revisions, binary hashes, hardware/settings, raw archives,
median/p95/p99 distributions and failed attempts. They are historical evidence,
not measurements of reviewed main `243ef34` or this documentation change.

| Check | Historical status / current disposition |
| --- | --- |
| Dense 500 interaction route | Passed historically: natural sleep, flight/landing/takeoff, support removal, unrelated sleepers and three pickup/drop cycles; [retained result](../issue-156/native-500-floor-navigation-2026-10-13.json) |
| Dense settled 500 and 1,000 timings | Six windows each retained historically; [500 summary](../issue-156/native-500-dense-settled-2026-10-12-summary.json), [1,000 summary](../issue-156/native-1000-dense-settled-2026-10-12-summary.json); no strict 60 FPS floor established |
| Corrected dense 1,000 interaction route | **Skipped, not passed.** Older route failed repeat pickup identity; corrected retry [terminated before readiness](../issue-156/native-1000-preparation-stopped-2026-10-13.json). Sleep/flight/support success from a partial attempt does not establish complete acceptance |
| Remaining reference matrix | **67 windows skipped**, including the remaining 500 cases; existing 221 retained windows are not a complete 288-window matrix |
| Fresh source/binary/hardware measurements | **Not run.** Earlier evidence is preserved with its original provenance; no current-build performance certification |

The historical missing windows remain:

| Selection | Missing windows |
| --- | ---: |
| Settled 500/1,000, scattered/surface, both views, three repeats | 24 |
| Active 250, dense away, third repeat | 1 |
| Active 250, surface, both views, three repeats | 6 |
| Active 500/1,000, all layouts, both views, three repeats | 36 |
| Total | 67 |

Existing windows requested as active can contain both moving and resting samples.
For example, [active 250 scattered facing repeat 1](../issue-156/native-matrix-active-250-partial-2026-10-13-summary.json)
has 100 frame intervals but only 46 moving update samples and 55 resting samples.
Its overall frame median/p95/p99 are 16.782/2,473.292/5,345.254 ms; those tails
combine states and must not be relabeled as sustained active-only distributions.
The window has fewer than 100 moving samples and cannot satisfy active-only
sample sufficiency. Expensive active physics remains unresolved; skipped active
500/1,000 windows cannot establish an interaction budget. See the
[measurement definitions](../../scripts/FRAGMENT_BENCHMARKS.md#measurement-definitions-and-gates).

## Validation

Documentation-only checks on Linux with Python 3.12.14:

- `python3` evidence review: parsed the five retained native matrix summaries;
  confirmed 6 + 6 + 108 + 90 + 11 = **221** windows, each with at least 100
  retained frame intervals, leaving **67** of 288; checked the mixed-state
  example and the stopped 1,000 result above.
- Relative Markdown file links checked for this report and the updated handoff.
- `git diff --check` passed.

Not run: Rust tests/builds/Clippy, Python harness tests, native launches, new CPU
benchmarks or Apple M1 runs. Source and harnesses are unchanged; the applicable
[documentation validation gate](../../docs/validation.md#cpu-and-tooling-gates)
is link/evidence integrity. No new screenshot or performance result is claimed.

## Future resumption

Issue closure records the user's waiver. It does not establish the original
full stress acceptance. If larger-load verification becomes necessary, reopen
#165 or create a separately scoped validation issue. Use the existing
[benchmark guide](../../scripts/FRAGMENT_BENCHMARKS.md) and
[correctness harness](../../scripts/fragment_sleep_acceptance.py), with a fresh
recorded revision/binary/hardware configuration and isolated unobscured native
runs. Keep preparation, fixed-step correctness, active-only sufficiency and
frame/CPU/GPU timings separate; retain three repeats and at least 100 intervals
per reference window. Reuse older evidence only where configuration and code
applicability have been established. Do not restart the historical jobs merely
to satisfy the superseded handoff instructions.
