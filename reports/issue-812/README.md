# #812 — Scanner-gated perspective overlays

Implemented on `issue-812-perspective-markers`, based on `main` at
`7cb0e7c8ef2e63552ab80d5854c5062c8047c0e0` (merged #811/#818, #810/#817 and
approved design #809). No active PR stack existed. Issue #812 has no formal
blocker edges; its explicit textual prerequisites are implemented in this base.
Code/docs/test revision: `a6c41052e5044d92e7f995dad17d501b0df91995`.
The following evidence commit only packages this report and original evidence;
[source hashes](source.sha256) bind the tested inputs without a self-referential SHA.

`perspectiveContacts.ts` projects current scanner contacts through inertial
camera directions, rejects rear/frustum misses and compares stable physical
surface-ray depths against optical geometry. Geometry may suppress a marker;
it cannot authorize an ID. The grounded globe supplies the simplified surface
occluder. A partially occulted center is conservatively hidden. CSS marker
size (32 px) never changes physical body size, positions or collisions.

`PerspectiveContacts.tsx` supplies live guarded scene selection, searchable
8-entry browser pages, lossless connected overlap groups and an optional
Set direction callback. The parent retains scanner ticking and authorized
flight-command execution. No Look at implementation, autopilot, scanner history,
through-body indicators or off-screen arrows were added. Production cockpit
integration is #815; the composed development fixture supplies the panoramic
foundation for it. Full compact HUD remains #814. Existing radar, physics,
scanner acquisition, inclusive 100,000 ls range and 300-tick/5-second pulses
are unchanged.

## Same-session implementation review

Reviewed by the implementing Codex agent in this active chat, not an independent
execution. Inspected the approved design, final unique source diff, actual logs,
raw state/DOM samples and all four final original captures. No unresolved source
findings. **QA pending / CI UNVERIFIED; final ACCEPT awaits the user's CI report.**

| Acceptance criterion | Local outcome and evidence |
| --- | --- |
| Optical visibility without pre-pulse identity/selection/actions | PASS — SENSOR-1 clicks visible globe with empty scanner; no markers/names/selection; [original](browser/attachments/1-before-pulse.png) |
| Detection enables matching IDs; range exit clears selection/actions even at telescope zoom | PASS — SENSOR-1 23 contacts after tick 300; scene/list selection, callback guard, tick 301 eviction; [raw states](browser/attachments/1-measurements.json) |
| Behind-camera, out-of-FOV and occluded bodies have no scene hit targets but remain searchable | PASS — browser selects hidden/back/outside IDs; markers contain only 20 visible IDs in three groups; unit limb, foreground-depth and near-ground surface tests |
| Dense groups preserve every underlying ID | PASS — one 18-ID group, three pages, 18 unique collected IDs, page selection and stale-group suppression |
| Huge split offsets, tiny bodies, 0.1° and motion stability | PASS — unit precision/limb/tiny-body tests; tiny telescope marker click; repeated and translated pixels byte-identical; six actual frame/DOM samples retain one finite marker during motion; [raw measurements](browser/attachments/2-measurements.json) |
| Camera/scanner independence and physical contracts | PASS — browser motion leaves tick/contact IDs unchanged, typing blocks observation, inclusive range/pulse/eviction unit tests; no physics/scanner source change |
| Broad browser/negative/repeat visual/baseline CI | UNVERIFIED — CI awaiting user report; existing workflows and approved baselines preserved |

Commands from `client/`:

- `node --experimental-strip-types src/perspectiveContacts.test.ts` — **5 passed**.
- `npm test` — **150 tests passed**, 0 failed; [log](unit.log).
- `npm run build` — **PASS**, typecheck and production Vite build (94 modules);
  existing large-chunk warning. Production bundle hash remains `index-Db3_inTB.js`.
- `CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=/workspace/salimon/reports/issue-812/browser npm run test:browser -- --project=perspective`
  — **2 passed (2.1m)**; [log](browser.log), [reporter index](browser/results.json).
- `git diff --cached --check` — **PASS** before the source commit.

Unit/build results are reused after fixture-only calibration/resolution and
browser-test edits: production modules, unit tests, lockfile and toolchain did
not change. No full local CI duplication or remote check-status query occurred.
The new Playwright project joins the existing all-project CI invocation.

## Originals, environment and failures

Final originals: [pre-pulse](browser/attachments/1-before-pulse.png),
[scanned groups](browser/attachments/1-scanned.png),
[tiny body at 0.1°](browser/attachments/1-tiny-telescope.png),
[telescope camera motion](browser/attachments/2-telescope.png).
The single raw measurement artifact per scenario is linked in the table above.
Reporter attachment bodies were materialized once as linked files; reporter
indexes retain outcomes and relative attachment paths instead of duplicate
base64 bodies. Default end-of-test screenshots and failure traces are retained.

Linux x86_64; Node v24.19.0, npm 11.9.0, Chromium 154.0.8037.57;
ANGLE/Vulkan SwiftShader. Full-resolution startup preflight:
[1280×800 capture](preflight.png), [renderer/empty-scanner state](preflight.json).
Final interaction scenarios use a 960×600 CSS viewport, DPR 1 DOM and a 480×300
optical raster (`?low-resolution`); this bounds software shader cost while
preserving the physical camera/aspect, geometry and CSS hit target sizes.
The default development fixture remains optical DPR 1. These are cloud software
rendering results, not hardware GPU/Metal/FPS qualification or approved baselines.

Initial 1280×800 runs timed out at 120 seconds: [log](initial-timeout.log),
[original results/traces](initial-timeout/results.json). Reducing the viewport,
removing trace filmstrips and using the established 180-second interaction budget
allowed SENSOR-2 to pass at optical DPR 1, while SENSOR-1 reached dense page 3
before timeout: [log](interaction-timeout.log), [original evidence](interaction-timeout/results.json).
The final reduced optical raster completed both scenarios, including an added
tiny-body telescope click and live DOM motion samples. No assertion, pixel
comparison, tolerance, baseline gate or scanner rule was removed or loosened.
An early unit-test vector assumed a 45° horizontal direction was outside a 75°
vertical FOV; corrected the fixture for viewport aspect before final unit checks.
The restricted subprocess/network context required approved expanded sandbox
execution for npm tests, Git/Vite and Chromium; no credentials/proxy settings changed.

Updated client API guidance, maintenance routing and current-client status.
Remaining scope is explicitly owned by #813–#816; required CI awaits the user
in this chat. No merge, deployment or release was performed.

## Environment additions

**None** for the reusable environment template. All runtimes/browser/tools were
preinstalled and reused. Project setup only: `cd client && npm ci` installed
98 locked packages. No global/system/browser dependency or saved-template change.
