# Salimon browser prototypes

Use the [2D documentation migration register](../docs/2d-migration-register.md) to distinguish the current browser implementation, approved future mechanics and archived native source contracts.

**2D gathering (#757; weight-only revision):** The continuous Scout/planet browser world uses one **10 kg** weight cap, **12 shared inventory slots**, and **up to 1 kg/s integer-gram** sample gathering with approved survival penalties with atomic local saves. `resourceInventory.ts` also models count-based items with mass and per-type stack sizes (future content). `resourceSession.ts` saves a schema-v11 resource/item/survival/hazard/Scout/site ledger and migrates valid v1
litre / v2 gram saves without loss. `resourceLedger.ts` validates exact transfers
and store mass; future stores require trusted ID/ship/capacity declarations.
The default world enables the 50 kg Scout Resource Container and deposit/withdraw panel (#784/#785). See [resource design](../docs/resource-inventory-and-gathering.md).

The default client is a React + TypeScript + Vite 2D world with a Three.js /
React Three Fiber panoramic cockpit. Start with [current implementation and legacy
contract routing](../docs/current-client-status.md) and the
[playable journey](../docs/playable-2d-foundation.md). This is a new prototype and does not migrate the historical native Rust gameplay. See [design contract](../docs/radar-navigation-prototype.md).

## Run

```bash
cd client
npm ci
npm test
npm run dev
```

Open the URL Vite prints. `npm run build` produces `dist/`.

## Cockpit gameplay (#766)

The default `/` (development and production) starts outside the Scout in 2D.
Walk around the aft hull, through the open entry, around the cabin crate and
forward to the cockpit chair. **E / Use cockpit** seats the owner and opens the
full-viewport panoramic observation scene. **Exit cockpit** returns standing beside the
chair in one action. Ship position, velocity, orientation, throttle and scanner
continue in the same `NavigationRuntime` while walking or observing the universe.
Both views share the grounded Scout/Vesper surface anchor (#802). Grounded manual propulsion, turning and alignment remain gated. An authorized
`ResourceSession.transitionScout({kind: 'begin-takeoff'}, revision, liveSeat)`
starts the assisted launch only after the aft door is closed and all occupants
are inside. The production Take off button shows admission/progress (#841); the development
`?flightFixture=1` command harness is excluded from the shipping build.

The #840 launch lifts 50 m over 360 fixed ticks (6 simulation seconds), with
nonnegative radial acceleration `2H/T² × (1 − cos(2πt/T))`. Peak acceleration is
5.56 m/s² and final outward speed is 16.67 m/s plus inherited inertial surface
velocity; acceleration vanishes at both endpoints. The conservative 50 MN
thruster budget admits at most 9,000 t loaded mass for this profile. No mass,
impulse law or Core balance is changed. `TAKEOFF_FUNDING` marks the future funding
adapter seam; the current prototype supplies unlimited energy. Admission sweeps
the complete fixed-step path against conservative hull/body spheres before any
state change. Launch reuses the validated saved surface mapping on Vesper, its moon or Caligo;
planetary ephemerides and gravity remain outside this prototype. See the surface-site contract below.

Manual inputs, Set direction, other assists and `finish-takeoff` cannot override
an admitted launch. The world clock continues through cockpit exit/observation
unmount and blur, and pauses normally while hidden or explicitly paused. Saves
retain the plan and validate its pose/velocity before resuming. Old grounded and
flying snapshots migrate with no launch plan; old synthetic taking-off snapshots
without a plan fail safely rather than fabricating motion. Completion clears the
plan and selects throttle zero without changing the terminal pose/velocity.


`TopDownView` owns one `NavigationRuntime`, one `ObservationCameraState` and the
only production navigation clock. `PanoramicNavigation` receives
`advanceSimulation={false}`; its standalone fixtures still advance their own
runtime. First entry/new character/reload starts at ship heading and 75°; exit
and re-entry within the same active session preserve direction/FOV. Reset view
aligns to heading. View state never enters the save or multiplayer domain.
Blur, hide, pause and unmount cancel held camera input and captured Look at.
Wheel over a hovered sensor marker shares canvas zoom; panels keep native
scrolling and focused forms do not steer observation.
The optical origin is the exact shared `scoutLocation.ts` ship position: Vesper
occludes the lower hemisphere; no screen-space horizon or independent ship is
created. Other scanned bodies support Look at; grounded Vesper self-body actions
remain disabled. Manual throttle/steering require flying mode; Take off starts
the authorized launch. Survival/pause remains available.

`cockpit.ts` owns one local chair reservation, owner-control admission and power
transitions. `TopDownView` keeps the 2D world mounted; only panoramic presentation is
mounted/unmounted. A seated avatar is drawn over its chair, in ship-local space.
Power loss closes observation and keeps the chair occupied; restoration stays seated
in 2D until **E**. **Stand up / Escape** releases the chair even without power.
Development-only `/?powerFixture=1` exposes explicit outage/restoration buttons;
this is a test fixture, not a Core implementation. No multiplayer/server authority,
backend persistence or multiplayer reconnect behavior is implemented. Browser-local
inventory/depletion/location persistence is implemented by `ResourceSession`. Occupant identity and
control gating are local hooks for a future authoritative adapter.

`/?scene=radar` retains the standalone navigation prototype and its regression
suite. `/?scene=top-down` in development retains the isolated inspection fixture
for the #761–#765 checks. These fixture queries do not implement in-play teleport.
`npm run test:browser -- --project=cockpit` walks from default spawn and covers
normal exit, outage/restoration, chair state, preserved camera/FOV, a persistent grounded anchor and
advancing ship/scanner ticks across view switches. The production slice also
checks repeated entry and local save/reload. It repeats original captures with the unchanged pixel policy.

## Top-down foundation (#761)

During `npm run dev`, open `/?scene=top-down` for a strict overhead illustrated
Scout and planetary site in one continuous scene. The player starts outside.
**W/A/S/D** walks the fixed landing area or the interior fixture; visible walls, furniture and a crate block movement. The camera follows with fixed-step
smoothing, finishing at the exact player anchor below a nanometre of lag; facing persists when movement stops. Focus loss, editable controls,
resize and hidden-page events release movement. The isolated development fixture uses inspection-only consoles; the default `/` integrates cockpit gameplay above.

`topDownFrames.ts` owns planet-fixed/ship-local anchors using the existing split
`Position`; `topDownRuntime.ts` owns stable IDs, immutable scene state, 60 Hz steps,
collision-aware walking, follow-camera state and deterministic draw order.
`topDownInteraction.ts` owns swept footprint queries and deterministic nearby
selection. Collision boxes use entity-local metres and ship/planet anchors;
conservative player clearance is 0.46 m. Wall/door/crate art uses those same
footprints. Sweeps prevent tunnelling and separate local axes allow sliding. `TopDownIllustration.tsx` owns the original SVG art. `topDownSprites.tsx` caches
it once at fixed resolution (64 texels/metre; terrain 32), using bounded blob
references. `TopDownView.tsx` consumes the draw list as SVG image layers: terrain, hull, cutaway interior, people, props and
foreground cues. Terrain fill precedes terrain objects; equal-layer objects use
stable ID ordering. There is no roof cover or roof fade. Astronomical planet
origins never enter camera-relative drawing; planet motion changes its universe
pose, and ship motion changes its planet-relative pose without rewriting anchors.
Local frame point conversion and aft doorway ownership transfer are explicit.

The fixture uses authored illustrated placeholders, not scientific terrain or
full gameplay. The isolated development fixture excludes cockpit/gathering/saves;
the default gameplay entry includes those systems. `topDown.test.ts` tests state/frames without graphics. The `top-down`
Playwright project exercises the production SVG adapter, pixel visibility, held
input cleanup, production radar isolation and byte-identical fresh-run captures.
Captures are new baseline candidates, not promoted navigation references.
Renderer readiness waits for authored sprites and exposes only a read-only flag;
asset caching never changes entity IDs, collision footprints, anchors or gameplay.
World traces retain actions, screenshots and sources; explicit state JSON replaces
redundant per-action snapshots of immutable image assets.


### Player interactions (#762)

Approach the Core or cockpit chair to select its stable ID; a cyan outline
and nearby prompt show the available **Inspect [E]** action. **E** or the Inspect
button starts a test inspection, and **Escape**, Cancel, movement, focus loss,
resize, hidden-page state or loss of the target cancels it. Out-of-range requests
produce feedback and make no gameplay changes. Inspections are explicit no-ops;
they do not run Core machinery or open navigation. The state exposes standing,
walking and interacting modes plus a target ID for downstream consumers.

The Scout has an open cargo passage and aft entry with automatic planet/ship
frame transfer (#765).
Walk around the crate on its starboard side, then through the open forward
partition to reach the cockpit. DOM focus clears held movement and editable
controls do not leak WASD/E input. Button activation returns keyboard control to
the world. `npm run test:browser -- --project=top-down` covers collision, the
keyboard detour, both consoles, cancellation, input cleanup, identical fresh-run
captures and built radar isolation. Position/facing data attributes are read-only
render observations, not state setters. Capture candidates are review evidence;
no approved navigation baseline changes.

### Seamless doorway traversal (#765)

From the outdoor spawn, walk south around the Scout and north through the
painted aft entry, then through cargo and the cabin to the cockpit. Return by the
same opening. `topDownTraversal.ts` transfers the collision-resolved player anchor
only when its path crosses the clear 3 m entry plane. Intact hull, jambs and
partitions remain physical. Conversion preserves world position; the camera
keeps its smoothing lag and zoom while adopting the player's new frame. The
interior, surface props and entity IDs remain in the same SVG scene throughout.

`capturePlayerLocation(state)` returns a detached in-memory snapshot of the exact
split anchor (bigint cells, metre offsets, frame ID) and facing. Ship-local
locations follow later ship pose changes; planet-fixed locations stay on the
surface. This is a hook for a future authoritative reconnect adapter, which must
validate spawn/collision and serialize the snapshot; the default entry serializes it in local `ResourceSession` saves, while
multiplayer reconnect remains future work. Simulated relocation is tested without adding flight controls.
`topDownTraversal.test.ts` covers rotated handoffs, boundary/blocked paths,
world/camera continuity and location retention. Browser `TRAVERSE-1` walks the
outdoor-to-cockpit round trip twice, measures both handoffs, keeps the same scene,
and compares simultaneous indoor/outdoor captures using the unchanged visual policy.

### Scout cutaway (#763)

The same world contains a graphite/slate Scout outline, exposed warm cabin,
cockpit desk and separate chair, bunk, partitioned aft cargo room, Core,
Fabricator and **50 kg raw-only Resource Container**. Cargo passage (2.6 m) and aft
entry (3 m) are open; visible jambs and room boundaries block walking. The
threshold marks the aft approach. No roof layer, transparency or interior scene
swap is used. Explicit line metrics keep the desktop SVG viewport on the device-pixel grid.
The world viewport and translucent caption use rectangular pixel-aligned edges;
curved ancestor clipping otherwise varies at compositor/capture boundaries. `scoutLayout.ts` owns ship-local named anchors, shared collision/
visual footprints and storage capacity metadata; Core/Fabricator interactions
inspect placeholders. The Resource Container opens its transfer panel in the default game.

`/?scene=top-down&view=exterior` starts a fresh development world outside the
Scout to inspect its always-visible interior. The default fixture starts outdoors;
`view=interior` explicitly selects the cabin for Scout-only checks. These initial fixture viewpoints
are not in-play teleport controls. Walk through cargo to the entry; cross-frame
ownership transfers at its open aft threshold (#765); cockpit seating/radar remains #766, and full playable-slice acceptance #767.
Browser SCOUT-1 exercises cargo module selection, doorway clearance and simultaneous
outside-player/interior visibility; TD-3 reaches the chair through the cabin.
These original captures are review evidence, not approved navigation baselines.

### Fixed planetary landing area (#764)

The development world now starts **outside** at the invariant planet-fixed
`[-6, 3]` metre anchor. `planetarySite.ts` owns the fictional version-1 site ID,
fixed Scout landing pose, flat illustrated strata, collidable boulders/stones,
and sparse alien plate colonies. This is one authored living-world sample,
not confirmed real-world biology or procedural whole-planet generation.
The sample's `planet:fixture` reference frame and object IDs are stable across
fresh visits, camera movement and ship pose changes.

The banded mineral outcrop (`source:landing:mineral:01`) and alien plate colony
(`source:landing:biology:01`) expose stable IDs, planet-fixed anchors, physical
footprints, source kind/archetype and site version identity. Approach them and
**E** inspects their placeholders through the shared interaction mechanism;
walking into either source stops at its visible footprint. Repeated inspection
has no extraction or carrying effect. In the default gameplay entry, equipping
the gathering tool changes **E** to gathering, with catalog/depletion and atomic
inventory transactions owned by `resourceSession.ts` (#757).
`SourceIdentity` is descriptive metadata, not an authoritative save schema.

`PLANET-1` drives a full outdoor keyboard journey twice and compares original
spawn, near-mineral, near-colony and near-Scout screenshots with the unchanged repository visual policy, while
checking stable source identity/positions, collision stops and the same scene
node. Replays use fresh browser contexts, exact observed simulation tick counts
for real keyboard input, and five simulated seconds for camera settling. The
existing `compareImages` policy permits zero pixels beyond its fixed 0.1
perceptual threshold, includes antialiasing and applies no masks; raw changed
pixels and diff images are retained. A wrong-checkpoint comparison must fail.
Full-viewport PNG readback preserves the viewer pixel grid; the SVG adapter
requests geometric precision. The earlier static TD-1/4 byte-identity check is
preserved. Domain tests cover clear outdoor spawn, source reach, repeated inert
inspection, footprint blocking and independence from the camera/ship pose.
The default production `/` integrates cockpit entry (#766), doorway frame transfer
(#765) and durable local gathering/location saves (#757).

## Playable-slice acceptance (#767)

See the [full journey, local commands and mining reuse boundary](../docs/playable-2d-foundation.md).
`SLICE-1` in the `cockpit` Playwright project exercises the built default entry,
both source archetypes, continuous doorway/camera handoffs, actual radar controls,
standing return and walking back outside, twice in fresh browser contexts.
Original screenshots and raw frame/source measurements are retained in the issue report.
This proves foundation playability. #757 adds gathering and local saves below;
Core, Fabricator, multiplayer and ship damage remain future features.

## Gathering and saved expeditions (#757)

In the default world, click **Equip gathering tool**, approach the banded outcrop
or plate colony and press **E / Gather**. Up to **1 kg** is collected per completed work interval; low hunger/fatigue reduce the rate. See [approved tuning](../docs/browser-survival.md#approved-work-and-action-tuning-799).
Movement, Escape, loss of reach, stowing the tool or focus loss stops gathering
and checkpoints source-bound partial work, which survives reload. The HUD shows weight/slot limits,
separate resource stacks and remaining source mass. A full inventory leaves
the source's uncollected material intact. No physical fragments or carrying
simulation are involved.

Completed gram transfers save immediately in this browser. Movement is checkpointed at
most once per simulated second, at movement stop and on page hide/blur. Reload
restores location and inventory, never active gathering. Cockpit reloads stand
beside the chair. A second same-origin tab cannot edit the open expedition.
Use HTTPS or localhost; blocked storage/invalid saves pause gathering and preserve
the saved bytes. Data is browser-local; there is no server/cloud/multiplayer save.

`npm test` covers exact grams, weight/slot caps, count-item stacks, interruptions, failed and
stale writes, malformed/incompatible saves, upgrades, depletion and payload mass.
The existing Navigation client CI includes `GATHER-1`/`GATHER-2` through
`npm run test:browser -- --project=cockpit --grep GATHER`, using the built entry,
real input, reload, competing tabs and repeat screenshot comparisons. Approved
navigation baselines are unchanged. Machine buffers/transfers, upgrade UI/costs
and broader resource balance are future work.

## Controls (desktop mouse and keyboard)

- **W/A/S/D** pans the independent camera up/left/down/right. Left-drag empty space to orbit; right-drag to pan.
- Mouse wheel zooms smoothly within bounds; **Reset camera** restores the viewer without changing the ship.
- Arrow keys turn ship pitch/yaw; manual turning cancels one-time **Set direction** alignment. Camera movement preserves alignment.
- Click a visible contact, or use the scanned-contact picker, then **Set direction**.
- Throttle slider controls forward/reverse thrust; **Space** sets zero throttle while navigating. Zero throttle coasts. Energy is infinite.
- **Help** opens the keyboard-accessible binding reference; Escape or Close help dismisses it and returns focus. Form controls retain their native keys.

For usability, visible sample distances are compressed *as input test fixtures*, not via a nonlinear rendering transform. All rendered positions use a linear conversion from simulated coordinates, and scanner range is exactly 100,000 light-seconds. The catalog is not astrophysically authoritative. Ship motion uses the approved SI impulse/mass model on a renderer-independent 60 Hz clock. `coordinates.ts` owns integer galactic cells/local offsets, `simulation.ts` owns motion and one-time alignment, and `App.tsx` owns input and presentation. Energy remains infinite. Tests run on Node 22.18 or newer without a separate test runner.

The first scanner pulse arrives after five simulation seconds. `scanner.ts` owns the immutable catalog index and disposable contact/selection state. `radar.ts` owns visible frustum groups; `ContinuousContacts.tsx` retains meshes by scanned ID and updates grouping/visibility/positions each render frame; click a group to select any member from its paged list. Zoom never reveals unscanned catalog bodies. The lattice stays fixed to galactic axes. Off-screen objects have no scene indicator; their scanned IDs remain in the picker. Yellow heading and light-green dashed velocity rays run from the projected ship to the viewport edge. Zero speed suppresses velocity; off-screen ship origins use forward-ray clipping.

`navigationRuntime.ts` composes motion, physical contact and scanning. `contact.ts` uses a conservative 5 m ship sphere and physical body radii, including rotating surface velocity. Impacts >=100 m/s end the run. Lower-speed contacts rebound/settle with zero thrust; <=20 m/s on a planet/moon is safe contact, separate from assisted landing. Nonplanetary contacts always recover as crashes. Restart navigation clears the standalone sample run. In the production cockpit,
`scoutLocation.ts` binds the 2D site and radar to the same grounded Scout on
Vesper until an authorized Take off detaches the ship interior and enters the
automatic launch. Manual steering, alignment and propulsion require flying mode;
the cockpit has no restart-to-flight action. Scanner/camera and simulation ticks
continue across cockpit exit and re-entry; ship and scanner snapshots persist. Warnings observe scanned contacts without steering.

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
cache, `npm ci`, domain tests, build and installed Chromium. PRs run the focused
`npm run test:browser:smoke` selection; daily/manual full runs retain the entire suite.
`test-results/` contains JSON results, original screenshots and failure traces;
CI retains these as a revision-named artifact for 14 days. New completion records
belong in the PR description/comment; keep local originals in ignored output and
do not commit new report/evidence archives. CI tracking belongs to the user.

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

These captures are evidence. Visual references/diffs are #751; the owner reviews
appearance in the deployed version without a separate approval-comment CI job. Review changes and originals in the active Codex chat;
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
intentional visual defect demonstration and retained image/diff evidence.
`npm run test:visual` compares all screens; run it twice for determinism and run
`npm run test:visual:negative` to prove the comparator rejects a real CSS defect.
The ordinary `test:browser` suite includes these captures. Candidate generation
does not certify appearance. The owner removed the human baseline-approval CI job
in PR #834; subjective review happens in the deployed version, with separate issues
for regressions. Automated comparisons, repeats and negative checks remain required.
Software WebGL does not qualify hardware GPUs.

## Scout container domain (#784)

The default ResourceSession declares `scout:storage`, a 50,000 g raw-only store.
Exact `deposit(resourceId, grams, revision)` / `withdraw(...)` transactions use
the shared ledger and atomic save. Schema v5 migrates old v1/v2/v3/v4 expeditions
without changing source/player mass, and rejects damaged current container data.
Store mass contributes once to ship simulation independently of player location.
The default game opens the playable panel described below; the isolated development
world retains inspection-only fixtures.

## Resource Container transfers (#785)

Approach the labelled 50 kg container on the starboard side of the aft cargo bay,
then press **E / Use container**. The modal shows both inventories in fractional
kg, player weight/slots and remaining container weight. Choose a raw resource and
an exact positive whole-gram amount, select **Review deposit** or **Review
withdrawal**, then **Confirm**. Reviewing/editing/cancelling never moves material.
A rejected request explains insufficient balance, weight/slots, stale revision or
save access; no smaller amount is silently transferred.

Escape, Close, window focus loss, resize/hidden state, movement/target range loss
or cockpit/scene transitions cancel only the draft. Committed transfers remain
saved. Modal focus contains keyboard/mouse interaction; editable fields keep their
keys. The normal world controls resume on close; ship simulation remains active.
`resourceContainerInteraction.ts` owns admission/draft/confirmation/cancellation;
`ResourceContainerPanel.tsx` and `resourceContainer.css` own accessible responsive
presentation through shared `EquipmentDialog`, Button and Progress primitives
(#829). The opaque industrial shell shares identity/title/status, machine content,
feedback and dismiss/action regions. Carried/container kg, exact g meters and free
capacity stay visible alongside the raw table. Draft, committed and rejected
states use explicit words; save-blocked controls retain reachable explanations.
Small desktop windows scroll inside the native dialog. No production state setters
are exposed.

`tests/storage.spec.ts` uses the built default entry to mine → enter → deposit →
withdraw → exit → reload twice, compares original panel/error captures and
checks a validated prior full-container save, overweight/stale-save errors and a
narrow viewport. `STORE-UI-3` verifies the production shell at 960×600,
1280×720 and 1920×1080: focus wrapping/isolation, accessible names/descriptions,
text contrast, reduced motion, panel-local overflow, repeat originals and a wrong
checkpoint negative. Captures/measurements are retained in
[the #829 report](../reports/issue-829/README.md). It is part of the existing cockpit project and full browser CI.
The player-slot feedback helper is tested with validated synthetic usage; the
current two-resource/empty-item catalog cannot fill all 12 slots in browser play.

The NAV-1 replay completes a 1,200 ms input window (75 × 16 ms RAF intervals,
72 fixed 60 Hz ticks) before equality comparison, and asserts that exact tick
count. A partial 1,088 ms window could quantize to 65 or 66 ticks depending on
renderer startup phase. The build/browser CI job allows 20 minutes for cold
Chromium installation plus the full matrix; individual test limits and assertions
remain in force.

### Generic count items (#786)

The field inventory displays raw kg and registered item counts/kg/shared slots.
`ResourceSession.changeItem` validates positive integer counts, trusted authored
weights/stack sizes and an expected revision, then atomically persists the full
candidate. The production item catalog stays empty until content is approved.
Schema v5 binds saved definitions to that trusted catalog and migrates valid
v1–v4 empty-item expeditions without changing raw mass or location.

`tests/items.html` is an explicit non-shipping fixture using the production
session/HUD with synthetic stack=1/stack=4 definitions and fixture-only controls.
Run `npx playwright test --project=cockpit items.spec.ts` for mixed inventory,
overfill, exact removal, reload, narrow layout and repeat/negative captures.
Normal gameplay has no query flag, controls or item grants from this fixture.

## Survival clock (#787)

The playable world includes persisted 200 HP, hunger and fatigue, foreground active-time drain, launch-only fatigue recovery and a labelled top-left HUD. Pause with P or the HUD button. A terminal character can explicitly restart without restoring depleted sources or granting items. See [rates, migration and deferred tuning](../docs/browser-survival.md). Unit/integration coverage is `src/survival.test.ts`; built browser coverage and originals are `tests/survival.spec.ts`.

## Hazard domain (#788)

The production session supports exact temperature damage, radiation/poison exposure
and 1,000 HP protection budgets without shipping hazard sources or equipment. See
[domain, save and fixture contracts](../docs/browser-hazards.md). `npm run test:browser`
builds a separate non-shipping hazard fixture on port 4275; ordinary `npm run build`
includes only the normal game. Unit/integration coverage lives in `src/hazards.test.ts`.

## Full-viewport 2D HUD (#803, #827)

The world fills the browser viewport. Compact labelled survival meters, Pause
and Help sit top left; carried kg, shared slots and the gathering tool sit top
right. **Field inventory / I** opens raw/count item details and save information
on demand. Escape closes that utility first, returning focus to its trigger;
entering the cockpit closes it. Focused utility/help controls isolate gameplay
keys and interrupt extraction while retaining exact partial work. Menus do not
pause active time. Survival warnings remain explicit beside pause state and in
open help. The contextual HUD shows an eligible nearby action, transaction/error
feedback and exact mining progress/capacity while working.

Panels scroll their own overflowing content; the page stays fixed. Resize
expands the SVG view without distorting world proportions. The cockpit-project
`tests/viewport.spec.ts` checks the built `/` at 960×600, 1280×720 and 1920×1080,
repeat/negative screenshots, DPR 2, inventory/help/focus and partial-work safety,
resize and the standalone dev fixture. See the
[layout contract](../docs/playable-2d-foundation.md#viewport-and-hud-803) and
[#827 evidence](../reports/issue-827/README.md).

The 2D camera universally keeps the active ship vertical with its bow screen-up.
All scene positions/facings and selection cues share that basis; W/A/S/D is
screen-relative through the inverse input transform. Physical ship headings,
anchors and saves are unchanged. Unit coverage tests arbitrary headings and
indoor/outdoor ownership; viewport coverage asserts the rendered bow is vertical.

## Perspective observation camera foundation (#810)

The development-only `/tests/observation.html` fixture exercises the new camera;
production cockpit uses this camera through #815; `/?scene=radar` retains the orthographic flight fixture.
Start with `npm run dev`. This fixture uses synthetic calibration targets and
actual fixed-step ship motion; it is not the physical sky or final cockpit HUD.
It is absent from the production build.

`observationCamera.ts` owns a session-local `ObservationCameraState`. Construct
it once from current ship yaw/pitch outside the mounted scene. Pass that owner
to `<ObservationCamera view={view}/>`. `snapshot()` returns detached quaternion
and FOV values; `restore(snapshot)` validates/normalizes them atomically and
clamps FOV. `command({type:'reset', heading: currentShip})` samples current
heading once and restores 75°. Look/zoom commands are separate from flight
controls; the view owner never writes a ship snapshot.

Use `observationFrame(view, currentShip).origin` as the split physical render
origin, and `observationRelativePosition(body.position, currentShip)` for scene
coordinates. The Three camera stays at zero in this ship-relative frame; never
narrow absolute bigint galactic cells to Number. A future optical renderer must
provide appropriate distance/depth staging (#811).

WASD rotates around the current screen up/right axes; left-drag grabs the view.
Pole crossings transport that basis continuously rather than snapping to world
up; there is no commanded optical-axis roll. Wheel zoom is exponential per
normalized wheel pixel, clamped to 0.1°–75° vertical FOV. Look increments scale
with FOV, including keyboard and drag. Arrow keys remain exclusively ship
steering. Form focus blocks keyboard/wheel observation; explicitly clicking the
canvas transfers focus back to observation. Blur, visibilitychange, focus,
resize, pointer cancellation and unmount release active input.

Run `npm test && npm run build`, then
`CHROMIUM_PATH=/usr/bin/chromium npm run test:browser -- --project=observation`.
OBS-1–5 cover real WebGL, independent ship steering/motion, reset/remount,
telescope sensitivity/pixels, focus cancellation, resize/DPR, spherical input,
repeat captures and a deliberately wrong wide-versus-telescope comparison.
These originals are evidence, not approved navigation baselines. Existing
automated regression gates remain required in CI.

## Physical optical sky foundation (#811)

`OpticalSky.tsx` consumes only catalog bodies and a `getOrigin()` callback to the
shared authoritative ship position. `opticalSky.ts` subtracts split coordinates
before converting bounded directions/radius ratios to GPU uniforms. It never
reads scanner contacts, emits identities, registers pointer actions, or changes
simulation. Production composes this camera through #815.

The development route `/tests/optical.html?scenario=wide` renders a single reused
quad with analytic per-pixel sphere intersections. Available scenarios are
`wide`, `telescope` (0.1°), `surface`, `below`, `translated` and `clear` (occulted
body removed as a negative control). Surface scenarios use `startingScoutLocation()`
and sample Vesper's catalog radius, leaving the real 5 m ship-bound clearance.
No eye translation or cosmetic horizon is added. Physical hit distances resolve
occlusion without a mixed astronomical/ground depth buffer. Factored CPU surface
terms retain near-surface clearance; perpendicular ray vectors retain tiny disks
at telescope magnification. Uniform spheres have restrained diffuse terminators;
no texture, atmosphere or geological detail is claimed. Subpixel bodies use an
energy-limited Gaussian PSF. A deterministic sparse decorative starfield carries
no contact identity. The explicit budget is 32 bodies, two triangles and one draw
call, with constant geometry regardless of FOV/distance; larger catalogs must be
partitioned upstream (overflow fails explicitly rather than silently hiding bodies).

Run `CHROMIUM_PATH=/usr/bin/chromium npm run test:browser -- --project=optical`.
OPT-1 checks pre-pulse disks/no selection, unchanged pixels after a real scanner
pulse, occulted-body removal, repeat stability and a huge translated origin.
OPT-2 captures telescope/tiny moon, the shared grounded horizon and downward
occlusion. These captures are review candidates, not approved baselines. Existing
CI negative/repeat regression gates remain required; no human approval job runs.

GPU ordering uses logarithmic physical ray distances, avoiding float overflow
without exposing absolute galactic coordinates. The shader/material is disposed
on unmount. Software-WebGL capture timings are not a GPU/Metal frame-rate claim;
the budget bounds catalog work and geometry, not the full future cockpit HUD.

## Perspective sensor overlay foundation (#812)

`PerspectiveContacts.tsx` is the reusable DOM companion to `ObservationCamera`
and `OpticalSky`; mount it in the same positioned viewport. The parent supplies
its live `Scanner`, `ObservationCameraState`, physical `getOrigin`, and optical
catalog geometry. `perspectiveContacts.ts` consumes **only current scanner
contacts** for markers/browser entries; geometry only vetoes line of sight.
Every selection/action handler rechecks live scanner membership and range.
The parent still advances scanner ticks independently of rendering/camera input.
An optional `onSetDirection` callback connects the existing authorized flight
command; its caller retains power/cockpit authority through `canPilot` and the live `getShip` snapshot. Grounding is also guarded by the overlay. Camera-only Look at is described below.

Projection rotates split-relative unit directions into camera space, explicitly
rejects behind-camera/outside-frustum centers, and compares physical surface
ray intersections using the optical layer's stable CPU geometry. No astronomical
near/far plane narrowing or physical-radius inflation is used. Occluded center
rays have no marker/hit target, including partially occulted centers; scanned
contacts remain searchable/paginated in the browser. The simplified physical
planet globe provides the grounded surface occluder; detailed terrain is not
introduced. Marker overlap components retain sorted unique IDs, with eight
entries per browser page. Menus lose their actionable entries if their current
visible group disappears; range eviction removes selection actions.

The separate development route `/tests/perspective.html` exercises the composed
panoramic scene; `?translated` repeats it at huge galactic cell offsets. Tests:
`src/perspectiveContacts.test.ts` and Playwright project `perspective`.
For bounded software rendering, the browser scenarios use `?low-resolution`
(480×300 optical canvas in a 960×600 CSS viewport; full-resolution DOM controls).
The default fixture retains DPR 1. Its Telescope calibration preset centers the
tiny test moon at 0.1°; it is not a player Look at implementation.
The production cockpit composes this sky through #815; this is not a
compact HUD (#814). The existing 32-body optical catalog budget still applies.
Run `npm test`, `npm run build`, and
`CHROMIUM_PATH=/usr/bin/chromium npm run test:browser -- --project=perspective`.
Original evidence and limitations: [issue report](../reports/issue-812/README.md).


## One-time contact observation (#813)

`PerspectiveContacts` exposes independent **Look at** and **Set direction**
commands for a currently scanned selection. Supply `getShip` and `getOrigin`
from the same authoritative ship snapshot, and a live `canPilot` callback for
cockpit/power authority. Recheck flight authority in the parent callback;
`setContactDirection` is the reusable scanner/flight guard. The current
orthographic flight fixture stays isolated; production uses panoramic integration #815.

`lookAtContact` captures the current ship-to-contact vector once, even for a
rear/occluded browser entry. `ObservationCameraState.lookAt` transports the
current camera frame along a shortest quaternion arc over one second of active
view time using smoothstep interpolation. At exactly opposite directions the
current screen-up axis breaks the tie deterministically. It preserves FOV,
ship heading, velocity, throttle and any ongoing ship alignment. It retains
no target reference and never tracks moving bodies or the moving ship.
`ObservationCamera` advances it on rendered frames (delta bounded at 50 ms).
Manual left press/drag, W/A/S/D, wheel, reset or snapshot restoration cancels it;
idle commands and focused typing do not. Manual ship steering cancels only
Set direction. Call `cancelLookAt()` for an explicit view interruption.

Grounded Scout can observe other detected bodies without flight authority.
Its currently landed planet is identified in the browser and both commands are
disabled for that body; Set direction remains disabled for all grounded targets.
Range loss clears selection/actions, while the captured animation may finish.
Optical geometry stays opaque; Look at never exposes hidden markers.
The overlay includes expandable keyboard/action help.

`/tests/look-at.html` composes these production owners in standalone flying and
`?grounded` shared Scout/Vesper fixtures. Verification:
`node --experimental-strip-types --test src/observationCamera.test.ts src/contactCommands.test.ts`
and `CHROMIUM_PATH=/usr/bin/chromium npm run test:browser -- --project=look-at`.
Broader regression and approved visual baselines remain in existing CI.

### Compact panoramic HUD (#814)

`PanoramicNavigation.tsx` composes the independent observation camera, optical sky,
scanner-gated perspective markers, collapsed contact browser and full-viewport HUD.
Pass a `NavigationRuntime` and parent-owned `ObservationCameraState`; optional
`canPilot`, `isPaused`, `onExit` and `onSceneReady` callbacks preserve the lifecycle
owner's authority. By default the component advances the supplied runtime while mounted. Production passes `advanceSimulation={false}` because `TopDownView` owns the sole navigation tick loop. Production cockpit
activation and view persistence are integrated by #815; the orthographic radar remains an isolated flight fixture. `/tests/panoramic.html` exercises the reusable production component;
`?grounded` uses the shared Scout/Vesper domain anchor.

The sky keeps #811's explicit 32-body uniform budget (`opticalBodies` may supply
that bounded physical render set separately from the scanner catalog). CPU sensor
occlusion uses the entire runtime catalog, with no 32-ID cap. No camera-owned
physics or server state is introduced. All scanned contacts remain searchable and
paged, whether grouped, occluded or behind the camera.

C opens contacts from the playfield, Tab traverses controls, and Escape dismisses
help, contacts, then telemetry before requesting cockpit exit. Focused controls
suppress WASD, ship arrows and optical wheel. Reset returns to the current ship
heading and 75° FOV. The compass uses a camera-relative hemispherical disk: center
is axial, rim is sideways, up/down follows the screen. Solid markers are front;
hollow markers are rear (exact 180° rear is at the center). Yellow triangles and
larger green circles distinguish coincident vectors; cyan marks the view axis.
Velocity at or below 0.01 m/s is Stationary. Speed explicitly uses the sample
inertial frame. Warnings/terminal actions have priority over expanded telemetry.

Run `npm test`, `npm run build` and
`CHROMIUM_PATH=/usr/bin/chromium npm run test:browser -- --project=panoramic --project=perspective --project=look-at`
when using an existing system Chromium (omit `CHROMIUM_PATH` for Playwright's
installed browser). The new browser project runs in the existing functional CI;
compass repeat captures and a front/rear negative comparison do not change or
approve any visual baseline.

## Panoramic acceptance matrix (#816)

`npm run test:visual` also runs `PANO-VIS` in the existing navigation-captures
project: 19 fixed-clock production-adapter screenshots cover 75°/0.1° optics,
huge split origin, scanner acquisition/loss, opaque occlusion and rear contacts,
overlap/pagination, grounded horizon, Look at, free-flight Set direction,
compass stationary/rear/overlap and 960×600 warning/terminal priority. The default
production chair/power/2D journey remains `COCKPIT-1`, with two fresh contexts,
originals, unchanged pixel policy and state assertions. `HUD-1` covers keyboard
Tab/Enter, search, Escape priority and focus restoration. Existing 2D/scanner/ship
regressions remain in the full browser suite.

`OPT-3` compares identical angular scenes from 1 cm to 1e24 m depth and repeats
0.1° optics at huge split coordinates. `SENSOR-4` stresses alternating keyboard
movement and high-frequency wheel events at telescope FOV, retaining every
sample, rendered zoom states and raw 24×24 GPU readbacks. A physical one-metre
stress target makes blank-frame checks meaningful. Marker precision, exact
origin, stable endpoint pixels and unchanged scanner time are asserted.
`OBS-5` covers full spherical turns and physical ship-origin following.
`SENSOR-3` rejects unscanned optical markers and stale marker click-through.
`npm run test:browser:negative` deliberately leaks optical markers in a disposable
source copy and must fail that named assertion, alongside the older mutations.
`npm run test:visual:negative` additionally changes one expected telescope PNG
in a disposable copy, updates its integrity hash, and requires exactly that
pixel comparison to fail. A crash, timeout or environment drift is not success.

To propose panoramic references locally, run:

```sh
SALIMON_PANORAMIC_CANDIDATES=/tmp/panoramic-candidates npm run test:visual -- --grep PANO-VIS
npm run test:visual
SALIMON_EVIDENCE_DIR=test-results/visual-repeat npm run test:visual
npm run test:visual:negative
```

Candidate generation writes a complete proposal outside the tracked references,
retaining the older images and shared manifest fields. Review the originals and
copy the proposed PNGs/manifest into a dedicated reviewable change before normal
comparison. It rejects relative/in-place output and is forbidden in CI. Generating
or matching candidates does not certify appearance. No separate human-approval
comment or digest gate is required; the owner reviews the deployed result and
files follow-up issues for visual problems. Never regenerate references to resolve
a failure. Keep originals, diffs, JSON metadata and traces in `test-results/`
and retain essential raw evidence with the issue report before artifacts expire.
Software SwiftShader measurements do not establish target-device GPU/Metal FPS;
physical input/platform and subjective qualification remain with #720–#722.

## Expedition Interface foundation (#826)

Import components from `src/ui/primitives.tsx` and wrap consumers in
`<div className="expedition-ui">`. No root/body/world selectors are installed.
Use `disabledReason` for an unavailable action, `pending` for work awaiting an
authoritative result, and `StatusLabel` for event-driven feedback. A controlled
`Dialog` requires `open`, `title`, `onDismiss` and children; it restores focus to
its opener. `EquipmentDialog` adds the mechanical accent and short interruptible
entry fade; optional `eyebrow`, `status`, `feedback` and `actions` slots share the
machine layout while children retain machine-specific controls. Native modal inertness and keyboard handling supplement the existing
view owner's shortcut/input-release gates. Widgets do not hold game state.

`/tests/ui.html` is a development-only bright-planet/dark-space component gallery;
it is absent from the production bundle. `npx playwright test --project=ui`
checks modal focus, blocked activation, units, reduced motion and deterministic
reference captures at 960/1280/1920 desktop widths. Set `CHROMIUM_PATH` if reusing a
system browser. Gallery captures require human visual review; they are not
automatically approved baselines. See [contract](../docs/hud-ui-style-guide.md#shared-foundation-implementation-826)
and [evidence](../reports/issue-826/README.md). Existing screens are not restyled.


### Shared panoramic styling (#828)

The production cockpit uses the Expedition Interface tokens and shared control,
utility and dialog presentation classes. Its native disabled controls, scanner,
optical camera, compass calculation and navigation owners retain their contracts.
`HUD-STYLE` in `tests/panoramic.spec.ts` checks grounded horizon and dark/dense
contacts, focused help, contrast and edge-to-edge geometry at 960×600, 1280×720
and 1920×1080. `HUD-BRIGHT` checks actual bright-star pixels and visible warning
summaries in focused help at the same sizes. Use `npm run test:browser -- --project=panoramic` for focused checks;
`LOOK`, `COCKPIT-1` and PANO-VIS retain command/lifecycle and visual coverage.
See [the delivery evidence](../reports/issue-828/README.md); new image candidates
remain reviewable evidence; there is no human baseline-approval CI gate.

## Cross-view Expedition Interface visual regression (#831)

The development gallery `/tests/ui.html` imports production tokens, primitives
and feedback components. `?context=planet` or `?context=space` isolates a desktop
state board: idle/active/hover/focus, disabled-with-reason, pending, confirmation,
error, warning, terminal and empty/full inventory. The query-free overview remains
scrollable. Fixture readings are illustrative; they cannot issue gameplay commands.

`npm run test:ui` uses the existing Playwright configuration, vendored fonts,
fixed clock, DPR 1 and strict unmasked pixel comparator at **960×600, 1280×720 and
1920×1080**. It covers the built `/` surface/gathering/inventory, critical modal,
terminal, cabin, container review/error (top and controls), grounded cockpit,
contact selection and return. Existing production sensor/bright-star composition
fixtures add unscanned/occluded/lost contact and bright-background warning frames;
these are fixture compositions, not new playable flight/optical mechanics.
Readouts are checked against domain-created prior-expedition saves and exact
transactions. Geometry assertions reject body scroll, clipped/horizontally
scrolling panels, HUD overlap and world interception at the player. Browser
keyboard checks exercise focus containment/return, input isolation, modal/utility
Escape ordering and grounded ship guards. The broader scanner, transactions,
reduced-motion and focus tests remain in their owning existing suites.

Run after `npm ci && npx playwright install chromium && npm run build`:

```sh
npm test
npm run test:ui
npm run test:ui:negative
# A fresh browser run, preserving the first run's originals:
SALIMON_EVIDENCE_DIR=test-results/ui-repeat npm run test:ui
```

`npm run test:browser` includes this project. The existing Navigation client CI
runs it, repeats it, and runs the two deliberate CSS mutations. Negative success
requires the **specific visual or focus assertion** to fail; setup errors, missing
references and timeouts are invalid demonstrations. Node tests also reject absent
manifest entries/files, corrupt hashes and changed pixels. Normal runs never
write reference images; absent references fail instead of bootstrapping a pass.

To propose intentional reference changes locally:

```sh
SALIMON_UI_CANDIDATES="$PWD/test-results/ui-candidates" npm run test:ui
```

Candidate generation is forbidden in CI and outside an absolute ignored
`client/test-results/` path. Review originals, repeats and metadata, then explicitly
copy the proposed PNGs and per-width manifests to `tests/expedition-baselines/` in
the reviewed change. Their `PENDING_OWNER_REVIEW` marker does not grant visual
approval; route intentional reference changes to the owner. References are test
fixtures, not completion reports. Existing navigation/panoramic references and
pixel tolerances are unchanged. No extra approval/status CI system is introduced.

Canonical scenario JSON includes tested SHA, source-input digest, viewport/DPR,
font hashes, clock, browser/backend, readouts, geometry and comparison hashes.
Original/expected/repeat/diff PNGs, negative failures and logs use ignored
`test-results/`; the existing CI artifact retains them for 14 days. Put delivery
results and limitations in the PR, not new committed `reports/` files. Synthetic
browser input and SwiftShader results do not qualify physical input, target
hardware FPS or native Metal rendering. No timing claim is inferred from captures.

### Shared Scout lifecycle (#838)

`ResourceSession.transitionScout(action, revision, cockpitSession, actorId)` admits explicit
`begin-takeoff` and `begin-landing` (`bodyId`, optional validated `surface`/`siteId`)
commands through the live powered owner seat. Both maneuvers complete automatically;
`finish-takeoff` and `finish-landing` pilot overrides are rejected. The caller must provide that exact
live cockpit-state token; an old seat token remains invalid after re-entry. These commands
start the saved domain trajectories; production Take off starts the launch profile.
Player-facing landing presentation is separate L07 work (#844).
The development-only `/?flightFixture=1` buttons call the same domain actions for
browser acceptance. They are removed from the production build.

Schema v11 persists ship pose/velocity/mode, surface address, visited site state,
active maneuver and scanner state in the resource transaction; v1–v10 migrate
losslessly. `ScoutLocation.body` and `siteId` identify the current/target physical region, while `binding` alone
indicates an active exterior. Airborne `frames.ship.parent === null` prevents
exterior conversion, reach, drawing and collision. Contact without a registered
site remains physically recovering and cannot silently open an exterior.

Run `npm test`, `npm run build`, and
`CHROMIUM_PATH=/usr/bin/chromium npm run test:browser -- --project=cockpit tests/flight.spec.ts tests/cockpit.spec.ts`.
New originals and traces stay in ignored `test-results/`; CI uploads that output.
Observation direction/FOV remain local to the active session, independent of pose.

### Fast PR checks and daily full regression

The existing **Navigation client / build** check keeps all domain tests and
build/typecheck, plus `npm run test:browser:smoke`: the built radar/WebGL check,
fixed-clock scanner check, complete flight journey, and built in-flight save
restore/inventory check. Its job limit is 15 minutes. Docker smoke stays in its
existing workflow. A passing smoke run does not qualify every full-suite scenario.

[Navigation client full regression](../.github/workflows/navigation-client-full.yml)
runs daily at 03:17 UTC on the default branch (GitHub may delay dispatch), or via
**Run workflow** on demand. The schedule starts after this workflow is merged to
the default branch. It runs domain/build checks once and shares the built client
with five independent browser jobs: world, navigation, perspective, visual, and
expedition. Every Playwright project runs once, plus the existing visual/UI repeat
and negative campaigns. Failed groups do not cancel others; remaining repeat and
negative steps still run after normal assertion failures. No pixel policy or
reference is changed by the split.

The isolated reporting job has `issues: write` and `actions: read`; test jobs keep
read-only repository access. A failed/timed-out group opens a `bug` issue with its
commit, ref, run/attempt, job logs and artifact link (14-day retention when produced).
Further failures comment on that group's existing open bug, preserving human
edits, labels and assignees. Same-attempt report retries are idempotent. Successful
runs stay quiet and never auto-close bugs. Reporting is restricted to this repo's
scheduled/manual full runs; PR jobs cannot write issues. No extra credentials or
external service is needed. Cancelling an entire workflow can prevent its reporter
from running; ordinary test failures/timeouts are handled by the downstream job.

CI automation contracts can be checked with
`node .github/scripts/navigation-ci.test.cjs` from the repository root. They cover
full-project coverage, retained checks, PR smoke selection, permissions, duplicate
bug handling and repository/event boundaries. After publishing a change, the owner
still tracks CI and brings results to the implementation chat.

### Aft boarding door and launch admission (#839)

Approach the aft threshold on either side and press **E** or click **Close door** /
**Open door**. The nearby label shows the current state, and the closed panel
blocks swept walking on both sides. Step clear of the threshold to close it.
The closed-state HUD remains visible at the cockpit; flight adds a lock indicator.
An open-top cutaway drawing does not imply an open exterior door.

The v11 resource save preserves this state (introduced in v10) atomically with player/resource/Scout
source. Grounded v1–v9 saves migrate open; airborne v9 saves migrate closed to
preserve the previous exit lock. Malformed/inconsistent saves fail safely and keep
their bytes. Landing preserves the door's closed state until explicitly opened.

`ResourceSession.transitionScout` admits launch only with the current revision,
a live enabled powered owner seat, a nonterminal eligible pilot and ship, a
validated supported grounded site, every occupant aboard, a closed aft door,
no active gathering/ship operation and no already-started maneuver. Rejections
explain the reason without moving the ship, advancing time or charging resources.
There is no automatic door closure. The development-only `?flightFixture=1`
exercises explicit lifecycle endpoints; production launch trajectories/controls
remain future work. Cockpit power is still a fixture, not a Core simulation.

Affected verification: `npm test`, `npm run build`, and
`npm run test:browser -- --project=cockpit tests/boarding-door.spec.ts tests/flight.spec.ts tests/cockpit.spec.ts tests/storage.spec.ts tests/gathering.spec.ts`.
The door test repeats the built `/` journey through real keyboard/mouse input,
retains original open/closed/occupied screenshots and unchanged-policy repeat
diffs. Flight tests cover visible open-door launch denial, closed airborne
interaction/traversal rejection, reload and return. Evidence goes to ignored
`test-results/` and the existing CI uploads (14-day retention); delivery summaries
belong in the PR. Existing visual references must not be silently rewritten.

### Production Scout takeoff and flight (#841)

On `/`, board the Scout, close its aft boarding door with E/Use, approach the
cockpit and sit. **Take off** displays the live admission reason until eligible.
Launch runs automatically for six seconds and cannot be cancelled by steering,
focus loss or cockpit exit. The compact HUD shows phase/progress. Once flying,
arrows steer the ship, Set direction aligns it, signed throttle selects thrust
and Space selects zero. WASD/left-drag affect only the independent camera.
Dialogs, text/control focus and unauthorized seats cannot issue flight shortcuts.

The world advances navigation exactly once per fixed tick; the mounted cockpit
only publishes held input. Exit cockpit returns to the ship-local 2D interior;
flight, scanner and selected throttle continue, while held steering is cleared.
Re-entry preserves trajectory and observation. The aft door remains locked in
flight pending EVA; no planetary exterior is presented. Landing is separate work.
The isolated `/?scene=radar` engineering fixture remains independently clocked.

Focused verification: `npm test`, `npm run build`, and
`npm run test:browser -- --project=cockpit flight.spec.ts cockpit.spec.ts playable-slice.spec.ts boarding-door.spec.ts`.
Original captures/traces and state records use ignored `test-results/`; existing
CI artifacts retain them for 14 days. Validation summaries belong in the PR.


### Deterministic surface sites (#842)

`surfaceSite.ts` resolves an explicit `SurfaceAddress { bodyId, radialM, yaw }`:
body-fixed metres from the body centre at `body.radiusKm * 1000 + 5` (the unchanged
physical hull bound), with yaw in radians in `[-π, π]`. Split galactic positions
are derived by translating the catalog body; camera/screen offsets never enter
identity. `resolveSurface` provides a planet-fixed tangent origin and orthonormal
east/north/up basis plus the ship-local planar origin and angle. Stable cubed-sphere
ratio cells of approximately 64 m use versioned `site:surface:v1:...` identities;
orientation changes the ship transform, not the region's saved world identity.
Axis ties/poles use deterministic rules. These are local illustrated patches,
not canonical geology, full-sphere walking, ephemerides or global terrain.

The original Vesper region is the +Y hemisphere with radial x within ±18 m and
z within ±14 m. Only that region resolves to `site:illustrated-landing:v1` and its
actual authored IDs. Other Vesper regions, Vesper Moon and Caligo contain one
sparse ground illustration, no mineral/biology sources, loot or detailed geography.
The validated physical 5 m bound, illustrated hull with player clearance, and
aft access apron must be clear. Unsupported bodies, off-surface coordinates,
body/site mismatches and obstructed footprints reject with a reason.

`ResourceSession.transitionScout({ kind: 'begin-landing', bodyId, surface }, ... )`
admits a validated target while retaining the detached interior. An optional
`siteId` must match; the legacy original `siteId` + `bodyId` action remains supported.
Without explicit coordinates the planner chooses the point below the ship in the
body-fixed frame. Automatic touchdown preserves cabin occupants and the sealed
door, then restores the matching saved local terrain. A collision/crash never
creates a traversable site or becomes landing admission.

The same localStorage key now writes schema v11. `scout.surface` retains the
body/coordinates/orientation in flight and across mid-launch/mid-landing restores.
`sites` stores visited region addresses and existing terrain anchors/removals.
Only trusted original entity definitions can be restored: no saved artwork,
collider or fabricated source definitions. Required ground/source IDs cannot be
removed. The single global gram ledger retains original source depletion, player
cargo, Scout stores and partial mining work; new sites never duplicate it or
permit remote extraction. Local terrain edits stay in their owning region.
Versions 1–10 migrate without changing original bytes until a successful write.
Missing/inconsistent mappings, stale writes and failed storage preserve old data.
Exterior entities are removed during flight and restored only after validation;
ship-local furniture, occupants and inventory persist without changing frames.

Verification: `npm test`, `npm run build`, and
`CHROMIUM_PATH=/usr/bin/chromium npm run test:browser -- --project=cockpit tests/surface-sites.spec.ts tests/flight.spec.ts tests/boarding-door.spec.ts`.
The surface journey uses production React/keyboard input, admitted landing-plan saves,
moon/Caligo exterior walking, launch/reload and a return to depleted authored
Vesper. Originals, repeat comparisons and canonical state JSON stay in ignored
`test-results/` and existing full-world CI artifacts (14-day retention). Existing
visual references and tolerances are unchanged; run the affected `UI-WORLD` case
in the `expedition` project to compare the original site/cabin/cockpit.

### Assisted landing domain (#843)

`landing.ts` owns deterministic powered-style prototype landing. Admission requires
free flight, a closed door, no other assist, a currently scanner-detected sampled
planet/moon, centre altitude within 0.15 body radii after the 5 m hull allowance,
and full surface-relative speed <=20 m/s (including tangential/spin velocity).
There is no heading gate. The live powered owner-seat/revision checks stay in
`ResourceSession`; saved terrain hull/apron clearance is checked before admission.
Low-speed safe contact remains physical recovery and cannot bypass this contract.

The saved `LandingPlan` captures the initial ship, target, start tick and dynamic
duration. A cubic Bezier in the rotating surface frame preserves initial position
and velocity and finishes with zero body-relative motion. Orientation follows a
bounded smooth shortest turn. Continuous convex-hull/tangent-plane clearance and
analytic velocity/acceleration bounds reject unsafe braking or interfering bodies
before any motion. The budget is <=100 m/s surface-relative speed, <=10 m/s² and
<=50 MN / loaded mass acceleration, with <=12 simulated hours and a site within
1 km of the point below the ship. The 930 km Vesper limit therefore takes hours,
not eight seconds. Conservative bounds can reject a technically possible path;
feedback asks the pilot to stop farther above a clear site and retry.

`NavigationRuntime` samples the plan on its sole 60 Hz clock before incidental
contact resolution, advances the same scanner, and publishes the validated
surface binding automatically. Steering, throttle, Set direction, duplicate/new
assists, cockpit exit, blur and cockpit power loss do not cancel admission.
The body-fixed address/region remains stable on rotating fixtures; grounded
physical position keeps following the moving surface with nonzero inertial
velocity. Catalog centres stay fixed: linear ephemerides, gravity, atmosphere
and true terrain are future adapters. The sample production bodies have no spin.
`LANDING_FUNDING` explicitly supplies unlimited prototype energy and the existing
1 N·s/J conversion seam; it neither reserves Core energy nor drains fuel.

Schema v11 adds optional `scout.landing` (missing means null for older non-landing
saves). Decode revalidates the plan and its exact sampled pose/velocity; malformed,
tampered or old endpoint-only landing snapshots fail safely instead of teleporting.
`landing.test.ts` covers boundaries, derivatives, large descent, partitions,
rotation, automatic contact-safe arrival and unsafe/terminal negatives.
`tests/flight.spec.ts` LANDING-843 and `tests/surface-sites.spec.ts` exercise
production domain admission, the sole tick, interruptions and built save/resume.
HUD/landing-command presentation is deferred to #844; no visual references change.
