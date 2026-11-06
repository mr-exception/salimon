# Ship-centered 360° panoramic navigation — approved future design

**Decision date:** 2026-11-06  
**Status:** Approved product/interaction design; **not implemented** in the current browser client.  
**Scope:** The 3D cockpit observation/navigation presentation in `client/`; the playable planetary surface and Scout interior remain strict-overhead illustrated 2D.

This specification supersedes the **presentation and camera requirements** in [the current orthographic radar contract](radar-navigation-prototype.md) **when its implementation lands**. Until then, `radar-navigation-prototype.md` accurately describes shipped browser behavior and regression tests. Preserve the existing simulation, scanner, cockpit authority, collision rules, and 2D/3D shared location unless an explicitly separate feature authorizes a change. The standalone `/?scene=radar` fixture remains useful for isolated development.

## Player experience and non-goals

The player observes the physical universe from a 360° panoramic sensor whose origin follows the Scout's actual position. The scene is realistic space, supplemented by **compact, visually restrained sensor overlays**. This is not a movable holographic map, an external ship-follow camera, an avatar's eyes, or a fully modeled 3D cockpit. Ship hull/window obstruction is intentionally omitted: the panoramic sensor sees in every direction unless occluded by world geometry. There is no persistent galactic lattice or rendered ship glyph at the camera origin.

No new flight mechanics, takeoff, 3D planetary exploration, server authority, canonical planetary generator, detailed planetary texture catalog, Core depletion, epoch-aware motion, light-delay simulation or navigation autopilot are introduced by this redesign. Production cinematic appearance is iterative; physical dimensions/directions and gameplay permissions are not.

## Camera and optical zoom

- **Projection:** Perspective, full spherical 360° horizontal and vertical observation. Camera translation is never user-controlled; the physical observation origin stays at the shared ship position (including when grounded). The orientation is a local inertial quaternion, **independent of the ship's heading and roll**.
- **Inertial stabilization:** Ship rotation does not rotate the camera. Ship motion translates the camera with the ship; nearby objects may drift across the view because of parallax or relative motion. This is not continuous target tracking.
- **FOV:** Default **75° vertical**, lower bound **0.1° vertical**. Wheel input changes FOV smoothly and multiplicatively/logarithmically, not the camera-to-body distance; enforce bounds for all inputs. Adjust look sensitivity with FOV, avoiding extreme jitter and abrupt jumps at telescope zoom.
- **Manual observation:** Left mouse drag and WASD rotate the observation direction only. Observation works at poles and through full turns with deterministic quaternion behavior, no gimbal flip or unwanted roll. Do not enable right-drag camera translation. Arrow keys retain ship pitch/yaw control; keyboard-focus rules prevent movement/flight shortcuts when interacting with text fields, dialogs, contact controls, or sliders.
- **First entry/reset:** The initial camera aligns with current ship heading. Reset View aligns to the *current* heading and resets FOV to 75°, without changing ship orientation or throttle. On voluntary cockpit exit or power interruption, retain orientation/FOV locally for re-entry within that active browser session. A new run/reload may initialize from heading; camera state is not authoritative save/multiplayer state.
- **View/power transitions:** Navigation remains accessible only through the powered, authorized cockpit. Normal exit returns the pilot standing in 2D; power loss returns seated in 2D and does not auto-reopen on restoration. Ship simulation continues independent of whether the view is mounted. Hidden-tab time and input release follow existing prototype policy.

## Separate observations and ship actions

Only a **currently detected scanner contact** may be selected from a scene marker or the contact browser. Physically visible but undetected objects have no identity/name/tooltip, selection, or navigation command.

- **Look at:** An independent, smooth **one-time** camera rotation to the direction captured at activation. Preserve FOV, do not rotate or thrust the ship, do not track the moving target afterward. Manual observation input cancels the transition immediately. For exact antiparallel directions, choose a deterministic, continuous shortest interpolation; no surprise roll. If scanner selection becomes invalid mid-transition, clear the contact/actions; the already captured camera rotation may finish.
- **Set direction:** Existing **one-time ship alignment** toward a currently scanned contact. Ship rotation at existing fixed angular speed; no camera movement, thrust, braking, target tracking or autopilot. Manual ship steering cancels alignment; camera operation does not. Controls remain disabled when grounded or otherwise forbidden by ship authority.
- **Grounded self-body:** The starting planet remains identified as the Scout's current location; do not offer Look at its center (which would direct observation through the local ground). Existing unsupported launch/steering/relocation remains gated.
- **Lost contacts:** Never retain a selectable/contact-action ghost when an object leaves range or is evicted by scanner logic. Reset selection and context controls; no historical rediscovery is added.

## Physical sky versus scanner knowledge

**Two independently maintained presentation layers are required.**

1. **Observable physical scene:** Render available celestial bodies and a decorative astronomical starfield regardless of scanner discovery. Bodies retain their catalog-derived geometric directions and physical angular diameters, shading/illumination and appropriate distance-dependent detail. A distant body may be subpixel; use optical point-spread treatment as needed, but do not inflate physical disks to a selectable size. Decorative background stars grant no catalog/contact identity.
2. **Sensor overlays:** Add a bounded, screen-readable marker/name/selection hit area **only** for scanner-detected bodies, including optically tiny contacts. Their marker sizes are independent of physical angular sizes. Keep all detected contacts searchable in the collapsible browser, whether off-screen or occluded. Dense markers may cluster without merging or discarding contact IDs.

**Realistic occlusion is mandatory:** Foreground planets, moons, stars, or solid terrain block farther physical objects and their scene sensor markers. Never draw a dashed/through-body contact indicator for an occluded target. The contact browser continues to show scanned occluded contacts; optionally show a non-positional "line of sight blocked" status. `Look at` toward such a contact rotates toward its last captured direction but reveals the intervening geometry. Occlusion affects *presentation*, **not** scanner acquisition/range or ship collision.

Do not reintroduce previous off-screen celestial arrows/lines. Scanned off-screen objects remain accessible from the contact list; clicking Look at is the explicit way to bring the camera toward one.

**Scanner contract unchanged:** Initial radius 100,000 light-seconds; successful pulses every five simulation seconds; all cataloged in-range bodies detected without geometric occlusion; temporary browser contacts only, first publication after the first five-second pulse; evict out-of-range entries per `scanner.ts`. Preserve visibility/permission tests so rendering cannot accidentally turn the physical catalog into a contact list.

## Geometry, depth and physical accuracy

- Keep integer-cell/split galactic positions. Compute ship-relative direction/range before narrowing to GPU coordinates. Build a stable perspective and angular presentation that represents nearby surfaces and distant celestial bodies without z-fighting, floating-point jitter or an impossible near/far frustum. Separate real-space geometry/depth handling from sensor marker projection; a normalized direction or staged optical representation must **not** change underlying physical distance/collision.
- At small angular size, point-like stars/bodies can have restrained optical brightness/point-spread sprites; resolved disks use physical angular radius, approximate terminator lighting and available body properties. No invented high-detail geology/atmospheres are canonical facts. Atmosphere is shown only when justified by available authored/catalog data.
- Sensor marker occlusion uses actual surface/depth or a defensible geometric line-of-sight test, not merely an NDC in-frustum check. Contacts behind the camera, outside the frustum, behind a body, or hidden by terrain never appear as through-space markers.
- Renderers must not leak undiscovered body identity via mesh names in user-facing UI, tooltips, contact menus, accessibility labels or navigation actions. Raw scene implementation data is not authoritative discovery state.
- Actual collision/contact physics uses physical radii and relative speed; never mesh/sprite/marker visual radii. Existing destructive impact threshold (100 m/s) and other flight rules are unchanged.
- Canonical light-delay, epoch-aware ephemerides, physics-driven atmosphere, production procedural textures, and photorealism guarantees are out of scope. The illustrative `sampleUniverse.ts` remains explicitly illustrative.

## Grounded planet and cockpit integration

The default `/` journey starts outside the **grounded Scout on Vesper**. `scoutLocation.ts` owns the sample site, planet binding, ship position and physical anchor; 2D and 3D must read the **same** location/lifecycle. The browser sample uses Vesper's spherical physical radius plus conservative 5 m ship bound. In the 3D navigation scene, a simplified local globe/surface must occlude below-horizon views. Horizon and sky update from the ship's actual pose; do not place a fixed cosmetic horizon or independently spawn the ship in free space.

Until compatible shared launch/return rules are implemented, propulsion, takeoff, turning and Set direction remain disabled in the actual grounded cockpit; the observation camera, scanner, contact browser and Look at (except grounded self-body) operate normally. The isolated `/?scene=radar` fixture may continue testing free flight. Later takeoff/landing should keep the same observation mode and move it continuously with ship state, not switch camera architectures.

## Compact full-viewport HUD

The 3D scene fills the available viewport in desktop cockpit mode: no permanent right sidebar, coordinate lattice, unused margins or normal body scrolling. All controls remain reachable through accessible HUD overlays, not hidden or sacrificed to appearance.

- **Always visible:** Small cyan observation reticle, scanner status/contact count/pulse; current speed with explicit reference frame, throttle and flight/grounded status; compact **3D mini orientation compass**; critical warnings when present.
- **Compass:** Cyan = camera/view axis; yellow = ship heading/thrust axis; light green = physical velocity direction. Express full spherical directions relative to camera (including behind/above/below). Use distinct solid/front versus hollow or dashed/rear glyphs, and distinguish overlapping heading/velocity. Suppress green direction near zero speed and show Stationary. **No** old viewport-reaching yellow/green vector rays.
- **On selection:** Small contact identification/status and independent Look at / Set direction actions. Contact details and ship stats can expand. The contact browser is collapsed by default but searchable and paginated; preserve access to all currently detected IDs, including grouped/off-screen/occluded contacts.
- **HUD safety:** Warning severity takes priority over normal panels; dangerous approach and terminal ship destruction remain visible. Inputs within dialogs/forms/panels cannot accidentally steer/turn/zoom the ship or camera. Escape closes the topmost panel before any cockpit-exit behavior. Keep Tab for accessibility; a proposed contact-browser shortcut is **C**, not Tab.
- Keyboard and mouse UI, help text, test IDs and screenshots must reflect actual bindings. No mobile-specific interface requirement.

## Non-regression and acceptance

Implement on top of the existing React/TypeScript + Three.js + React Three Fiber client; simulation and scanner remain renderer-independent.

1. **Camera unit tests:** Camera origin coincides with the shared ship position; moving/rotating the ship preserves inertial camera quaternion; user observation never changes ship pose or velocity; reset and cockpit re-entry behave as specified. Full-sphere, poles, antiparallel Look at, manual cancellation and FOV `75° ↔ 0.1°` behave deterministically.
2. **Projection/geometry tests:** Known bodies have correct directions, angular sizes and zoom behavior at both ends; large galactic offsets do not cause jitter, NaN or vanishing surfaces; near-planet horizon masks below-surface objects without changing catalog coordinates. Representative high-zoom and wide-zoom screenshots are reproducible.
3. **Discovery/security tests:** Optical render before first pulse, while sensor selection remains empty; detection after pulse enables name/action; range exit clears actions; unscanned visible body cannot be selected; optically occluded/behind-camera contacts remain in browser but cannot emit visible through-body markers; grouped contacts preserve IDs.
4. **Interaction tests:** Look at moves only camera, Set direction moves only ship; neither is auto-tracking; ship steering cancels Set direction and camera input cancels Look at; grounded Scout gates flight and grounded self-body Look at.
5. **HUD/browser tests:** Full-screen 3D cockpit with no sidebar; compact compass for forward/rear/overlap/stationary; contact search and pagination; focus, Escape, help, keyboard controls, zoom, resize, warnings, collision terminal; optical selection/click targets work at 0.1° without changing physics.
6. **2D ↔ cockpit E2E:** Spawn outside grounded Scout, enter cockpit, inspect the visible horizon, observe and select a scanned external contact, exit to 2D and re-enter preserving local view, exercise power interruption; physical anchor remains consistent. Preserve standalone `/?scene=radar` flight fixture and existing simulation/scanner/collision regressions.
7. **Cloud acceptance:** Use `cd client && npm ci && npm test && npm run build && npm run test:browser` and relevant deterministic screenshot suites; retain captures and exact outcomes. Baseline changes need owner review. A cloud/software WebGL pass is not a measured GPU/Metal/60 FPS qualification.

## Suggested implementation packages (planning only)

1. Camera projection, inertial orientation and extreme optical FOV.
2. Physically consistent optical bodies, sky, depth/occlusion and grounded horizon.
3. Scanner-gated perspective markers, contact grouping and optical-selection isolation.
4. One-time Look at plus mini-compass and compact cockpit HUD.
5. 2D/cockpit lifecycle integration, persistence, accessibility and complete browser verification.

## GitHub implementation tracking (not delivered)

- [#810 — Inertial perspective camera and 0.1° zoom](https://github.com/mr-exception/salimon/issues/810)
- [#811 — Physical sky, angular-scale bodies and grounded horizon](https://github.com/mr-exception/salimon/issues/811)
- [#812 — Scanner-gated perspective overlays, selection and occlusion](https://github.com/mr-exception/salimon/issues/812)
- [#813 — Independent one-time Look at](https://github.com/mr-exception/salimon/issues/813)
- [#814 — Compact full-screen HUD and 3D orientation compass](https://github.com/mr-exception/salimon/issues/814)
- [#815 — Grounded Scout/cockpit integration and camera persistence](https://github.com/mr-exception/salimon/issues/815)
- [#816 — Browser E2E, visual, telescope-precision and accessibility QA](https://github.com/mr-exception/salimon/issues/816)

Each implementation issue is separately scoped and blocked by the documentation PR #809 merge (plus its actual technical prerequisites). This document is not an implementation status report.

These are **planning packages**, not claims of implemented code. Track actual prerequisites, scope and cloud verification in GitHub issues. Coordinate with historical radar tickets #737, #744, browser QA #750/#751 and shared location #802 rather than reopening already-delivered orthographic requirements.
