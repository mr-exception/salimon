# #730 — browser input, accessibility and reproducible verification

Reopened desktop requirements: see the [desktop cleanup supplement](desktop-update/README.md). The original delivery evidence below is preserved at its recorded revision.

Parent: `issue-729-physical-contact` at `f9f002f` (PR #734). Stack order:
#732 → #733 → #734 → this PR. The common prototype prerequisite #731 is merged.

Implemented independent held-input ownership, pointer capture release/cancel/loss,
Space/Enter button holds, focus-aware global shortcuts, blur/visibility/resize
cleanup, and a frame-time adapter that pauses hidden intervals without catchup.
Forms retain normal typing/slider keys. Camera zoom/reset has keyboard and touch
controls and never mutates the ship. Labelled search/paged contact selection keeps
all IDs accessible while limiting DOM rows. Scan, selected target, warning and
flight status are explicit; terminal/grounded ship controls disable.

The lockfile introduced with #727 is now used by `npm ci` and CI caching. The
Navigation client workflow runs domain/integration tests, TypeScript (including
browser tests/config), build, installed Chromium and all 12 desktop/mobile/built
production routes, retaining revision-named raw JSON, screenshots and failure
traces for 14 days. Reports preserve selected originals permanently.

## Validation

Linux 6.18.57, Node 24.19.0 / npm 11.9.0, TypeScript 5.9,
Playwright 1.64.1 and Chromium 154.0.8037.57. Actual ANGLE Vulkan SwiftShader WebGL
2; synthetic browser/CDP input. Desktop 1280×800; touch mobile 390×844,
resized to landscape/400 px widths. No hardware FPS or physical-input claim.

- `npm ci --prefix client --no-audit --no-fund`: pass, 95 locked packages installed.
- `npm test --prefix client`: 20/20 pass, [raw output](domain-tests.log).
- `npm run build --prefix client`: pass, [raw output](build.log); Vite's existing
  >500 kB chunk advisory remains. No threshold was suppressed.
- `CHROMIUM_PATH=/usr/bin/chromium npm run test:browser --prefix client`:
  11/12 initially passed. The dense fixture expected numeric page order but used
  unpadded lexical IDs. The fixture IDs now match its zero-padded names. This was
  a test-data ordering error, not dropped detection. Original
  [results](initial-attempt/results.json), [log](initial-attempt/browser.log),
  [failure capture](initial-attempt/dense-order.png) retained.
- `CHROMIUM_PATH=/usr/bin/chromium npm run test:browser --prefix client -- --project=desktop --grep 'dense groups'`:
  corrected dense route passes; [results](dense-recheck/results.json),
  [log](dense-recheck/browser.log), [capture](dense.png).
- `CHROMIUM_PATH=/usr/bin/chromium npm run test:browser --prefix client -- --project=mobile`:
  2/2 pass after making grouped labels size to their text while staying within
  the mobile viewport; [results](mobile-recheck/results.json),
  [log](mobile-recheck/browser.log). Other behavior is unchanged and retains its
  original passing coverage. CI runs all 12 together on the published head.
- `npm run preview --prefix client -- --host 127.0.0.1 --port 4173 --strictPort`,
  then `node reports/issue-730/capture.mjs`: actual final built bundle captures,
  [desktop](desktop.png), [mobile](mobile.png), [backend/state](backend.json).
  A capture attempt against the long-running dev server after replacing
  node_modules timed out; the fresh built preview passes with no page errors.
- `git diff --check` and new relative Markdown link/path checks: pass.
  [Production/test/lockfile hashes](source.sha256).

The 12 browser routes cover scanner cadence/no early reveal, changed camera pixels
with unchanged ship state, real-frustum edge selection, manual alignment
interruption, input focus and release, independent coasting, warning/destruction/
safe/asteroid production-App fixtures, all dense contact IDs with bounded DOM,
mobile touch/cancel/resize and the built production WebGL surface. Development
fixtures inject state/catalog into production App/runtime and are not distributed
HTML entry points. Domain tests independently cover split coordinates, fixed-step
motion/impulse, exact scanner boundaries, 20,000-body queries, contact thresholds,
spin, complete crossings, settling, terminal/restart and clock/input ownership.

## Documentation impact and limits

Updated client guide, browser contract, validation/maintenance routing and active
architecture/root entry-point notes. Historical native documentation remains
explicitly historical. The browser uses an illustrative static catalog, infinite
energy and simplified physical spheres/settling. Gravity, ephemerides, native
migration, Core integration, detailed damage, planetary gameplay and assisted
landing are not delivered here. No browser accessibility certification, physical
input/device qualification, native platform or hardware performance is claimed.
Remote current-head checks and durable run/artifact links are recorded on the PR
and source issue after publication; local results above retain actual failures.
