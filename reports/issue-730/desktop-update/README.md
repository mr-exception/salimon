# Reopened #730 — desktop navigation cleanup

Parent: `main` at `e6881cb` (merged prototype and later radar work). The earlier
report/evidence in the parent directory describes the original #735 delivery;
this supplement addresses the subsequently reopened desktop requirements.

WASD now pans the independent orthographic camera in screen space. Arrow keys
own manual ship pitch/yaw and alignment interruption. Wheel zoom retains the
0.001–30 bounds. Removed every direction-pad and zoom +/- button. Reset camera,
selection, signed throttle and statuses remain. Help is a native modal dialog
with current bindings, focus containment, Escape/close dismissal and focus return.
Form focus, keyup, blur, visibility and resize release held keyboard inputs.
Pointer release drains residual orbit/pan damping, so a released drag stops.

Owners: `CameraControls.tsx` (camera), `HelpDialog.tsx` (bindings/accessibility),
`input.ts`/App (separate held inputs). Updated client/design/maintenance/validation
guides. Removed the mobile-only browser project and obsolete touch/button tests:
desktop mouse and keyboard are the supported path in the updated issue.

## Verification

Cloud Linux; Node 24.19.0, npm 11.9.0, TypeScript 5.9, Playwright 1.64.1,
Chromium 154; actual ANGLE SwiftShader WebGL 2, synthetic mouse/keyboard input.
A representative production-scene GPU/pixel preflight passed before editing.

- `npm ci --prefix client --no-audit --no-fund`: pass, 95 locked packages.
- `npm test --prefix client`: 29/29 pass ([log](domain.log)).
- `npm run build --prefix client`: TypeScript + Vite pass ([log](build.log));
  existing large-chunk advisory remains.
- `CHROMIUM_PATH=/usr/bin/chromium npm run test:browser --prefix client`:
  initial 13/15 pass ([original results](initial/results.json),
  [log](initial/salimon-730-browser.log)). The translated wheel test used too small
  a delta to reach the former button zoom; corrected to the equivalent zoom.
  The other failure exposed real residual pan damping after pointer release
  ([original capture](initial/pointer-drift.png)); fixed by consuming it at drag end.
- Full recheck: 14/15 passed ([results](final-full/results.json)); the combined
  input test exceeded its 30-second budget during focus on software WebGL.
  Split it into three shorter scenarios without changing assertions. Typecheck
  passed, then `CHROMIUM_PATH=/usr/bin/chromium npm run test:browser --prefix client
  -- --project=desktop --grep 'WASD pans|camera keys release|wheel zoom bounds'`:
  3/3 pass ([results](split/results.json), [log](split/browser.log)). The final
  17 scenarios have complete local coverage; CI runs them together on the head.
  [Desktop](split/desktop-controls.png), [Help](final-full/help.png) and
  [backend/state](split/desktop-state.json) are original captures/results.
  Embedded redundant screenshot bytes were replaced by hashes in the result
  indexes; required states and selected original images are retained separately.
  The suite covers all four WASD directions without a prior radar click, camera/
  ship and alignment independence, release/focus/blur/visibility/resize, wheel
  bounds, absent controls, Help focus/return, slider behavior, contact outcomes,
  scanner cadence, dense groups, actual frustum selection and built WebGL.

No native platform, physical-device, hardware FPS or mobile qualification is
claimed. No merge is performed; source issue stays open until PR merge.

## Idle camera follow-up discovered in descendant CI

The initial #747 [CI run](https://github.com/mr-exception/salimon/actions/runs/41784573915)
passed 18/19 browser routes but found ~1e-12 percent idle projection changes after
key release ([original failed-step log](ci-idle-failure.log)). The existing exact
stability assertion remains unchanged. OrbitControls was reconstituting spherical
coordinates every idle frame. CameraControls now updates it only during active
pointer interaction or keyboard pan; pointer end still consumes residual damping.
This fixes the production cause on its owning branch and is propagated through
the stack. Required updated-head CI runs are linked on each PR/source issue.

Updated production code: `npm run build --prefix client` passes, then
`CHROMIUM_PATH=/usr/bin/chromium npm run test:browser --prefix client --
--project=desktop --grep 'WASD pans|pointer release|faceted navigation'`: **3/3 pass**
([results](idle-recheck/results.json), [log](idle-recheck/browser.log),
[build](idle-recheck/build.log)). The exact unchanged key-release assertion,
right-drag release and proxy/reset checks all pass. Domain code is unchanged.
