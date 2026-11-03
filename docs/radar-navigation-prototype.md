# Navigation-only React radar prototype (2026-11-03)

Status: approved navigation design and prototype scope. This is **not** a claim that the previous Rust/native gameplay, planetary exploration, interiors, Energy Core, or save systems were migrated.

## Scope boundary

Implement a browser-based 3D holographic navigation *view*, built with React, TypeScript, Three.js and React Three Fiber under `client/`. Ship energy is **infinite in this prototype only**. No Energy Core UI, depletion, resources, 2D planetary surfaces, interior, survival, crafting, or detailed ship meshes. The deterministic **sample universe** is illustrative test data, not the production Gaia catalog.

The main branch contains the browser client; older documents mentioning Rust packages refer to a previous architecture. This prototype establishes a fresh browser entry point but does not delete or rewrite historical contracts. Production migration is separate future work.

## Approved navigation interaction

- Space appears as a sparse holographic 3D radar, centered on the ship, not a rendered physical galaxy. Stars, planets, moons and asteroids have simple wireframe/symbol contacts.
- The camera is **orthographic** and independent from ship pose, rotating around its center. The fixed galactic coordinate orientation establishes the grid. Zoom does **not** alter distances; object positions are **linear** at any chosen view scale.
- Starting scanner detection radius is **100,000 light-seconds** (about 200 AU). All cataloged celestial bodies within range are detected on a **five-second pulse**. No contact history remains after objects leave range; the catalog itself remains unchanged.
- Contacts outside the *current zoom* but within scanner range use edge indicators. Out-of-scanner-range bodies are never exposed by zoom or indicators. Dense contacts can be clustered in presentation, not in detection.
- Clicking a celestial contact opens a menu whose only action is **Set direction**. The action captures a one-time vector toward the contact, smoothly rotating the ship at a fixed angular speed. It does not apply thrust, brake, follow the moving target or autopilot. Any manual rotation cancels alignment.
- Hold up/down/left/right buttons or W/S/A/D to rotate ship pitch and yaw. Roll stays stabilized relative to galactic up. Ship rotation and camera motion are independent.
- Throttle ranges -100% to +100%, with the preexisting impulse/thrust design intended for eventual integration. **Thrust continues while turning**; changing heading does not rotate existing velocity. Zero throttle coasts.
- Show heading and velocity directions, speed with an explicit reference frame, contact distance, and concise basic proximity / dangerous approach warnings. Keep grid and indicators visible without contacts.
- At zero Core power in eventual production, controls and propulsion stop. **Energy is unlimited in this prototype**, so no power failure occurs.
- Relative impact >= **100 m/s** destroys the ship. Lower-speed contact uses existing bounce/recovery handling; initially safe landing threshold is 20 m/s, upgradeable in future. Actual contact uses physical radii and surface-relative speeds, not hologram sizes.
- Existing automated landing/takeoff and distance-based jump-drive mechanics remain **design contracts**, but are not implemented by this navigation-only sample.

## Prototype sample and limitations

The sample in `client/src/sampleUniverse.ts` contains an invented miniature test system with the ship, a star, planets, moon and asteroid. It intentionally uses short *illustrative* distances for visible camera interaction; the actual scanner cap remains 100,000 light-seconds. The catalog is illustrative; its coordinates are converted once to split SI positions. `simulation.ts` owns authoritative browser ship motion: 60 Hz fixed steps, integer galactic cells with local meter offsets, 100,000 kg dry mass, 100/50 MW forward/reverse and 1 N·s/J with unlimited energy. Throttle clamps to signed integer percent. Position integrates constant step acceleration; orientation uses 5°/s yaw/pitch with galactic-up stabilized roll. Manual input cancels the captured one-time alignment, including opposing held controls. Existing velocity stays inertial. React and the radar only publish snapshots; a bounded per-frame work budget retains elapsed-time debt. Hidden-page time is paused. This does not implement gravity, ephemerides, collision, light delay, jump drive or the historical native runtime.

The prototype must not silently masquerade as the original native gameplay. Its purpose is to validate controls, symbols, selection, scanning cadence, frame-independent heading and camera, range filtering and the client build.

## Acceptance checks

1. `cd client && npm ci && npm test && npm run build` succeeds in a supported Node environment.
2. Browser displays a ship-centered radar and multiple distinct holographic celestial contacts from a typed sample catalog.
3. Camera orbit/zoom do not change ship orientation or motion.
4. A selected contact opens only Set direction; alignment is one-time; manually turning cancels.
5. Hold controls rotate while held; forward/reverse/zero throttle leave preexisting velocity independent of orientation.
6. Five-second pulses refresh in-range temporary contacts; contacts outside range never appear. Zoomed-out-of-frame contacts can be reached via edge indicators.
7. No planetary scene, ship interior, Core charge limits or 2D implementation is introduced.

## Migration/engineering follow-ups

Keep sample simulation separate from rendering. Split coordinates, ship-relative conversion and impulse/mass motion are implemented for the browser ship. `scanner.ts` now indexes immutable catalog bodies into integer 32-cell buckets. It queries the inclusive 100,000 ls sphere on ticks 300, 600, etc.; first contact publication waits five simulation seconds. Between pulses it only evicts out-of-range contacts, including selection, without recording history. `radar.ts` projects these contacts through the actual orthographic frustum. One symbol/edge button per 44 px bin bounds presentation; grouping retains every ID and exposes paged selection (32 rows). Edge markers use frustum clipping, never radial zoom approximations. The grid translates along a fixed galactic lattice. Remaining work includes epoch-aware body positions, contact handling, browser/input accessibility tests and eventual native-domain migration. The prototype's apparent local speed/distance scale must be replaced by the real simulation rather than retuned into fake astronomy.
