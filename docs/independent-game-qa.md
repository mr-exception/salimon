# Independent game QA (#752)

The [trusted runtime](../.github/qa/review.py) reads one open Salimon PR and calls
a separate, stateless multimodal OpenAI Responses request. It sends no developer
chat, tools or write authority to the model. This is implemented infrastructure;
the interactive Salimon skills alone do not provision a persistent agent.
The owner retains merge authority. Nothing merges, deploys, approves baselines,
closes issues, edits code or retries a developer fix automatically.

## Evidence and verdict

Use exactly one local `Closes #N` reference and an issue with 1–20 bullets under
`## Acceptance criteria`. The runtime assigns AC-1 onward without rewriting the
issue. It reads the full issue/discussion, paginated PR file patches, current
head/base/merge, current completed Navigation client PR run, latest executions of
`build` and `baseline-approval`, and the primary navigation artifact. A partial
job rerun can reuse successful jobs from that same exact revision. It never
confuses the deliberate negative captures or repeat directory with the primary.

Every one of eight frames must include source-bound actual/expected/diff PNGs,
state/environment, the approved manifest hash and retained trace. The model sees
all 24 images, per-frame state, comparison measurements and E2E results; a trace
citation proves availability, not that a model has replayed it. Human baseline
approval is independently verified from exact base + trusted main inheritance or the existing
exact human approval comment. AI cannot substitute for it. Changed CI workflow
or mandatory npm scripts are BLOCKED for human review rather than trusted as
successful deterministic checks. Ordinary source/test diffs remain reviewable.

The strict response schema requires one PASS/FAIL/UNVERIFIED row per criterion,
known evidence citations, finding and action. Unknown citations, malformed or
incomplete output, missing/expired capture, unavailable provider, stale evidence,
unfinished/skipped required CI or active human change requests produce BLOCKED.
Failed deterministic CI yields REQUEST_CHANGES even if AI says ACCEPT. Only
all-pass criteria plus successful required CI can yield ACCEPT. Head/base/merge,
issue body, blocking reviews and current run/job results are rechecked before
publishing. An approval is attached only to the exact original head SHA.

The PR comment contains a durable cycle index, actionable findings and complete
machine-readable JSON, including capture/state/trace hashes, source/base, CI run
and attempt, model response identity, limitations and artifact link. The same
JSON is uploaded for 90 days. Original browser artifacts retain the existing
14-day policy; preserve essential originals beside the issue report. An expired
artifact blocks replay even if a previous review accepted it.

## Bounded operation and invalidation

At most three model cycles per PR, including failed provider calls. There are at
most two API requests per cycle, retrying only HTTP 429/500/502/503/504 once after
two seconds. Each call has a 90-second timeout and 4,000 output-token cap; there
are no model tools. Maximum six requests/24,000 provider output tokens across three
cycles (retries may consume provider tokens too). Authentication errors do not
retry. A fingerprint binds head/base/merge, issue/discussion, review state, diff,
model/configuration presence, trusted runtime bytes, CI attempt/results and artifact hashes. Identical
requests revalidate live evidence and restore the status without another model
call or duplicate comment. A transient failure remains BLOCKED on replay;
fix configuration or rerun CI to create a new evidenced cycle. At the limit,
human investigation/new delivery is required; changing source does not reset it.

Per invocation: at most 100 GitHub calls, 500 entries per list, 200 E2E results,
200 KB PR patches, 350 KB packet JSON, 8 MiB total images, 128 MiB compressed /
256 MiB expanded artifact data, 2,000 ZIP entries and 32 MiB per entry. No ZIP
is extracted or executed. Oversized, unsafe, duplicate or missing data blocks.
Large PRs must be split. The workflow has an eight-minute wall-clock cap.

Navigation CI completion and explicit dispatch request a review; unrelated
pushes never spend model tokens. PR synchronize/base edits, branch pushes and a
15-minute scan invalidate statuses when the current head/base/merge differs from
the latest recorded review. Each PR has its own workflow concurrency group.
Scans are bounded to 80 open PRs. Per-head GitHub statuses cannot atomically bind
a moving base: use strict up-to-date branch protection or a merge queue as well.
The scan bounds missed-event recovery, not permission to merge stale evidence.
After a stacked parent merges/retargets/advances, rerun CI and independent review.

## Installation and required-check rollout

1. Owner reviews and merges this infrastructure through the normal independent
   review process. `workflow_run` and schedules activate only from default main.
   Pre-merge push invalidation explicitly skips if trusted main has no runtime;
   this bootstrap is not an independent-qa ACCEPT. No auto-merge is enabled.
2. Configure repository secret `OPENAI_API_KEY` with vision/model access. Optional
   repository variable `SALIMON_REVIEW_MODEL` defaults to `gpt-4.1`. Without the
   secret, actual captures are still checked and the result is BLOCKED.
3. Run Navigation client on a reviewable PR, then dispatch **Independent game QA**
   with its number if no completion event has triggered review. Read the exact
   JSON/comment and confirm both failure and success paths before requiring it.
4. Only after those jobs/statuses actually exist, an owner may add existing
   `build`, `baseline-approval` and `salimon/independent-qa` checks to protection,
   with strict base freshness. This change does not alter repository protection.
   Keep human review for modifications to trusted QA/CI code; do not allow an
   AI approval or a bypassed test to replace that decision.

The workflow checks out only trusted `main`, disables persisted Git credentials
and uses Python standard library. It never checks out PR code into the privileged
job, runs npm from a PR, executes artifact content or uses model output as code.
PR artifacts and issue text are untrusted data with explicit prompt boundaries.
Write token scope is limited to issue comments and commit statuses; model secrets
are passed only to the review job. Branch protection and provider installation
remain owner configuration, not silently inferred from a green unit fixture.

## Verification and controlled demo

From repository root:

```sh
python3 -m unittest discover -s .github/qa -p 'test_*.py' -v
```

Fixtures run the production orchestration with a labeled fake GitHub API and
mock HTTP provider transport. They cover deterministic veto, missing provider /
capture, stale head/base/merge, mid-review parent changes, expired evidence on
replay, independent baseline approval, protected CI scripts, bounded retries,
cycle exhaustion and idempotency. They are not live service passes.

For the stronger image demo, capture the real red-velocity CSS defect in a
disposable committed worktree, restore it in another commit and run the normal
suite. A distinct reviewer must inspect originals, references, diffs and states,
then supply source/hash-bound JSON. Replay with:

```sh
python3 .github/qa/demo.py --wrong /tmp/wrong --fixed /tmp/fixed \
  --wrong-review /tmp/wrong-review.json --fixed-review /tmp/fixed-review.json \
  --output /tmp/qa-demo
```

Only GitHub and reviewer transport are substituted; the verdicts are the actual
independent reviewer's, validated against exact supplied capture hashes. The
record explicitly distinguishes genuine browser/AI execution from simulated
PR status transitions. See [issue evidence](../reports/issue-752/README.md).
Hardware GPU/native/physical-input qualification is outside this infrastructure.
