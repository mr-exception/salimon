# #728 — indexed temporary scanner and bounded radar

Parent: `issue-727-navigation-simulation` at `96a5f12` (PR #732).
The required prototype #731 is merged. This PR adds only scanner/presentation work.

`CatalogIndex` uses bigint spatial buckets; its spherical query does not scan the
whole catalog. The first pulse occurs at tick 300 (five seconds), with no early
reveal. Contacts/selection are immediately evicted on the next fixed step after
leaving range; re-entry waits for another pulse. No navigation history is stored.

Radar symbols/edge controls are bounded by viewport bins. Groups retain every
member ID and expose a paged contact menu. Only scanned in-range contacts reach
presentation; actual orthographic frustum coordinates determine edges. Galactic
lattice offsets use integer cells before narrowing to local render coordinates.

Validation: Linux, Node 24.19.0, npm 11.9.0, Chromium 154.0.8037.57,
1280×800, ANGLE Vulkan SwiftShader WebGL 2 (software GPU).

- `cd client && npm test`: 10/10 pass. Includes exact range and huge/negative
  anchors, 20,000-body indexed-vs-brute query, pulse/selection eviction/re-entry,
  10,000-member overlap preservation, viewport bounds and frustum/depth cases.
- `npm run build`: passes, existing Vite bundle-size advisory remains.
- `git diff --check`: passes.
- `npm run dev -- --host 0.0.0.0`, then
  `PLAYWRIGHT_MODULE_URL=/tmp/salimon-qa/node_modules/playwright/index.mjs node reports/issue-728/browser-check.mjs`:
  passes against actual React/R3F WebGL. No markers before first pulse; zoom then
  exposes correct Caligo edge selection; further zoom preserves ship state.
  [Script](browser-check.mjs), [assertions/backend](browser.json),
  [original capture](scanner.png), [source hashes](source.sha256).
  Use any installed Playwright 1.64.1 module URL for the environment variable.

The initial browser probe exposed a negative startup frame interval; it was fixed
on the owning #727 branch and propagated before this delivery. Probe assumptions
about initial Caligo clipping and single-event wheel zoom were corrected to use
actual frustum behavior and repeated wheel events; those failed attempts are not
claimed as passes. Current checked-in probe passes.

Documentation impact: browser guide/contract and maintenance routing updated.
The catalog remains illustrative/static; detection indexing does not imply a
production Gaia publication. Software rendering/synthetic browser events do not
qualify hardware FPS, physical input or native platforms. Collision is #729;
input/accessibility and consolidated browser CI remain #730.
