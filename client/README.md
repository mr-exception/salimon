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

## Controls

- Drag on empty space to orbit the independent orthographic camera; mouse wheel zooms the radar.
- Click a contact (or its edge marker) to open the **Set direction** menu.
- Hold **W/S** for pitch and **A/D** for yaw; or hold the on-screen buttons. Manual turning cancels Set direction.
- Throttle slider: forward or reverse thrust, 0% to coast. **Energy is infinite**.
- Reset camera only changes the viewer's orientation.

For usability, visible sample distances are compressed *as input test fixtures*, not via a nonlinear rendering transform. All rendered positions use a linear conversion from simulated coordinates, and scanner range is exactly 100,000 light-seconds. The catalog is not astrophysically authoritative. Ship motion uses the approved SI impulse/mass model on a renderer-independent 60 Hz clock. `coordinates.ts` owns integer galactic cells/local offsets, `simulation.ts` owns motion and one-time alignment, and `App.tsx` owns input and presentation. Energy remains infinite. Tests run on Node 22.18 or newer without a separate test runner.

The first scanner pulse arrives after five simulation seconds. `scanner.ts` owns the immutable catalog index and disposable contact/selection state. `radar.ts` owns frustum edges and dense groups; click a group to select any member from its paged list. Zoom never reveals unscanned catalog bodies. The grid is anchored to galactic axes/lattice, with the ship at its center.

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
Playwright starts isolated dev (:5175) and built-preview (:4175) servers. The 12
browser routes cover desktop controls, actual camera/frustum behavior, contact
fixtures, dense groups, mobile touch/resize and the production WebGL bundle.
TypeScript also checks test/configuration files. CI uses Node 22, an npm lockfile
cache, `npm ci`, domain tests, build, installed Chromium and the same suite.
`test-results/` contains JSON results, original screenshots and failure traces;
CI retains these as a revision-named artifact for 14 days. Task reports retain
selected original evidence permanently.

The contact picker is labelled, searchable and paged at 32 items. Buttons have
accessible names and visible focus; hold orientation buttons with pointer,
Space or Enter. Focus the radar for W/A/S/D and Space shortcuts. Focused form
controls retain native keyboard behavior. Zoom in/out and Reset camera are
keyboard/touch accessible and never change ship state. Blur, hidden-page events
and viewport resize clear held inputs; hidden time does not advance simulation.
Releasing one pointer/key preserves other independently held sources. Scanner,
selected target, flight and warning status are explicit; throttle has a label
and a signed forward/reverse/coast value. Browser automation uses synthetic input
and software WebGL; it does not certify physical input, native platforms or FPS.

## Navigation representations (#737)

Navigation objects use shared, flat-shaded low-poly meshes: a directional swept
ship, faceted planet/moon spheres, an irregular angular asteroid and a restrained
emissive star. Sizes are fixed CSS-pixel proxies at every orthographic zoom;
positions stay linear and physical collision radii remain separate. The sparse
3D galactic reference lattice replaces the rectangular grid surface. No ground
plane, textures, bloom or physical ship asset is introduced.

`src/visualGeometry.ts` owns shared deterministic geometry/materials;
`src/navigationVisuals.tsx` owns screen sizing and scene composition. Run the
normal unit/build/browser checks. `tests/visuals.spec.ts` measures original WebGL
pixels, draw/triangle/resource bounds and zoom/selection screenshots using the
production App with a development-only scene observer.

## Off-screen contacts (#738)

Scanned, in-range contacts outside the actual orthographic viewport use subtle
ship-to-boundary lines. Lines clip to the real viewport, including entry/exit
when the ship is off-screen; a path missing the viewport draws no false segment.
Selected lines and labels are brighter. Stable perimeter groups retain every
contact ID, expose the existing paged picker, and prioritize the selected ray.
Labels are inset for access; line endpoints remain on the viewport boundary.
Contacts outside camera depth alone get a labelled contact entry without an
invented 2D direction. No scanner or detection history policy changes.

Left-drag orbits, right-drag pans and the mouse wheel zooms the independent
camera. Reset camera restores its target, pose and zoom without moving the ship.
`src/radar.ts` owns segment clipping and grouping; `tests/edges.spec.ts` covers
all edges/corners, pan/orbit/zoom, off-screen ship, pulse/expiry and dense
selection against the production App and actual WebGL canvas.
