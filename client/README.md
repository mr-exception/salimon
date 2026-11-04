# Salimon browser prototypes

**2D resource foundation, not radar UI:** The client contains a pure whole-unit 10 kg / 10 L resource inventory and gathering domain (src/resourceInventory.ts), with node tests (src/resourceInventory.test.ts). It is not yet wired to the navigation-only interface. See [resource design](../docs/resource-inventory-and-gathering.md).

A self-contained React + TypeScript + Vite + Three.js + React Three Fiber radar. This is a new prototype and does not migrate the historical native Rust gameplay. See [design contract](../docs/radar-navigation-prototype.md).

## Run

```bash
cd client
npm ci
npm test
npm run dev
```

Open the URL Vite prints. `npm run build` produces `dist/`.

## Top-down foundation (#761)

During `npm run dev`, open `/?scene=top-down` for a strict overhead illustrated
Scout and planetary site in one continuous scene. **W/A/S/D** walks the fixture cabin and cockpit; visible walls, furniture and a crate block movement. The camera follows with fixed-step
smoothing, and facing persists when movement stops. Focus loss, editable controls,
resize and hidden-page events release movement. The normal `/` entry and built
production bundle remain the 3D radar. This development URL is not a cockpit
interaction or a global gameplay mode switch.

`topDownFrames.ts` owns planet-fixed/ship-local anchors using the existing split
`Position`; `topDownRuntime.ts` owns stable IDs, immutable scene state, 60 Hz steps,
collision-aware walking, follow-camera state and deterministic draw order.
`topDownInteraction.ts` owns swept footprint queries and deterministic nearby
selection. Collision boxes use entity-local metres and ship/planet anchors;
conservative player clearance is 0.46 m. Wall/door/crate art uses those same
footprints. Sweeps prevent tunnelling and separate local axes allow sliding. `TopDownView.tsx`
consumes the draw list as SVG: terrain, hull, cutaway interior, people, props and
foreground cues. Terrain fill precedes terrain objects; equal-layer objects use
stable ID ordering. There is no roof cover or roof fade. Astronomical planet
origins never enter camera-relative drawing; planet motion changes its universe
pose, and ship motion changes its planet-relative pose without rewriting anchors.
Local frame point conversion is explicit; automatic frame transfer is deferred.

The fixture uses authored illustrated placeholders, not scientific terrain or
full gameplay. Cross-frame traversal, cockpit seating/navigation, multiplayer, persistence,
resource systems remain later work. `topDown.test.ts` tests state/frames without graphics. The `top-down`
Playwright project exercises the production SVG adapter, pixel visibility, held
input cleanup, production radar isolation and byte-identical fresh-run captures.
Captures are new baseline candidates, not promoted navigation references.


### Player interactions (#762)

Approach the Core or cockpit chair to select its stable ID; a cyan outline
and nearby prompt show the available **Inspect [E]** action. **E** or the Inspect
button starts a test inspection, and **Escape**, Cancel, movement, focus loss,
resize, hidden-page state or loss of the target cancels it. Out-of-range requests
produce feedback and make no gameplay changes. Inspections are explicit no-ops;
they do not run Core machinery or open navigation. The state exposes standing,
walking and interacting modes plus a target ID for downstream consumers.

The Scout has an open cargo passage and aft entry; automatic planet/ship frame
transfer remains #765.
Walk around the crate on its starboard side, then through the open forward
partition to reach the cockpit. DOM focus clears held movement and editable
controls do not leak WASD/E input. Button activation returns keyboard control to
the world. `npm run test:browser -- --project=top-down` covers collision, the
keyboard detour, both consoles, cancellation, input cleanup, identical fresh-run
captures and built radar isolation. Position/facing data attributes are read-only
render observations, not state setters. Capture candidates are review evidence;
no approved navigation baseline changes.

### Scout cutaway (#763)

The same world contains a graphite/slate Scout outline, exposed warm cabin,
cockpit desk and separate chair, bunk, partitioned aft cargo room, Core,
Fabricator and **50 kg / 50 L** storage placeholder. Cargo passage (2.6 m) and aft
entry (3 m) are open; visible jambs and room boundaries block walking. The
threshold marks the aft approach. No roof layer, transparency or interior scene
swap is used. `scoutLayout.ts` owns ship-local named anchors, shared collision/
visual footprints and storage capacity metadata; machine/storage interactions
only inspect placeholders and do not process or transfer material.

`/?scene=top-down&view=exterior` starts a fresh development world outside the
Scout to inspect its always-visible interior. The normal fixture starts in the
cabin; `view=interior` selects it explicitly. These initial fixture viewpoints
are not in-play teleport controls. Walk through cargo to the entry; cross-frame
ownership/persistent locations remain #765, cockpit seating/radar #766, the
starting planetary area #764, and full playable-slice acceptance #767.
Browser SCOUT-1 exercises cargo module selection, doorway clearance and simultaneous
outside-player/interior visibility; TD-3 reaches the chair through the cabin.
These original captures are review evidence, not approved navigation baselines.

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

### Deterministic navigation regression (#750)

This suite targets the fresh browser prototype, not the removed native codebase.
`npm run test:browser -- --project=deterministic` uses a uint32-seeded sample
catalog (default seed 750), a 1280×800 viewport, DPR 1, en-US/UTC, and a paused
Playwright browser clock starting at `2026-11-04T00:00:00Z`. Tests advance RAF and
the production simulation with `page.clock.runFor`; input comes from actual
Playwright keyboard/mouse actions. The seed controls illustrative fixture
positions, not production astronomy. The ordinary desktop/production projects
also exercise the real-time runtime.

`tests/fixture.tsx` sets initial scenarios before play, including named vector
and camera presets. Its development-only observers return detached snapshots,
project positions, and record at most 120 completed WebGL frames. They expose
no ship/camera setters and never force a render. The normal entry and built
production bundle expose no observers or fixture page. `tests/deterministic.ts`
owns controlled boot, metadata and original screenshot capture.

| Criteria | Coverage |
| --- | --- |
| NAV-1 | Same-seed/input replay, changed seed, each WASD direction/release, wheel direction and removed buttons |
| NAV-2 | Tick-300 first discovery, selected contact and body/framebuffer continuity during keyboard and pointer pan |
| NAV-3 | Yellow heading/light-green dashed velocity reaching different edges, zero velocity, absent object guides/arrowheads; existing edges tests cover corners, opposite/overlap, zoom and off-screen origins |
| NAV-4 | Detached read-only observer state and production isolation |

Run `npm run test:browser:negative` **after build and the normal suite**. It copies
the client to temporary directories, reverses WASD movement or changes the
heading color, and requires the matching browser assertion to fail. Crashes,
timeouts, missing reports or unrelated failures fail the demonstration. Source
is restored by discarding the copies. `test-results/negative/` retains each
failure's original screenshot/trace, JSON results, command log and mutation hash
summary. CI runs both the normal and negative suites; a successful negative
demonstration means the intentionally broken copy failed, not that broken code
passed. Set `SALIMON_EVIDENCE_DIR` / `SALIMON_NEGATIVE_DIR` to separate output paths.

These captures are evidence, not approved pixel-regression baselines. Baseline
approval/diffs are #751. Review changes and originals in the active Codex chat;
the automated #752 review/status gate is retired.
The fixed Ubuntu CI runner and lockfile-selected Chromium improve repeatability;
a browser/OS update needs relevant revalidation. SwiftShader does not qualify
hardware GPUs, physical input, mobile/touch, native platforms or hardware FPS.

The contact picker is labelled, searchable and paged at 32 items. Buttons have
accessible names and visible focus. WASD and arrow shortcuts work without a
prior radar click when no form/button/dialog has focus. Focused form controls
retain native keyboard behavior. Blur, hidden-page events and viewport resize
clear held inputs; hidden time does not advance simulation. Scanner, selected
target, flight and warning status are explicit; throttle has a label and signed
forward/reverse/coast value. Desktop mouse/keyboard is the supported input path.
Browser automation uses synthetic input and software WebGL; it does not certify
physical input, native platforms or hardware FPS.

## Navigation screenshot regression (#751)

The [visual reference guide](tests/navigation-baselines/README.md) owns the eight
frozen browser screens, fixed fonts/Fontconfig/browser/backend, pixel comparison policy,
intentional visual defect demonstration and explicit independent baseline approval.
`npm run test:visual` compares all screens; run it twice for determinism and run
`npm run test:visual:negative` to prove the comparator rejects a real CSS defect.
The ordinary `test:browser` suite includes these captures. Candidate generation
never approves a baseline; CI blocks changed references until a human collaborator
approves the exact manifest digest. Software WebGL does not qualify hardware GPUs.
