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
