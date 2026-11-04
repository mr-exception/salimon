# Independent review evidence gate (#752)

Independent AI review runs in the active Codex session, using a distinct reviewer
subagent when available. The [trusted CI runtime](../.github/qa/review.py) verifies
that review's submitted record against current tests, source and screenshots.
CI does not invoke a model, provision an agent or require an external AI API key.
The user explicitly excluded unattended external AI review from this delivery.
Human retains merge authority; the runtime never merges, closes issues, edits
code, deploys, or approves screenshot references.

## Session review and deterministic gate

A delivering PR must have exactly one local `Closes #N` reference and 1–20
acceptance bullets under `## Acceptance criteria`. The runtime assigns AC-1
onward. It reads full issue/discussion, PR patches, exact head/base/merge, latest
completed Navigation client PR run and successful `build`/`baseline-approval`
executions. Partial job reruns can reuse same-revision successful jobs.

The primary artifact must contain all eight source-bound actual/expected/diff
PNGs, state/environment, approved manifest and trace. Negative/repeat directories
cannot substitute for the primary evidence. Human reference approval is verified
independently from exact base plus trusted-main inheritance or an exact existing
human approval comment. Changed CI workflows or mandatory npm scripts require
human review and block automated acceptance. AI cannot override those gates.

Without a current session verdict, the job records BLOCKED and exports
`handoff-request.json` alongside `report.json` as a 90-day artifact. The request
includes criteria, full issue/discussion, reviewed diff, test results, state,
measurements, manifest, citations, exact revisions and the original artifact link.
Give those materials and actual images to a distinct Codex reviewer. Require one
PASS/FAIL/UNVERIFIED row per criterion, known evidence citations and actionable
findings. Missing capability/evidence must remain BLOCKED; never invent agents or
screenshots. Trace availability is distinct from interactive trace replay.

Create the local submission after genuine independent review:

```sh
python3 .github/qa/handoff.py --packet /tmp/handoff-request.json \
  --review /tmp/independent-review.json --reviewer /root/reviewer \
  --implementer /root --output /tmp/review-comment.md
```

The helper only writes a local comment file. Publish it to that PR through an
existing authorized GitHub tool. Its marker is `<!-- salimon-codex-review:v1 -->`.
Only OWNER/MEMBER/COLLABORATOR comments authored by a User are eligible. The
submitter attests actual separate-agent execution; identities are **not**
cryptographic proof of agent execution. Keep independent review evidence and
human review of trusted CI code. A comment alone cannot provision a reviewer.

The record binds the entire reviewed packet, issue/criteria, CI run/attempt,
source head/base/merge and every capture/state/trace hash. Re-run the evidence
preparation and independent review when context changes. CI validates that record
and deterministic checks, then publishes actionable comments, machine JSON and
`salimon/independent-qa` status on the exact original head. Failed CI vetoes AI
approval; missing/unrun checks, absent/invalid handoff, stale context, expired
artifacts and active human change requests block acceptance. Only all-pass
criteria and mandatory CI may produce ACCEPT.

Immediately before publication, the runtime rechecks head/base/merge, issue
body/discussion, current CI attempt/results, blocking reviews and the latest
matching handoff. A newer change request wins over an older acceptance. Edited,
withdrawn or deleted records trigger revalidation; bot report comments do not
recursively trigger more reports.

## Limits and invalidation

At most three consumed session-review cycles per PR; source edits do not reset
that budget. Failed preparation or a missing reviewer does not consume a cycle.
After the cap, human investigation/new delivery is required. There are no model
calls, model retries, API credentials or model resource costs in this workflow.
An exact fingerprint binds input/runtime bytes, submissions, CI and artifacts;
duplicate events revalidate live evidence and restore status without duplicate
cycle comments. Expired/deleted originals invalidate replay.

Each invocation permits at most 100 GitHub calls, 500 entries per list, 200 E2E
results, 200 KB patches, 350 KB packet JSON, 8 MiB aggregate images, 128 MiB ZIP /
256 MiB expanded entries, 2,000 ZIP entries and 32 MiB per entry. Artifacts are
read as data, never extracted or executed. Oversized/unsafe data blocks. The job
has an eight-minute timeout. Source scripts use only Python's standard library.

PR changes, pushes and a 15-minute bounded scan invalidate head/base/merge drift.
The scan also checks withdrawal of accepted handoffs and issue-context changes;
it supports up to 20 open PRs within the GitHub call budget. Statuses cannot
atomically encode a moving base or eliminate all event races. Require strict
up-to-date branch protection or a merge queue as well. Stacked parent changes
require new evidence and session review. Preserve essential originals beside the
issue report because navigation artifacts expire after 14 days.

## Rollout

1. Owner reviews/merges the infrastructure through normal independent review.
   Trusted workflows run only default-main code with persisted credentials off;
   they never execute PR files with write credentials. Bootstrap skips an absent
   runtime on trusted main; that is not an independent QA acceptance.
2. Run Navigation client, obtain its evidence packet, conduct independent review
   in Codex and submit the exact record. PR comment events or workflow dispatch
   validate it. No extra AI service, key, subscription or unattended worker setup.
3. Only after jobs/statuses actually exist and failure/success paths have been
   observed should an owner require existing `build`, `baseline-approval` and
   `salimon/independent-qa`, with strict base freshness. This PR changes no branch
   protection. The owner still decides whether to merge.

## Verification

```sh
python3 -m unittest discover -s .github/qa -p 'test_*.py' -v
```

Controlled fixtures use simulated GitHub transport and test session records.
The stronger demo reuses real wrong/repaired browser captures at distinct actual
commits and the actual distinct reviewer's source/hash-bound verdicts. Only GitHub
transport is simulated; the production handoff validator and veto policy run.

```sh
python3 .github/qa/demo.py --wrong /tmp/wrong --fixed /tmp/fixed \
  --wrong-review /tmp/wrong-review.json --fixed-review /tmp/fixed-review.json \
  --output /tmp/qa-demo
```

[Issue report](../reports/issue-752/README.md) retains originals, diffs, states,
traces, logs and transition JSON. Hardware/native/physical-input qualification
remains outside this infrastructure.
