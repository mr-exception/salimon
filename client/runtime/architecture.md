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
   state with a bounded monotonic delta. Map the six catalog bodies to `f64` sphere/material
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
`winit` types. All six catalog names and nonnegative camera-to-nominal-surface
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
character-owned floor containment, stable object order and mass-derived radius,
then writes velocity/pose results through existing IDs. Carried pieces are
excluded. Gravity, release/ejection, restitution, deck/hull response, spherical
contacts and substeps live in the portable physics crate. World retains identity,
material, mass, orientation and validated poses; runtime retains cross-domain
frame/session sequencing. See the canonical
[ownership decision](../../docs/technical-architecture.md#physical-object-simulation-decision-114).

## First-person reticle

`reticle.rs` selects a static outlined dot or `+` RGBA image directly from
`MiningTool.equipped` each gameplay redraw. Precision-tour view hides it. The
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
variants. Each follows the same stable identity and authoritative-size rules. Rotation remains presentation-independent as before;
carried, dropped, ship-local and streamed pieces use the same identity mapping.
World mass/volume and runtime contact/carrying controllers are unchanged.

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
route: on an initial press, carrying wins if an object is held or a reachable
object is aimed at; otherwise it starts keyboard mining. MiningTool latches F
until release, preventing a consumed grab/drop press from mining or repeating.
Independent mouse and keyboard mining state is combined into `held`; lifecycle
resets and stowing clear both and the latch. Automation `grab_drop` plus legacy
`mine`/`pickup`/`drop` names map to F with identical contextual semantics.

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
Toolbar rendering and deriving mining-tool presentation from selection remain
separate dependent work: the existing M equip control is preserved for now.
Automation inspection exposes `equipment.slots` (tool names/null) and
`equipment.selected_slot` (1–5/null).
