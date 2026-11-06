# Issue #803 — full-viewport 2D world and HUD

**Current follow-up:** [universal ship-up camera and current validation](ship-up/README.md).
The initial delivery record below describes the earlier tested revision.

Implemented the default `/` as an edge-to-edge world with survival top left,
inventory/save status top right, and nearby interaction/live feedback/key hints
bottom left. The title remains accessible without reserving world space. Panels
retain their native controls and internal overflow. SVG resize observation expands
the reference view to the window aspect without stretching or cropping it.

- Issue: https://github.com/mr-exception/salimon/issues/803
- Source revision: `696caa021b04e01383d7bed0dabffd8c9100a092`.
- Base: `main` / `4b27aa62366c3ca57a97f7a2965f6d1882d5de1e`.
- Branch: `issue-803-viewport-hud`; no active PR stack at discovery/publication.
  Base is an ancestor; all related 2D foundations already exist there. Related
  #761–#767 are background, not outstanding prerequisites.
- [Source hashes](evidence/source-hashes.json) bind the tested code, test/config
  and lockfile. The subsequent evidence-only commit changes no executable input.
- **QA pending: required CI awaiting user report.** No merge or baseline approval.

## Criteria and executed evidence

| Criterion | Result | Verification |
| --- | --- | --- |
| A1: fresh production world fills 1280×720 and smaller desktop with no body scroll | PASS locally | Built `/`, 1280×720 and 960×600: scene/stage bounds equal viewport; document scroll dimensions equal viewport; SVG projected edges cover device-pixel grid |
| A2: survival, inventory, interactions/status and hints are usable overlays | PASS locally | HUD bounds inside viewport and pairwise non-overlap; followed player hit point remains in scene; original captures below; storage dialog stays in browser top layer |
| A3: walking, gathering, storage, doorway, cockpit and pause/terminal controls | PASS locally | Nine existing affected browser tests pass, plus focused-button versus walking, pause/resume, resize-release and DPR 2 assertions; production survival journey covers doorway and navigation return |
| A4: deterministic screenshots at two sizes and visual comparison/build/tests | PASS locally; CI UNVERIFIED | Three viewport tests pass, fresh/paused repeats and deliberate inset negatives; build/typecheck/domain and rendering preflight pass; broad CI delegated below |
| A5: related docs and obsolete layout assumptions | PASS locally | Updated client guide, playable foundation and art direction; removed inset stage/header/footer/mobile-flow CSS. Existing tests needed no assertion removal; TD-1/4 and affected gameplay checks remain intact |

### Commands and outcomes

Commands below run from `client/` unless stated otherwise.

| Category | Command / procedure | Result / raw evidence |
| --- | --- | --- |
| Unit + integration (mandatory) | `npm test` | PASS, 22 tests; [log](evidence/unit.log) |
| Build/typecheck (mandatory) | `npm run build`; final test edits additionally `npx tsc -b` | PASS; [build log](evidence/build.log). Existing large-bundle advisory only |
| Rendering preflight | `CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=test-results/803-preflight npx playwright test --project=top-down --grep 'TD-1/4'` | PASS, 1 test, two identical original renders; [log](evidence/preflight.log) |
| Affected E2E + gameplay visual checks (mandatory) | `CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=test-results/803-final npx playwright test --project=cockpit viewport.spec.ts gathering.spec.ts storage.spec.ts survival.spec.ts cockpit.spec.ts items.spec.ts` | Nine gameplay tests PASS; DPR test PASS; two initial viewport resize assertions FAIL as documented below. [Log](evidence/gameplay.log), [canonical raw results and attached measurements/comparisons](evidence/gameplay-results.json) |
| Final viewport E2E + visual/negative (mandatory) | `CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=test-results/803-viewport-final npx playwright test --project=cockpit viewport.spec.ts` | PASS, 3/3; [log](evidence/viewport.log), [canonical raw results with layout/CTM/DPR/error and comparison attachments](evidence/viewport-results.json) |
| Source hygiene/docs | `git diff --check`; inspected final diff and guide targets | PASS |
| Physical/hardware | N/A for this desktop browser layout issue; synthetic keyboard/mouse and software rendering only | No physical-device, mobile, M1/Metal or hardware performance qualification claimed |

The final 12 distinct affected browser scenarios pass across the gameplay and
final viewport runs. Production code and existing gameplay tests did not change
between these runs; only viewport-test measurement/capture coverage was revised.
The screenshot comparator, masks, thresholds, approved references and CI gates
are unchanged.

### Originals and comparison results

- [1280×720 fresh](evidence/1280x720/fresh-0.png) / [repeat](evidence/1280x720/fresh-1.png): **0 policy-changed pixels**, 39 raw channel-different pixels (below the existing perceptual threshold).
- [960×600 fresh](evidence/960x600/fresh-0.png) / [repeat](evidence/960x600/fresh-1.png): **0 policy-changed and raw-different pixels**.
- Paused/tool-equipped: [1280×720](evidence/1280x720/viewport-0.png) / [repeat](evidence/1280x720/viewport-1.png), [960×600](evidence/960x600/viewport-0.png) / [repeat](evidence/960x600/viewport-1.png): **0 policy-changed and raw-different pixels** at both sizes.
- Deliberate inset-stage negatives: [1280×720](evidence/1280x720/inset-negative.png), **126,931** policy-changed pixels; [960×600](evidence/960x600/inset-negative.png), **89,391**. Both correctly rejected.
- Representative [interior](evidence/interior.png), [storage dialog](evidence/storage.png), [terminal/reset state](evidence/terminal.png), [cockpit radar](evidence/radar.png).

Policy remains threshold 0.1, maxDiffPixels 0, includeAA true, no masks. Repeats
are evidence, not approved baselines. Raw numerical observations and comparison
attachments are embedded as base64 JSON bodies in the two Playwright JSON reports;
no duplicate state archives are needed. Local full traces/captures remain under
`client/test-results/803-*`; the representative originals above are retained here.

### Failures and corrections

Initial viewport tests passed both-size repeat/negative captures but measured
resize before ResizeObserver/React finished (projection origin 6 px). Added an
explicit asynchronous projection wait. [Initial log](evidence/failures/initial-viewport.log)
and [raw results](evidence/failures/initial-viewport-results.json) preserve this run.
The affected run then exposed `viewBox.baseVal` float32 rounding: projected height
679.9999949137369 versus 680, a 0.000005086263 px difference. The final geometry
assertion checks coverage on the device-pixel grid and retains unrounded CTM data;
[original failed capture](evidence/failures/resize-measurement.png) and the gameplay
results preserve that finding. Pixel-comparison policy was never changed. Added
fresh-start captures and 1600×700 wide-aspect resize coverage; focused rerun passes.
An initial build command from repository root was corrected to `client/`.

## Same-session review

The implementing Codex agent applied `review-salimon-task` in this same chat:
reviewed the unique diff against the exact base, all five issue criteria, test
results and original fresh/smaller-window/storage/terminal captures. Checked
resize cleanup, uniform projection, keyboard admission, pointer/focus boundaries,
modal stacking and survival placement on radar. No remaining local blocking
finding. This is same-agent review, not independent execution. A1–A5 have the
local results above; final ACCEPT remains pending the user's current-head CI
report. The issue remains open until an authorized merge.

Full Navigation client CI (all browser projects, navigation negatives, approved
visual comparisons/repeat, visual negative and baseline approval) and applicable
Docker smoke are **UNVERIFIED / awaiting user report**. No CI status was queried
or polled. Approved navigation references were not edited. Resume review here
with results bound to the delivered unchanged PR head.

## Environment additions

**None** to the reusable environment template. Reused Linux x86_64, Node 24.19.0,
npm 11.9.0, system Chromium 154.0.8037.57 and Playwright 1.64.1 from the lockfile.
Project setup only: `cd /workspace/salimon/client && npm ci --no-audit --no-fund`
installed 98 locked packages; no runtime/browser/system package was installed or
upgraded. Use `CHROMIUM_PATH=/usr/bin/chromium` for the existing browser.

Built production preview `:4175`, dev `:5175`; SVG CPU rasterization and cockpit
WebGL/ANGLE SwiftShader, controlled browser clock from 2026-11-04, en-US/UTC,
reduced motion, DPR 1/2. Two screenshot sizes above; resize checks 1100×680 and
1600×700. No environment template edits or new CI tooling.
