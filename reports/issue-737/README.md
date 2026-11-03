# #737 — faceted navigation representations

Delivered a swept hard-surface ship proxy with a clear -Z nose, clipped hull,
wings, canopy and paired rear engines; faceted planets/moons, an irregular
angular asteroid, and a restrained emissive star. Shared deterministic geometry
and flat materials follow the Option A shape/palette guidance of #273/#278.
No textures, bloom, physical ship asset or simulation changes. Selected members
of overlap groups supply their own representative class and selection ring.

Removed the rectangular grid helper. The replacement is a sparse, transparent
3D galactic point lattice with no faces, ground surface or universe boundary.
Orthographic proxy sizes stay readable in CSS pixels; positions remain linear,
ship-relative SI conversions and physical radii remain authoritative elsewhere.

Parent: `main` at `c37120d` (prototype #731 and #728/#730 merged). No active PR
existed at selection. No formal blockers for #737/#738. Source inputs and test
fixture hashes are in [source.sha256](source.sha256).

## Validation

Linux 6.18.57 x86_64, Node 24.19.0/npm 11.9.0, Chromium 154.0.8037.57,
actual ANGLE Vulkan SwiftShader WebGL 2, synthetic browser/CDP input. Main
viewport 1280×800; existing mobile regression 390×844 and resized landscape.

- `cd client && npm ci --no-audit --no-fund`: pass (95 locked packages).
- Baseline `npm run build && CHROMIUM_PATH=/usr/bin/chromium npx playwright test --project=production`: pass, live WebGL/adapter preflight before implementation; [original baseline](before.png).
- `npm test --prefix client`: 22/22 pass, including asset bounds/finite vertices,
  directional nose/wing silhouette, outward hull winding and nonspherical rock, plus unchanged
  scanner/precision/physics/input tests. [Raw output](domain-tests.log).
- `npm run build --prefix client`: pass, [raw output](build.log).
  Existing Vite >500 kB bundle advisory remains; no threshold was changed.
- `CHROMIUM_PATH=/usr/bin/chromium npm run test:browser --prefix client`:
  13/13 pass locally at the hull-corrected source (`7fb7e70`), including all prior desktop/mobile/production routes and the new
  production-App visual fixture. [Raw output](browser.log), [results](browser-results.json).
  Pixels are read directly from the rendered WebGL framebuffer. All four classes
  have visible shaded pixels; the ship remains readable at zoom 0.1/0.5/1.
  No plane/GridHelper is present. Draw calls <30, triangles <1000 and geometry
  resources <20 in every measured view; exact counts/pixels/colors/backend are
  in [raw measurements](geometry-backend.json).
- Original captures: [normal](zoom-0.5.png), [far zoom](zoom-0.1.png),
  [near zoom](zoom-1.png), [selected moon](selected.png).
- `git diff --check`: pass. All final local checks pass. Final geometry review found inward hull triangle winding in the initial delivery; it is corrected on this owning branch, with an outward-face regression and refreshed original captures. The initial evidence remains in the first PR commit (`10c59c6`).

CI run [41719915839](https://github.com/mr-exception/salimon/actions/runs/41719915839)
at `7fb7e70` passed install/unit/build and 12/13 browser routes but failed edge
selection. Its [raw log](ci-selection-failure/browser.log),
[original capture](ci-selection-failure/selection-failed.png) and
[replay trace](ci-selection-failure/trace.zip) are retained. The trace shows
residual orbit damping changing a single-contact marker into a group during a
click. Reset now drains pending orbit/pan damping before restoring pose/target.
The visual fixture asserts restoration of original body projections and absence
of later drift. The first recheck exposed a separate test assertion counting
visible overlap menus as off-screen indicators: [log](reset-recheck/browser.log),
[original](reset-recheck/visible-group.png). A `data-offscreen` attribute now lets
the test check actual off-screen indicators while preserving visible group access.

Final reset correction: `npm run build --prefix client` passes, then
`CHROMIUM_PATH=/usr/bin/chromium npm run test:browser --prefix client -- --project=desktop --grep 'scanner pulse|faceted navigation'`
passes 2/2 affected browser routes: [log](reset-browser.log),
[results](reset-browser-results.json), [reset original](camera-reset.png).
The 22 portable/asset tests and other previously passing routes have unchanged
inputs and are reused; final CI repeats the full 13-route gate on the new head.

CI repeats the full locked build/unit/browser checks on the published head;
its run link and outcome are recorded on the PR and issue. Screenshots, raw JSON
and traces remain available in revision-named CI artifacts for 14 days; useful
originals above are retained permanently here.

This qualifies browser/cloud presentation, not hardware FPS, physical input,
native gameplay migration or art-direction signoff. The static sample catalog,
unlimited energy and existing prototype simulation limits remain. Off-screen
line presentation is the separate child delivery #738.
