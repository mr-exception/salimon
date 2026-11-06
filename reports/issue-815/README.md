# #815 — Grounded panoramic cockpit integration

Default `/` now opens `PanoramicNavigation` from the existing powered owner chair.
The optical scene uses the exact grounded Scout/Vesper ship anchor, the existing
scanner and contact authority, and a session-local observation quaternion/FOV.
Exit stands the player; power loss retains the occupied seat and restoration
requires explicit interaction. Re-entry preserves the local view; new character,
reload and Reset view initialize from ship heading. No camera state is saved.

`TopDownView` retains the world, runtime and observation state and runs the sole
production navigation clock; the panoramic component receives
`advanceSimulation={false}`. Unsupported flight controls stay disabled, while
scanner, camera and valid external Look at remain available. Interrupted input
and captured camera animation cancel on blur, hide, pause and unmount. Escape
also works when pause/resume leaves focus on body or the separate survival HUD.
The orthographic free-flight engineering fixture remains at `/?scene=radar`.

## Revisions and scope

Base: `main` at `bfbb339f24fe4a4f997ea83ec357d1a52980b718`. No active
PRs were present at preflight; #809 and #810–#814 are merged ancestors. Formal
blocked-by metadata is empty; textual prerequisites were verified in source.
Full domain/browser matrix source revision: `b3eaf3c0a72307af1701cb4d4ef9c8a865042a86`.
The matrix client tree is `e18c0c4ab39cc27019c610c7d18fb16cde2cf2d7`.
Final source: `280c453e31d95c07d4747a8a38ac6a5dfdb0a37a`, client tree
`b965672133f4f640bb260351627bbde0c02c0d3d`. The only later source changes focus
panoramic Help at its heading and assert that initial focus/visibility in HUD-1.
Build and both panoramic/legacy Help scenarios passed on that final source;
ship/scanner/camera/world/save/assets/lockfile and domain-test inputs are unchanged,
so the other matrix evidence remains applicable. The final report/evidence
commit changes no runtime/test/asset/configuration input.
The delivering PR and final head/base binding are recorded in the
[issue evidence update](https://github.com/mr-exception/salimon/issues/815)
and the PR review comment.

Changed owners: `TopDownView.tsx`, `PanoramicNavigation.tsx`,
`ObservationCamera.tsx`, contextual help and survival HUD placement. Existing
cockpit/ship/scanner/collision/save domain contracts are reused. Client setup,
current status, migration register, architecture/maintenance routing and design
status now distinguish the production cockpit from legacy engineering fixtures.

## Checks and review

Exact commands: [command record](evidence/commands.txt). All rows are mandatory
for local delivery except the explicitly separate hardware context.

| Criterion / category | Result and evidence |
| --- | --- |
| C1: default journey, shared horizon, observation, re-entry (E2E/integration) | PASS: two fresh contexts; same anchor/heading, preserved quaternion and FOV 50.274°; [raw ship/view records](evidence/browser/COCKPIT-1/cockpit-measurements.json), [horizon](evidence/browser/COCKPIT-1/panoramic-horizon-0.png), [external observation](evidence/browser/COCKPIT-1/external-body-observation-1.png) |
| C2: blackout/seat, explicit reactivation, input cancellation, no doubled clock (integration/E2E) | PASS: 61 and 60 ticks across nominal one-second samples (59–61 boundary tolerance), one canvas, no orthographic mount; [seated blackout](evidence/browser/COCKPIT-1/blackout-seated-1.png), [restored seat](evidence/browser/COCKPIT-1/restored-seated-1.png), same raw records |
| C3: grounded authority; isolated flight steering/alignment/coasting (unit/E2E) | PASS: self-body commands/throttle disabled, external Look at works; LOOK-1/2, HUD-3 and two legacy desktop flight scenarios in [13-scenario results](evidence/browser/results.json) |
| C4: movement, inventory, interactions, save restore, scanner, landing and collision (unit/integration) | PASS locally: `npm test`, 27 test-file entries, zero failures; [domain log](evidence/domain.log). Built SLICE-1 and SURV pass; [journey/save records](evidence/browser/SLICE-1/slice-journeys.json), [survival/pause/reload records](evidence/browser/SURV/survival-records.json) |
| Build/typecheck | PASS: `npm run build` on final focus-corrected source; [final build log](evidence/build-final.log) |
| C1–C5 affected browser matrix | PASS: 13/13, no skips/retries/flaky results, 9.4 min; [log](evidence/browser.log), [normalized attachment index/results](evidence/browser/results.json). Final Help correction: 2/2, [log](evidence/help-final.log), [results](evidence/help-final/results.json) |
| C5: original/repeat visual evidence, readable HUD/help, status docs (visual/docs) | PASS: 12 repeat comparisons under unchanged policy; accepted changed-pixel count zero (some 2D originals have nonzero raw anti-alias/color differences, recorded in each repeat JSON). [Built horizon](evidence/browser/SLICE-1/slice-grounded-panoramic-0.png), [final Help](evidence/help-final/HUD-1/help.png), [960×600 HUD](evidence/browser/HUD-3/grounded-960x600.png). 458 source-doc local links resolve; `git diff --check` passes |
| Broad CI browser, navigation negative, visual repeat/negative and baseline approval | UNVERIFIED: awaiting user report for the delivered head. No CI status queried or baseline changed |
| Manual/physical hardware qualification | N/A to routine cloud delivery: no physical target or reference promotion requested; software WebGL and synthetic input only, no hardware FPS/M1/Metal claim |

The index extracts attachment bodies to their original byte files and normalizes
paths for repository access. Inherited process environment is omitted from the
reporter configuration (only the font-config path is retained); log trailing
whitespace is trimmed. See [artifact hashes](artifact-sha256.txt). [Earlier completed journey records](evidence/initial-journey-measurements.json)
are retained as history; the final matrix above supersedes that pre-focus-fix run.

The review is by the implementing Codex agent in this same active chat; no
independent execution is claimed. Original horizon/observation and seated/standing
captures, final Help, repeat pixel comparisons and raw ship/view measurements were inspected.
No unresolved local findings remain. Local review PASS; **QA pending / CI UNVERIFIED**.
No visual baseline is promoted. CI's broad browser, negative/repeat visual and
baseline approval remain UNVERIFIED, awaiting the user's report at the delivered
head. Local verification does not establish QA acceptance or authorize merge.

## Failed attempts and corrections

- [Live-reload run](failures/live-reload.log): editing source while the development
  browser's virtual clock was paused reloaded the app, yielding an invalid
  [white capture](failures/live-reload-white.png). Stable reruns use fixed source.
- [Original settling interval](failures/old-settle-timeout.log): the legacy capture
  helper rendered five seconds of the panoramic shader and exceeded 180 seconds
  on software WebGL. [Captured state](failures/old-settle-timeout.png). The helper
  now settles 64 ms in the panoramic view, retaining five seconds in the 2D world;
  actual Look at progression, one-second clock checks and fresh replays remain.
- [Wheel delivery](failures/wheel-delivery.log): waiting did not fix the unchanged FOV.
  A direct reproduction found the Look at-centered marker intercepted wheel events
  bound only to canvas. Optical wheel now routes through the observation surface
  for canvas/marker hits; panel/form scrolling keeps its own input. The test uses
  the actual mouse wheel over that marker and waits for the observed FOV change.
  [Original failure](failures/wheel-delivery.png).
- [Replay budget](failures/replay-budget-timeout.log): both observation repeats
  matched, but rendering every virtual frame of the idle clock/blur intervals
  exceeded 360 seconds during the second blackout. Those intervals now advance
  through ten 100 ms RAF steps per second (under the world catch-up cap), keeping
  the elapsed time, nominal 60 Hz clock check (59–61 tick boundary tolerance), cancellation assertions and two complete
  fresh-context replays. [Original timeout](failures/replay-budget-timeout.png).
- [Focus cancellation regression](failures/focus-cancellation.log): LOOK-1
  exposed that clearing held input on ordinary focus also cancelled a captured
  Look at during scanner loss. Focus now clears only held input/drag; actual
  blur/hide/resize/pause/unmount cancel the animation. [Original failure](failures/focus-cancellation.png).
  That matrix was stopped after 7 passes, 1 failure and 1 interrupted built
  journey (3 unrun), then rerun on the correction. Explicit originals/raw state
  remain; the three affected world journeys disable redundant per-frame trace
  filmstrips while retaining failure action traces.
- Visual review found panoramic Help initially scrolled to the bottom because
  its Close button took autofocus after the extra grounded instructions.
  [Original capture](failures/help-initial-scroll.png). Panoramic Help now focuses
  its heading; legacy Help keeps its existing Close-button focus. HUD-1 and the
  legacy desktop Help scenario are rerun on this final focus-only correction.
- Local review found Escape could miss the cockpit after focus moved to the
  separate survival HUD/body. A shared dismissal handler now preserves overlay
  priority in that case; the HUD keyboard scenario exercises it.

## Environment

Debian 13 x86_64, preinstalled Node 24.19.0, npm 11.9.0, Chromium
154.0.8037.57; Playwright 1.64.1 from the project lock. Browser input is Playwright
synthetic keyboard/pointer/wheel. World journeys use the fixed 2026-11-04 UTC
clock, 1280×800/DPR 1, dark/en-US/reduced-motion contexts; compact HUD fixtures
also cover 1280×720 and 960×600. [Direct X11 WebGL preflight](evidence/preflight.png)
used existing `/workspace/.salimon-tools/start-display.sh`: WebGL 2, ANGLE/Mesa
Vulkan llvmpipe (LLVM 19.1.7). Browser journeys record their actual renderer.
This establishes software-rendered behavior, not physical hardware performance.

Environment additions: None. Project setup installed 98 locked packages with
`cd client && npm ci --cache /tmp/salimon-npm-cache`; the initial default-cache
attempt failed because `/home/agent/.npm/_cacache` was unavailable to the sandbox.
Use a writable npm cache. Browser selection: `CHROMIUM_PATH=/usr/bin/chromium`.
No runtime/system/browser installation or saved-template modification occurred.
Build retains the pre-existing large-bundle advisory. No CI status was queried.
