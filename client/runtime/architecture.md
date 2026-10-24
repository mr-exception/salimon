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
renderer recovery/redraw scheduling, native admission/budgets and evidence IO.
It contains one `GameplaySession`; portable composition does not contain a window,
renderer, automation reader or benchmark recorder.

| Module | Responsibility |
| --- | --- |
| `session.rs` | Typed `Command`/`Snapshot`, gameplay state, held controls, interactions and portable tick order |
| `session/frames.rs` | Ship/surface frame adapters and nearby solid-body selection |
| `session/interaction.rs` | Ship-local target/range/aim and interior authority gates |
| `app/input.rs` | Physical-key translation and native focus/capture/repeat/synthetic admission |
| `app/interaction.rs` | Contextual text and prompt-placement mapping |
| `app/scene.rs` | World/ship renderer DTO mapping at absolute `f64` positions |
| `app/diagnostics.rs` | Observational metrics, frame measurements and formatting |
| `app/native.rs` | Window attributes, drawable size and cursor operations |
| `mining.rs` | `ResourceGameplay`: world mining session, fragment motion/sleep/history/scratch, ship anchors and cargo ledger; a separate `MiningTool` stores only input latches |

`GameplaySession::advance_game` preserves the existing sequence: presentation
history, precision camera, ship steering/advance, power gates, character nearby
surface/movement/nearby surface, mining, loose physics, then carried-object/anchor
and mass synchronization. The native shell advances transient action-bar expiry
by the same completed simulation duration. Commands return transient feedback via
a consumed observation; only the shell rasterizes/expires text.

Native scheduling still admits elapsed time to the same session-owned
`UpdateClock`, permits at least one tick, then enforces the existing tick-count
and wall-clock catch-up budgets. `advance_automation` drains the identical fixed
ticks without a wall-clock budget and retains the fractional backlog. Lifecycle
resets clear held controls/history and exclude backlog exactly as before.

`automation::execute_session` and `inspect_session` run against the production
session without constructing `ClientApplication`. Native `execute` adds window,
capture, upload and performance observations/operations to that same route.
The deliberate automation desktop-admission bypass remains explicit; portable
view, carrying, cockpit, power and assist gates still apply. E2E and benchmark
initializers take `GameplaySession` directly. Headless benchmark sampling advances
the whole production tick and records solver/adapter costs separately, rather
than running a second fragment-only simulation. Session-only regressions and
existing declarative routes cover cross-domain behavior; OS delivery and native
visuals/FPS remain separate #312/#313 verification.

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

Opt-in native benchmarks observe real winit focus/visibility/minimization and
occlusion before admitting warmup. Lifecycle loss resets pre-sampling clocks or
fails an already sampled window. Their PID and monotonic transition evidence is
required by the Python summary/comparison gate; see the canonical
[benchmark protocol](../../scripts/FRAGMENT_BENCHMARKS.md#native-wall-clock-matrix).
Successful present-call intervals remain distinct from physical scanout.

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

All contextual world prompts, screen-space flight guidance and transient action
feedback use the shared action-bar bitmap density: 2 pixels per glyph cell at
1× UI scale, rounded to integer density and capped at 3 (1.5× and higher).
Seven-cell glyphs are 14/21/21 physical pixels tall at 1×/1.5×/2×, respectively;
single-line panels are 26/39/39 pixels tall. The high-DPI cap keeps contextual
labels compact; integer rasterization preserves sharp edges. The font/palette is now shared with physical instruments (#466); anchors,
visibility and expiry retain their existing owners. This sizing remains independent of cockpit
displays, diagnostics, equipment-toolbar artwork and the reticle.

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
world session state for both GPU mapping and inspection, and presents the authored tool through `ResourceGameplay::held_item` / `HeldItemInstance`
and resource geometry through generic renderer DTOs without exposing gameplay
types to the renderer. No procedural tool or aim cuboid remains.
Physical fragments are world-owned session entities. Runtime maps their
mass-derived presentation size and absolute pose to generic presentation DTOs; the same
nearby query supplies automation state even when the tool is stowed.

## Physical surface/ship transfer (#47)

`carrying.rs` composes surface and interior targeting/placement with the shared
character collision layout. `ResourceGameplay.ship_fragments` holds supporting ship-local
`salimon_character::ShipLocalPoint` coordinates keyed by existing physical
fragment IDs, never inventory quantities. Typed project/unproject boundaries and
explicit solver/JSON array adapters prevent accidental absolute-position storage.
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
The unequipped dot replicates each original pixel as a centered 3×3 block:
white diameter 9 pixels (previously 3), outlined diameter 15 (previously 5).
Both images keep the same 17×17 physical-pixel canvas and unchanged center;
the equipped cross is unchanged. This deliberately retains the existing
fixed physical-pixel reticle policy independently of prompt/toolbar DPI scale.

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

## Gameplay input dispatch (#302)

`app/input.rs::InputPolicy` is the canonical press-admission policy. Native
initial presses require focus and a non-occluded window; gameplay additionally
requires capture and Gameplay view. OS repeats and synthetic focus-replay presses
are ignored for all keys. F2/F3 and engineering-camera commands remain global
view/diagnostic controls; tour numbers address bodies, never toolbar slots.
Release edges clear both movement/steering meanings and F/throttle latches even
after a view, location or capture change. Left-mouse mining and mouse look use
the same native gameplay admission conditions.

For dedicated native input diagnosis, `RUST_LOG=info,salimon_client::app=trace`
records received mouse deltas with capture/focus/occlusion/isolation admission
state, key edges with ship controls/orientation, and capture release. Focus
transitions are logged at debug level. These observations do not change admission,
controller state or timing. Posting an OS event alone is not proof of delivery;
compare this boundary trace with native state changes, as in the
[#453 report](../../reports/issue-453/README.md).

Native key translation and automation call `dispatch_key`. The explicit
`InputSource::Automation` harness policy bypasses desktop focus/capture/occlusion,
while retaining view, equipment, cockpit, power and assist authority. Protocol
`interact` and `landing` use the E/L dispatch route and report `applied: false`
in PrecisionTour; this is admission, not proof that a domain interaction succeeded.
The legacy one-point `thruster` operation also requires Gameplay view.

Escape/capture loss, focus loss, view changes and lifecycle clock resets use
`clear_held_controls`: movement, steering target/ramp, F/mining and held throttle
are discarded, preserving persistent throttle selection. Occlusion, suspension,
resize/minimize and renderer recovery retain their existing clock-reset policy.
Production commands are covered portably; real OS-input verification remains
[#312](https://github.com/mr-exception/salimon/issues/312).

## Physical interaction input (#126)

E routes cockpit/door and mechanical Core interactions. `automation_key` shares the native F
route: an initial press grabs an aimed reachable object or drops the carried
object; a miss never starts mining. The MiningTool input state latches F until release to prevent
repeat grab/drop. Only left mouse controls `held`; releasing F never changes it.
Lifecycle resets and stowing clear mining input and the latch. Automation
`grab_drop` plus legacy `pickup`/`drop` names map to F. The old `mine` key alias
is rejected. Automation `mouse` with `button:left` and boolean `pressed` shares
`mining_mouse_input` with native left-button events, preserving gameplay,
equipment and carrying gates; native events additionally require cursor capture.

## Equipment toolbar state (#128)

`equipment.rs` owns a portable five-slot loadout and optional typed selection,
composed by `GameplaySession.equipment`. Slot 1 contains `MiningTool`; slots
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

Automation inspection exposes `mining.held_item_visible` from the shared `GameplaySession::held_item` DTO helper used by redraw,
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
See the canonical [surface emission contract](../world/resource-contracts.md#legacy-facet-emission-150).

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

`ResourceGameplay.fragment_sleep` owns the in-memory `SleepTracker<FragmentId>` lifetime.
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
silently clamped to 250 ms. Lifecycle resets (OS suspension and renderer reconstruction) deliberately exclude the gap and clear already queued
debt plus interpolation history; cleared debt is added to `lifecycle_excluded`.
Occlusion/zero drawable instead clear held controls and presentation history,
then wake the same bounded scheduler every 16 ms without rendering. Focus loss
also continues gameplay. Unobserved OS lifecycle gap duration is not represented as gameplay debt. Saturating
`Duration` arithmetic bounds representation, not a gameplay cargo/time cap.
Automation still advances the exact requested fixed delta directly, and renders
current authoritative state; it never uses native backlog or interpolation.

Before each native tick, runtime captures ship/character snapshots and selected
loose-fragment poses. `session/presentation.rs` keeps the last completed pair;
rendering only reads it. One alpha (backlog / 16 ms, clamped to 1 under overload)
drives ship, character and fragment presentation. Centers blend in `f64` and
orientations use normalized shortest-sign quaternion interpolation. Interior
camera positions blend in ship-local space and map through the interpolated
ship pose, as do loose ship-local fragments. Exterior camera/fragment positions
blend in world space. Immediate live look direction is retained, transformed
through the render ship frame for interior views. Door fraction interpolates;
instrument/telemetry values always come from the current snapshot.

Initialization, lifecycle/view resets, cockpit entry/exit, doorway/EVA/frame
transitions and flight-state transitions bypass history for the entire scene.
Doorway blending uses current poses throughout. Out-of-tick ship/character pose
changes (including automation fixture teleports) invalidate sampling. New,
growing, frame-transferred, carried and just-released fragments bypass their
incompatible object history, while retaining their current render-frame
attachment. Removed entities are never retained in presentation. Carried objects,
growing previews and held tools follow the presented camera with immediate look.
Ship and fragment prompt eligibility still reads authority; only their anchors
follow the presented geometry (fragment anchors retain stable IDs).

Interaction, mining, physics, cargo and diagnostics authority never consume
interpolated poses. Automation disables all gameplay interpolation, so inspection
and image fixtures stay exact and deterministic. Rendering does not advance the
simulation. Interpolation adds up to one tick of positional/ship-orientation
visual delay; mouse look remains immediate. Portable synthetic 60/120/144 Hz,
zero/multiple tick, fractional/overloaded backlog, distant-anchor and frame-reset
regressions do not establish native smoothness or input latency. Native signoff
requires normal-clock evidence and explicit human review; see the
[#477 report](../../reports/issue-477/README.md).

Native benchmark samples sum solver work/time across all completed ticks in a
render and retain the final activation counts. Zero-tick renders record zero new
work while retaining the last activation observation. Samples include `game_ticks`,
`game_backlog_ms` and cumulative `lifecycle_excluded_ms`; CPU-only samples have no
native scheduler observation. Periodic frame debug logs include ticks/backlog. Opt-in
`RUST_LOG=info,salimon_client::frame=trace` records every presented-frame sample
for matched reference measurements (#483); ordinary debug cadence remains one
in 300 frames. Logging is observational and its IO overhead is present in both
matched variants; it does not alter clocks, input, domain state or GPU readback.

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

`Bouncing` and `GroundedAfterCrash` map their selected solid body's surface frame
and ship-owned velocity/telemetry. They do not satisfy ordinary `Landed` ramp
or takeoff gates. Automation inspection retains the distinct flight-state names.

## Physical mass accounting (#187)

Ship's `ShipMass` caches 100,000 kg dry hull plus positive finite physical masses
by stable `MassObjectId` (separate fragment/equipment namespaces). Upsert/removal
is idempotent; handling and storage do not add another identity. Installed
components can use the same ledger; the current asset has no separately defined
installed-equipment masses beyond dry hull. No inventory/container implementation
or invented component weights is added. `reconciles` is an explicit diagnostic,
not flight work. `set_total_mass_kg` preserves the impulse coefficient and velocity.

World coalesces creation/growth, pickup, carried/loose pose changes and physical
removal into an ordered mutation journal. `remove_fragment` is the future Core
consumption/despawn boundary and never reuses IDs. Streaming/read-only queries
emit nothing. Runtime drains changes after carrying/physics synchronization,
classifies centers in the current authored cabin/nose frame, and updates cached
mass for the next ship step. Door animation/furniture/handling do not change
containment. Existing physics visits and anchored frame writebacks provide pose
events even for sleeping contents; no extra full scene scan computes mass each
tick. Explicit scene reconciliation is available for validation. Full inertial
cabin motion, arbitrary future physical items and Core intake are separate tasks.

## Core power integration (#188)

Runtime maps `ShipSnapshot::power_available` to character artificial gravity,
fragment floor power and renderer `CockpitInstruments::power_off`. Physical
cockpit occupancy is independent of power. No runtime energy arithmetic is added.

## Mechanical Core intake and conversion (#189)

Normal E targeting within 3 m of the authored Core center, aimed within 30°,
accepts the one carried fragment indoors, including blackout/zero-G. No
cockpit authority or electricity is required. Ship validates finite mass ×
material yield before removing the stable identity from the world session;
runtime immediately drains removal events into the mass ledger and clears
physical/presentation caches. Busy intake leaves the next fragment untouched.

Ship owns a single automatic process, 200 MJ/s by default. Catalog gameplay
yields default to 1 GJ/kg iron, 0.5 GJ/kg silicate, 2 GJ/kg ice; these are tuning
values, not chemistry. `ConversionConfig` permits finite nonnegative yields
and a positive finite rate. Zero yield consumes immediately. Generated joules
settle with ordinary discharge before capacity clipping; excess is wasted and
full batteries keep processing to completion. Updates split at process completion
so later idle time is not incorrectly charged by an already finished fragment.

## Interior zero-G (#191)

On outage, interior character movement uses ship-local view-relative three-axis
acceleration (WASD, Space up, Shift down), retaining walking/jump momentum.
Released controls coast; controlled speed is bounded by the existing 3.8 m/s
movement speed without reducing faster inherited momentum. Body-clear fixture/wall/deck/ceiling contacts remove blocked
velocity components. Exterior planetary gravity never enters this path. Power
return preserves position and vertical velocity before ordinary deck gravity
settles the player. Airlock EVA inherits ship plus local drift; seated occupancy
remains mechanically fixed. Held fragments follow existing carrying rules in
both powered and weightless interior modes.

Loose cargo's unpowered surface preserves velocity without ambient damping and
adds conservative radius-based ceiling containment; horizontal hull/fixture
containment and precise deck support remain unchanged. Surface changes invalidate
sleep support through existing snapshot matching. No acceleration-induced cabin
forces, crew inertial simulation or new contact engine is introduced.

## Cockpit blackout authority (#192)

Cockpit occupancy is physical; commands require powered cockpit ownership and
free flight. Steering uses the exact powered interval, stops at energy depletion,
and clears target/ramp/held-key residue at outage or cockpit exit. Signed throttle,
heading and momentum persist. Charging/conversion automatically resumes selected
nonzero thrust, including while the cockpit is empty; manual control still needs
a powered pilot. Assists remain authoritative and reject manual overrides.
Native and automation steering share gates; blackout/assist/exit clears the runtime
latch, and OS repeat/focus replay cannot reinstall a stale steering press.

The Core-local display maps `ShipSnapshot::core_status` field-for-field to
`CoreInstruments` alongside explicit bus power. Display formatting/cache and
physical surface routing belong to renderer; mechanical intake remains
available without power.

Crash-grounded L input routes through the same ship-domain assisted action as
normal landing. Automation uses that production route; tests cover lift/recovery
and safe relocation on representative terrain with authored support geometry.

## Realistic flight regression (#195)

`flight_regression.rs` owns opt-in deterministic initial conditions and tests for
`flight-regression`, `core-restart`, `crash-recovery`, `cargo-flight` and
`cargo-blackout` E2E setups.
After setup the normal automation actions/controllers own every transition.
Inspection exposes explicit power, conversion and ship-owned navigation telemetry.
The 500-piece fixture uses 0.032 kg authored iron fragments inside the aft cabin;
checks compare the live ledger with actual physical identities, including Core
consumption, rather than assuming every overfull stress-fixture piece is onboard.
See the [runner commands](../../scripts/README.md#realistic-flight-regression-195).

## Flight/cargo CPU profiling (#196)

`flight_profile.rs` composes fresh production E2E/contained-cargo fixtures and
measures fixed-count event-ledger, boundary, moving powered/zero-G, transition
and dense-contact workloads in release mode. Observations and identity/mass
checks run outside each timed region. Reports retain update distributions and
motion counts; no presented frames or hardware FPS are synthesized. The
[runner](../../scripts/FRAGMENT_BENCHMARKS.md#flight-mass-and-cabin-power-cpu-profiles-196)
validates report completeness/raw statistics and applies explicit baseline/absolute
limits when explicitly invoked and in weekly diagnostics; PR CI only compiles source. This path introduces no gameplay timing or physics shortcuts.

## Native M1 integrated follow-up (#198–#200)

`cargo-blackout` initializes one held 2 kg bootstrap fragment and 499 contained
0.032 kg loose fragments with an empty Core. Production input, conversion and
physics then own zero-G drift, recharge, gravity restoration and subsequent
depletion. No new state-mutation command is exposed. The realistic-flight suite
also tests the 20 m/s landing boundary reached through ordinary tangential thrust.
See reports/issue-198 through issue-200 for dated native evidence and limits.

The opt-in fragment benchmark's `onboard` layout reuses that contained cargo
initializer (0–500 pieces) for native wall-clock frame sampling. Active initial
motion is explicit; settling still runs ordinary physics with no installed sleep
flags. Older multilayer scattered/dense stress layouts remain distinct cases.

## Continuous spherical mining (#231)

`ResourceGameplay` translates held mouse/valid target and current f64 tool attachment
into world-owned `tick_spherical`. Input reset and mouse-up detach immediately;
shared native/automation `apply_look_delta` admits zero-time aim transitions so
rapid away/back releases without adding mass. Runtime never computes mining
mass or debits energy. Session owns all reserved/released identities/journal.
Attached entries are excluded from pickup and loose physics, then retain the
same identity after release. See [source-of-truth lifecycle/integration](../world/resource-contracts.md#continuous-session-integration-231).
Spherical mesh/contact and final carry/Core migration remain #232–#234.

## Sphere presentation (#232)

`resource_presentation::fragment_mesh` selects the single sphere, established
resource tint and `GrowingSphere::radius_meters()/0.48`, preserving pose/ID.
The reserved attached identity is omitted from the ordinary loose draw list;
`ResourceGameplay::growth_preview` supplies exactly one visual at this frame's current
camera-local front-left pose (including orientation), without a rigid body or
second material record. It hides when stowed/outside surface gameplay and
vanishes on release; the same session ID becomes an ordinary world visual.

Legacy contact proxies and inflated gameplay size remain explicitly independent
until #233 (spherical physics) and #234 (release/carry/contact integration).
Consequently this visual migration alone does not claim matching sphere contact
or final resting heights. Existing fragment interpolation compares the physical
snapshot size to physical size, independently of the new authored visual scale.
Automation exposes `sphere_visual` and `mining.growth_preview` without changing
legacy observation fields used by existing fixtures. No disk migration is needed
for the in-memory Phase 0 session.

## Sphere contacts (#233)

`fragment_physics::snapshots` maps every loose fragment to an analytic sphere
using the exact bounded `GrowingSphere` radius already used by presentation.
Ground support is rotation-independent. Grid, support/sleep tracking and frame
conversion remain in physics; no per-fragment hull allocation/import is required.
Attached previews and carried identities never enter the solver. Final raycast,
carry clearance and old emission-adapter migration follow in #234.

## Sphere interaction integration (#234)

World defines the single radius used by pickup and all runtime clearance, prompt,
visual and contact adapters. Attachment pose now includes current camera rotation;
release retains that same reserved ID, mass, radius and pose, with ordinary gravity
providing the drop motion. Tool previews have no loose body or pickup target.
Legacy hull selection is removed from emission/carrying/benchmark setup; historical
geometry evaluations still explicitly load their original assets. Checkpoints
identify `collision_shape: sphere` rather than exporting stale shard vertices.
Core intake uses the ordinary mechanical E route and authoritative ship conversion.
The portable `sphere_interactions` test covers all materials at 10 kg and a short
0.25 kg deposit through F pickup, carry, F drop, reacquisition, E intake and
streaming restoration. Native M1 remains #236.


## Checked spherical fixtures (#303)

Test-only `sphere_fixtures.rs` constructs append-only, released default-lifecycle
spheres and checks actual mass/radius/source/ID/count before carrying/contact
assertions. `sphere_workloads.rs` owns separately versioned portable 500/1,000
current-sphere workloads, with raw population and CPU samples. Their canonical
initialization now lives in `benchmark_fixture.rs`, exposed as `current-cap` and
`current-mixed` to native benchmarks and opt-in E2E, including 1,000 objects (#313).
Raw benchmark accounting includes actual per-object contact radius. Historical flight
and #235 populations retain their original definitions. See the canonical
[fixture/workload contract](../../scripts/FRAGMENT_BENCHMARKS.md#current-sphere-fixture-and-cpu-workload-contracts-303)
for commands, fixture identities, tolerances and interpretation limits.

## Nearby preparation observations (#307)

`mining.rs` retains one Rc-owned immutable nearby result plus raw candidates,
with exact position/seed/mode invalidation and world deposit revision-based
reconciliation. Current rays/obstruction are never cached. Terrain selection,
generation, complete preparation and renderer CPU upload preparation are measured
separately in automation inspection; see the canonical [budget guide](../../docs/terrain-validation.md#nearby-query-reuse-307).

Automation inspection's `visual_context` observes the renderer-selected backend,
adapter and driver plus physical window size and DPI scale. Corpus tooling uses
these observations to prevent comparisons between incompatible capture contexts
(#308); they never affect gameplay or automatically approve images.

## Fragment update scratch and mass invalidation (#306)

ResourceGameplay retains snapshot/state/ID buffers. Snapshot filling preserves session
order and range/frame gates; states move into the contiguous solver input rather
than cloning a second population. Capturing presentation history reuses the same
snapshot storage and retains its simulation-frame interpolation policy.

Exact identical validated carried/loose poses emit no world journal entry;
pickup/drop remain explicit events even at the same pose. Physics still stores
velocities, angular state and local anchors, and ship synchronization still
computes absolute poses when a supporting frame moves. Growth, Core removal,
release and wake invalidations keep their existing routes.

Mass synchronization drains mutations in the stationary containment frame.
Changing that frame reclassifies retained centers once, including unchanged
unanchored world objects that a moving boundary can pass. Repeated redraws in
the same frame do no extra scan; a moving frame pays O(retained count) for exact
boundary membership. This deliberately replaces the assumption that every
unchanged pose creates an event. The return count is ledger visits for portable
profiling, not a gameplay input or native performance promise.

## Frame and dimension boundary inventory (#311)

See the canonical [boundary inventory and compatibility policy](../../docs/technical-architecture.md#position-boundaries-and-resource-dimensions-311).
Interior character history interpolates typed ship-local points; fragment anchor
storage also requires that type. Renderer mesh scale and analytic sphere diameter
have separate names. Automation serializes explicit arrays and preserves its
existing JSON keys and values.


## Consumable raw resources (#337)

See the canonical [resource catalog and compatibility contract](../world/resource-contracts.md#consumable-raw-resources-337) for new feedstock generation, physical lifecycle, distinct presentation and Core yields.

## Fabricator physical input (#339)

The session owns a portable Fabricator. E inserts the one held world fragment
through a universal interior port, within 3 m and 30 degrees with unobstructed
sight. Core/cockpit/door targets retain priority; blackout does not gate intake.
The port uses the generated ship-local Fabricator input marker (#342).
Successful insertion removes the source exactly once and drains its physical
journal into mass, motion, anchors and presentation caches immediately. Full
reserves and invalid milligram masses leave both domains unchanged. The whole
accepted object is consumed, with overflow discarded. F pickup/drop is unchanged.

## Continuous Fabricator power (#340)

Runtime registers the runnable head recipe's watts with ShipController before
every shared gameplay tick, regardless of view, player location or flight state.
The Core settles conversion, propulsion, idle and Fabricator demand together.
Every ordinary bus consumer shares the exact funded interval, while reserved
assist energy remains isolated. Runtime floors that interval to nanoseconds
(no free progress; less than one nanosecond of paid work can be lost at an
exhaustion boundary). No blackout time is banked. Recipe completion splits the
tick and removes its load immediately; idle and blocked tray demand zero watts.
Core telemetry exposes Fabricator watts separately and includes them in total
draw, depletion and braking-energy estimates. Recipe efficiency is watts times
actual worked duration; there is no second battery or assumed full powered frame.

## Fabricator mass transitions (#341)

The runtime updates two dedicated cached ledger keys for integer raw reserves
and the one physical tray item. Every machine command and completion synchronizes
immediately; zero contributions remove their keys. World fragment consumption
drains the same physical journal before the input command returns, so carrying
never creates a second cargo identity. Enqueue removes committed ingredients
from accounted mass deliberately, collection removes tray mass, and cancellation
never refunds. Queue/in-progress/consumable inventory contribute zero. Existing
loose cargo and Core-consumed identity accounting retain their owners. Runtime
UI/inventory adapters must use execute_fabricator and validate receiving slots
before collection; no consumable collection UI is introduced by this issue.

## Authored Fabricator module and tray presentation (#342)

The cabin component owns the wall chassis, monitor, input throat and tray, plus
a stationary generic output package. Runtime reads the generated input marker.
The renderer receives only a tray-occupied boolean; the existing ship uniform's
spare component gates the authored output surface in the opaque shader. The
package appears at completion and disappears on collection, with no animation,
new draw or physical fragment identity. It follows the presented ship pose and
remains visible during blackout. Monitor artwork/crafting UI are owned by #343.

## Fabricator crafting screen (#343)

`session/crafting_ui.rs` maps validated recipe/job snapshots to a keyboard modal.
E targets disjoint authored monitor/input face regions; only the port inserts.
Character's machine sight query excludes its own conservative fixture and retains
other occluders. Opening clears held movement/mining/throttle; modal input cannot
reach gameplay. E/Escape close, Up/Down browse every recipe (including locked),
Left/Right choose positive quantity, Enter adds, Tab selects any queued job and
Delete cancels without refund. Native repeats/synthetic presses use the existing
focus/capture policy. Focus loss/view lifecycle clearing closes the modal.
The screen shows all raw bins, fixed per-unit costs/time/watts, precise shortages,
active time and selected FIFO jobs, quantities and tray/power status. It uses a
centered, high-contrast 3x bitmap panel, wrapping long feedback at 66 characters.
Renderer's fifth instrument-atlas section displays observational Fabricator status
on the authored monitor. No gameplay state lives in the GPU or rasterizer.

## Deliberate Fabricator output collection (#344)

E targeting the separate authored tray face validates the portable consumable
receiver before clearing one stable `(job, unit)` output. Full category leaves
the tray, queue block and mass unchanged and reports feedback. Empty/repeated
collection cannot duplicate inventory. The package is an existing authored ship
submesh, never a loose world/carrying entity; it follows the presented ship pose
and remains visible without Core power. Category selects ration stripes or a
medical cross in its UV surface. Only occupied tray output contributes mass;
collection removes that contribution. Reserved/queued jobs remain distinct.

### Consumable HUD (#346)

Runtime maps the existing three food and two medical slots into a separate
renderer DTO row. Equipment selection remains keys 1–5. M/N do not consume
items in gameplay; #377 owns Q-wheel selection. Survival effects use the shared #370 transaction.

### Fabricator automation (#347)

Inspection exposes stable ordered raw bins/jobs, tray identity and the existing
3/2 inventory slots without advancing simulation. `crafting` actions invoke only
the same open-modal action handler as native input; they cannot insert free
resources, open a remote machine or bypass transactional validation. Portable
session regressions run in PR CI; graphical and native signoff remain separate.


## Logical save boundary (#364)

The [canonical save inventory](../../docs/save-contract.md) defines schema-1
source state and excluded caches. Domain-owned save state is validated before
restore; runtime composes temporary owners and rebuilds mass, immutable terrain
queries and presentation/query/solver caches without advancing simulation.
Stable IDs, depletion, carrying, motion, Core/assist progress and logical
contact/sleep/dwell state survive. No input latch, OS handle or GPU state is
restored. An attached mining sphere becomes a detached existing physical entity,
without allocating a new ID or losing mass. Whole-session/Fabricator publication
is composed in #365; disk checkpoints remain #366.

## Whole-session saves (#365)

`FabricationSaveState` validates recipe definitions, ordered complete bins, queue
IDs/captured recipes/committed quantities/progress, allocator high-water marks,
tray and exact inventory slots together. Cross-owner item identities cannot be
both collected and on the tray. Runtime's `SessionState` is the concrete payload
inside `SaveGame`; `save_bytes` and transactional `restore_bytes` integrate all
physical state, equipment and exact elapsed/backlog clocks. Stored mass and
recipe watts are reconstructed without charging or refunding. UI/input/focus,
feedback and render caches reset; interactions are recomputed from restored
world state. The native host's automation policy stays host-owned. No disk IO,
autosave scheduling or offline simulation is implied; #366 owns durable storage
and shell lifecycle integration. See [canonical save contract](../../docs/save-contract.md).

## Hold-Q consumable wheel (#377)

Hold Q in captured living gameplay to open five stable inventory segments: three
food, then two medical. Release cancels; clicking an occupied segment consumes
and closes only after the shared #370 prepare-before-remove transaction succeeds.
Hover/keyboard selection captures `(slot, job, unit)`; stale replacements and
invalid effects cannot remove items or apply partial benefits. Full vitals allow
waste; exhaustion permits emergency consumption. Hover feedback reads the saved
recovery catalog. Tab/arrows navigate, Enter/numpad Enter consume; equipment stays
on keys 1–5 and M/N never consume items.

Opening clears held movement/steering/throttle-adjust/grab/mining controls.
World, ship, crafting and survival continue ticking; movement/look/world actions
are gated. Pointer geometry shares the rendered physical-pixel fit. A pre-held
mouse must release before use; consumption cannot click through to mining.
Focus/lifecycle loss and restore clear transient wheel state. Escape closes the
wheel and opens full pause; death closes it and freezes authority. Cursor capture
resumes only in living, unpaused, focused gameplay. Vital meters remain beside
the wheel through the generic secondary UI layer normally occupied by the hidden
reticle, without adding a GPU layer or simulation authority. Native OS-input,
DPI/appearance/M1 signoff remains separate under #376.

## Durable active-run orchestration (#366)

Runtime `checkpoints.rs` owns normal-play startup restore, a bounded save worker,
60-game-second/discrete-event scheduling, persistent native-title warnings and
verified clean-exit capture. Domain state stays on the event thread; owned bytes
cross to the worker. `salimon-persistence::storage` owns verification and native
filesystem durability. Startup renderer creation preserves restored clock backlog;
E2E and benchmark launches remain isolated from user saves. See the
[canonical save contract](../../docs/save-contract.md#durable-active-run-366)
for paths, recovery/failure policy, Windows behavior and future lifecycle owners.

## Survival restriction composition (#369)

`session/survival.rs` derives the [canonical restriction policy](../survival/architecture.md#movement-and-interaction-restrictions-369)
from the session's typed survival owner. Shared commands and each tick apply
voluntary character speed, pilot revocation, mining interruption and crafting
permission. Both native input and portable protocol use these gates, including
legacy direct `thruster` automation. Contextual cockpit/machine prompts filter
for the same permissions. Discrete nonessential E/F/crafting mutations share an
active-time recovery timestamp; doors/menus/emergency consumption bypass it.
Mining scales only its bounded extraction delta, not world/physics/job time.

Automation inspection exposes exact `survival_restrictions` basis points for
future HUD consumption and regression checks. #371 activates the vital clock and whole-game save publication. #370
and #377 supply actual effects and wheel dispatch separately. No schema-1 save
is silently changed; derived character scale and interaction cooldown reset on
restore and are reapplied by the session before the next tick.

## Consumable survival transaction (#370)

`session/consumables.rs::consume_selection` is the shared wheel/protocol seam.
Selection captures a named inventory slot plus expected `JobId`/unit, not a
category/first-occupied request. Gameplay view, non-crafting state and typed
Consume permission gate use; fatigue zero remains eligible. Stale/empty identity,
unknown recipe, mismatched category, bad effect data or terminal state reject
without changing either owner. Prepare all benefits on a survival clone; only
then take the exact slot and publish the prepared owner under exclusive session
ownership. Neither operation changes ship mass or Fabricator jobs/raw/tray.

The portable protocol supports `consume` with 1-based slot 1..5 and positive
`job`/`unit`, all required. It calls this same transaction and reports applied/
rejection; it does not introduce a gameplay hotkey. Inspection includes recipe
recovery definitions and active survival recovery snapshots. Native desktop
focus admission stays shell-owned; #377's wheel uses this seam, captures the selected identity and closes only
after success; recovery definitions supply hover feedback. M/N remain absent.

See the [canonical recovery data](../survival/architecture.md#consumable-recovery-definitions-370).
Runtime #371 activates production survival ticking and full-save
publication; #377 still owns the native wheel and #378 advanced foods. Serialized
recovery and inventory continue exactly and remain frozen offline.


## Active survival, terminal state and restart (#371)

`session/lifecycle.rs` owns typed run/journey source and deterministic fresh-run
preparation. `session.rs` advances survival first to determine the admitted portion
before death, then composes physical work for that duration. Terminal state clears
held controls/mining, pilot authority and backlog, and rejects subsequent world
commands. Ordinary menus continue active time; true pause remains #381.

`session/persistence.rs` captures survival meters, fractional clocks, captured
status/healing rules/intervals, terminal cause, offline watermark, item catalog,
restriction tuning and run/journey source with world/ship/Fabricator/inventory in
one schema-3 envelope. V1 migration initializes the new survival source;
v2 migration adds journey tracking. Missing/invalid version-required source rejects. Restore resets transient UI/view/input and
repairs derived permissions without stepping or granting offline effects.

`checkpoints.rs` performs verified launch recovery before exposing a temporary
candidate; only fatigue may be credited. Its bounded worker publishes terminal
pairs, immutable archive summaries and both fresh generations during restart.
The #374 Journey Ended screen exposes one verified Restart Game action through
initial focused Enter/R or a pointer hit; it replaces the earlier minimal notice. See the
[canonical save/lifecycle contract](../../docs/save-contract.md#survival-lifecycle-and-launch-transaction-371)
for failure/crash guarantees and remaining #373/#377/#379 scope.


## Journey accounting (#372)

`session/journey.rs` owns framed last valid physical samples, combined travel and
named location DTOs. `session.rs` measures ship travel around its update, then
character travel around its controller using the same completed ship frame.
Cabin-local samples exclude inherited ship translation/rotation; outside/EVA
samples remain absolute f64. Neither camera/render origins nor out-of-tick changes
create distance. The [canonical save contract](../../docs/save-contract.md#journey-statistics-372)
defines the metric, schema-3 v1/v2 migration, background timing and version-2
archive policy. `checkpoints.rs` archives on terminal publication, not only restart.
Maintain `session/journey/tests.rs`, native background-scheduler and checkpoint
archive/recovery/version tests when changing these boundaries.


## Survival HUD and status details (#373)

`survival_hud.rs` maps typed survival snapshots, immutable effect snapshots and the
existing restriction DTO into observational UI data; `survival_hud/raster.rs`
produces cached RGBA through the existing generic TopLeft overlay. The renderer
has no survival dependency. `Survival::effect_snapshots` exposes captured behavior,
stack severity, lifetime/remaining duration and removal policy without JSON peeking
or mutation. Registered sources retain their stable source ID in descriptions.

Gameplay shows health/cross, hunger/cutlery and fatigue/bolt with thin bars and no
permanent meter numbers. Active effects have a separate icon strip. Active hunger
and fatigue speed reductions remain labeled below the bars; H opens/closes the
read-only detail panel, and [ / ] pages through the entire bounded effect registry.
The panel shows per-tick amounts, stack count, duration rounded up to seconds,
exposure/persistent lifetime and treatment-only policy, plus exact hunger, fatigue,
combined speed and interaction-rate reductions from the authoritative basis points.
No threshold or restriction math is duplicated in UI authority.

H requires normal gameplay focus/capture and initial nonsynthetic input. The
panel does not pause time or consume world controls; it uses keyboard navigation
and no pointer capture. Capture/focus loss closes it. Crafting, engineering tour,
diagnostics overlay and terminal state suppress this HUD. Overlay priority is
crafting, diagnostics, then survival. #381 must extend this same visibility and
input admission when adding actual pause menus; #377 retains meters while the
wheel is open and uses its own world-input gate. Both interfaces share these admission gates.

Presentation-only thresholds are health <25, hunger <20 and fatigue <=20 for
critical colors; other values <50 use amber. A subdued 400 ms color pulse and a
three-second warning accompany entry. Warning rearm requires two points above
the threshold, preventing repeated warnings near a boundary. These choices do not
change gameplay rules. Positive observed net health/hunger changes tween bars for
450 ms and float the actual +N gain for 1.2 s; damage shows immediately. Passive
hunger drain updates rather than cancels recovery animation. Full-meter waste has
no fabricated gain. New-run/time-rewind observations establish a fresh baseline.

All animations use admitted survival time, never render frame count, and never
modify save state. Hidden panels close safely. Unchanged quantized healthy frames
reuse the image/revision; presentation state is host-owned and excluded from saves.
Icons are original bitmap shapes; shared action-bar glyphs include UI punctuation.

Raster scale is bounded 2x–3x then reduced to fit. The renderer-owned
`toolbar_message_inset` is shared with the native HUD: no copied inventory dimensions.
Layout reserves that fitted lower band, a 96 px action/margin allowance and the
screen center. At the minimum window size, detail text replaces the meter rows
while H is open; status/equipment are not overlaid on one another. Below usable
native bounds, a non-fitting image hides rather than overflowing or allocating
unbounded pixels. Text is bounded to validated typed formats, with effect pages
and two-line descriptions preserving all current registered definitions.

Portable regressions live in `survival_hud/tests.rs`, its raster layout tests and
app status-key admission tests. Inspection exposes `survival_hud` DTOs; native E2E
keys `status`, `status_previous`, `status_next` use the same host dispatch as H/[ / ].
Native inspection adds `survival_details_open`. CPU PNG previews in the completion
report show actual raster output; they are not native GPU screenshots. #376 owns
remaining desktop/DPI/Metal visual signoff and #383 optional sensory feedback.


## Journey Ended screen (#374)

`journey_ended.rs` owns observational report formatting, bounded physical-pixel
layout, pointer hit testing and image caching; `journey_ended/raster.rs` owns the
bitmap final-transmission card, dim backdrop, button/hover and failure footer.
It consumes the same typed `session/lifecycle.rs::ended_run()` as the archive,
not JSON or independent guesses. Source states/schema/report bytes are unchanged.

`app.rs::restart_terminal` is shared by Enter/R, the one left-click action and
native automation `restart`. It calls the existing verified checkpoint/archive/
fresh-pair transaction before replacing session state. Error retains the terminal
session and usable retry feedback. Completion resets host HUD/action/terminal/
terrain/presentation caches and leaves input capture off. Default new-run state
and starter-supply ownership (#379) remain unchanged.

Keyboard dispatch admits initial nonsynthetic Enter/numpad Enter/R only while
terminal, focused and not occluded; ordinary terminal keys cannot open gameplay,
engineering or status views. Pointer events require the same native focus gate,
fresh finite physical coordinates and the half-open button bounds. Capture/focus
loss, cursor leave, resize/DPI and lifecycle reset invalidate pointer state.
Terminal rendering suppresses diagnostics/crafting/HUD, reticle, held tool and
inventory/equipment over the frozen world. See the
[canonical terminal contract](../../docs/save-contract.md#journey-ended-presentation-374)
for publication guarantees, display rounding, fitting and native limitations.

Tests live in `journey_ended/tests.rs` and `app/input_tests.rs`: typed report/archive
parity, all causes, extreme time/distance formatting, DPI/aspect/button-pixel parity,
cache/hover/failure, guarded real keyboard restart, archive conflict/retry and
terminal/fresh relaunch using real temporary native storage. Existing restart
fault injection and terminal world freeze tests remain required. Native inspection
exposes `journey_ended` report text; `restart` is a native-host operation and does
not add a diskless portable-session bypass. E2E hosts without verified storage
report failure instead of pretending to restart. #375 owns broader scenario/save
DSL and expansion verification; #376 owns native OS/Metal visual acceptance.

## New-run initialization (#379)

`GameplaySession::new_run` composes fabrication-owned starter inventory/reserves
and reconciles ship mass immediately. Normal-play startup and the existing
verified restart candidate use it; `Default` remains empty for restore and
isolated fixtures. `restore_bytes` replaces complete owners and never grants
supplies. The [canonical contract](../../docs/save-contract.md#starting-supplies-379)
records identities, publication and conservation. Portable E2E coverage lives in
`session/starting_supplies_tests.rs`, including real disk launch/restart/reload.


## Gameplay pause (#381)

Escape in gameplay opens the pause menu and releases the cursor. Enter or Escape
resumes; F1 explicitly releases the cursor without pausing. P/R/N camera commands
belong only to the F2 precision tour. Pausing closes crafting/status overlays and
clears held movement, steering, throttle keys and mining. Requested ship throttle
is preserved. Full simulation, Core, fabrication, loose cargo, survival effects,
journey accounting and autosave game-time intervals freeze at the session boundary.
UI/rendering and checkpoint IO continue. The transient pause flag is not saved.
Both native/background and portable automation tick paths exclude paused elapsed
time and clear pending backlog on entry/resume; focus/minimize/suspend cannot
resume a paused session or credit offline fatigue. Resume requires fresh controls.
The pause menu takes priority over diagnostics, HUD, reticle and equipment.
Save Now/Save & Exit use the verified #382 worker; Q-wheel input must respect
`GameplaySession::is_paused()` and the shared consumption gate.


## Optional condition presentation (#383)

`condition_feedback.rs` samples immutable survival snapshots. Health <25 gives a
faint edge vignette and a finite two-beat cue; hunger <20 gives one warning (the
existing survival HUD owns its pulse); fatigue <20 gives a quiet breathing cue
and walking-only camera roll, capped at 0.003 radians at zero fatigue. The center
aim ray and all authority remain unchanged. Three-point hysteresis rearms cues;
there is no endless sound loop. Exponential fades use active time. Pause, crafting,
tour, terminal, unfocused and occluded presentation suppress effects immediately.
The renderer stretches a cached 128x128 generic image before HUDs, preserving
center readability. No postprocessing or gameplay-domain dependency is added.

Pause -> S opens keyboard accessibility: C camera motion, A condition audio, V
vignette; #465 adds M master mute, B ship audio and G ship volume (0–100%).
S returns and Enter/Escape resumes. Version-1 preferences live in
`settings.json` beside the active-run directory (or `SALIMON_SETTINGS_FILE`) and
survive restart/new runs. Save errors stay visible; corrupt/unknown settings use
defaults with a warning. Settings do not enter run checkpoints.

Continuous audio (#582) is composed by `scout_audio.rs` and the shared
`native_audio.rs` CPAL output with fixed-memory `native_audio/mix.rs`.
Survival thresholds remain in `condition_feedback.rs`; settings keep their owner.
See the [canonical soundscape contract](../../docs/scout-presentation.md#soundscape-boundary-465)
for voice limits, lifecycle, device recovery, telemetry and validation.

## Advanced food composition (#378)

The shared exact-identity consumption transaction accepts all four food recipe
IDs from the survival catalog. `session/survival.rs` derives fatigue penalty relief
from captured active modifiers; movement, interaction work and HUD receive that
same policy. Status details describe continuous modifiers and remaining active
time. The existing pause/full-save/offline gates own their timing and persistence.
Native wheel dispatch is integrated by #377. See the
[food effect contract](../survival/architecture.md#four-food-catalog-378).

### Deterministic E2E desktop-input isolation (#313)

The scenario runner uses `SALIMON_E2E_ISOLATE_INPUT=1` only on its `--e2e`
child. Runtime then ignores native keyboard/pointer events and preserves
protocol-held controls and fixed-step backlog across native focus/resize clock
resets. Window/GPU lifecycle and visibility still operate; this is not native
input/lifecycle certification. Normal play and benchmark mode ignore this E2E
setting. Direct `Game` probes default to actual native input; their child removes
the variable. `inspect.native_input_isolated` makes the boundary observable.

## Scout reference cameras (#463)

`scout_reference.rs` loads four checked-in ship-local camera poses shared with
Blender. The `scout_reference` automation operation is admitted only in the
existing native `--e2e` boundary. Its optional camera overrides presentation,
never character position/aim, interaction rays, power, time or saved state.
`clear` restores the gameplay camera. The reference scenario uses real Core
feeding/restoration to compare blackout/powered views. See
[the capture contract](../../docs/scout-presentation.md).

## Scout audio composition (#465)

Continuous audio (#582) is composed by `scout_audio.rs` and the shared
`native_audio.rs` CPAL output with fixed-memory `native_audio/mix.rs`.
Survival thresholds remain in `condition_feedback.rs`; settings keep their owner.
See the [canonical soundscape contract](../../docs/scout-presentation.md#soundscape-boundary-465)
for voice limits, lifecycle, device recovery, telemetry and validation.

## Scout machinery composition (#464)

`scout_feedback.rs` derives `MachineryFeedback` from existing authoritative ship,
Core, Fabricator and active elapsed snapshots. `app.rs` attaches it to the same
presented ship pose; `automation.rs` exposes read-only inspection. No event queue,
animation controller, save state or additional elapsed admission is introduced.
See the [canonical feedback contract](../../docs/scout-presentation.md#machinery-feedback-boundary-464).

`scout_walkthrough.rs` supplies opt-in `scout-fabricator` and
`scout-work-blackout` E2E initial conditions for #489: ordinary starter supplies
plus the existing exact 2 kg Core-restart fragment. The first starts landed by
the machine; the second starts with an unfunded job and Core aimed at the held
fragment. Initial placement/energy are explicit fixtures, not normal-play mining
or travel. There are no new mutation protocol operations: physical E intake,
menu commands, production ticks and exact tray collection retain their gates.
Portable regressions preserve save/restore/pause feedback and identity/mass;
`reports/issue-489/walkthrough.py` drives the real window adapter and GPU captures.
Two extra presentation-only close cameras live in `scout-machinery-cameras.json`;
the four shared lighting/Blender poses remain in their original file.

## Pause checkpoint controls (#382)

The paused menu offers Enter/Escape Resume, N Save Now, S Settings and X Save &
Exit. R retries failures, C cancels exit and Q explicitly exits without saving
only after a failed exit checkpoint. Resume cancels exit intent. Menu keys require
initial focused admission; repeats/synthetic/unfocused input cannot request IO.
The existing center panel uniformly fits its drawable; Settings keep their keys.

`checkpoints/control.rs` owns bounded coalescing, verified UTC receipts and exit
intent. Manual/automatic captures share the same worker, atomic store and active
run. Polling continues while paused/minimized; a newer request cannot be satisfied
by an older in-flight snapshot. Retry bypasses frozen game-time backoff. Save &
Exit and living-gameplay window close await verification or explicit failed-save
risk choice. Final exit does not start a second write after either choice.
Terminal archives/restart and 60-second/event autosaves retain their owners.
No manual request ticks gameplay or credits offline fatigue. Native OS-input,
DPI/appearance and M1 signoff remain separately tracked by #376.

## Persistent survival onboarding (#380)

`session/tutorial.rs` owns seven monotonic per-run objective bits and durable skip:
inspect H survival details, open hold-Q, eat a basic ration, inspect the Fabricator
screen, finish a basic ration, collect it, and view checkpoint status in Escape.
Production success boundaries record events even before their current prompt;
failed consumption/collection and canceled jobs do not complete objectives.
T skips permanently for this run. Death/restart creates fresh objectives; legacy
saves and empty fixtures migrate dismissed. No resources, clock immunity, input
capture or forced ordering comes from the tutorial.

Compact two-line guidance uses the existing bottom action band only when no
interaction prompt/transient is present; details, wheel, crafting, diagnostics,
pause and terminal screens suppress it. Thus it adds no competing overlay or
world-anchored label. Existing DPI fitting and toolbar reservations apply.
Native appearance and complete physical input review remain #376.

## Portable survival balance (#384)

`src/survival_balance.rs` runs six bounded cross-domain tests through production
mining, carrying, E/Q transactions, funded crafting, save/lifecycle and EVA routes.
Travel/setup are explicit time budgets with stage fixture placements; see the
[canonical evidence/limitations](../../docs/survival-balance.md). Native and owner
budget/tuning approval remain #496; no production constants change.

## Funded engine composition (#493)

`session` maps B to the ship-owned engine switch; native/protocol input share
focus/modal/pilot/power gates. Scene mapping passes ON/OFF, actual acceleration
and braking to the existing cockpit instruments. Audio/machinery suppress drive
feedback while explicitly off. Schema 5 migration is in `session/persistence.rs`;
see the [ship math/safety contract](../ship/architecture.md#funded-core-impulse-and-engine-switch-493).

## Portable persistence/survival scenario composition (#375)

Test-only `session/persistence_scenarios.rs` owns a strict versioned JSON DSL,
whole-save comparisons and deterministic continuation across real disk/launch
boundaries. `checkpoints/scenarios.rs` exercises actual asynchronous manual-save,
startup and terminal restart transactions with storage publication faults.
Production ownership, schemas and native input stay unchanged. See the
[portable DSL, isolation and commands](../../scenarios/portable/README.md).

## Scout UI presentation (#466)

`action_bar.rs::glyph_rows` delegates uppercase prose to renderer `ui_style`,
which also serves physical instrument, toolbar and Q-wheel text. Action/crafting
panels use the shared opaque dark background, cyan rules and ivory text;
`survival_hud/raster.rs` shares text/secondary/accent inks. Runtime retains prompt
eligibility, absolute authored anchors, expiry, modal focus and all input routing.
`app/interaction.rs` supplies compact `[E]` controls; crafting maps status enums
to readable words without changing commands, ingredient precision or queue order.
The [canonical UI matrix](../../docs/scout-presentation.md#scout-ui-readability-contract-466)
records reticle invariants, scanner blockers and native acceptance separately.


## Scout Archive (#467)

The separate flush wall terminal at ship-local `[-3.0, 1.7, -4.13]` faces +Z,
opposite the Fabricator. E opens within 2.5 m, aimed at the screen, inside the
ship with bus power and a clear solid ray. Up/Down (also Left/Right/Tab) select
records; E closes and Escape opens pause. Reading blocks movement/world actions,
Q and H but keeps simulation running. Range/location/power loss, focus/lifecycle
reset, pause, death and restore close the transient reader and clear held input.

`session/archive.rs` owns seven fixed record identities, provenance, explicit
unknown timestamps, retained sequence, per-run read identities and selection.
Five intact departure/Mars/routine/exit records and two unreadable packets carry
no coordinates, reconstructed claims or confirmed missing history. Recovery work is now implemented by #468; destination granting, backend and Hall
UI remain deferred. #469 owns player-known charting. Reader text uses
the existing shared-font modal raster and input admission, never a new UI stack.

Schema 6 requires an inner version-1 Archive payload. Validate known IDs,
selection, bounded read set and no duplicates before transactional publication.
Versions 1–5 explicitly initialize an unread Archive; no prior history is invented.
Read/selection changes enter discrete checkpoint scheduling. Save/restore retains
records but never modal/input state; a new run resets the Archive.

Renderer `archive_terminal.rs` code-authors twelve immutable vertices for flush
backing and screen in the existing ship-local opaque draw, with no new buffers,
pipeline or authored GLB edits. It projects no additional collision volume into
the existing cabin lane. The sixth 512x256 instrument atlas region shows Archive
identity and E affordance, blanks with the bus and restores through the existing
cache. This is a modest code-authored panel, not a Blender asset replacement.
Native appearance/input/DPI/full-frame performance require separate evidence.

## Data Recovery Module (#468)

`session/recovery.rs` owns per-run module identity, captured config and paid
record progress. `archive.rs` composes R start/stop and bounded evidence text;
`fabrication.rs` moves the one component tray identity into the installed mass
ledger. Ordinary ship/Fabricator tick settlement precedes low-priority Core
funding; recovery only spends surplus above an emergency floor. Schema 7
validates installed/pending/tray identity and reconstructs mass without charging.
See [canonical contract](../../docs/data-recovery-module.md). No offline work,
free coordinates or reconstruction of irrecoverable bytes. Native review #515.

### Modal/world overlay composition (#348)

Native redraw in `app.rs` uses one visibility decision for world action-bar
pixels and the reticle. Gameplay without a modal shows both normally; pause,
terminal, Archive/crafting/settings text or the Q wheel suppresses them so a
world-anchored prompt cannot cover modal rows. This only gates rendering;
session input authority and retained prompt/expiry state keep their owners.

### Opt-in native save-review fixtures (#376)

`session/persistence_scenarios.rs::native_reference_checkpoints_376` is an ignored,
test-only export seam over existing #375 scenario initialization, production
consumable/damage/crafting transactions and `SaveManager<NativeStore>`. It writes
only to a new explicitly supplied directory and verifies exact save roundtrips.
It adds no production launch flag, save format, simulation clock or authority.
See [validation](../../docs/validation.md#reference-savehud-fixtures-376).

## Archive cloud verification (#505)

`scout_walkthrough::archive_initial_conditions` adds an opt-in `scout-archive`
E2E fixture at the physical terminal, with ordinary new-run source and no records
read or reader opened. Subsequent E/arrow/Tab commands use production input and
range/aim/power/modal gates. `automation::inspect_session` exposes immutable
Archive source/open/text; inspection does not mutate knowledge or simulation.
The `interact` key alias maps to physical E, including reader close, unlike a
synthetic reader action. `session/archive/tests.rs` covers real Save Now worker
completion, normal relaunch and verified death/restart reset. No save/schema,
record content, renderer DTO, discovery or navigation authority changes.

## Recovery cloud verification (#515)

The opt-in `scout-recovery` fixture reuses the Fabricator-side held 2 kg iron
identity and declares 0.5 kg raw silicate initially in the bin. It creates no job,
tray output, installed module or paid progress. Subsequent production intake,
menu enqueue, funded ticks, E tray collection, walking and Archive R actions
retain their existing gates. `inspect.recovery` is an immutable serialized source
snapshot supplied by `GameplaySession::recovery_observation`; `key: recovery`
maps to physical R. No protocol command sets recovery state. Session regressions
verify concurrent craft/thrust/recovery joule separation, schema-6 resources/jobs
and actual Save Now/relaunch/restart with seven days of zero offline work.

## Blueprint tool and placeholders (#542)

`session/construction.rs` composes ship-owned placement with actual authored
character geometry, individual physical cargo and protected starter escape/service
lanes. `catalogs/blueprints.json` declares seven furniture/wall/door/hull/deck/room/
module entries. Its costs/durations are **proposed estimates**, with
`approved_economy=false`; they authorize no resource debit or construction progress.
#537 owns economy approval before #543/#544 integration. Fine pitch is per-entry
data (currently 0.5 m), not a mandatory global furnishing rule.

C opens/closes the tool, 1–7 select, R rotates, left click places, E inspects the
aimed project, right click cancels the aimed reachable reservation. Escape closes
before pausing. Native and protocol inputs use the same session commands and
native focus/repeat/synthetic admission remains in the shell. Opening stows the
mining tool; closing never re-equips it. Existing essential carrying/escape paths
are protected. The initial coarse structural graph is an explicit support model;
it does not move authored anchors or generate physical hull. Exterior reservations
also reject the conservative existing hull, wings and engine volumes. Actual hull
modification/solid generation remain #545.

`app/scene.rs` maps hollow colored edges into ordinary oriented cuboid instances:
green clear preview, red declined preview, cyan placed project. These render-only
edges never become solid colliders or installed mass. Requirements/status stay
visible together with transient feedback in the action bar. Whole-session schema 8
preserves project source and rejects malformed restore atomically, while tool UI
resets. See the canonical [save contract](../../docs/save-contract.md#blueprint-placeholder-schema-8-542).
Portable and native routes use `scenarios/evidence/blueprint-placement.json`.

The opt-in `sphere_workloads::current_sphere_stress_cpu` records flushed JSONL
inventories, exact-snapshot solver/contact/query/activation timings and all natural
sleep preparation steps. Moving pools collect 100 actual moving samples across
fresh launches; quiet intervals remain raw. It is CPU evidence, not native FPS.
Historical comparisons must retain the original character/layout revision;
current Scout geometry can change supported columns without changing physics.

#566 extends only the report-side frozen-frame collector: a named cloud
llvmpipe profile complements the strict default M1 profile. Both retain actual
focus/visibility/source-state admission. No runtime scheduling, shader or frame
logging rule is changed by that tooling extension. Software device timestamps
cannot be interpreted as hardware GPU budget evidence.


## Physical cargo door and open bins (#539)

Character `cargo.rs` supplies the authored aperture, opposed sliding-leaf bounds
and hollow bin shells. Walking retains conservative bin envelopes. Loose cargo
and pickup rays use the actual 4 cm side walls, 12 cm bottom and open top; no
slot inventory, identity replacement, mass exemption or zero-G pin is introduced.
Runtime `session/cargo.rs` composes manual door motion and swept player/object
clearance, independent of the aft ramp, power and flight state. Aim at the
port-side door handle and press E; reverse with a second press. Partial openings
remain conservatively impassable. Door position is saved in schema 9 and
legacy captures migrate open; see the [save contract](../../docs/save-contract.md#cargo-door-schema-9-539).


## Base habitation (#540)

The bunk's two authored panels use the generated
`habitation_geometry.rs` hinge/swept bounds, sourced from
[the category definition](../../models/assets/ships/salimon-scout/habitation-geometry.json).
The renderer maps a finite stowed/deployed fraction to rigid rotation and normals;
runtime admits manual motion only after full player/belonging clearance. Static
poses cost no repeated clearance scan. Reserved walking clearance is unchanged.
Galley E opens the existing exact-identity consumable wheel, with no generated
food or additional inventory; hygiene remains an inspectable base fixture.
No water/hygiene meter, sleep-time advancement or additional survival benefit is
introduced. Sitting/lying, privacy mechanisms and expanded tactile presentation
remain #577. Schema 10 saves source motion; see
[the save contract](../../docs/save-contract.md#habitation-schema-10-540).


## Interior fixture composition (#575)

`session/fixtures.rs` maps existing saved habitation state to the versioned
[interior layout](../assets/ship/interior-layout-v1.json), preserving the stable
berth ID and ship-rooted f64 transform. `app.rs` supplies borrowed fixture DTOs;
renderer resolves/batches presentation and never owns collision, installed mass,
construction, cosmetic age or damage authority. Native debug inspect includes
actual fixture draw/memory/upload counters; `window_state` reports true focused/
occluded flags. Schema 10 is unchanged. Future room/module owners supply their
real parent IDs and local poses through this boundary. See
[architecture and budgets](../../docs/modular-interior-fixtures.md).


Continuous audio verification: `native_audio/mix/tests.rs` covers callback-size
invariance and reset/mute/stale control; original 60 Hz stereo fixtures and real
device telemetry are opt-in. Linux builds require ALSA development headers.


Archived solver stress (#592) remains separate from current Scout gameplay.
`sphere_workloads.rs` accepts `SALIMON_STRESS_PREFIX_SAMPLES=1..100` only for
explicit diagnostic terminal snapshots. Values below 100 emit
`diagnostic-prefix`, never a completed 100-sample pool. Ordinary workloads retain
100 moving samples, exact archived radius/mass bits and natural 300-step
preparation; no sleep flags are installed. Reproduction uses a disposable
historical-layout checkout and current production physics, preserving the actual
current cabin/physical geometry in gameplay. Query clocks remain a subset of
contact clocks; retain every timeout, partial sample and failed preparation.


Capture verification (#568): native `inspect` with `scope: capture` returns only
readback status after the usual E2E envelope/admission checks. It is observational
and never builds the world snapshot. Ordinary inspection is unchanged; old
executables may ignore the optional scope and return the full snapshot. The
runner restores the real window before GPU capture and logs that OS request;
it does not fake focus, disable occlusion or grant physical input qualification.
See [runner protocol](../../scripts/README.md#bounded-native-capture-polling-568).

## Physical mounts (#576)

`session/mounts.rs` composes G placement, K explicit restraint, J empty accessory movement, shared physical carry identity, frame transport and schema-11 save validation. See [the canonical contract](../../docs/physical-mounts.md).


Scout cabin #577: follow [the canonical source, collision, posture and presentation contract](../../docs/cabin-mechanisms.md). Runtime owns admitted mechanism/posture source; renderer never grants physical authority. Manual mechanisms preserve original belongings and halt on swept obstruction.


Persistent finishes #579: [canonical ownership, schema 13, bounded render factors and controls](../../docs/room-finishes.md). Ship owns room/item cosmetic identity; runtime owns nearby physical control admission; renderer applies the explicit nonfunctional surface allowlist. Cosmetic changes never edit physical material/age/damage.
