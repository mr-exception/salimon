# Retire the automated independent QA gate

The owner requested removing the resource-consuming independent QA gate and
reviewing changes and evidence in the same active Codex chat instead. That
instruction supersedes the former mandatory separate-reviewer/packet workflow.

Removed the scheduled/event-driven `independent-qa.yml` workflow and its Python
review, status, invalidation, handoff/demo runtime and fixtures. Removed only the
QA-specific Python setup and fixture step/path triggers from Navigation client.
Its install, domain, build, browser, negative/repeat visual and baseline-approval
steps are preserved exactly. Acceptance policy, PR/issue templates and active
owner guides now allow a labelled same-session review by the implementing agent;
extra agents are optional only when explicitly requested. Historical reports
and original evidence remain unchanged.

Base: PR #769, branch `issue-762-player-interactions`, head
`b990a634c8d97247ccb93ef2c956bb82751641bb`. Child branch
`task-retire-independent-qa` targets that parent branch. Merge the parent first,
then retarget/revalidate this child against main. No source issue is closed by
this separately requested maintenance task. Final comment binds published SHA/CI.

## Verification and review

| Requirement | Result / evidence |
| --- | --- |
| Stop live QA automation | PASS. Workflow 406890869 is `disabled_manually`; [observed state](evidence/live-workflow.json). Listing all its current runs found none queued/in progress, so no cancellation was needed. No other workflow was disabled. |
| Remove gate source and preserve functional checks | PASS. YAML parsed; compared the entire resulting Navigation workflow against the parent after allowing only the two QA-specific steps and two obsolete path triggers to be removed. All remaining steps, baseline approval, permissions, concurrency and triggers match. Deleted workflow/runtime absence verified. [Validation](evidence/validation.json). |
| Update active instructions and review changes here | PASS. Same-session `/root` reviewed final source/configuration diff and policy consistency. New local guide links and `git diff --check` pass. Gameplay inputs did not change, so no duplicate local gameplay campaign was run. Current-child CI runs the existing workflow to verify the edited CI path; final status is bound in the PR comment. |
| Update Salimon Issue Manager | PASS. Existing private USER plugin updated in place to 0.8.0. Six skills/identity/audience and existing default prompts preserved; implementation, review, batch, planning and shared guidance now default to review in this chat, without required subagents, handoff packets, review jobs/statuses or scheduled scans. All nine files read back byte-for-byte. [Release record](evidence/plugin-update.json). Future host invocation not exercised. |

Validation used Python 3 with PyYAML 6.0.3 for workflow/frontmatter parsing and
JSON/ZIP readers for the plugin. Compared retained workflow semantics against
`git show b990a634:.github/workflows/navigation-client.yml`; parsed both plugin
manifests and all skill frontmatter, resolved local references, preserved the
existing complete defaultPrompt values and inspected the nine-entry ZIP before
upload. No new ongoing validation job or test suite was introduced.

## Existing delivery review and limits

PR #769 is accepted by [review in this active chat](https://github.com/mr-exception/salimon/pull/769#pullrequestreview-5773202600)
at head `b990a634`, base `ecae30f8`, tested merge `dcc7b029`.
[Current-source CI](https://github.com/mr-exception/salimon/actions/runs/41878078126)
passed 28 browser cases, domain/build, navigation/visual negative demonstrations,
repeat comparisons and baseline approval. The code, 49 source hashes and original
Core/cockpit screenshots were reviewed. The old queue/packet blocker is superseded;
this is same-session review, not a newly run independent agent or gate.

The old pending `salimon/independent-qa` commit-status entry is historical.
Updating it to reflect retirement was denied with HTTP 403 `Resource not accessible
by integration`; no alternate credential/tool was used to bypass that permission.
Main is unprotected and the repository ruleset list is empty, so it is not a
required check. New review jobs/statuses will not be created by the disabled workflow.
The earlier browser timeout overlapped an input edit/HMR; the final clean run
passed, and no sole cause is claimed for the original blank-viewport failure.

Same-session review found no blocking defect in this removal. Required functional
CI failures still block delivery; changed baselines still require human approval.
No merge, deployment, release or issue closure was performed.
