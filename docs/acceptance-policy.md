# Issue acceptance and completion (#749)

This is the current acceptance policy for development issues, including the active
browser navigation client. It supplements [cloud and human verification](verification-workflow.md)
and [validation](validation.md). Routine delivery uses reproducible cloud evidence;
the owner retains merge authority. Older native commands apply only to the native
revision they describe. A successful build alone does not prove gameplay or UI behavior.

## Plan observable acceptance before implementation

Read the current issue, discussion, approved design and actual prerequisites.
Give each criterion a stable ID and describe an observable result, including
relevant negative, boundary and failure behavior. Use the
[development issue template](../.github/ISSUE_TEMPLATE/development.md) for new work;
ordinary bugs and documentation work use the same matrix with proportionate checks.

Every development issue needs a verification matrix covering unit, integration,
E2E, visual and manual/hardware qualification. Each applicable row states its
criterion IDs, production path, exact command or procedure, expected assertion,
whether it is mandatory, and evidence location. Mark inapplicable categories
`N/A` with a concrete reason; never use N/A to hide missing implementation or tooling.
Record actual outcomes separately as PASS, FAIL, BLOCKED or NOT RUN. Only executed
successful checks are PASS. A required skipped check remains unverified.

For browser behavior, use the running production adapters and real Chromium/WebGL
canvas where relevant. DOM/state assertions support functional checks; rendering
needs originals and measurable geometry, pixel or temporal assertions. Record
software adapters truthfully. Browser synthetic input does not prove physical input.
Use [client commands](../client/README.md#verification-and-accessibility) rather
than obsolete Cargo commands. Documentation-only changes need link/path checks and
semantic review, with runtime/E2E/visual checks explicitly N/A when behavior is unchanged.

## Status and authority

These are workflow states recorded in the issue/PR/report, not newly provisioned
GitHub labels or automatic checks. Verification labels still describe ownership.

| State | Entry evidence and next step |
| --- | --- |
| Implementation in progress | Developer records criteria, blockers and verification plan; issue stays open. |
| QA pending | Implementation, report and checks are available at an exact PR head/base; review in the active Codex chat requested. Missing or failed mandatory checks are disclosed and prevent acceptance. |
| Changes requested | Reviewer identifies failed/unverified criteria and actionable fixes; developer revises, reruns affected checks and returns to QA pending. |
| QA accepted | The active-session reviewer records ACCEPT for the current head/base, every mandatory criterion/check passes, and no blocking review remains. Issue stays open pending merge. |
| Merged | Owner authorizes/executes merge; verify the accepted change reached default without invalidating evidence. Merge alone is not proof of acceptance. |
| Closed | Delivering PR merged and mandatory acceptance remains satisfied; verify automatic closure and retain PR/report/review links. |

Review changes and actual evidence in the same active Codex chat after
implementation. The implementing agent may perform this clearly labelled review
pass; a separate agent or external reviewer is optional only when the user asks.
Do not claim independent or separate-agent execution for the implementing
agent's own review. Inspect the issue, final diff, originals, relevant results
and current head/base, then record criterion-by-criterion PASS/FAIL/UNVERIFIED,
evidence references and ACCEPT, REQUEST_CHANGES or BLOCKED/INCONCLUSIVE.

Missing evidence, unresolved prerequisites, stale revision evidence, failed/unrun
mandatory functional checks or blocking review veto ACCEPT. AI visual judgment
cannot override deterministic failures or approve baseline changes. Record who
reviewed, the same-session capability and any revisions. Keep review proportionate:
reuse unchanged source-bound evidence and stop when remaining blockers require
external action. Do not create review handoff packets, automated review statuses,
scheduled scans or extra review CI jobs.

## Revision-bound evidence and report

Write `reports/issue-N/README.md` in the delivering PR (or
`reports/task-ID/README.md` for authorized work without an issue). Include:

- Outcome, criterion matrix, changed production/test paths and documentation impact.
- Exact tested source SHA and PR head/base SHAs and branches, commands, results,
  tool/OS versions, browser/backend, viewport/DPR, seed/clock and input method as applicable.
- Durable links to original screenshots, traces, logs, raw measurements and diffs;
  failures, skips, unrun checks, limits and separately owned qualifications.
- Active-session reviewer, verdict and evidence, plus delivering PR and follow-ups.

A report cannot embed the SHA of its own commit. Record the tested source commit
in the report and bind the final head/base, CI and reviewer verdict in a PR comment
or check after publication. For report-only commits, explicitly verify unchanged
source/test/asset/configuration hashes and let the reviewer assess that reuse;
never relabel old-head CI as a new-head pass. Record any check that does not run
at the final head and why. Review the final published diff, including report edits.

A head update, rebase or parent change invalidates acceptance until the reviewer
checks ancestry, unique diff and evidence again. Rerun affected checks; exact
commands and unchanged input/environment hashes can justify reuse of unaffected
local evidence. Required CI must describe the current head/merge revision; pending
checks and old successes are not green current-head CI. A stacked child targets
the parent head branch and includes `Closes #N` from PR creation. It stays open
until the delivering changes reach default; after parent merges, retarget and
revalidate descendants as needed. Check that the source issue actually closes.

CI artifacts may expire (the navigation workflow currently retains them for 14
days). Keep essential originals/raw evidence beside the report or in a durable
accessible archive, with hashes and run links; preserve failure evidence as well.
The [PR template](../.github/PULL_REQUEST_TEMPLATE.md) collects this record.

## CI, exceptions and closure

Run the affected existing checks in the validation matrix. For client changes,
the current [Navigation client workflow](../.github/workflows/navigation-client.yml)
runs domain tests, build/typecheck and browser tests. It is path-filtered:
documentation-only PRs may have no run. Record that as NOT RUN / not applicable
to runtime validation, and perform the planned documentation checks. Do not claim
a passing workflow from an empty check list. Branch protection/rulesets may impose
additional mandatory checks; inspect actual current requirements before acceptance.
This policy defines acceptance; implementation of automated jobs and screenshot
baselines is owned by #750–#752. It does not itself alter branch protection.

The owner retired the automated #752 review/status gate on 2026-11-04.
[Review in the active chat](independent-game-qa.md) replaces its handoff packets,
status publication and scheduled invalidation. The Navigation client domain,
build, browser and negative/repeat checks remain required
where applicable. No separate reviewer identity, machine review record or
`salimon/independent-qa` status is required. A retired legacy status is historical;
it does not replace the active review or grant merge authority.

Fix mandatory cloud failures or leave the task incomplete. Missing cloud tooling
requires a repair attempt or a meaningful equivalent reviewed against the same
criterion. An external outage remains BLOCKED; it cannot become a pass. Never
weaken a threshold, add masks or regenerate approved baselines to get green tests.
The owner removed the separate human baseline-approval CI job in PR #834.
Intentional reference updates still need reviewable originals, diffs, hashes and
passing automated comparisons/repeats/negative checks. No human approval comment
or manifest digest approval is required. The owner reviews the deployed version
and files separate issues for subjective visual regressions; this does not waive
functional checks or authorize merge/deployment.

Keep genuinely physical, hardware-specific, subjective or rights decisions in
separately linked human-verification work with exact scope, owner and instructions
under the cloud/human workflow. Record them as unverified for that context. They
need not block unrelated routine cloud delivery; they cannot absorb missing
functionality or required cloud acceptance to make an issue look complete.
An approved scope change must be explicit in the issue before acceptance, with
original criteria/history and follow-up preserved; never retroactively call an
unrun criterion passed.

Create one complete PR per source issue, include `Closes #N` from creation, link
the report and publish actual review/check evidence to the issue. Keep it open
before merge; do not merge or deploy without authorization. If GitHub closes an
issue on a merge lacking required acceptance, report the mismatch and restore
open tracking rather than treating automatic closure as proof. The existing
evidence-only human decision exception (no repository change) remains valid;
it requires a fully evidenced issue record and never closes an implementation early.

## Existing issues and reports

Apply this prospectively when an open issue is next worked: preserve its original
text/discussion, add stable observable criteria, dependencies and a verification
matrix, and identify obsolete design requirements explicitly. Do not relabel old
build logs as QA acceptance. Bring an active PR to QA pending until its current
evidence is reviewed. Do not mass-close, mass-reopen or bulk-rewrite historical
issues/reports. Reassess only affected work and keep separately tracked release
qualification visible. The [navigation example](acceptance-navigation-example.md)
shows both a completed design's criteria and a known regression rejection.
