# Issue #810 — perspective observation camera

Implemented the camera foundation in an isolated development fixture. The current
orthographic radar, cockpit permissions, simulation, scanner and approved visual
baselines remain the production path. No physical sky, Look at, final HUD or
cockpit lifecycle integration is claimed; those belong to #811–#815.

Source: `9f0355506ad1b37714a8ac5bb042e6c1f4e226f2`.
Base: `main` at `818108fc255c347a85a3858d0b71c30328ada781` (#809 merged).
No active PR stack existed at discovery or publication preflight. No formal
blockers were registered; the textual prerequisite #809 is in branch ancestry.
The published head is the report-only descendant identified in the PR review.
[Input hashes](evidence/inputs.sha256.json) bind the retained execution evidence.

## Implementation and local review

Same-session review by the implementing Codex agent, not an independent reviewer:
inspected the final unique source diff, issue criteria, actual logs, detached
measurements and original wide/telescope screenshots. No unresolved local
implementation findings. **QA pending; CI UNVERIFIED / awaiting user report.**

| Criterion | Local result and evidence |
| --- | --- |
| Ship origin and inertial independence | PASS. Split physical origin is the current ship position; Three camera is zero in the ship-relative frame. Domain test covers huge bigint cells, sub-metre precision, turning/thrust and no ship mutation. [OBS-1](evidence/OBS-1/states.json) / [OBS-5](evidence/OBS-5/states.json) show real steering and 500 m translation without camera rotation/offset. |
| Full sphere, poles, antiparallel | PASS. Unit tests step 720 increments per axis through both poles/backward and assert continuous angle, normalization and zero incremental optical-axis roll. [OBS-5](evidence/OBS-5/states.json) samples more than 360° on each axis through actual WASD. Screen basis is parallel-transported, not reconstructed from world up. |
| 75°–0.1° optical zoom and sensitivity | PASS. Exponential normalized wheel input, finite clamps, FOV-scaled keyboard/drag, zero render-camera translation. [OBS-2](evidence/OBS-2/states.json), [wide](evidence/OBS-4/run-0-wide.png), [telescope](evidence/OBS-4/run-0-telescope.png). |
| Resize and DPR | PASS. OBS-2 checks aspect/unchanged view at 960×600; OBS-3 checks DPR 2 / actual framebuffer dimensions ([capture](evidence/OBS-3/dpr2.png)). |
| Input/focus and persistence | PASS. OBS-1 reset/current heading, no tracking and remount persistence; OBS-2 typing, arrows, focused wheel, blur and right-drag rejection; OBS-5 visibilitychange/pointercancel cleanup. Unit tests cover detached restore, atomic invalid-snapshot rejection, FOV clamps and pole-heading reset. Visibility cancellation is synthetic DOM event coverage; no OS hidden-window qualification is claimed. |
| Existing radar/cockpit | PASS. Built production radar smoke and existing COCKPIT-1 round-trip/reserved-seat/blackout/persistent-simulation test. [Backend](evidence/production/backend.json), [log](evidence/regression.log). Production modules have no imports of the new camera/fixture. |
| Reproducible evidence | PASS locally. [Index](evidence/index.json) includes seven passing scenarios and artifact hashes. Repeat wide/telescope comparisons: zero raw or perceptual changed pixels. Wrong wide-vs-telescope comparison rejects 21,366 pixels; telescope target covers 19,170 cyan pixels versus subpixel wide view ([raw results](evidence/OBS-4/comparisons.json)). No approved baseline files changed. |

The state owner lives outside the mounted Canvas and exposes independent
look/zoom/reset commands plus detached snapshot/restore. The parent supplies
current ship state to the split-origin helpers. The reusable adapter owns only
observation input and projection. The client guide documents this integration
contract and the development-only `/tests/observation.html` route.

## Commands and exact outcomes

Run from `client/` unless stated otherwise:

- `npm ci --cache /tmp/salimon-npm-cache` — PASS, 98 locked project packages installed; lockfile unchanged.
- `npm test` — PASS, runner reports 23 passed / 0 failed ([log](evidence/unit.log)).
- `npm run build` — PASS, TypeScript + Vite ([log](evidence/build.log)); existing large-chunk warning remains. Final subsequent changes were only an EOF whitespace removal and OBS-5 timeout budget, not runtime behavior.
- `CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=test-results/observation-frozen npx playwright test --project=observation` — OBS-1–4 PASS; OBS-5 initially exceeded the default 30 s budget ([log](evidence/camera.log)).
- `CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=test-results/observation-sphere npx playwright test --project=observation --grep OBS-5` — PASS after giving the ~960-frame software-WebGL test an explicit 120 s limit; completed in 21.9 s ([log](evidence/sphere.log)). OBS-1–4 inputs/behavior unchanged by that timeout-only edit; their prior passing evidence is reused.
- `CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=test-results/observation-delivery npx playwright test --project=observation --project=production --project=cockpit --grep 'OBS-|built production|COCKPIT-1'` — existing production and COCKPIT-1 PASS; earlier OBS-5 timing failure retained ([log](evidence/regression.log)). Production and cockpit sources unchanged since these passes.
- `git diff --cached --check` — PASS at local review.

Logs retain original outcomes with trailing spaces normalized. Earlier failures
are preserved under [failures](failures/): initial stale remount
observation (fixed by waiting for mounted canvas/current frame), BigInt JSON
serialization (fixed with the existing string-serialization convention), cyan
calibration predicate versus Three tone mapping (fixture marker explicitly
untonemapped), real time leaking between spherical samples (clock now paused
before navigation), and default test-budget exhaustion. No visual tolerances,
baselines or required regression gates were relaxed. The first sandbox browser
launch could not bind its local web servers; enabled network access was needed.

**Unrun locally / UNVERIFIED:** full existing browser/deterministic matrix,
existing navigation mutation suite, full repeat/negative baseline suites and
baseline-approval CI. Existing GitHub workflow runs them plus the new observation
project. CI belongs to the user; no statuses were queried or monitored. Final
ACCEPT awaits results for the delivered unchanged head reported in this chat.
Software WebGL is not hardware/Metal/FPS, physical-input or subjective art
qualification. Merge/deployment remain owner decisions.

## Environment additions

**Reusable environment prerequisites: None.** Reused Linux x86_64, Node 24.19.0,
npm 11.9.0 and system Chromium 154.0.8037.57. Actual renderer: ANGLE Vulkan
SwiftShader (Subzero), recorded in raw states/backend evidence.

**Project setup:** `cd /workspace/salimon/client && npm ci --cache /tmp/salimon-npm-cache`
installed the locked dependencies (Three 0.180.0, React 19.3.1,
Playwright 1.64.1, TypeScript 5.9.3). Initial plain `npm ci` failed because the
restricted environment could not write its default `/home/agent/.npm/_cacache`;
a writable cache resolved it. Select the preinstalled browser with
`CHROMIUM_PATH=/usr/bin/chromium`; nothing was installed for browsers or system
libraries. Network-enabled command execution was used for remote Git, npm and
local web servers. No saved environment template was changed.
