## Problem and resulting behavior

Closes #<!-- source issue number; include from PR creation, also for stacked children -->

Describe the observable change and any negative/boundary cases.

## Revision and stack

- Head branch/SHA and base branch/SHA:
- Parent PR (or default branch if no active stack), verified ancestry and unique diff:
- Tested source SHA; final-head binding/review comment (fill after publication):
- Status: QA pending (independent acceptance is required before QA accepted).

## Acceptance and verification

Use [the acceptance policy](../docs/acceptance-policy.md). Map every issue criterion
and all five categories; N/A needs a reason. PASS requires actual execution.

| Criterion IDs / category | Mandatory? / N/A reason | Exact command/procedure | PASS / FAIL / BLOCKED / NOT RUN | Evidence / tested SHA |
| --- | --- | --- | --- | --- |
| | | | NOT RUN | |

- Report: `reports/issue-N/README.md`
- Environment, browser/adapter/backend, input, viewport/DPR/seed/clock as applicable:
- Original screenshots, traces, raw measurements/logs and approved baseline/diffs:
- Current-head CI links/results and actual required checks; explain absent path-filtered jobs:
- Failed/skipped/unrun checks, unresolved blockers and separately owned qualifications:

## Documentation impact

List updated canonical guides and map changed production/test paths to affected
architecture/invariants/contracts, plus the report and link/path checks. If no
behavior/ownership/usage changes, state the exact no-impact reason; a report alone
does not document a changed contract. Preserve historical evidence and future scope.

## Independent review (complete after publication)

- Reviewer identity/capability (distinct agent or disclosed separate review pass), cycle:
- Exact reviewed head/base, criterion PASS/FAIL/UNVERIFIED with evidence, and verdict:
- Blocking findings and fixes/rechecks; acceptance revalidated after head/parent changes:

Do not self-accept or waive failed/unrun mandatory checks. Keep the source issue
open before merge. The owner retains merge authority; this PR does not authorize deployment.
