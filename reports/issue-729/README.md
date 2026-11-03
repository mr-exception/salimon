# #729 — physical contact and approach warnings

Parent: `issue-728-indexed-scanner` at `cdca6c8` (PR #733); authoritative
simulation #727 is in its ancestry. Implemented in the production per-step
`NavigationRuntime`, independently of rendering/symbol sizes.

Conservative physical sweeps catch full crossings and choose the first hit.
Impact speed includes rotating surface velocity. At >=100 m/s the run ends;
otherwise throttle/alignment clear and bounded damped rebounds settle to grounded.
Planet/moon contact <=20 m/s is safe contact, not automatic assisted landing.
Nonplanetary low-speed contact always follows crash recovery. Grounded state
follows surface rotation; restart clears clock/scanner/controls. Warnings only
observe scanned contacts and neither steer nor change thrust.

Validation on Linux, Node 24.19.0/npm 11.9.0:

- `cd client && npm test`: 18/18 pass, including 19.999/20/20.001 and
  99.999/100/100.001 m/s boundaries, total vs normal speed, rotating surfaces,
  departure/tangent/flyby, full crossings/earliest hit, bounded settling,
  deduplicated warnings, frozen terminal state, restart and composed cadence.
- `npm run build`: TypeScript/Vite pass; existing bundle-size advisory remains.
- `git diff --check`: pass.
- Vite running on :5173; `PLAYWRIGHT_MODULE_URL=/tmp/salimon-qa/node_modules/playwright/index.mjs node reports/issue-729/browser-check.mjs`:
  production React/R3F App with explicitly injected deterministic runtime/catalog
  through development-only `client/tests/fixture.html`; warning, destruction,
  safe-contact and asteroid crash routes pass. Non-flight thrust is disabled and
  restart restores flight. No page errors. Chromium 154.0.8037.57, Playwright
  1.64.1, 1280×800, ANGLE Vulkan SwiftShader as in the parent browser capture.
  [Script](browser-check.mjs), [state assertions](browser.json),
  [warning](warning.png), [destroyed](destroyed.png), [safe](safe.png),
  [asteroid](asteroid.png), [source hashes](source.sha256).

Initial strict float equality at a 1 km normalized contact differed by 1e-13 m;
the regression now uses a documented 1e-9 m arithmetic tolerance. No production
threshold, impact classification or acceptance tolerance was changed.

Documentation impact: browser contract, client guide and maintenance map updated.
The illustrative catalog is static apart from explicit surface spin. Sweeps use
fixed-step chords and conservative spheres, not detailed hull/terrain collision.
Settling is a simplified crash response; no planetary 2D scene, gravity, damage
modules, Core consumption or assisted landing migration is claimed. Software
WebGL and synthetic input do not qualify hardware, native platforms or FPS.
Input/accessibility and reproducible browser CI remain #730.
