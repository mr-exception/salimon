# Approved survival work tuning (#799)

Implemented the owner's [2026-11-06 approval](https://github.com/mr-exception/salimon/issues/799#issuecomment-6679680904): hunger <20 and fatigue ≤20 each multiply mining by 0.75; together 0.5625. Baseline remains 1 kg/s. Movement has a 10% floor (current components already bottom out at 22.5%); terminal health still stops movement/work. Hunger/fatigue/offline durations and 200 HP remain unchanged.

Mining uses 960 work units per extraction and 16/12/9 units per active tick, sampled before that tick's survival change. Work stays with its source and checkpoints across interruptions. Schema v8 stores canonical partial work with clock, inventory, location and depletion; v1–v7 migrate without work/resource grants. Extraction atomically debits work and credits capacity/source-limited integer grams. Failed saves restore the entire last checkpoint and freeze play.

Future Core intake, Fabricator submit and tray collection share a persisted 15-active-tick recovery through `completeDiscreteAction`. Validate the machine operation first, then atomically save its result and returned survival deadline. Emergency interactions bypass recovery/exhaustion and do not extend it. Production machines remain inspection placeholders; no playable machine integration or catalog is claimed. Continuous mining, storage transfers, inspection and cockpit exit are exempt.

## Criteria and verification

| ID / category | Required observable result | Command / evidence | Result |
| --- | --- | --- | --- |
| TUNE-1 / decision | Explicit owner approval precedes coding | Linked owner decision, unchanged issue history | PASS |
| TUNE-2 / unit | Exact threshold/composition; work, clock and deadline boundaries; unchanged durations/HP | `cd client && npm test`; [unit log](logs/unit.log), 132 tests | PASS |
| TUNE-3 / integration | Per-tick/batch equivalence, source-bound interruption/reload, malformed work rejection, v7 migration, capacity, failure rollback, shared action deadline and emergency bypass | Same complete unit/integration command; `miningPolicy.test.ts`, existing session/save suites | PASS |
| TUNE-4 / build | Production typecheck and build | `cd client && npm run build`; [build log](logs/build.log) | PASS |
| TUNE-5 / browser | Production mining 0.75/0.5625 rates; saved partials; exact source debit; repeated reload without duplication; HUD warnings | `npm run test:browser`; final originals and raw save records collected below | Pending final browser/CI result |
| TUNE-6 / visual | Readable active/low-condition HUD, existing repeat/negative/navigation baseline gates | Existing browser/visual workflow; no baseline changes | Pending final browser/CI result |
| Manual/hardware | Subjective balance feel, real-device performance and physical input | N/A to routine cloud delivery; synthetic input/software rendering only | N/A |

## Revision and environment

Base/default `main`: `a42d97042701fa0c00ddaee04a83690544ec1de1` (includes #787 and #788). No open PR stack or formal issue blockers existed at preflight. Head branch: `issue-799-survival-work-rate`; PR targets `main`. Tested source commit: `91182a2ed99a45b5764246536f7849af170e1c45`; subsequent guide/report commits do not change source, tests, assets, lockfile or configuration. Final head/base, CI and review are bound in the PR comment.

Linux 6.18.57 x64, Node 24.19.0, npm 11.9.0, Playwright 1.64.1. Initial production diagnostic used system Chromium 154; pinned Chromium 156.0.8078.4 is installed for exact visual-environment comparisons. SVG uses CPU rasterization; cockpit WebGL uses ANGLE SwiftShader, not physical GPU hardware. Viewport 1280×800/DPR1, en-US/UTC, dark/reduced-motion. Controlled clock starts 2026-11-04; low conditions are saved-meter fixtures through the actual decoder. Inputs are Playwright synthetic keyboard/mouse.

Earlier iteration failures were stale schema-version/migration fixture assumptions and an invalid test survival timer, corrected before the final 132-test pass. Initial npm install failed because its default cache was not writable; installation succeeded using `/tmp/salimon-npm`. An initial Git fetch could not reach the proxy in the default sandbox; network-enabled execution succeeded. These are not runtime passes. The production preflight proved all three rate cases and retained original screenshots; final evidence is required after the zero-elapsed work-entry fix.

## Active-chat review

Reviewer: implementing Codex agent in this same active chat; no independent execution claimed. Reviewed approval, source/unique diff, deterministic boundaries, atomic work/resource accounting and original low-condition screenshot. Fixed a zero-elapsed interaction edge case before final testing, with a regression test. Current verdict: **QA pending final browser and current-head CI**. Acceptance is withheld until required checks pass. The issue stays open until owner merge. No baseline promotion, merge, deployment or release is authorized.

Canonical guides: `docs/browser-survival.md`, current-client status, maintenance routing, resource design and client README. Existing navigation baselines and all survival timing/HP constants are preserved.
