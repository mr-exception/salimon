# #784 — Scout Resource Container domain

The default ResourceSession now declares the existing ship-local `scout:storage`
fixture as an empty 50,000 g raw-only store. Explicit `deposit` / `withdraw`
commands use #783's revision-checked ledger and atomic storage replacement.
Schema v4 adds lossless v3 missing-container migration alongside v1/v2 migration;
current missing/corrupt stores fail closed. No UI or container slot cap is added.

Base: `main` at `cf46092` (merged #783/#790); no active PR stack existed.
Tested runtime/source commit: `05b4273659368c9d035c0afc0c58437e0c2a876c`.
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
