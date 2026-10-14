# Runtime Architecture

## Responsibility

The runtime is the native host and composition root. It translates `winit`
lifecycle/window events into a small set of renderer operations and owns timing
consumed by current portable character, ship, world, and resource controllers.

```text
winit event loop
    -> salimon-client lifecycle + frame/update clocks + typed key mapping
        -> salimon-character movement / camera snapshot
        -> salimon-ship pose / interaction snapshot
        -> salimon-world static Solar System + camera/precision snapshot
        -> salimon-diagnostics aggregation / RGBA view
        -> runtime contextual action-bar state / RGBA view
        -> map snapshot -> salimon-renderer new / resize / render
            -> wgpu surface, camera-relative conversion, depth, and presentation
```

The dependency direction is one-way: the runtime depends on character, ship,
world, physics, diagnostics, and renderer. Supporting crates never call into runtime;
renderer and diagnostics do not depend on behavior domains. Runtime mapping
prevents portable types from acquiring `wgpu` or `winit` dependencies.

## Internal composition modules

`src/app.rs` owns `ClientApplication`, the `ApplicationHandler` lifecycle,
renderer recovery/redraw scheduling, automation dispatch and gameplay update
ordering. Its private `src/app/` modules hold cohesive translation helpers:

| Module | Responsibility |
| --- | --- |
| `input.rs` | Physical-key commands, initial-press/repeat gates and held movement/steering state |
| `interaction.rs` | Ship-local cockpit/door targeting, interior authority gate and contextual text selection |
| `frames.rs` | Ship/surface frame adapters and nearby solid-body selection |
| `scene.rs` | World spheres/markers/light and live ship instrument DTO mapping; absolute `f64` positions stay intact |
| `diagnostics.rs` | Body distances, gameplay/tour metrics, presented-frame measurements and metric log formatting |
| `native.rs` | Window attributes, drawable size and cursor grab/release operations |

These are internal composition helpers, not new domain or platform interfaces.
Pure helper regressions live beside their owners. Existing crate-visible frame
helpers remain available through `app` for automation/carrying consumers; domain
and renderer DTOs are unchanged. Renderer and diagnostic maps observe snapshots
without advancing controllers or taking ownership of authoritative state.

`advance_game` stays in `app.rs`: preserve ship/character/camera advancement,
nearby surface updates, mining, carried-object synchronization and loose-fragment
stepping in their existing order. Redraw maps snapshots after those updates and
records diagnostics only after a successful presentation. Automation continues
to run on the native event thread with explicit fixed steps and production gates.

## Lifecycle flow

1. On resume, create the native window and initialize its renderer when needed.
2. On a nonzero resize, update the renderer's drawable size. A zero-sized window
   is not configured or rendered.
3. Schedule redraws while the application has a live, drawable window, and use a
   short delayed retry when the presentation surface is temporarily unavailable.
4. Before each drawable render attempt, advance portable character/ship/camera
   state with the shared bounded tick scheduler below. Map the six catalog bodies to `f64` sphere/material
   DTOs, map Sun lighting, preserve separate marker cuboids, calculate camera-to-surface
   distances, and measure that real update work. In gameplay view, map the ship's
   pose, door state, and latest speed/thruster snapshot into `ShipMeshInstance`.
   Its `CockpitInstruments` is refreshed regardless of cockpit control authority,
   so the physical screens keep following autonomous ship changes after exit.
5. For each successfully presented redraw, record monotonic frame timing after
   the renderer submits and presents the frame, then combine it with renderer,
   update, and camera measurements for diagnostics.
6. Recover from surface loss through renderer reconstruction/reconfiguration;
   treat transient acquisition failures as nonfatal and report unrecoverable
   renderer failures before exiting.
7. On a close request, stop the event loop and release window/GPU state cleanly.

## Diagnostics flow

F2 toggles gameplay/precision-tour view and F3 toggles diagnostics. P toggles the camera fixture, R
restarts it, and N selects and pauses the exact near-surface inspection view;
releases and key-repeat events are ignored. The runtime maps native keys to typed
commands; in precision-tour view, 1–6 restart inspection of Sun, Mercury, Venus, Earth, Moon, and Mars.
It maps renderer/world measurements to diagnostics without sharing `wgpu` or
`winit` types. All six catalog names and nonnegative camera-to-sampled-surface (nominal in flat fixtures)
observations flow through `BodyDistance`; diagnostics selects the closest for its
single nearby-body row. Diagnostics owns aggregation and the RGBA panel, while
the renderer owns only generic image composition. Gameplay view supplies live
player/ship diagnostics; precision-tour view supplies its camera metrics.

## Contextual action-bar flow

The runtime field-maps ship-owned Core and nearby-body telemetry into renderer
instrument DTOs every gameplay frame; it never derives proximity or radial
velocity. Runtime selects one applicable prompt with the existing door/ship/resource
priority, and rasterizes its text in `action_bar.rs`. Cockpit and door anchors
use authored interaction markers transformed by the current ship frame;
resource anchors use the selected deposit or fragment's absolute position.
While carrying, drop guidance follows the carried object, even when another
fragment is aimed at. `resource_context::Prompt` pairs text with typed renderer
placement. State-derived prompts disappear when eligibility changes.

The renderer receives borrowed RGBA pixels and `OverlayPlacement::World`
(absolute `f64` center and conservative visibility radius), or screen placement
for global flight/tool guidance. Renderer owns projection, fitting and depth
visibility; runtime never rebases or projects an anchor. Immediate blocked-door
feedback overrides the current prompt at bottom center for three seconds and
then expires without changing ship state. Precision tour hides gameplay prompts.

## Evolution

An explicit `--e2e` launch selects a fixed update duration and one known initial
scenario before the event loop starts. The runtime composes existing character,
ship, and immutable catalog values; subsequent frames and interactions use the
same production controllers. The ready signal follows successful renderer setup.
Only E2E mode starts a stdin JSON reader. It passes requests to the native event
thread, where held input, look, interaction, thruster, landing, and explicit fixed
steps use the gameplay controllers. Rendering never advances E2E simulation;
inspection reads domain snapshots without mutating them.

Diagnostics remains observational and non-authoritative. When additional native
targets or platform services appear, move OS-specific policy behind
`client/platform/` without moving portable timing or client orchestration out of
the runtime.

## Mining composition

`mining.rs` translates equip/hold state plus character camera/ship obstruction
into portable `salimon_world::mining` calls. World owns target selection, rate,
validated deposit mutation, and session mass deltas. Character exposes ray hits
against its solid ship proxies. Runtime queries nearby generation, applies
world session state for both GPU mapping and inspection, and presents the authored tool through `MiningTool::held_item` / `HeldItemInstance`
and resource geometry through generic renderer DTOs without exposing gameplay
types to the renderer. No procedural tool or aim cuboid remains.
Physical fragments are world-owned session entities. Runtime maps their
mass-derived presentation size and absolute pose to generic presentation DTOs; the same
nearby query supplies automation state even when the tool is stowed.

## Physical surface/ship transfer (#47)

`carrying.rs` composes surface and interior targeting/placement with the shared
character collision layout. `MiningTool.ship_fragments` holds supporting ship-local
coordinates keyed by existing physical fragment IDs, never inventory quantities.
Successful interior release installs an anchor; pickup removes it before following
the player; surface release stays world-local. Every gameplay update and look/input
synchronization maps loose anchors through the current ship frame. The same world
session owns every entity and its immutable mass/material/source throughout.
`fragment_physics.rs` adapts loose session fragments to `salimon-physics` in
ship-local or absolute planet coordinates. It supplies selected body geometry,
character-owned floor containment, stable object order and mass-derived scale/radius plus cached authored convex geometry,
then writes velocity/pose results through existing IDs. Carried pieces are
excluded. Gravity, release/ejection, restitution, deck/hull response, convex
contacts, angular motion and substeps live in the portable physics crate. World retains identity,
material, mass, orientation and validated poses; runtime retains cross-domain
frame/session sequencing. See the canonical
[ownership decision](../../docs/technical-architecture.md#physical-object-simulation-decision-114).

## First-person reticle

`reticle.rs` selects a static outlined dot or `+` RGBA image directly from
`EquipmentToolbar::mining_equipped()` each gameplay redraw. Precision-tour view hides it. The
renderer composites it independently of diagnostics/action text at the drawable
center, without scene depth or a world-space aim cuboid. This center is NDC
(0, 0), matching the existing eye-to-look-target interaction/mining ray; target
selection and extraction remain unchanged. Equip/stow changes the image revision
so the existing overlay texture cache uploads only on state changes.

## Authored fragment presentation

`resource_presentation::fragment_mesh` selects one of two Blender exports by
stable fragment ID parity and supplies the world-owned position/physical side.
All materials emit authored meshes without procedural fragment cuboids. Iron
uses the evolved original chunk and a taller shard; silicate uses slab/ridge
variants. Each follows the same stable identity and authoritative-size rules. The world pose quaternion drives the rendered rotation;
carried, dropped, ship-local and streamed pieces use the same identity mapping.
World mass/volume and one-object carrying remain unchanged; #148 adds orientation-aware convex contacts.

## Authored water-ice deposits (#88)

Four opaque Blender-authored spire/crown/ridge/shelf exports use the shared
resource mesh batch. Runtime selects `DepositId.local % 4` in that order and
emits no water-ice cuboid. Depleted deposits emit no authored visual. Baked
coordinates fit ±0.48 m; uniform scaling by `bounds_radius_meters /
(0.48 * sqrt(3))` inscribes the visual cube in the authoritative spherical bound
at its absolute deposit center, at every body/latitude. Variant choice ignores
query order, camera, streaming and remaining mass. World generation, mining,
mass, session persistence and streaming rules are unchanged. See the
[asset guide](../../models/assets/resources/water-ice-deposit-spire/README.md).

## Authored silicate deposits (#87)

Four opaque Blender-authored boulder/slab/ridge/scree exports share the resource mesh
batch. Runtime selects `DepositId.local % 4` in that order and emits no silicate
cuboid. Depletion hides the mesh. The centered ±0.48 m baked cube uses uniform
`bounds_radius_meters / (0.48 * sqrt(3))` scaling inside the authoritative sphere.
Identity, mining, mass, session persistence and streaming stay world-owned.
See the [asset guide](../../models/assets/resources/silicate-deposit-boulder/README.md).

## Authored iron deposits (#86)

Four opaque Blender-authored nodule/vein/ledge/rubble exports complete deposit
migration to the shared resource mesh batch. Runtime chooses `DepositId.local % 4`
in that order. All resources now emit authored meshes; no deposit cuboid path
remains. Depletion hides the visual. The centered ±0.48 m baked cube uses uniform
`bounds_radius_meters / (0.48 * sqrt(3))` scaling inside the authoritative sphere.
Identity, targeting, mining, mass, session persistence and streaming stay world-owned.
Automation retains the legacy `visual` field as null and reports mesh/center/scale
through `authored_visual`. See the [asset guide](../../models/assets/resources/iron-deposit-nodule/README.md).

## Shared vector primitives

Compatible `f64` component arithmetic comes from the dependency-free
`salimon-math` leaf crate. Normalization, frame and quaternion policies stay
with their owning domain. See the canonical
[decision and inventory](../../docs/technical-architecture.md#shared-math-decision-115).

## Physical interaction input (#126)

E routes only cockpit/door interactions. `automation_key` shares the native F
route: an initial press grabs an aimed reachable object or drops the carried
object; a miss never starts mining. MiningTool latches F until release to prevent
repeat grab/drop. Only left mouse controls `held`; releasing F never changes it.
Lifecycle resets and stowing clear mining input and the latch. Automation
`grab_drop` plus legacy `pickup`/`drop` names map to F. The old `mine` key alias
is rejected. Automation `mouse` with `button:left` and boolean `pressed` shares
`mining_mouse_input` with native left-button events, preserving gameplay,
equipment and carrying gates; native events additionally require cursor capture.

## Equipment toolbar state (#128)

`equipment.rs` owns a portable five-slot loadout and optional typed selection,
composed by `ClientApplication.equipment`. Slot 1 contains `MiningTool`; slots
2–5 are empty. Initial selection is absent. Gameplay numeric keys 1–5 select
one slot, including empty slots, without toggling the current selection off.
Native selection accepts only captured-cursor, nonsynthetic initial presses.
The same mapping is used by automation. Numeric camera inspection commands
only apply in precision-tour view; P/R/N retain their engineering shortcuts.

Successful physical pickup immediately clears selection. Selection reads the
world session's carried identity and is refused while it is occupied; release
never restores a previous selection. Renderer state does not own the loadout.
`EquipmentToolbar::mining_equipped()` derives active equipment from the selected
slot and its contents. Mining target/extraction, held mesh, reticle and prompts
read this decision; MiningTool stores no equipped boolean. Slot changes cancel
held mining; pickup clears selection and held input while preserving the consumed
F latch. M and automation `equip_mining_tool` are removed; scenarios explicitly
select `slot_1` to equip and empty slots to stow.
Automation inspection exposes `equipment.slots` (tool names/null) and
`equipment.selected_slot` (1–5/null).

## Equipment toolbar presentation (#129)

`EquipmentToolbar::presentation` field-maps runtime loadout/selection to typed
renderer-neutral icons/slots. Each gameplay redraw sends that DTO plus the native
window scale factor; precision tour sends no toolbar. The renderer owns raster,
cache and bottom-center stacking with global/transient action images. Runtime
never draws this HUD in `action_bar.rs` or changes selection for display.

## Carrying and equipment regression contract (#131)

A failed pickup leaves the selected slot and held mining input unchanged. A
successful pickup stows the tool and cancels extraction in the same F edge;
all five slot inputs stay blocked until release. Drop leaves no selection and
no held mining input. An explicit slot 1 selection equips again without resuming
an old mouse hold. The toolbar remains visible with its usual five
slots and no highlight while carrying; carrying never creates a sixth slot.

Automation inspection exposes `mining.held_item_visible` from the shared `ClientApplication::held_item` DTO helper used by redraw,
including the same view/location/equipment presentation gates, so carrying scenarios
check the absence of the held tool as well as selection/extraction state.
Baseline and evidence carrying/resource-loop scenarios cover an F miss before
extraction, equipped pickup, lockout and explicit selection after drop.

## Toolbar scenario coverage (#132)

The carrying scenario executes production numeric/F/mouse routes, including all four
empty-slot mining attempts at a reachable surface deposit. Its synchronized
evidence variant adds equipped/empty-selected checkpoints to carrying and
post-drop screenshots. Existing inspection fields establish loadout, selection,
held mesh, mining eligibility and conserved output; no test-only gameplay
mutation or new protocol action is needed. See [runner coverage](../../scripts/README.md#toolbar-gameplay-coverage-132).

## Enlarged resource geometry (#144)

The world size policy scales fragments/deposits by 10 in each dimension without
changing mass. `resource_presentation` lifts generated deposit centers using
renderer-neutral CPU vertex support from the selected immutable mesh. The
fragment adapter sends conservative broad-phase radius, current mass/scale/orientation and an immutable authored convex hull to physics. Carrying clears the entire cube around the player and rejects
ship floor drops whose enlarged footprint crosses hull/furniture proxies.

## Authored convex fragment contacts (#148)

`fragment_physics` caches six convex envelopes from renderer-neutral CPU vertices,
using the same stable variant selection as presentation. Every snapshot refreshes
scale and mass, preserving growing piece identity. Planet objects use absolute
world poses; ship objects use local orientation/angular velocity, mapped through
the current ship basis at synchronization and writeback. Carried objects remain
excluded and release resets angular motion. Physics owns all response rules.
Rendering and oriented pickup cubes consume the same world quaternion; hand
support is queried on the rotated geometry. Circumscribed spheres remain only
broad-phase and conservative player/hull clearance.

The opt-in `fragment-pile` initial fixture (seed 0 cabin, seed 1 surface) creates
six 2 kg pieces through normal extraction, then releases them from deterministic
initial poses. Subsequent fixed steps use production controllers and contacts;
no protocol operation teleports objects. Baseline/evidence scenarios compare
settled poses across two checkpoints.

## Surface emission (#150)

`mining_emission.rs` adapts the selected authored deposit triangle positions into
exposed facet contacts and full-growth fragment clearance. The ray contact is
preferred; nearby eye-facing supporting facet centers provide deterministic
alternatives when crowded. World still splits/allocates output, invoking this
adapter only for new IDs. A rejected placement pauses extraction without pending
mass. Normal convex physics then supplies motion; runtime installs
`surface_ejection_velocity` only in the successful creation callback. Missing
motion defaults to rest rather than inferring newness from map membership.
See the canonical [surface emission contract](../world/resource-contracts.md#surface-emission-150).

## Automation command deadlines

The ready event advertises the ordinary 5-second command deadline. Valid
protocol-1 `step` requests of 1–600 frames receive a bounded 120-second
server deadline: even one update can rebuild a large loaded contact island. The stdin
reader and native request expiry use the same command-specific duration.
Malformed/out-of-range requests and ordinary inputs/inspection stay at 5 seconds.
Scenario overall deadlines remain independent; no fixed steps are skipped.

## Fragment load benchmarks (#155)

`benchmark_fixture.rs` creates bounded opt-in test populations through normal
source extraction, validates mass/IDs/deck containment and selects mixed authored
variants and sizes. It changes initial conditions only. `--e2e --fragment-count`
and optional `--fragment-layout` extend scenario setup; all subsequent fixed
steps/inputs retain production controllers and gates. Normal launches reject
these options. The 1,000 limit belongs to the fixture, not gameplay cargo.

`benchmark.rs` owns benchmark launch parsing, accounting, bounded sample storage
and JSON output. `--benchmark` native runs keep the ordinary monotonic update
clock and renderer; they never enable stdin automation. Physics-only runs time
production fragment adapters at 16ms simulation deltas with no window, renderer
or FPS claim. Samples record actual moving-object counts; a requested settled
fixture does not prove a settled sample. Persistent awake/sleeping counters are supplied by the physics activation cache.
Optional `--settle-steps` prepares requested settled fixtures through the same
production adapter at 16 ms simulation deltas before renderer startup. It stops
only on full persistent sleep and fails at its explicit limit otherwise; it
never installs sleep flags or settled poses. Preparation steps/simulation/wall
duration are logged separately from measured native warmup and samples. Active
initial conditions reject preparation. The matrix runner rejects prepared cases
that wake during their sample window.
Settled `--repeats` resets independent bounded sampling windows on that same
population and validates accounting each time; active repeats use fresh processes.
Native `--camera both` switches after the facing repeats and independently warms
all away windows. Optional stdin preparation barriers permit concurrent fixture
preparation followed by isolated native recording; no window/clocks start until
the explicit start marker. Optional baseline replays use the canonical adapter
snapshots and cloned state/cache before recording, without changing live cargo.
Native benchmark windows retain normal focus/cursor capture but ignore gameplay
mouse/keyboard input. Their titles identify count, layout, requested state, camera and
repeat. Native timing and production updates remain unchanged. Recorded renderer
metadata includes DPI scale. Initial occlusion before any retained frame interval
restarts the full warm-up. Occlusion during measured sampling or changed surface
configuration after renderer initialization fails; closing before completion also fails.
See the benchmark guide for limits, counter meanings and CPU-only interpretation.
Physics #157 changes pair visits to spatial candidates and the legacy matrix
allocation metric to sparse-table growth; historical matrix capacities differ.
Optional `--minimum-samples` requires both elapsed sampling duration and a retained
sample count before a window completes. Native mode counts completed frame
intervals; CPU mode counts updates. Actual sample wall time is reported, including
extensions for expensive production updates. The process deadline remains the
outer bound. This does not change simulation delta or drive artificial motion.
E2E setup's `fragment_settle_steps` uses the same fixture preparation before
readiness. Inspection reads cache activation by stable identity, excluding
carried pieces, through aggregate/per-fragment sleeping fields. Preparation is
initial-condition work; later interactions retain production gates and stepping.

Frame intervals use a separate uncapped monotonic clock, preserving long spikes
hidden by rolling diagnostics' 250ms display clamp. Native gameplay uses the shared bounded tick scheduler below; simulation delta
is the sum of completed 16 ms ticks and backlog is reported separately. Samples are retained in memory and written once at
completion; high-frequency CPU runs decimate deterministically to bounded storage
and report stride/observed count. Native early closure must produce no successful
benchmark result. See the [benchmark guide](../../scripts/FRAGMENT_BENCHMARKS.md).


## Fragment activation adapter (#156)

`MiningTool.fragment_sleep` owns the in-memory `SleepTracker<FragmentId>` lifetime.
The adapter supplies stable IDs alongside current frame-local snapshots; selection
uses current-frame world positions for ship anchors even before final pose
synchronization, so high-speed flight cannot omit last frame's trailing cargo.
Selection omission (carrying or distant streaming) invalidates cached support, including
an entirely empty selection. Return starts awake without changing session IDs,
mass or local anchors. Growth/release/frame transfer changes snapshot fields and
wakes old contact neighbors. Ship-local rigid motion leaves sleep intact; world
presentation still synchronizes every sleeping pose/orientation through f64
frame conversion. Current interior gravity abstracts ship acceleration, so
flight itself introduces no new local force. Future force/support policies must
change snapshots or explicitly invalidate activation.

Physics owns thresholds, contact support and waking. Runtime/benchmark map
awake/sleeping/integrated counters; renderer and interaction continue consuming
the complete session, including sleeping pieces. No cargo cap/deletion or disk
persistence is introduced. See the [sleep contract](../physics/architecture.md#persistent-contact-island-sleep-156).

## Bounded gameplay ticks and fragment interpolation (#159)

`update_clock` retains every observed monotonic interval during drawable gameplay
in a `Duration` backlog. Redraw runs at most two **16 ms** `advance_game` calls,
with a **4 ms** catch-up admission budget checked after the first tick. A single
contact solve is not preemptible; the budget is not a hard wall-time guarantee.
Camera tour, ship, character, mining, fragment physics and action expiry all
receive the same completed tick durations. No solver-only time clamp or enlarged
catch-up tick is used. Native input takes effect on the next tick using current
held state; backlog is elapsed-time debt, not a recorded input event replay.

Overload retains debt and slows the entire gameplay simulation relative to wall
time. Recovery drains it in bounded batches. A long unreported stall is no longer
silently clamped to 250 ms. Lifecycle resets (occlusion, suspension, zero drawable,
renderer reconstruction) deliberately exclude the gap and clear already queued
debt plus interpolation history; cleared debt is added to `lifecycle_excluded`.
Unobserved lifecycle gap duration is not represented as gameplay debt. Saturating
`Duration` arithmetic bounds representation, not a gameplay cargo/time cap.
Automation still advances the exact requested fixed delta directly, and renders
current authoritative state; it never uses native backlog or interpolation.

Before each native tick, runtime captures selected loose fragment poses in their
simulation frames. Presentation blends previous/current centers in f64 and uses
normalized shortest-sign quaternion interpolation. Ship-local history maps
through the current ship frame, avoiding interpolation across rigid world-space
ship travel. The alpha is backlog / tick, clamped to 1 under overload; using a
modulo remainder would produce a backward sawtooth. New/growing/frame-transferred,
carried and just-released objects bypass incompatible history. Release invalidates
history immediately; held/drop/pickup authority and interaction queries never use
interpolated poses. Interpolation adds up to one tick of loose-object visual delay;
native visual quality is a separate graphical gate. Ship/camera/controller poses
remain authoritative; this change does not smooth all native controller motion.

Native benchmark samples sum solver work/time across all completed ticks in a
render and retain the final activation counts. Zero-tick renders record zero new
work while retaining the last activation observation. Samples include `game_ticks`,
`game_backlog_ms` and cumulative `lifecycle_excluded_ms`; CPU-only samples have no
native scheduler observation. Periodic frame debug logs include ticks/backlog.

## Bottom-hinged boarding ramp (#169)

The [authored ramp contract](../../models/assets/ships/salimon-scout/README.md#boarding-ramp-169)
owns geometry and generated dimensions. Runtime enables traversal only when the
700 ms animation is fully deployed, supplies movement-envelope collision and
rejects E while the player's body occupies the sweep. Ship's
`toggle_door_with_clearance` rejects an obstructed toggle without reversing the
target state; takeoff retains its closed-state/zero-fraction interlock.
Open-space EVA retains the same flight permission and inherited drift rules;
there is no planetary support on the lowered door in space. Renderer tags the
leaf and tray separately, using one uniform for phased lowering/extension in
the existing opaque batch. Character contact follows the complete sloping
surface, supports a stopped or carrying walker, and joins radial ground at its tip.

## Terrain integration (#179–#181)

Follow the canonical [integration/budgets/validation guide](../../docs/terrain-validation.md)
and [world definition contract](../world/terrain-contract.md). Keep local rules
and their consumers aligned; flat deterministic fixtures are explicit.

## Shared elapsed-time admission (#185)

Native redraws and E2E `step` commands admit elapsed time to `UpdateClock`'s
16 ms accumulator. Native catch-up retains its two-tick/4 ms budget and keeps
all remaining backlog. Automation drains admitted whole ticks deterministically
and retains fractional time across commands. Every completed tick advances ship,
character, mining, fragments and feedback in the same order. Lifecycle reset
explicitly pauses across the gap and accounts for queued time as excluded.
Ship updates no longer truncate direct travel to 100 ms; character callers
with longer intervals use bounded slices rather than silently losing time.

The `coasting-space` E2E fixture explicitly seeds 25,000 m/s world +X momentum
and zero throttle for moving EVA. `open-space` continues to start at rest.
Selected throttle no longer instantly assigns speed.
The `eva-approach` fixture seeds -25,000 m/s world Y momentum at zero
throttle for deterministic nearby-body influence crossings.
`coasting-orbit` seeds the same +X momentum near Earth for the existing loaded
cargo sleep/assist acceptance route, retaining its high-speed coverage.

## Held throttle routing (#186)

Native Up/Down and E2E `key` values `thruster_up`/`thruster_down` share the typed
ship edge API. Synthetic press replay and OS repeats are ignored, releases always
clear holds. Focus/cursor loss, view changes and lifecycle clock resets clear
held throttle. The existing `thruster` command remains a gated one-point action
and reports whether the powered flight authority accepted it. Renderer DTOs and
the cached cockpit atlas retain signed percentages and show reverse magnitude.
