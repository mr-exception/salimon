# Runtime Architecture

## Responsibility

The runtime is the native host and composition root. It translates `winit`
lifecycle/window events into a small set of renderer operations and owns timing
that future portable simulation code may consume.

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
world session state for both GPU mapping and inspection, and presents a generic
cuboid tool/aim marker without exposing gameplay types to the renderer.
Physical fragments are world-owned session entities. Runtime maps their
mass-derived cube size and absolute pose to generic presentation DTOs; the same
nearby query supplies automation state even when the tool is stowed.

## Physical surface/ship transfer (#47)

`carrying.rs` composes surface and interior targeting/placement with the shared
character collision layout. `MiningTool.ship_fragments` holds supporting ship-local
coordinates keyed by existing physical fragment IDs, never inventory quantities.
Successful interior release installs an anchor; pickup removes it before following
the player; surface release stays world-local. Every gameplay update and look/input
synchronization maps loose anchors through the current ship frame. The same world
session owns every entity and its immutable mass/material/source throughout.
Greybox cubes remain world-axis aligned; conservative bounding-sphere clearance
keeps them off the deck/walls as the supporting ship rotates. This intentionally
abstracts acceleration through internal gravity without adding rigid-body physics.
