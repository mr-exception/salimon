# Issue #814 — compact panoramic HUD

Full-viewport panoramic composition with a cyan reticle, scanner pulse/count,
framed speed, signed throttle, domain flight status, priority warnings/terminal
restart and a camera-relative mini compass. Contact search/pagination is collapsed
by default; selected Look at / Set direction actions preserve scanner and flight
authority. Telemetry expands separately. C opens contacts; Tab traverses controls;
Escape closes help, contacts and telemetry before cockpit exit. Controls retain
focus and suppress observation/flight/wheel input.

The reusable `PanoramicNavigation` receives the domain runtime and parent-owned
view. `/tests/panoramic.html` exercises it, including the grounded Scout. As with
#810–#813, production cockpit activation/persistence remains #815; the existing
orthographic entry is preserved until that separately scoped integration.
No simulation/server state, old lattice, projected ship or long vector rays are
introduced by the panoramic composition.

Base: `main` at `6c785484491c502f99c6332ca45cda5b879cadb5` (#820).
No active PR stack existed at preflight. Dependencies #810/#811/#812/#813 and
approved documentation #809 are present in this ancestry. Implementation branch:
`issue-814-compact-hud`; PR base: `main`.

## Acceptance and local evidence

Final commands, captures, source hashes and results are recorded below after the
final affected checks. Same-session review is performed by the implementing
Codex agent; this is not a separate or independent reviewer.

- Full spherical compass: pure mapping tests cover front/rear, exact 180°,
  up/down/poles, camera relativity, stationary and coincident directions.
- Browser matrix: 1280×720 and 960×600 full canvas/no body scroll, scanner-only
  selection, 40 IDs across pages/search, keyboard traversal, input focus,
  Escape priorities, heading rotation with inertial camera, zoom/reset/throttle,
  compass render repeat/negative, expanded warnings and terminal restart.
- Shared sensor/Look at browser scenarios retain occlusion, behind-camera and
  grounded authority coverage. No baseline is changed or self-approved.
- CPU marker occlusion now handles catalogs larger than the sky's GPU uniform
  budget without dropping IDs/blockers. The fixture explicitly uses two optical
  bodies with 40 sensor IDs; the sky's existing 32-body budget is unchanged.
- Direct X11/Chromium WebGL preflight: [capture](preflight.png),
  [adapter and canvas measurement](preflight.json). ANGLE Vulkan SwiftShader,
  software rendering; no hardware FPS/Metal qualification.

## Findings corrected during implementation

Initial browser fixture incorrectly submitted 40 optical bodies to the 32-body
sky budget ([original results](failures/initial-browser-results.json)). The
expanded scenario then exposed the same unwanted limit in CPU sensor occlusion;
that limit is now removed from the CPU path with a 40-ID regression. Browser
keyboard testing exposed focus loss after modal unmount; post-unmount focus
restoration fixes subsequent Escape handling ([failure](failures/focus-restoration.md)).
Intermediate interrupted runs are retained in the workspace test-results;
publication evidence records the final complete run separately.

## Environment

Preinstalled Node 24.19.0, npm 11.9.0, system Chromium 154.0.8037.57 and existing
Xvfb/sysroot tooling reused. No reusable environment packages installed/upgraded.
Project setup: `npm ci --no-audit --no-fund --cache /workspace/.npm-cache`
(installed locked project packages). The initial default-cache install failed
because `/home/agent/.npm/_cacache` was not writable/available; the successful
command selects a writable cache. Browser checks select system Chromium using
`CHROMIUM_PATH=/usr/bin/chromium`; X11 preflight uses existing display `:99` via
`. /workspace/.salimon-tools/start-display.sh`. No saved template changed.

CI and broad regression/visual negative/repeat/baseline-approval jobs are awaiting
the user's report for the delivered head. No CI status queried. QA pending;
final ACCEPT requires the user-reported required CI results bound to this head.

## Final results and same-session review

Implementation source revision: `3fe06bf2c403524c36dd7ba0d31166beb8fd5e35`.
The final publication adds only this report/evidence on top of that source.
`git diff --check` passes. Build retains the pre-existing large bundle advisory.

| Criterion/check | Result | Evidence |
| --- | --- | --- |
| Domain tests (`cd client && npm test`) | PASS: 27 test-file entries, zero failures | [domain log](evidence/domain.log) |
| Typecheck/production build (`npm run build`) | PASS | [build log](evidence/build.log) |
| Final focused compass/occlusion tests (`node --experimental-strip-types --test src/orientationCompass.test.ts src/perspectiveContacts.test.ts`) | PASS: 2 file entries | [focused log](evidence/focused.log) |
| HUD + shared Look at/sensor WebGL matrix | PASS: 7/7, 4.7 minutes | [results and original attachment index](evidence/browser-results.json), [command log](evidence/browser.log) |
| Final Help style/capture + keyboard scenario | PASS: 1/1 | [results](evidence/HUD-1-final/results.json), [Help capture](evidence/HUD-1-final/help.png) |
| Full viewport and 40-ID accessible browser | PASS | [1280×720](evidence/HUD-1-final/desktop-1280x720.png), [raw state](evidence/HUD-1-final/browser-state.json) |
| Rear/coincident compass, stationary and independent ship rotation | PASS; byte-identical repeat and distinct front/rear negative pixels | [rear](evidence/HUD-2/rear.png), [coincident front](evidence/HUD-2/overlap.png), HUD-2 assertions |
| Warning/terminal priority with panels expanded | PASS | [warning](evidence/HUD-2/warning-960x600.png), [terminal](evidence/HUD-2/terminal-960x600.png) |
| Grounded authority and smaller desktop | PASS | [960×600](evidence/HUD-3/grounded-960x600.png), LOOK-2 measurements in attachment index |
| CI broad browser, negative/repeat visual and baseline approval | UNVERIFIED — awaiting user report | Existing workflow preserved; no baseline changes |

Reviewed the unique diff against the base, original desktop/rear/overlap/grounded/
terminal/help images, scenario assertions and raw state/measurements. Confirmed
no sidebar, page scrollbar, old rays/lattice or overlapping expanded HUD in the
captured layouts. Fixed the catalog-budget and modal-focus findings described
above. No unresolved local findings. Review identity: implementing Codex agent
in this active chat. Local review PASS; **QA pending / CI UNVERIFIED**, final
ACCEPT pending the user's report for the unchanged publication head. Nothing
merged or deployed. This standalone PR targets `main`; #815 may follow it.

## Owner-reported CI failure and repair

The initial published head failed Navigation client run 42232661938 (58/60 browser
scenarios passed). The prior local PASS did not establish CI acceptance. See the
[diagnosis, test synchronization repairs and new evidence](ci-repair/README.md).
QA remains pending for the updated head until the owner reports its required CI
results in this chat.
