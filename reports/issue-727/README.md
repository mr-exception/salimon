# #727 — authoritative browser navigation

Delivered renderer-independent SI ship motion, split galactic coordinates, fixed
60 Hz state transitions, mass-based signed impulse and captured one-time alignment.
Parent: `main` at `be612f1f07504318410ac06c8efe7bb9a3f1049c` (merged #731).
The browser model supersedes sample acceleration; historical native gameplay is not migrated.

Validation on Linux 6.18.57, Node 24.19.0 / npm 11.9.0:

- `cd client && npm install --no-audit --no-fund`: passed; lockfile committed for the new domain tests.
- `npm test`: 4/4 passed (precision/boundaries, batching/debt, impulse/mass/coasting, alignment/cancellation).
- `npm run build`: passed, with Vite's existing >500 kB bundle advisory.
- `git diff --check`: passed after whitespace cleanup.
- `npm run dev -- --host 0.0.0.0`, then Playwright 1.64.1 with system Chromium
  154.0.8037.57, headless, `--use-angle=swiftshader --enable-unsafe-swiftshader`:
  1280×800 real React/R3F WebGL render; camera drag preserved all ship readouts;
  held A changed yaw; 100% thrust increased speed; zero throttle followed by held D
  preserved speed. No page errors. Original [capture](navigation.png),
  [state/backend assertions](browser.json). Browser events are synthetic, not physical input.

Production source hashes: [manifest](source.sha256). Initial validation caught a
test syntax error and unused import; both were fixed before passing.

Documentation impact: updated browser contract, client guide, maintenance routing
and validation commands. Relative guide links resolve in this checkout.
Remaining planned scope: indexed scanner #728, collision #729, browser/input and
CI hardening #730. Catalog remains illustrative, energy infinite, no gravity,
Core integration, planetary scene, native/hardware qualification or performance claim.
