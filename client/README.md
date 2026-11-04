# Salimon navigation-only browser prototype

A self-contained React + TypeScript + Vite + Three.js + React Three Fiber radar. This is a new prototype and does not migrate the historical native Rust gameplay. See [design contract](../docs/radar-navigation-prototype.md).

## Run

```bash
cd client
npm ci
npm test
npm run dev
```

Open the URL Vite prints. `npm run build` produces `dist/`.

## Controls (desktop mouse and keyboard)

- **W/A/S/D** pans the independent camera up/left/down/right. Left-drag empty space to orbit; right-drag to pan.
- Mouse wheel zooms smoothly within bounds; **Reset camera** restores the viewer without changing the ship.
- Arrow keys turn ship pitch/yaw; manual turning cancels one-time **Set direction** alignment. Camera movement preserves alignment.
- Click a visible contact, or use the scanned-contact picker, then **Set direction**.
- Throttle slider controls forward/reverse thrust; **Space** sets zero throttle while navigating. Zero throttle coasts. Energy is infinite.
- **Help** opens the keyboard-accessible binding reference; Escape or Close help dismisses it and returns focus. Form controls retain their native keys.

For usability, visible sample distances are compressed *as input test fixtures*, not via a nonlinear rendering transform. All rendered positions use a linear conversion from simulated coordinates, and scanner range is exactly 100,000 light-seconds. The catalog is not astrophysically authoritative. Ship motion uses the approved SI impulse/mass model on a renderer-independent 60 Hz clock. `coordinates.ts` owns integer galactic cells/local offsets, `simulation.ts` owns motion and one-time alignment, and `App.tsx` owns input and presentation. Energy remains infinite. Tests run on Node 22.18 or newer without a separate test runner.

The first scanner pulse arrives after five simulation seconds. `scanner.ts` owns the immutable catalog index and disposable contact/selection state. `radar.ts` owns visible frustum groups; `ContinuousContacts.tsx` retains meshes by scanned ID and updates grouping/visibility/positions each render frame; click a group to select any member from its paged list. Zoom never reveals unscanned catalog bodies. The lattice stays fixed to galactic axes. Off-screen objects have no scene indicator; their scanned IDs remain in the picker. Yellow heading and light-green dashed velocity rays run from the projected ship to the viewport edge. Zero speed suppresses velocity; off-screen ship origins use forward-ray clipping.

`navigationRuntime.ts` composes motion, physical contact and scanning. `contact.ts` uses a conservative 5 m ship sphere and physical body radii, including rotating surface velocity. Impacts >=100 m/s end the run. Lower-speed contacts rebound/settle with zero thrust; <=20 m/s on a planet/moon is safe contact, separate from assisted landing. Nonplanetary contacts always recover as crashes. Restart navigation clears the run. Warnings observe scanned contacts without steering.

## Verification and accessibility

```sh
npm ci
npm test
npm run build
npx playwright install --with-deps chromium
npm run test:browser
```

For an installed system Chromium use `CHROMIUM_PATH=/usr/bin/chromium npm run test:browser`.
Playwright starts isolated dev (:5175) and built-preview (:4175) servers. Desktop
routes cover camera/ship independence, every-frame pan/body/pixel continuity, actual camera/frustum behavior, contact
fixtures, dense groups, help, focus/release cleanup and production WebGL.
TypeScript checks test/configuration files. CI uses Node 22, an npm lockfile
cache, `npm ci`, domain tests, build, installed Chromium and the same suite.
`test-results/` contains JSON results, original screenshots and failure traces;
CI retains these as a revision-named artifact for 14 days. Task reports retain
selected original evidence permanently.

The contact picker is labelled, searchable and paged at 32 items. Buttons have
accessible names and visible focus. WASD and arrow shortcuts work without a
prior radar click when no form/button/dialog has focus. Focused form controls
retain native keyboard behavior. Blur, hidden-page events and viewport resize
clear held inputs; hidden time does not advance simulation. Scanner, selected
target, flight and warning status are explicit; throttle has a label and signed
forward/reverse/coast value. Desktop mouse/keyboard is the supported input path.
Browser automation uses synthetic input and software WebGL; it does not certify
physical input, native platforms or hardware FPS.
