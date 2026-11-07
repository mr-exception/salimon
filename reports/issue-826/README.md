# Issue #826 — Expedition Interface shared foundation

Implementation and same-session local review; QA pending CI report and human visual review.

- Issue: https://github.com/mr-exception/salimon/issues/826
- Base: `main` / `f329fe41c0c6bf83f12fde73bfc372b9c1fc278f` (merged documentation PR #825).
- Tested source: `c3a38eb20c8b8b52d48182b5ae142f4b71f2aecb`, branch `issue-826-expedition-primitives`.
- No active parent PRs at discovery. Formal blocked-by metadata is empty; the textual #824/#825 prerequisite is verified merged and present in branch ancestry.
- Report/capture/documentation-status-only commit follows tested source; [source hashes](source-hashes.json) bind unchanged runtime/test/config inputs. Final head is recorded in the PR review comment.

## Scope and acceptance matrix

| Criterion | Implementation / observable check | Result |
| --- | --- | --- |
| AC1: one token source, light/dark contexts, compass semantics | Scoped `src/ui/tokens.css`; gallery uses identical components over bright ground/near-black space. Opaque shared surfaces preserve contrast. Heading/velocity remain compass-only; native meter fill explicitly uses cyan. | PASS local source/capture review |
| AC2: component states and units | Button/IconButton, utility/equipment panels, Dialog, Meter/Progress, KeyHint/ContextPrompt and StatusLabel. Gallery exposes idle, hover, visible focus, active, unavailable-with-reason, pending, rejected and committed states; explicit HP/kg/s/g values. | PASS component/browser checks |
| AC3: keyboard and accessible semantics | Native buttons/meters/progress, named panels/dialog, explained keyboard-reachable blocked actions, native modal inertness plus explicit Tab wrapping, Escape/close focus return; input and modal shortcut sentinel remains zero. | PASS Chromium synthetic keyboard checks; physical input/screen-reader listening NOT RUN |
| AC4: preserve live views and world authority | No imports into existing game views; no existing styles, world coordinates, collision, gameplay state or save changes. Build retains existing distributed assets; full browser regression below. | PASS: all 68 browser checks, including live 2D/panoramic/storage |
| AC5: reproducible checks and originals | Unit bounds regression, type/build, gallery Playwright state tests; 960×600, 1280×720, 1920×1080, DPR 1; repeated screenshots byte-identical, intentional red-button mutation must differ. | Gallery PASS; human baseline approval NOT RUN |

## Commands and evidence

Run from `client/` unless specified. Retained console logs have trailing whitespace trimmed; result values and original PNG bytes are unchanged:

- `npm ci`: PASS, 98 locked project packages installed; lockfile unchanged.
- `npm test`: PASS, 158 tests. [Original result](logs/unit.log). The final UI bounds test/source is unchanged since this run; later edits affected only TSX/CSS/browser tests/docs, covered by final build/browser verification.
- `npm run build`: PASS (TypeScript + Vite). [Original result](logs/build.log). Existing large-chunk advisory remains; production output names `index-B5bgRJ8u.css` / `index-CoTLVLat.js` are unchanged between initial/final builds because the new gallery is development-only.
- `CHROMIUM_PATH=/usr/bin/chromium npx playwright test --project=ui`: PASS, 4 tests after correcting initial-focus Shift+Tab wrapping. [Initial failure and successful reference states](logs/initial-ui-focus-failure.log) preserve the defect; final captures below supersede these earlier images.
- `npm run test:browser`: **PASS, 68 passed / 0 failed / 0 skipped**, 38.7 minutes, pinned Chromium 156.0.8078.4. Includes all four gallery checks, both existing navigation/panoramic reference matrices, production WebGL and live 2D/cockpit/storage/survival checks. [Original log](logs/browser.log), [canonical per-test outcomes and durations](browser-results.json). No full suite rerun is needed for the subsequent documentation/evidence-only commit; runtime/test/config hashes were verified unchanged.
- [Contrast measurements](contrast.json): WCAG sRGB luminance ratios for eight actual opaque surface/text pairs, minimum 4.8559:1 (muted on control surface); focus cyan vs panel 8.3679:1. These measurements do not certify a future transparent HUD or renderer materials.

### Original gallery captures

Each image is a review **candidate**, not an approved baseline. Both contexts use the same dark instrument surfaces. The gallery itself scrolls; full-region captures retain all states. Native dialog bounds are checked within each actual desktop viewport. Repeat-image hashes and original dialog hashes are retained alongside each width.

| Desktop viewport | Planet context | Space context | Dialogs | SHA-256 index |
| --- | --- | --- | --- | --- |
| 960×600 | [planet](captures/planet-960.png) | [space](captures/space-960.png) | [planet](captures/dialog-planet-960.png), [space](captures/dialog-space-960.png) | [hashes](captures/hashes-960.json) |
| 1280×720 | [planet](captures/planet-1280.png) | [space](captures/space-1280.png) | [planet](captures/dialog-planet-1280.png), [space](captures/dialog-space-1280.png) | [hashes](captures/hashes-1280.json) |
| 1920×1080 | [planet](captures/planet-1920.png) | [space](captures/space-1920.png) | [planet](captures/dialog-planet-1920.png), [space](captures/dialog-space-1920.png) | [hashes](captures/hashes-1920.json) |

## Same-session review and limits

The implementing agent reviewed the unique diff against the exact base above, API ownership, scoped selectors, actual unit/build/browser results and original bright/dark/dialog captures. This is same-agent review, not independent execution. Review found and fixed modal reverse-Tab escape and native meter green semantics before publication. Final local review found no outstanding material findings: AC1–AC4 PASS; AC5 automated checks/originals PASS, human approval UNVERIFIED. No visual baseline files or approvals were changed. Existing UI migrations and future input/tooltip/notification primitives remain separately owned work.

`npm run test:browser:negative`, the additional repeat invocation `npm run test:visual`, `npm run test:visual:negative`, and `npm run test:visual:approval`: NOT RUN locally as a duplicate CI campaign; CI outcomes remain UNVERIFIED awaiting the user report. The two visual reference matrices already passed within the complete browser run. Remote CI, existing negative/repeat campaigns and baseline-approval policy remain intact. **CI awaiting user report**; final ACCEPT is pending that report for the delivered head. Human visual approval is not inferred from screenshot generation. No hardware performance, physical keyboard or assistive-technology listening qualification is claimed.

## Environment and retained failures

Debian 13 x86_64; preinstalled Node 24.19.0, npm 11.9.0, system Chromium 154.0.8037.57. Project versions: React 19.3.1, TypeScript 5.9.3, Vite 7.3.7, Playwright 1.64.1. Headless browser, en-US/UTC, DPR 1, reduced motion; DOM gallery uses scoped sRGB CSS. Existing WebGL tests use ANGLE SwiftShader, not physical GPU qualification.

The first unconfigured focused launch failed because Playwright's default browser was absent. Reusing system Chromium proved the component path. The initial focus failure is preserved above. The system-browser broad run was deliberately interrupted after 8 passes because checked-in visual references require Chromium 156; [interrupted run](logs/browser-154-interrupted.log), 1 interrupted / 59 not run. A first browser download was cancelled on discovering the system browser, then resumed once its incompatibility with the reference manifest was verified. No tolerances or baselines were changed.

Environment additions (reusable template): Playwright Chromium / Chrome Headless Shell **156.0.8078.4**, revision **1248**, plus bundled FFmpeg **n7.0.1-playwright-build-1013**, installed by `cd client && npx playwright install chromium` (after locked `npm ci`). Browser cache: `/home/agent/.cache/ms-playwright`; final checks leave `CHROMIUM_PATH` unset to select this pinned browser. No system libraries, runtimes or package managers were added/upgraded. Project setup: `cd client && npm ci` (98 locked packages); this is separate from reusable prerequisites. Node/npm/system Chromium were preinstalled. No saved environment template was changed.
