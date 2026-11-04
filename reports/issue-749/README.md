# #749 — evidence-based acceptance policy

Implemented on `issue-749-acceptance-policy`, targeting `main` at
`6dc4a117613b0e0f12d210297ffb9a88a85ba497`. No active PR stack existed at selection.
The four open issues were #749–#752; #749 had no formal/textual prerequisites or
existing delivering PR. This is documentation/template work; runtime, dependencies,
assets, browser tests and workflow definitions are unchanged.

## Acceptance and evidence

| Issue criterion | Verification / result |
| --- | --- |
| Policy, issue/PR templates and navigation matrix | Added [policy](../../docs/acceptance-policy.md), [example](../../docs/acceptance-navigation-example.md), [issue template](../../.github/ISSUE_TEMPLATE/development.md) and [PR template](../../.github/PULL_REQUEST_TEMPLATE.md). Template walkthrough uses #744's current design and #742's known temporal defect. |
| No acceptance with failed/unrun mandatory checks, unresolved blocking review or stale evidence | Policy explicitly vetoes acceptance, defines six workflow states and independent reviewer/cycle/verdict, binds final head/base in PR evidence, and revalidates after head/parent changes. Build-only and stale-head example verdicts are BLOCKED; defective frames require changes. |
| Compatible migration, no mass closure | Prospective per-issue migration preserves original criteria/history and reports; no bulk issue mutations. Required cloud failures stay incomplete. Separately scoped release qualifications and the evidence-only human exception remain explicit. |

The implemented status/closure wording is aligned in verification-workflow and
coding-conventions; validation, root README and docs index route to the current
policy. No new automatic gate or baseline approval is claimed. #750–#752 remain
separate work.

## Verification matrix

Environment: managed Linux cloud, Python 3.12.14 (standard-library link collector),
Git and connected GitHub APIs. No browser/GPU execution required for prose changes.
Exact tested document/collector hashes are retained in `source.sha256`; final
head/base and independent verdict are bound in the delivering PR conversation.

| Category | Requirement / result |
| --- | --- |
| Unit | N/A: no runtime logic changed; no tests mirroring policy prose. |
| Integration | Mandatory: template/schema and guide semantic walkthrough, plus added local link/path/anchor checks. See commands/results below. |
| E2E | N/A: no application behavior changed. Existing frozen evidence replay is supporting documentation validation, not a new browser run. |
| Visual | N/A: no UI/assets changed; original #742 evidence is referenced without promoting baselines. |
| Manual/hardware | N/A: no physical or subjective qualification in #749. Independent document review is mandatory and recorded below. |

Commands (repository root):

- `python3 reports/issue-749/check_links.py 6dc4a117613b0e0f12d210297ffb9a88a85ba497`
  checks only links introduced by this change, not historical native documentation debt.
- `git diff --check`: whitespace validation.
- `python3 reports/issue-742/verify_frames.py reports/issue-742/before`:
  expected exit 1 at the missing-body/pixel assertion; frozen known defect rejected.
- `python3 reports/issue-742/verify_frames.py reports/issue-742/camera-idle-recheck`:
  exit 0; keyboard 65 frames/143 observations and mouse 65 frames/177 observations pass.
  These are archived-frame checks, not current-app acceptance or fresh captures.

Template walkthrough: both templates collect criterion IDs, production path,
mandatory/N/A decisions, commands/assertions, results, report/originals, tested
revision and final head/base, reviewer/cycle/verdict, CI and unresolved limits.
The example maps NAV-1–4 to unit/integration/browser/visual checks and explains
the desktop-only hardware exclusion. PAN-1 rejects missing bodies during keyboard
pan even when mouse frames or build pass. Missing current evidence or stale CI
blocks acceptance; repaired compatible evidence permits independent review.

Remote policy inspection: `main` is unprotected and repository rulesets are empty
at preflight. The sole Navigation client workflow only triggers for `client/**`
or its own definition. This PR changes neither: no runtime CI run is expected.
An absent workflow is NOT RUN, not a CI pass. Runtime build/tests/browser commands
and legacy unavailable `scripts/check_docs.py` are not run for this docs-only PR;
the local added-link check and semantic review cover its affected scope.

## Independent review and delivery

Status: QA pending. Cycle 1 will use a distinct reviewer subagent under the
Salimon QA workflow, which independently inspects criteria, final diff, template
walkthrough, archived failure/pass evidence and remote checks. Its exact reviewed
head/base, verdict and limitations are recorded in the PR conversation; this
developer report does not self-accept. The source issue remains open until merge.

No deployment, merge, issue closure, branch-protection mutation, baseline promotion
or qualification claim is part of this delivery. No remaining implementation
scope is transferred to another issue.
