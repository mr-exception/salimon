# #742 — continuous body rendering during camera pan

Parent: `issue-744-ship-vector-lines` at `57266cd` (PR #746), on #745.

## Diagnosis and fix

The previous scene recalculated camera-frustum bins every 100 ms and used bin
IDs as React mesh keys. This delayed visibility as bodies entered the viewport
and remounted proxies whenever camera bin identity changed. The production
WebGL regression on the parent found **3 missing meshes and 3 missing body-pixel
observations** during 65 keyboard-pan frames (143 in-view body observations).
The matching mouse route had 177 observations without a miss. This demonstrates
a real temporal culling gap; no drag flag or scanner suspension was found.

`ContinuousContacts.tsx` now retains meshes by scanned body ID. After camera
input and before rendering, it projects current contacts, updates visible
representatives and positions, and reads current ship orientation. Shared
geometries/materials stay mounted. Dense groups still draw one representative;
all scanned IDs remain available through the picker. The 100 ms React update is
only for contact menus; it cannot control body rendering. Ship-vector rays from
#744 keep their per-frame camera/ship update. Scanner cadence, expiration,
physical simulation and selection are unchanged. No off-screen guides return.

## Verification and evidence

Cloud Linux, Node 24.19.0/npm 11.9.0, TypeScript 5.9, Playwright 1.64.1,
Chromium 154, actual ANGLE SwiftShader WebGL 2 and synthetic desktop mouse/keys.
Same locked dependencies/toolchain as #730/#744.

- Before implementation: `npx --prefix client tsc -b client/tsconfig.json` passed;
  `CHROMIUM_PATH=/usr/bin/chromium npm run test:browser --prefix client --
  --project=desktop --grep 'continuous.*pan renders'` reproduced the bug:
  1/2 failed ([log](before/browser.log), [results](before/results.json),
  [keyboard raw frames](before/keyboard-frames.json),
  [mouse raw frames](before/mouse-frames.json),
  [original missing-body frame](before/keyboard-missing.png)).
- `npm test --prefix client`: 28/28 pass ([log](domain.log)).
- `npm run build --prefix client`: TypeScript/Vite pass ([log](build.log));
  existing large-chunk advisory remains.
- Full post-fix browser verification and its original frames are recorded below.

The after-render observer samples the actual production framebuffer without
requesting an extra render. It records every sampled frame's expected projection,
visible mesh identity/position, local bright pixels, ship/vector observations
and scan status. Original PNGs are captured synchronously from that framebuffer.
The fixtures inject a deterministic four-kind catalog into production App/runtime;
the production bundle has separate browser coverage and excludes the fixture.
No physical-input, hardware FPS, mobile or native-platform qualification claimed.
No merge performed. Scope and owners are documented in the maintained guides.

Final full command: `CHROMIUM_PATH=/usr/bin/chromium npm run test:browser
--prefix client`: **19/19 pass**, no retries/skips
([results](after/results.json), [log](after/browser.log)).
[Before/after counts and backend](frame-summary.json): keyboard 65 frames /
143 body observations, **0 missing** after the fix (previously 3); mouse 65 frames /
180 body observations, **0 missing**. Both routes exercise all four body kinds;
64 keyboard and 60 mouse samples change the camera while input remains held.
All captured scan statuses retain four temporary contacts, and selection stays
stable through movement/release. The 2,000-contact browser picker/group route,
proxy pixel/draw-count checks, vector matrix and built production route pass.

Original post-fix frames: [keyboard](after/keyboard-visible.png),
[mouse](after/mouse-visible.png); raw [keyboard](after/keyboard-frames.json) and
[mouse](after/mouse-frames.json). `python reports/issue-742/verify_frames.py
reports/issue-742/after` also passes: reuses these exact frames to verify body
positions/pixels, movement, scanner continuity and mathematically correct
heading/zero-velocity rays on every sampled frame, without another GPU campaign.
[Source hashes](source.sha256) identify the verified implementation and drivers.

## Parent correction and final stack

Initial #747 CI at `1ff579c` passed 18/19 routes but caught ~1e-12 percent idle
camera drift in the unchanged exact key-release assertion. #745 fixes the
production OrbitControls idle update; see its desktop supplement for the
original CI failure and focused checks. The correction was merged through #746
and this branch without rewriting published history. Current parent is
`issue-744-ship-vector-lines` at `7989918`, containing #745 at `213338c`.

`npm run build --prefix client` passes on the integrated code. Then
`CHROMIUM_PATH=/usr/bin/chromium npm run test:browser --prefix client --
--project=desktop --grep 'continuous.*pan renders|ship rays track'`: **3/3 pass**
([results](camera-idle-recheck/results.json),
[log](camera-idle-recheck/browser.log),
[build](camera-idle-recheck/build.log)). Both keyboard/mouse pan routes and
pan/zoom/turn/off-screen ship-vector clipping pass again. New raw pan frames and
original images are in that directory. `python reports/issue-742/verify_frames.py
reports/issue-742/camera-idle-recheck` passes all body/projection/pixel/vector/
scanner assertions. CI repeats the complete 19-route suite on the updated head.
