# Navigation-only React radar prototype (2026-11-03)

Status: approved navigation design and prototype scope. This is **not** a claim that the previous Rust/native gameplay, planetary exploration, interiors, Energy Core, or save systems were migrated.

## Scope boundary

Implement a browser-based 3D holographic navigation *view*, built with React, TypeScript, Three.js and React Three Fiber under `client/`. Ship energy is **infinite in this prototype only**. No Energy Core UI, depletion, resources, 2D planetary surfaces, interior, survival, crafting, or detailed ship meshes. The deterministic **sample universe** is illustrative test data, not the production Gaia catalog.

The main branch currently contains documentation only; older documents mentioning Rust packages and `client/` refer to a previous architecture. This prototype establishes a fresh browser entry point but does not delete or rewrite historical contracts. Production migration is separate future work.

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

The sample in `client/src/sampleUniverse.ts` contains an invented miniature test system with the ship, a star, planets, moon and asteroid. It intentionally uses short *illustrative* distances for visible camera interaction; the actual scanner cap remains 100,000 light-seconds. The sample uses a fixed-point local frame, scale conversion and simple motion for demonstrability, **not production-grade gravity, Kepler ephemerides, collision, precision split-cells, light-speed information delay, long-distance jump drive or authoritative ship impulse**.

The prototype must not silently masquerade as the original native gameplay. Its purpose is to validate controls, symbols, selection, scanning cadence, frame-independent heading and camera, range filtering and the client build.

## Acceptance checks

1. `cd client && npm install && npm run build` succeeds in a supported Node environment.
2. Browser displays a ship-centered radar and multiple distinct holographic celestial contacts from a typed sample catalog.
3. Camera orbit/zoom do not change ship orientation or motion.
4. A selected contact opens only Set direction; alignment is one-time; manually turning cancels.
5. Hold controls rotate while held; forward/reverse/zero throttle leave preexisting velocity independent of orientation.
6. Five-second pulses refresh in-range temporary contacts; contacts outside range never appear. Zoomed-out-of-frame contacts can be reached via edge indicators.
7. No planetary scene, ship interior, Core charge limits or 2D implementation is introduced.

## Migration/engineering follow-ups

Keep sample simulation separate from rendering. Production work needs real galactic split coordinates, epoch-aware body positions, floating-origin renderer, spatial indexed scanner (100,000 light-seconds), the correct pre-existing thrust/mass model, 100 m/s contact handling, input and scene unit tests, accessible keyboard controls, and eventual navigation integration. The prototype's apparent local speed/distance scale must be replaced by the real simulation rather than retuned into fake astronomy.
