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
| TUNE-5 / browser | Production mining 0.75/0.5625 rates; saved partials; exact source debit; repeated reload without duplication; HUD warnings | `npm run test:browser`; [originals and raw save records](evidence/mining-policy-records.json), [browser summary](evidence/browser-summary.json) | PASS locally; current-head CI bound in PR review |
| TUNE-6 / visual | Readable active/low-condition HUD, existing repeat/negative/navigation baseline gates | `PLAYWRIGHT_BROWSERS_PATH=/workspace/playwright-browsers npm run test:visual`; [pinned visual log](logs/visual-pinned.log); CI supplies negative/repeat/baseline gates; no baseline changes | PASS locally; current-head CI bound in PR review |
| Manual/hardware | Subjective balance feel, real-device performance and physical input | N/A to routine cloud delivery; synthetic input/software rendering only | N/A |

## Revision and environment

Base/default `main`: `a42d97042701fa0c00ddaee04a83690544ec1de1` (includes #787 and #788). No open PR stack or formal issue blockers existed at preflight. Head branch: `issue-799-survival-work-rate`; PR targets `main`. Tested source commit: `91182a2ed99a45b5764246536f7849af170e1c45`; subsequent guide/report commits do not change source, tests, assets, lockfile or configuration. Final head/base, CI and review are bound in the PR comment.

Linux 6.18.57 x64, Node 24.19.0, npm 11.9.0, Playwright 1.64.1. Initial production diagnostic used system Chromium 154; pinned Chromium 156.0.8078.4 is installed for exact visual-environment comparisons. SVG uses CPU rasterization; cockpit WebGL uses ANGLE SwiftShader, not physical GPU hardware. Viewport 1280×800/DPR1, en-US/UTC, dark/reduced-motion. Controlled clock starts 2026-11-04; low conditions are saved-meter fixtures through the actual decoder. Inputs are Playwright synthetic keyboard/mouse.

Earlier iteration failures were stale schema-version/migration fixture assumptions and an invalid test survival timer, corrected before the final 132-test pass. Initial npm install failed because its default cache was not writable; installation succeeded using `/tmp/salimon-npm`. An initial Git fetch could not reach the proxy in the default sandbox; network-enabled execution succeeded. These are not runtime passes. The production preflight proved all three rate cases and retained original screenshots; final-source evidence after the zero-elapsed fix is retained under `evidence/`. The complete system-Chromium browser run produced 42 PASS / 1 FAIL in 539 seconds: the exact visual-environment check rejected Chromium 154 against the Chromium 156 reference. Re-running only that failed visual suite with pinned Chromium 156 passed all eight screens in 19 seconds. The failed run/log is preserved, and no tolerance or baseline changed.

## Active-chat review

Reviewer: implementing Codex agent in this same active chat; no independent execution claimed. Reviewed approval, source/unique diff, deterministic boundaries, atomic work/resource accounting and original low-condition screenshot. Fixed a zero-elapsed interaction edge case before final testing, with a regression test. Local criterion review: PASS. Current verdict: **QA pending current-head CI**, including negative/repeat/baseline gates; the final PR review comment records acceptance only after those checks resolve. Acceptance is withheld until required checks pass. The issue stays open until owner merge. No baseline promotion, merge, deployment or release is authorized.

Canonical guides: `docs/browser-survival.md`, current-client status, maintenance routing, resource design and client README. Existing navigation baselines and all survival timing/HP constants are preserved.

## Retained originals and measurements

[Active combined-penalty HUD](evidence/active-mining-10-20.png),
[low-hunger HUD](evidence/active-mining-10-100.png), and
[low-fatigue HUD](evidence/active-mining-100-20.png) preserve the production rate
text, progress and warning layout. Post-extraction originals accompany them.
[Raw mining saves](evidence/mining-policy-records.json) show 444 units after
37 ticks for either individual penalty and 333 units for both. After restart,
80 total ticks credit exactly 1,000 g for either 0.75 rate; 107 total ticks at
0.5625 credit exactly 1,000 g and retain three work units. Every restored source
retains 59,000 g. [Survival records](evidence/survival-records.json) retain the
production ANGLE Vulkan/SwiftShader backend, pause/terminal and return states.
[Browser results](evidence/browser-summary.json), [failed broad run](logs/browser.log),
and the pinned visual result preserve executed results and environmental failure.
[Evidence hashes](evidence/SHA256SUMS) index the retained originals/raw data.
