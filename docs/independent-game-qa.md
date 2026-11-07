# Review changes and evidence in the active chat

The owner retired the automated independent QA gate (#752) on 2026-11-04 because
its handoff/status jobs and scheduled scans cost more than their added value.
The live `Independent review evidence gate` workflow is disabled. The removal
PR deletes its workflow, Python runtime/fixtures and Navigation-client QA-only
setup/test steps. Historical reports and captures remain evidence of their
original revisions; they are not current workflow instructions.

## Current review procedure

After implementation, review the issue criteria, final diff, applicable design,
actual test results and original screenshots in the same active Codex chat.
The implementing agent may perform this explicit review pass; identify it
honestly as same-session review. Use a separate reviewer only if the user asks.
Record exact head/base, each criterion's PASS/FAIL/UNVERIFIED result, evidence,
findings, unrun checks and ACCEPT, REQUEST_CHANGES or BLOCKED. Correct failures
and rerun only affected checks, then reassess the changed evidence.

Do not generate handoff packets, attestations, automated review statuses,
scheduled review scans or external AI calls. No review workflow, extra CI job or
machine identity record is required. Old `salimon/independent-qa` statuses are
retired history and are not required acceptance checks.

## Checks and evidence that remain

The existing Navigation client workflow continues to run locked installation,
domain/integration tests, build/typecheck, browser gameplay and production WebGL,
negative regression demonstrations and repeated visual comparisons. The owner
removed the human baseline-approval CI job in PR #834 and reviews appearance in
the deployed version, filing separate issues for visual problems. No human
approval comment or manifest digest approval is required. Failed or unrun mandatory
functional checks still block acceptance. Never loosen pixel tolerances or promote candidate baselines
to manufacture a pass. Reuse current-source CI and existing originals instead
of repeating full campaigns merely to prepare a review record.

Keep useful originals, logs and measurements beside the task report; bind local
evidence to its source and remote CI to its actual head/merge revision. An update
to the head or stacked parent requires checking ancestry, unique diff and affected
evidence again. Follow the [acceptance policy](acceptance-policy.md). Human retains
merge, deployment and release authority. Synthetic/software-browser evidence does
not qualify physical input, hardware performance or native/mobile platforms.
