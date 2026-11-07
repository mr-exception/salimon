# Issue #827 — Adaptive compact surface HUD

Implemented on `issue-827-adaptive-hud` against `main` / `01c31816f155b48b513761cd768e146811fbe0f2`.
Final runtime/test source: `5a8fde7a0ac0fbf4bc289132c4ad1911503c5cf0`.
Publication adds documentation/evidence only; [source hashes](source-hashes.json) bind unchanged runtime/test/config/asset inputs. Final publication head is recorded in the PR review comment.
Same-agent review in the active implementation chat; **QA pending / CI awaiting user report**.

The default playable world retains its full viewport and ship-up projection.
Labelled Health/Hunger/Fatigue meters and Pause/Help sit top left; carried kg,
shared slots and actual gathering-tool state sit top right. Field inventory / I
opens raw/count item details on demand. The contextual region advertises only an
eligible nearby action and expands for exact extraction progress, source identity,
rate and remaining capacity. Weight, slot, depletion, survival and power restrictions
remain explicit. No simulation, persistence, units, targeting or world-coordinate
owner changed.

## Criteria and evidence

| Issue criterion | Actual checks | Local result |
| --- | --- | --- |
| 1. Full viewport at 960×600, 1280×720, 1920×1080 | Built `/`: page/stage bounds, uniform projection, ship-up bow, no body scrolling, HUD non-overlap and unobstructed player; DPR 2 and resize | PASS |
| 2. Compact summary; on-demand complete raw/count details | Collapsed default, keyboard/pointer inventory, integer-gram kg display, count/stack mass and shared slots, empty/mixed/reloaded inventory | PASS |
| 3. Accurate work and capacity; no false tool/E actions | Gathering repeats, interruption/save/reload, weight-full and separate 12-slot count-item fixture; exact progress values from ResourceSession, no Gather affordance when blocked | PASS |
| 4. Low/zero health, restrictions, pause/focus and cockpit | Survival, hazard, pause/background, terminal/reset, reduced mining rate and exact partial work; cockpit power/seat/clock round trip | PASS |
| 5. Keyboard/pointer/focus and reviewed screenshots | Inventory/help isolation, Escape/focus return, focus/blur input release, storage confirmation/cancellation, repeated originals and intentional negative comparisons | PASS |

Original [broad run outcomes/measurement index](broad/evidence-index.json) retains
all 25 checks, including both superseded failures. Named JSON artifacts contain
raw state, exact save snapshots, world transforms, viewport measurements and pixel
comparison results. Original PNGs remain beside their scenario; images are review
candidates, not approved navigation references. No reference PNG, comparator policy,
negative test or baseline-approval gate was altered.

Representative originals reviewed in this chat:

- [960×600 exploration](broad/viewport-VIEWPORT-producti-18de9-eat-and-negative-comparison-cockpit/fresh-0.png), [1920×1080 exploration](broad/viewport-VIEWPORT-producti-919fd-eat-and-negative-comparison-cockpit/fresh-0.png).
- [Expanded inventory](broad/viewport-HUD-inventory-hel-5b462-cape-menus-keep-active-time-cockpit/inventory-expanded.png), [slot limit](broad/items-HUD-full-by-slots-us-78809-and-never-advertises-Gather-cockpit/slot-limit.png).
- [Panoramic survival/pause/help](broad/cockpit-COCKPIT-1-panorami-aeb94-t-and-persistent-simulation-cockpit/panoramic-horizon-0.png).

The slot/count-item capture is a non-shipping fixture whose external test controls
can scroll its page; production desktop viewport qualification comes from the
built `/` matrix above. Production count-item content remains empty, as before.

## Verification and revision binding

Commands run from `client/`:

- `npm ci`: PASS, 98 locked project packages; lockfile unchanged ([log](logs/npm-ci.log)).
- `npm test`: PASS, 158 tests ([log](logs/unit.log)). All owners imported by the Node unit/script suites and their tests/lockfile are unchanged by this issue.
- `npm run build`: PASS on final source, TypeScript + Vite; existing large-chunk advisory only ([log](logs/build-final.log)).
- `CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=../reports/issue-827/broad npm run test:browser -- --project=cockpit --project=top-down`: 23 PASS / 2 FAIL, no skipped checks, 15.5 minutes, at source `0386f86` ([log](logs/broad-browser.log)). The run originally wrote to `browser/`; its evidence was moved to `broad/` before rerunning only affected cases.
- `CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=../reports/issue-827/browser npm run test:browser -- --project=cockpit cockpit.spec.ts playable-slice.spec.ts viewport.spec.ts`: **PASS, 8/8, 8.0 minutes**, at final source `5a8fde7` ([log](logs/review-browser.log), [results/measurement index](browser/evidence-index.json)). Both earlier failures are resolved; all **26 distinct affected checks** have passing applicable results across these two runs.

The final source adds only inventory dismissal on cockpit transition, a copy of
critical condition text in open help, two focused input regressions and migration
of the slice's save-detail assertion into the new on-demand utility. Unchanged
world/gathering/storage/survival/hazard/standalone fixture results from the broad
run remain applicable: their domain/test/asset inputs, dependency lock, toolchain,
rendering configuration and exercised behavior are unchanged. Final viewport,
help, cockpit and complete slice coverage is rerun on the final source.

Full cross-project navigation/component/reference/negative CI is **UNVERIFIED**,
awaiting the user's result report for the published unchanged PR head. It was not
polled or duplicated as a second full local campaign. Human retains merge and
baseline approval authority.

## Findings, corrections and limits

- [Initial development run](initial/browser/evidence-index.json) preserves three failures and the interrupted campaign. A live development-module edit reset the cockpit during its first replay; that run is not final cockpit evidence. The hazard test caught rounded accessible health values; the final meter again exposes the exact authoritative value, rounding only visible text. Deferred utility focus interfered with a fixture click; focus now runs after the React commit with `preventScroll`.
- The broad cockpit run exceeded its six-minute wall-clock harness timeout during its second software-WebGL replay. The harness allowance is now ten minutes; all simulated durations, assertions, repeated states, negative checks and image thresholds remain unchanged.
- The broad slice failure was an old always-visible save-status assertion. It now opens Field inventory, verifies the same status and dismisses it before continuing. No save or gameplay assertion was removed.
- Source review found hidden inventory expansion could survive cockpit entry. The final source closes it on view transition and verifies its collapsed state after entry. Help also retains explicit condition text when open.
- Original captures and numerical/repeat/negative outcomes were reviewed by the implementing agent in the active chat. This is same-agent evidence review, not independent execution. Final ACCEPT awaits the user-reported required CI results.

Environment: Debian 13, Linux x86_64; reused Node 24.19.0, npm 11.9.0 and system
Chromium 154.0.8037.57. SVG uses CPU rasterization; panoramic WebGL uses ANGLE
SwiftShader, confirmed in the cockpit browser measurements. Playwright input is
synthetic keyboard/pointer; viewport contexts use en-US/UTC and reduced motion.
Hardware GPU, physical input, screen-reader listening and M1/Metal are not claimed.
The repository's pinned CI browser/reference environment remains separate.

## Environment additions

**None** to the reusable environment template. Project setup only: `cd client && npm ci`
(98 lockfile-pinned packages). Node/npm/Chromium were preinstalled and reused;
no browser/system/runtime package was installed or upgraded, and no saved template
was changed.
