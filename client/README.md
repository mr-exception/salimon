# Salimon browser prototypes

Use the [2D documentation migration register](../docs/2d-migration-register.md) to distinguish the current browser implementation, approved future mechanics and archived native source contracts.

**2D gathering (#757; weight-only revision):** The continuous Scout/planet browser world uses one **10 kg** weight cap, **12 shared inventory slots**, and **up to 1 kg/s integer-gram** sample gathering with approved survival penalties with atomic local saves. `resourceInventory.ts` also models count-based items with mass and per-type stack sizes (future content). `resourceSession.ts` saves a schema-v8 resource/item/survival/hazard ledger and migrates valid v1
litre / v2 gram saves without loss. `resourceLedger.ts` validates exact transfers
and store mass; future stores require trusted ID/ship/capacity declarations.
The default world enables the 50 kg Scout Resource Container and deposit/withdraw panel (#784/#785). See [resource design](../docs/resource-inventory-and-gathering.md).

The default client is a React + TypeScript + Vite 2D world with a Three.js /
React Three Fiber cockpit radar. Start with [current implementation and legacy
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
existing 3D radar. **Exit navigation · Stand up** returns standing beside the
chair in one action. Ship position, velocity, orientation, throttle and scanner
continue in the same `NavigationRuntime` while walking or viewing the radar.
The illustrative landed 2D ship pose stays fixed: radar sample motion does not
implement flight or move the landing site.

`cockpit.ts` owns one local chair reservation, owner-control admission and power
transitions. `TopDownView` keeps the 2D world mounted; only radar presentation is
mounted/unmounted. A seated avatar is drawn over its chair, in ship-local space.
Power loss closes radar and keeps the chair occupied; restoration stays seated
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
normal exit, outage/restoration, chair state and nonzero throttle/velocity across
view switches. It repeats original captures with the unchanged pixel policy.

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
presentation. No production state setters are exposed.

`tests/storage.spec.ts` uses the built default entry to mine → enter → deposit →
withdraw → exit → reload twice, compares original panel/error captures and
checks a validated prior full-container save, overweight/stale-save errors and a
narrow viewport. It is part of the existing cockpit project and full browser CI.
The player-slot feedback helper is tested with validated synthetic usage; the
current two-resource/empty-item catalog cannot fill all 12 slots in browser play.

For stacked pull requests, the baseline-approval CI job fetches the declared
parent SHA explicitly before comparing its manifest. GitHub's synthetic stacked
merge ref can place the actual parent outside a depth-2 checkout. Missing history
is repaired by fetching that exact commit; changed reference digests still require
the existing explicit human approval. The gate itself and reference bytes are
unchanged.

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
