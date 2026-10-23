# #479 — Legacy active-budget decision package

## Outcome

Reanalysed the complete retained #255 native archive on 2026-10-22. Every
admission, exact workload/identity/mass, moving-sample, matched-empty-prefix,
sleeping-reference and historical-pair check passes. The proposed limits below
are unchanged. This is a proposal for the owner's explicit review; no approval
is inferred from requesting a PR, collecting evidence or passing its numbers.

## Proposed decision

Adopt the existing #288 **launch/settling-only** budget for ten fresh, no-warmup,
seed-155 drops of 500×0.032 kg onboard cargo on reference M1/Metal, physical
1920×1080, 2× scale, AutoVsync, built-in 60 Hz. Retain at least 100 completed
moving intervals. It does not establish sustained active 60 FPS or change the
17.5 ms settled guidance, provisional 5 ms physics partition or the independent
20% same-device regression rule. Current-cap/mixed spheres remain #455.

| Metric | Unchanged proposed maximum | Retained observation |
| --- | ---: | ---: |
| Moving frame p95 | 33.333 ms | 23.446 ms |
| Moving frame maximum | 50 ms | 33.947 ms |
| Physics + adapter p95 | 8 ms | 5.932 ms |
| CPU render p95 | 4 ms | 0.422 ms |
| Latest asynchronous GPU p95 | 12 ms | 3.562 ms |
| Maximum backlog | 50 ms | 38.430 ms |
| Frame p95 / matched empty prefix p95 | 1.2 | 1.0641 |
| Moving intervals over 50 ms | 0 | 0 |

Rationale: fresh drops settle rapidly, so a long whole-window percentile hides
active cost. Matching the exact number of empty launch-prefix intervals controls
startup/presentation cadence without deleting spikes; the long empty and fully
sleeping rows independently bound settled cadence. CPU/GPU overlap means their
limits must not be added together. The 150 moving intervals cover natural
settling, not sustained or arbitrary dense motion. Present-call intervals and
latest-GPU samples do not certify optical scanout.

## Historical failures and limits

Six historical whole-window CPU-render relative failures and three later
same-state settled failures remain failures. Fresh exact historical binary
pairs pass all 20% comparisons, but non-reproduction does not explain the old
failures. The proposed decision is to retain this unresolved attribution as an
explicit limitation, subject to owner acceptance or a request for further
profiling. It does not retrospectively waive those runs. Larger legacy scattered
preparation naturally settled at update 93 within the unchanged 120 bound;
that result does not address current-mixed 1,000-sphere stress.

## Reproduction and review

```sh
python3 reports/issue-255/2026-10-21/verify.py --root reports/issue-255/2026-10-21/native-data --output artifacts/479-archive-reanalysis.json
```

The fresh [reanalysis](evidence/reanalysis.json) and
[archive SHA-256 audit](evidence/verification.json) are retained beside this report. Only ten recorded file paths differ from the original result (committed gzip
archive paths replace historical artifact paths); all other fields match exactly.
Raw bytes, provenance,
counts, exact historical binaries and commands remain in the canonical
[#255 refresh](../issue-255/2026-10-21/README.md); they are not duplicated or
relabelled as fresh native measurements. No completed native collection was
repeated simply to remove historical failures. This PR changes documentation
and decision evidence only; runtime/physics/render behavior is unchanged.

Required owner decision: approve these exact scoped limits and retained
limitations, or identify which limit/attribution needs revision. Until that
explicit decision, `acceptance_granted=false` and Project Q&A remains open.
[The pending decision record](decision-proposal.json) hashes the one canonical
budget instead of introducing a second definition. [#563](https://github.com/mr-exception/salimon/issues/563)
owns this exact approval/attribution decision after this delivery; closing #479
does not approve the proposal. Documentation/path/base-impact and whitespace
checks pass; no unrelated runtime tests or native collection were repeated for
this evidence/documentation-only change.
