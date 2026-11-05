# Issue #787 — browser survival

Implemented deterministic active hunger/fatigue, 200 HP, starvation/passive recovery,
launch-only offline fatigue credit, schema-v6 persistence, movement/exhaustion policy,
terminal gathering/movement freeze, explicit fresh-character reset and accessible HUD.
Reset preserves the entire resource ledger and source depletion; no supplies are granted.

Branch: `issue-787-survival`, based directly on `main`
`d80eb51616b8334f7a28814b92358571ba64b46f`. No active PR stack existed at branching
or publication preflight. #783 is merged (PR #790); its required ledger code is in
this ancestry. #782's current/native reconciliation and #786's count-item changes
are also present. There is no dependency on an unmerged branch.

Latest tested source: `f1ac352ec935326ebc2e9187f19d3e1cea358a45`.
Unit/build and the ten-test affected browser campaign used
`5145acfb266a7783e10cec2523f635774ac2972b`; the only subsequent client change moves
the interior capture inside the ship and asserts its frame. That changed test
passed again at `f1ac352`. Unit/build/other browser inputs are unchanged.
`source-inputs.sha256` binds all tracked client/workflow inputs. The final report
commit changes only documentation and evidence; the published PR comment binds
its actual head/base, current-head CI and final review verdict.

## Acceptance evidence

| Criterion | Local result | Evidence |
| --- | --- | --- |
| SURV-1 | PASS | Exact tick rates, partition invariance, 600-tick starvation, pause/background, cockpit clock, launch credit/rollback/repeat tests; `logs/787-final-unit.log`, `evidence/survival-final/records/survival-records.json` |
| SURV-2 | PASS | 200 HP clamp, regen thresholds, terminal truncation, action/movement policy boundaries, terminal reload/control/reset journey |
| SURV-3 | PASS | Whole-ledger atomic publication, v1–v5 migrations, injected quota/stale writes, rejected/corrupt survival, failed launch credit, no reset/grants on failure; gathering/storage/item browser journeys |
| SURV-4 | PASS | Production labelled meters outdoors/indoors/in navigation and after return, numerical text and warning transitions; five originals below |
| SURV-5 | PASS | Explicit tuning follow-up [#799](https://github.com/mr-exception/salimon/issues/799); current mining stays 1 kg/s. Future machine actions/hazards/consumables are excluded, not claimed implemented |

| Category | Exact command/procedure | Actual result |
| --- | --- | --- |
| Unit + integration (mandatory) | `cd client && npm test` | PASS, 115 tests; production sessions, deterministic elapsed ticks without timeouts, failed/stale save injection |
| Build (mandatory) | `cd client && npm run build` | PASS; existing large-bundle advisory only |
| Full browser (mandatory) | `CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=../reports/issue-787/browser npm run test:browser` | Initial diagnostic: 35 PASS / 5 FAIL. Four gathering/storage cases and one reference environment mismatch corrected/rechecked below. Retained as failure evidence, not a final full-suite pass |
| Affected E2E + exact visual reference | `SALIMON_EVIDENCE_DIR=../reports/issue-787/final-browser npm run test:browser -- --project=cockpit --project=navigation-captures` | PASS, 10/10 on locked Chromium; eight reference comparisons pass. `logs/787-final-browser.log`, `evidence/final-browser/results.json` |
| Changed interior capture | `SALIMON_EVIDENCE_DIR=../reports/issue-787/survival-final npm run test:browser -- --project=cockpit --grep 'SURV built'` | PASS, 1/1; asserts the player is in `ship:scout` before the interior original |
| Baseline approval | `SALIMON_BASE_SHA=d80eb51616b8334f7a28814b92358571ba64b46f npm run test:visual:approval` | PASS, inherited unchanged baseline; `evidence/baseline-approval.json` |
| Full final browser, navigation negative mutations, repeat visual comparison, visual negative mutation | Current-head Navigation client workflow | Pending at report creation; final run links/results and acceptance are recorded in the PR review comment. No local pass is claimed for these final-head CI steps before execution |
| Manual/hardware | N/A for routine browser-local acceptance | Balance feel, physical input and real-device performance remain separate release qualifications; software rendering is not hardware-performance evidence |

## Original captures and measurements

The final five originals are under
[`evidence/survival-final/survival-SURV-built-active-9de1a-n-return-and-terminal-reset-cockpit/`](evidence/survival-final/survival-SURV-built-active-9de1a-n-return-and-terminal-reset-cockpit/):
`outdoor.png`, `interior.png`, `navigation-return.png`, `low-condition.png`,
`terminal.png`. Inspected in the active chat. The interior frame assertion avoids
mistaking the doorway approach for an indoor capture. Image decoding is awaited
before capture; earlier blank-scene candidates remain diagnostic evidence only.

`evidence/final-browser/records/` retains cockpit, gathering, storage, item,
repeat-comparison and eight-reference measurements. `evidence-index.json` maps
original paths to retained PNGs and SHA-256 hashes. Original result JSON and logs
retain failures as well as final passes. Full local trace trees remain at
`/workspace/evidence-issue-787`; essential PNGs/state/measurements are committed
here, so acceptance does not depend on that machine or expiring CI traces.

Environment: Linux x64, Node v24.19.0, Playwright 1.64.1, locked Chromium
156.0.8078.4; initial diagnostic used system Chromium 154.0.8037.57 and correctly
failed the exact-environment gate. SVG uses CPU rasterization; cockpit WebGL 2
reports ANGLE Vulkan / SwiftShader Device (Subzero), not a physical GPU.
1280×800, DPR 1, en-US/UTC, dark/reduced-motion; controlled epoch
2026-11-04T00:00:00Z. Input is Playwright synthetic keyboard/mouse plus explicit
visibility/blur events. Low/terminal conditions use a saved-state fixture through
the real decoder, with no shipping query/runtime test backdoor.

## Review and documentation

Reviewer: implementing Codex agent, same active chat; no independent execution is
claimed. Reviewed final source, criterion boundaries, production integration,
original PNGs and raw results. Findings fixed: development save-lock remount race,
radar accumulator reset, reset-button keyboard focus, clock-only confirmation
expiry, read-only storage inspection after save failure, reload/pagehide clock
assertions, browser UTC normalization, and premature/mislabelled captures.
The delayed-confirmation regression was observed failing before its fix
(`logs/787-container-regression.log`) and passes in the final 115-test run.

Local criterion review: PASS. Final verdict is **QA pending current-head CI**;
ACCEPT is withheld until all required workflow steps resolve. See the PR review
comment for the final exact head/base, CI and verdict. The issue stays open until
owner merge; no merge, deployment or release is authorized here.

Canonical behavior and deliberate follow-ups are documented in
[`docs/browser-survival.md`](../../docs/browser-survival.md); inventory/save,
playable-world, status, maintenance and client guides are aligned with schema v6.
Historical native 100 HP and provisional mining defaults are explicitly separated
from current 200 HP behavior. Follow-up #799 owns approval of mining/discrete-action
tuning; new food/medkit content and hazards remain outside this issue.
