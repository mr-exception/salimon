# Salimon navigation-only browser prototype

A self-contained React + TypeScript + Vite + Three.js + React Three Fiber radar. This is a new prototype and does not migrate the historical native Rust gameplay. See [design contract](../docs/radar-navigation-prototype.md).

## Run

```bash
cd client
npm install
npm run dev
```

Open the URL Vite prints. `npm run build` produces `dist/`.

## Controls

- Drag on empty space to orbit the independent orthographic camera; mouse wheel zooms the radar.
- Click a contact (or its edge marker) to open the **Set direction** menu.
- Hold **W/S** for pitch and **A/D** for yaw; or hold the on-screen buttons. Manual turning cancels Set direction.
- Throttle slider: forward or reverse thrust, 0% to coast. **Energy is infinite**.
- Reset camera only changes the viewer's orientation.

For usability, visible sample distances are compressed *as input test fixtures*, not via a nonlinear rendering transform. All rendered positions use a linear conversion from simulated coordinates, and scanner range is exactly 100,000 light-seconds. The sample model is not astrophysically authoritative.
