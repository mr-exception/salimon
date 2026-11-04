# #750 — deterministic browser navigation QA

The fresh React/Three.js navigation prototype now has seeded, fixed-clock browser
regression routes and a controlled source-mutation demonstration. This does not
restore the removed Rust/native application. Historical #23/#26/#28 infrastructure
is replaced here by equivalent browser fixtures over the actual production App,
runtime, input, camera and WebGL rendering paths. #738's object-guide design is
superseded by #744; no such guides are reintroduced.

Branch `issue-750-deterministic-browser-e2e` starts from `main` at
`e2ccba2c4d5800ade890aa8b394429d62835f6d2`; no active PR stack existed. Tested source
for the full run is `52bb6c3485750bb1828b163a1a7486e752e6a0c1`; the focused edge
rerun, final build and negative demonstrations use `5b5f1a7` (full SHA in
`verification.json`). Only the first edge test timeout changed between these
commits; other check inputs are unchanged. Final published head/base and CI/reviewer
binding belong in the PR conversation. `source.sha256` records the browser source,
tests, configuration, lockfile and workflow inputs, allowing report-only commits
to reuse local evidence without claiming old-head CI as a new pass.

## Acceptance matrix

| Issue requirement / category | Verification |
| --- | --- |
| AC-1 / regression detection | NAV-1–3 cover exact WASD direction/release, wheel zoom, absent removed buttons, tick-300 scanning, selected contact stability, actual framebuffer continuity and live heading alignment during pan, distinct yellow/light-green edge rays, zero velocity and no object guides/arrowheads. Existing edge routes retain corners, differing/opposite/overlap, off-screen ship and camera tracking. Controlled copies reverse WASD or turn the heading red; the matching assertion must fail, with unrelated failures/timeouts vetoing the demonstration. |
| AC-2 / repeatable cloud browser | Fixed 1280×800, DPR 1, en-US/UTC, reduced motion, seed 750, controlled RAF/performance clock, lockfile-selected Chromium and SwiftShader. NAV-1 replays the same input twice with identical positions/zoom/tick deltas, then verifies a different seed changes the catalog. CI uses Ubuntu 24.04 and the same commands. |
| AC-3 / traceability and artifacts | Criterion IDs in tests and maintained client guide; raw Playwright JSON, originals, failure traces, mutation hashes/summary and revision-named CI artifacts. NAV-4 verifies detached observations and production isolation. |
| Unit/integration — mandatory | Existing browser domain/input/projection tests plus typechecked observer/scenario contracts and live production composition. |
| E2E/visual — mandatory | Real Chromium/WebGL, actual keyboard/pointer/wheel events, framebuffer observations and original screenshots. These are evidence, not independently approved screenshot baselines (#751). |
| Manual/hardware — N/A for delivery | Desktop synthetic input/software rendering only. Physical devices/input, hardware GPUs/FPS, native platforms and mobile/touch are not qualified by this issue. |

## Implementation and ownership

`client/tests/fixture.tsx` initializes the seeded catalog and named vector/camera
presets. Browser observers return copies/observations; old ship/camera setters
are removed. `sceneHarness.ts` samples completed production frames, never forces
a render, and bounds recordings to 120 frames. Existing tests use boot presets
instead of browser-side authoritative writes. `deterministic.ts` owns paused-clock
boot and evidence metadata. The clock pauses one second ahead of its fixed epoch
before navigation to avoid a race with real time during installation; only
`runFor` advances it during play. Ordinary desktop/production routes retain
real-time coverage. Production source/behavior is unchanged.

`client/scripts/negative-navigation.mjs` creates disposable source copies, checks
that each mutation matches exactly once, requires one failed test with its exact
expected assertion, and retains failure output before deleting each copy.
Exceptions, timeout, missing report and a different assertion do not count as
successful detection. It does not modify baseline images or the checkout.

Documentation impact: [client guide](../../client/README.md#deterministic-navigation-regression-750),
[validation](../../docs/validation.md#browser-radar-checks) and
[maintenance routing](../../docs/maintenance-map.md) describe the fresh client,
commands, criterion ownership, read-only observations and retained limitations.
No native crate contract applies to this delivery.

## Validation and evidence

Environment: managed Linux cloud, Node 24.19.0 / npm 11.9.0, Playwright 1.64.1,
lockfile Chromium 156.0.8078.4, SwiftShader; browser synthetic keyboard/pointer.
CI uses Node 22 (>=22.18), Ubuntu 24.04 and the lockfile-selected browser.
Local dependencies/browser caches are under `/tmp` because the home cache is read-only.

Local commands (from `client/`; installed browser via
`PLAYWRIGHT_BROWSERS_PATH=/tmp/salimon-playwright`):

- `npm ci --no-audit --no-fund --cache /tmp/salimon-npm-cache`: PASS.
- `npm test`: PASS, 28 tests, zero failures/skips ([log](unit.log)).
- `npm run build`: PASS, including final test/config typecheck ([log](final-build.log)).
- `SALIMON_EVIDENCE_DIR=test-results/final npm run test:browser`: 22 PASS,
  one edge/corner route hit the previous 30-second timeout ([log](browser.log)).
- `SALIMON_EVIDENCE_DIR=test-results/edges-rerun npm run test:browser -- --project=desktop --grep 'ship rays reach every'`:
  PASS in 45.8 seconds after its sole timeout increased to 90 seconds ([log](edges-rerun.log)).
  All 23 routes are therefore covered; no full-green local run is claimed.
- `SALIMON_NEGATIVE_DIR=test-results/final-negative npm run test:browser:negative`:
  PASS, both broken copies fail at the intended assertions ([summary](negative-summary.json), [log](negative.log)).

[Machine-readable results](verification.json), [original evidence/hashes](evidence-index.json)
and original captures/traces under `evidence/` retain numerical and failure results.
Text log/context exports trim trailing whitespace; images, traces and numerical
JSON observations are unchanged.
Current-head CI must run the complete suite, including the corrected timeout.
Final remote results are bound in the PR conversation. Required current-head CI and
independent review remain mandatory before QA acceptance.

Iteration failures are preserved in logs: initial build caught a readonly fixture
copy assignment in the isolation test; initial deterministic runs hit the old
30-second timeout, and a subsequent run exposed a clock-install/pause race.
The assignment is now tested through a detached copy, the deterministic project
has a bounded 120-second timeout, and boot pauses ahead before navigation.
The expanded immutable edge/corner route also needed its bounded 90-second
capture budget; its assertions did not change. Initial sandboxed server startup failed; running with the environment's authorized
network capability resolved it. The existing production chunk-size warning is
preserved; no performance claim or threshold change is made.

## Independent review

QA pending at report creation. A distinct reviewer must inspect the published
head/base, complete diff, original evidence, required CI and all three issue
criteria, then record ACCEPT, REQUEST_CHANGES or BLOCKED with cycle/identity.
Final review binds the exact published SHA in the PR; this report never predicts
acceptance. No merge, deployment, baseline approval or early issue closure is
part of this delivery. #751 and #752 remain separate follow-up implementation.
