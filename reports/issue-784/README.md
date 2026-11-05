# #784 — Scout Resource Container domain

The default ResourceSession now declares the existing ship-local `scout:storage`
fixture as an empty 50,000 g raw-only store. Explicit `deposit` / `withdraw`
commands use #783's revision-checked ledger and atomic storage replacement.
Schema v4 adds lossless v3 missing-container migration alongside v1/v2 migration;
current missing/corrupt stores fail closed. No UI or container slot cap is added.

Base: `main` at `cf46092` (merged #783/#790); no active PR stack existed.
Tested gram-domain source commit: `05b4273659368c9d035c0afc0c58437e0c2a876c`.
Final head/base and current-head CI are bound in the publication review comment.
Later documentation/report edits leave the hashed source/test/config inputs unchanged.

| Criterion | Result / evidence |
| --- | --- |
| STORE-1 | PASS: empty identified 50 kg raw store; unknown/count IDs and count metadata rejected; no volume/slot cap. `scoutResourceContainer.test.ts` |
| STORE-2 | PASS: five 10 kg deposits fill exactly 50 kg; overweight, insufficient, fractional/unsafe/negative and stale requests preserve bytes/state; explicit 333 g withdrawal. Generic 13-resource fixture proves player 12-slot cap and uncapped raw store |
| STORE-3 | PASS: real session reload retains ID/grams/revision; v1/v2/v3 migrations, failed migration writes, current missing/changed metadata, quota/stale save bytes fail closed. Existing session and new container tests |
| STORE-4 | PASS: production session payload, store plus onboard/offboard player, repeated simulation mass recomputation, unchanged world/source entities. Production TopDownView already consumes this payload; no fragments |
| STORE-5 | PASS: built GATHER-1/2, SLICE-1 and focused COCKPIT-1 rerun; source depletion, mining, cockpit flow and screenshot repeats unchanged |

## Commands and environment

From `client/`: `npm test` PASS (16 test files); `npm run build` PASS (existing
bundle-size warning). Final logs retained here. `CHROMIUM_PATH=/usr/bin/chromium
SALIMON_EVIDENCE_DIR=/tmp/issue-784-preflight npm run test:browser -- --project=production`
PASS (1 test), proving a real built WebGL render/SwiftShader backend before campaign.
`CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=/tmp/issue-784-browser npm run
test:browser -- --project=cockpit` initial result: 3 passed, 1 timed out (7.1 min). GATHER-1/2 and SLICE-1 passed, including repeat visuals. COCKPIT-1 hit 180 seconds in replay two; only it is rerun with `--grep COCKPIT-1 --timeout=360000`, unchanged assertions/tolerances. Original failure captures/log/context and successful raw results are retained; the 21 MB timing trace stays in temporary execution output. No functionality assertion failed in the timed-out run.
Broad navigation/negative/repeat/baseline approval and Docker smoke use existing
current-head CI. No new jobs, gate, baseline or tolerances.

Linux cloud; Node v24.19.0/npm 11.9.0, system Chromium, SVG CPU rasterization and
ANGLE SwiftShader radar. Browser metadata is in original JSON results/attachments;
1280×800/DPR 1, en-US/UTC, fixed 2026-11-04 clock, synthetic keyboard/mouse input.
Domain-only visuals N/A; existing affected screenshot comparisons remain required.
Hardware/physical input/backend authority N/A; browser-local single-writer scope.

Setup failures: initial fetch lacked network permission; initial npm install used
an unwritable default cache. Both repaired with command network permission and
`/tmp/salimon-npm-cache`. Initial browser preflight could not start the localhost
server in the default network sandbox; rerun with network permission passed.
No functional failure was waived. A duplicate unit/build invocation was retained
as final verification after source edits; only final logs establish acceptance.

Canonical resource/client/architecture/maintenance/status/playable guides updated.
`python reports/issue-782/check-links.py` PASS (346 links, 8 guides); `git diff --check`
PASS. Essential originals/raw results are retained, with hashes. Branch protection
read returned 403 for the GitHub integration; ruleset list was empty. Actual
current-head CI checks are followed separately, never inferred from an empty list.

Same-session implementing-agent review inspects the unique diff, criteria,
source-bound tests and original browser results. No independent execution is
claimed; final verdict and current-head checks are published on the PR/issue.
Human retains merge authority; source issue remains open until merge.

## Final local review

Focused cockpit recheck: **PASS**, 1/1 with the 360-second execution budget;
see `cockpit-recheck.log`, original PNGs and raw backend/simulation/comparison
attachments. The initial timeout remains recorded. All four mandatory world
scenarios are now passed; no assertion or image policy was relaxed. Seven
gathering/slice and four cockpit repeat comparisons passed with zero pixels
beyond the unchanged perceptual threshold. Wrong-checkpoint negatives rejected
as expected. Original radar/full-inventory/standing-return captures were inspected
and remain legible with unchanged fixtures and controls.

Same-session implementing-agent local review: **ACCEPT**, STORE-1–5 PASS, no
material findings. Required current-head CI is still pending publication; final
acceptance is bound on the PR after those checks resolve. New-widget visual and
hardware qualification are N/A for this domain-only browser-local change.

## Concurrent documentation stack integration

PR #793 (`issue-791-2d-doc-corpus`, `bf1cb0df46d46047e9ee01460a26a4d083f85cce`)
appeared during implementation (created 12:44 UTC), after initial discovery found
no active PRs. To preserve the required linear stack, this branch incorporates
that documentation-only history non-destructively and now targets #793. Merge
order is #793 → #794 → the #785 child. The client README conflict preserves both
the new migration-register routing and schema-v4 container documentation. Its
register row is updated for the newly delivered domain.

All runtime/test/asset/config/lockfile hashes remain equal to source-hashes.json;
existing local evidence is reused. Link/diff checks are repeated and current-head
CI reruns before final acceptance. The previous exact-head acceptance is superseded
by the final stack review comment, which binds new parent/head and CI.

The complete unique-diff check also found trailing spaces in two preserved
initial-timeout log lines. Trimmed whitespace only; failure text/data are retained.
Final source hashes remain unchanged.

## Required visual regression repair and re-review

Final-head CI 42033304505 passed all four world/cockpit/gathering scenarios but
failed the existing TD-1 byte-equality gate. Both PRs reproduced exactly eight
changed pixels in the bottom-left rounded caption corner; originals, exact RGB
deltas and failed CI output are retained under `failures/ci-42033304505`.
A passing local parent control was insufficient to waive this observed failure.

The translucent caption now has its own 7 px inner bottom curve, matching the
stage's 8 px radius minus 1 px border. This removes compositor-dependent blending
without changing the frame/art, thresholds, masks or approved references. Eight
replay probes were byte-identical with this alignment, while the unmodified
control varied. The existing unmodified TD-1 test passed **3/3 fresh repeats**
(30.1 s); originals and results are under `clip-browser`. New build/typecheck
passed (`clip-build.log`). The tested renderer change is `ee96f02`; all gram
domain/test inputs remain unchanged, justifying unit reuse. Final source hashes
include the CSS change; full new-head CI remains mandatory. Visual qualification
is now applicable to this small shared-view repair, with TD-1 originals and
current-head full browser/negative/repeat checks; new-widget/hardware remain N/A.

Previous exact-head acceptance is superseded pending fresh CI and final active-chat
review. This repair is needed for the named issue's existing mandatory screenshot
gate; no extra task is counted and no rendering failure is dismissed as flaky.

## Merged-main conflict resolution and final clipping correction

The owner merged #793 and #797. This branch now includes main `f51350a`,
with README/schema conflicts resolved for v4 and the latest 109-row migration
register/backend contracts preserved. Runtime, test, lockfile and configuration
inputs are unchanged by that documentation merge. Parent is now main; merge order
is #794 then #795. No history was rewritten or PR merged by the implementing agent.

CI for prior `8f9a924` passed in full, but descendant CI 42037274989 still found
seven corner pixels under rounded clipping. Thus the 7 px caption adjustment was
not sufficient and is superseded. The final viewport/caption use rectangular,
pixel-aligned clipping; the only visual change is removal of the viewport
corner curve. World art, coordinates, camera, footprint geometry, dimensions,
assertions, thresholds and approved references are unchanged. No failing visual
is waived. The same original TD-1 assertion passed **5/5 fresh runs** (ten capture
originals) under `square-browser`; build/typecheck passed. Tested rendering source:
`9a907ba8d88daf56569e98c7f6ad0e471a30a898`. Pure gram/Node inputs are unchanged;
unit results remain applicable. Final source hashes include the corrected CSS.
Full current-head CI and final exact-ref review remain required after publication.
