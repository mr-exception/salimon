# #744 — heading and velocity rays

Initial parent: `issue-730-desktop-navigation` at `9398291` (PR #745).
The subsequent idle-camera correction on #745 was merged here without rewriting
history; its unchanged exact-stability checks and local results are recorded in
the #730 desktop supplement. Updated parent/head CI evidence is on this PR.

Removed celestial off-screen lines, edge indicators and heading/velocity arrow
helpers. Every scanned contact remains selectable in the paged picker. Visible
contact bins remain bounded. Added yellow solid heading and light-green dashed
velocity rays; different widths/dashes preserve both when overlapping. Zero/
negligible speed (<=1e-6 m/s) suppresses velocity and displays Stationary.
A vector parallel to the view axis has no screen direction and is suppressed.

`vectorGuides.ts` owns finite, forward-ray/rectangle intersection and orthographic
vector projection. It handles off-screen origins, misses, all edges/corners and
nonfinite/degenerate inputs. `ShipVectorGuides.tsx` reads live authoritative ship
state and the updated camera, publishing SVG endpoints every render frame before
WebGL draws. These display guides never change ship or scanner state. Updated
the client/design/maintenance guides to supersede the former #728/#738/#743
object-indicator presentation while preserving detection/selection semantics.

## Validation

Cloud Linux, Node 24.19.0/npm 11.9.0, TypeScript 5.9, Playwright 1.64.1,
Chromium 154, actual ANGLE SwiftShader WebGL 2, synthetic desktop input.
The same locked install/toolchain from #730 was reused; no dependency changed.

- `npm test --prefix client`: 28/28 pass ([log](domain.log)). Replaced obsolete
  target-line expectations with visible-group and infinite ship-ray tests.
- `npm run build --prefix client`: TypeScript/Vite pass ([log](build.log));
  existing >500 kB bundle advisory remains.
- `CHROMIUM_PATH=/usr/bin/chromium npm run test:browser --prefix client`:
  16/17 pass ([initial results](initial/results.json), [log](initial/browser.log)).
  The vector matrix read an initial camera line before the first new two-line
  frame; corrected the test to wait for both lines before inspecting styling.
  All other desktop, contact, selection, scanner and built WebGL routes passed.
- TypeScript and the affected vector matrix were rechecked after that test-only
  correction. Originals and detailed results are linked below.

The matrix covers all viewport edges/corners, stationary, differing, opposite,
overlapping/nearly overlapping rays, color/dash distinctions, actual WASD/right-
drag/wheel/turn input, off-screen origin entry/miss, no arrow helpers/markers or
object indicators, picker availability and scanner expiration. Raw numerical
state/backend data and original selected screenshots are retained; redundant
embedded screenshot bytes in JSON indexes are identified by hashes.

No hardware FPS, physical input, mobile or native platform qualification is
claimed. Body continuity while crossing the viewport during panning is the
separate next issue #742; no claim of that fix is made here. No merge performed.

Final affected recheck: `npx --prefix client tsc -b client/tsconfig.json`, then
`CHROMIUM_PATH=/usr/bin/chromium npm run test:browser --prefix client --
--project=desktop --grep 'ship rays reach every'`: 1/1 pass
([results](recheck/results.json), [log](recheck/browser.log)). All 17 final routes
have passing local coverage; CI repeats them together. Originals:
[differing](recheck/differing-vectors.png),
[opposite](recheck/opposite-vectors.png),
[overlap](recheck/overlapping-vectors.png),
[near overlap](recheck/nearly-overlapping-vectors.png),
[off-screen origin](initial/ship-off-screen.png),
[vector/backend state](recheck/vector-state.json),
[camera state](initial/camera-vector-state.json).
