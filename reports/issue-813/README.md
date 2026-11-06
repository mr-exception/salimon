# #813 — Independent one-time contact observation

Implemented on `issue-813-look-at`, based on `main` at
`fc3c9bbd9c53714f4a047d0572570d9d5ba1b53e` (merged #809, #810/#817,
#811/#818 and #812/#819). No active PR stack or formal issue blocker edges
were present; the textual prerequisites exist in the verified base.
Tested source: `e600e2a3980bbe944c1e8bb2465e309ec1ff1009`.
[source hashes](source.sha256) bind the inputs; the following evidence-only
commit changes no source/test/configuration/lockfile. Final head/base binding
is recorded in the published PR review comment.

`ObservationCameraState` owns a captured direction, start/end quaternion and
one-second smoothstep transition. It transports the current camera frame along
the shortest arc; exact antiparallel rays use the current screen-up axis for a
deterministic tie-break. FOV, physical position, heading, throttle, velocity and
ship alignment remain independent. Manual camera press/drag, WASD, wheel,
reset/restore or explicit cancellation interrupts Look at immediately. No target
reference/tracking mode persists. Idle commands do not cancel interpolation.

`contactCommands.ts` rechecks current scanner/range authority at activation.
`PerspectiveContacts` exposes independent commands, live `getShip`/`canPilot`
gates, expandable help, and grounded current-planet identity. Grounded Scout
can Look at other detected bodies; Vesper self-body actions and all grounded
Set direction commands are disabled. Hidden/rear browser contacts remain usable
without revealing through-body markers. Scanner loss clears selectable commands;
the captured animation may finish. Manual ship steering still cancels only
ship alignment. The new help disclosure also participates in the shared input
focus guard.

This composes the production perspective owners in `/tests/look-at.html`,
including the **shared** `startingScoutLocation()` Vesper anchor. The existing
orthographic production cockpit remains usable; final panoramic composition,
compact HUD and lifecycle integration remain #814–#816. No new flight authority,
physics, scanner acquisition, collision law or approved baseline was introduced.

## Same-session implementation review

Reviewed by the implementing Codex agent in this active chat, not an independent
execution. Inspected the issue/design, unique final diff, actual test logs, raw
measurements and original captures. No unresolved source findings.
**QA pending: required CI UNVERIFIED / awaiting user report; final ACCEPT pending.**

| Criterion/category | Local result and evidence |
| --- | --- |
| A1 — unit: camera-only vs ship-only mutation, FOV, antiparallel/poles, captured-once and cancellation | PASS — 8 affected tests within the 154-test domain suite; tuple mutation cannot retarget captured orientation; frame-by-frame bounded continuous antipodal rotations preserve transported roll; invalid vectors rejected |
| A2 — integration/E2E: browser rear contact -> camera turn, no ship turn; Set direction -> ship turn, stable camera | PASS — LOOK-1 exercises real production camera/command/simulation adapters through Chromium; ship snapshots and FOV preserved; camera input retains alignment, arrow input cancels it; [raw measurements](browser/attachments/1-measurements.json) |
| A3 — visual: opaque foreground after Look at | PASS — LOOK-1 canvas pixels are byte-identical with the occluded target present/removed, and its marker remains hidden; [original](browser/attachments/1-occluded.png) |
| A4 — grounded shared Scout/Vesper | PASS — LOOK-2 identifies Vesper with disabled self-actions; other detected body Look at changes camera only while Set direction stays disabled; [Vesper](browser/attachments/2-vesper.png), [observation](browser/attachments/2-grounded-observation.png), [raw states](browser/attachments/2-measurements.json) |
| A5 — scanner authorization, loss during animation and keyboard focus | PASS — command unit tests reject unscanned/unknown/lost IDs and missing flight authority; LOOK-1 starts with no actions, evicts during interpolation, clears selection/actions, finishes the captured direction without a ghost; text/help focus does not steer |
| A6 — scoped regressions/help | PASS — expandable observation help, client API/status/maintenance guides; SENSOR tests cover scanner authority, telescope, grouping, loss, motion, repeat and translated pixels; OBS tests cover camera independence, FOV, resize/DPR, remount, focus and negative/repeat comparisons |
| Broad CI browser/negative/repeat/baseline gates | UNVERIFIED — existing workflow and approved references preserved; CI awaiting user report |
| Manual/hardware qualification | N/A for routine scope: no physical-input/hardware-specific criterion; synthetic Chromium events and software WebGL do not establish hardware GPU/Metal/60 FPS qualification |

Commands from `client/` (Linux x86_64):

- Project setup: `npm ci --cache /tmp/salimon-npm-cache` — PASS, 98 locked
  packages; [log](project-setup.log).
- `node --experimental-strip-types --test src/observationCamera.test.ts src/contactCommands.test.ts`
  — 8 focused tests passed during iteration; included in the final full suite.
- `npm test` — **154 passed**, 0 failed; [log](unit.log).
- `npm run build` — **PASS**, TypeScript (including fixtures/tests) + Vite;
  existing chunk-size warning; [log](build.log).
- `CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=/workspace/salimon/reports/issue-813/browser npm run test:browser -- --project=look-at --project=perspective --project=observation`
  — **9 passed (3.4m)**; [log](browser.log), [reporter index](browser/results.json).
- `git diff --cached --check` and source-hash verification — PASS before publication.

The source/test/lockfile/configuration hashes are unchanged between these checks,
source commit and evidence commit. No remote CI status was queried. The existing
workflow includes the new project in its all-project browser invocation.

## Evidence, failures and limitations

Preflight proved real ANGLE/Vulkan SwiftShader rendering before the browser
campaign: [capture](preflight.png), [raw renderer/state](preflight.json).
Node v24.19.0, npm 11.9.0, Chromium 154.0.8037.57; software WebGL.
LOOK/SENSOR fixtures use 960×600 CSS pixels, DPR 0.5 optical raster (480×300),
with full-sized DOM controls; OBS uses its existing 1280×800 DPR 1/2 cases.
LOOK/SENSOR run live requestAnimationFrame time with bounded 50 ms view steps;
OBS retains its controlled clock. Input is Playwright synthetic keyboard/pointer.
Fixtures/scanner/catalog are deterministic; no save or production settings changed.
Reporter inline attachments are materialized once in `browser/attachments/`;
original end screenshots, raw states and failure trace remain accessible.

Initial `npm ci` could not write `/home/agent/.npm/_cacache`; the successful
command above supplies a writable temporary cache. No proxy/credentials changed.
An initial build found an unused fixture import, removed before final validation:
[original failure](initial-build-failure.log).
The first browser attempt had LOOK-1 pass, then grounded LOOK-2 time out looking
for Caligo after source edits triggered a Vite reload and cleared scanner state:
[log](initial-browser.log), [original reporter/trace](initial-browser/results.json).
This invalidated attempt is retained, and the final complete affected run uses
frozen source. No assertion, timeout, tolerance, rendering rule or baseline was
weakened to obtain a pass.

## Documentation impact

Updated the client API/help contract, current-client source status and maintenance
routing. `observationCamera.ts` owns camera-only interpolation;
`contactCommands.ts` owns scanner/grounding command guards; `simulation.ts` still
owns ship-only alignment. `ObservationCamera.tsx` and the overlay compose these
owners. Changed local documentation/evidence links and source hashes were checked.
The approved panoramic design remains the architecture contract; production HUD
and cockpit integration remain separately owned by #814–#816.

## Environment additions

**None** for the reusable template: runtimes, Chromium and system dependencies
were preinstalled. Project dependencies only: `cd client && npm ci --cache
/tmp/salimon-npm-cache` (98 locked packages). Temporary npm cache selection is a
project setup workaround, not a runtime/tool upgrade. No saved template change.
