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
world, diagnostics, and renderer. Supporting crates never call into runtime;
renderer and diagnostics do not depend on behavior domains. Runtime mapping
prevents portable types from acquiring `wgpu` or `winit` dependencies.

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
commands; 1–6 restart inspection of Sun, Mercury, Venus, Earth, Moon, and Mars.
It maps renderer/world measurements to diagnostics without sharing `wgpu` or
`winit` types. All six catalog names and nonnegative camera-to-nominal-surface
observations flow through `BodyDistance`; diagnostics selects the closest for its
single nearby-body row. Diagnostics owns aggregation and the RGBA panel, while
the renderer owns only generic image composition. Gameplay view supplies live
player/ship diagnostics; precision-tour view supplies its camera metrics.

## Contextual action-bar flow

The runtime field-maps ship-owned Core and nearby-body telemetry into renderer
instrument DTOs every gameplay frame; it never derives proximity or radial
velocity. The runtime maps typed ship messages and the aimed cockpit interaction into one
bottom-centered action bar for the normal gameplay view. State-derived actions
remain visible only while applicable. Immediate blocked-door feedback overrides
the current action for three seconds and then expires without changing ship
state. The renderer receives a borrowed RGBA image and placement only, allowing
the action bar and optional diagnostics panel to be composited independently.

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
`fragment_physics.rs` composes deterministic loose-fragment motion and spherical
contacts in ship-local or planet-relative meters, including gravity, release
velocity, deck/hull contact and fragment piling. `MiningTool.fragment_motion`
tracks velocities by existing fragment ID; carried pieces are excluded. This is
a small custom simulation, without Rapier or another physics engine. World
continues to own validated poses, identity, material and mass; runtime owns
cross-domain support frames and contact composition.

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
