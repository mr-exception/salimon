# Salimon Phase 0 Scout

A custom, editable scout with a broad living cabin, ivory armor, graphite
structure, copper seams, layered wing plates and paired aft ion thrusters. The
protruding port cargo module has been removed. A shorter nose and canopy keep
the ship's exterior silhouette cohesive. The cabin deck retains its human-scale
walking lanes; the character controller accounts for the player's 0.24 m radius.
Current measured dimensions are recorded in the asset manifests and export report.

Three tall window bays on each side and two aft observation panes surround the
living area. The shorter forward canopy has no central sightline mullion. The
contoured pilot chair and lowered instrument station retain clear forward and
downward views for seated and standing players. All panes share inexpensive,
lightly tinted, double-sided glass. The aft hull contains a real door aperture;
opening the door does not leave a solid cap across the exit.

The interior uses warm ivory lining, copper lamp housings, terracotta pilot
cushions and woven runners. The cabin bench, storage cabinet, drawers, and
worktop are removed to clear the deck. Amber
ceiling and floor guidance lights complement cyan instrument displays. The
central **Energy Core** has three cyan energy cells in a graphite cage, copper
containment rings, segmented ceramic armor, an open crown, inset light strips,
locking indicators, service vents and a charge gauge. Its stepped pedestal and
housing fit the existing 2 × 2 m collision footprint, preserving the two broad
walking routes around it. This follows
Notion's Core definition: the vast storage battery and recoverable heart of the
ship. Phase 0 adds a bounded live stored/capacity telemetry fixture for the
monitors, but no consumption, generation, damage, fuel, repair, or production
energy simulation.

The pilot station has one center console with three flat physical monitors,
recessed housings, ivory bezels, copper fasteners, tactile keys, service vents
and deck pedals. The screens retain their distinct live runtime instrument
artwork: speed and common thrust, Core energy, and nearby-body distance/radial
state. Both sides stay open as walking routes from the cabin past the
console into the cockpit nose. Monitor faces, collision and the walkable floor
must remain aligned in the authored source and exported asset.

The pilot chair is a contoured bucket with a reclined tapered back, split
terracotta cushions, side bolsters, compact headrest, armrests, a short stick and
throttle-shaped hand control, plus a suspension pedestal on floor rails. The
hand controls are visual props; the existing flight input remains authoritative.
Eight-corner profile rings and shared materials provide shape without dense
smoothing or subdivision.

The aft engines have hollow flared nozzles, recessed emitters, stepped liners,
armor rings, cooling fins and running lights. Their forms use 8–12 sided
cross-sections instead of dense smoothing or displacement.

## Cargo module removal

The dedicated port cargo room and its protruding hull have been removed from the
current scout. Loose fragments may still rest on clear main-cabin deck space and
move with the ship. The ship does not define a dedicated cargo storage volume or
cargo-room membership count. The historical [issue #34 evidence](../../../reports/issue-34/README.md)
documents the earlier layout, not the current asset.

## Cockpit glazing and nose

The shortened nose and canopy retain forward and lower cockpit glazing. The
lower nose, cheek panes and structural glass floor shoulders have matching deck
and belly apertures. Copper rims identify the floor panes; deck collision
continues to support their load-bearing glass. The single center console
must leave the walking routes on both sides and the seated downward sightline clear.
The `Cockpit_Window_Frame` uses narrow posts and fine lower-pane trim aligned
with the glazing aperture.

`Cockpit_Lower_Glazing` uses the existing lightly tinted, double-sided glass draw,
with alpha blending and no depth writes. Its exported outward normals include
upward floor panes and downward nose glazing. The manifest records validated
downward pitch/yaw samples from the seated eye. Exported-triangle
rays must hit lower glazing and no opaque mesh; they cover the actual deck,
belly, consoles, trim and hull rather than only pane bounds. The asset remains
within 6,000 triangles, 120 primitives, 13 materials and the 512 KiB GLB budget.

The native visual scenario follows free-look, assisted landing, low-altitude
approach, landed inspection, takeoff and cockpit exit. See
[issue #38 evidence](../../../reports/issue-38/README.md) for historical lower-window
screenshots and authoritative state/logs; recheck these views after nose edits.

## Source and regeneration

The [Blender scout source](../../../models/assets/ships/salimon-scout/README.md)
is authoritative for visual geometry and materials. The scout adapter uses the
generic exporter and checks all preserved contracts before replacing exports.
The procedural geometry generator and its software preview tool are retired.
The scout export and validator use authored GLB data and declarative contracts.
Spatial Rust layouts are derived from authored proxies/markers.

- `../../../models/assets/ships/salimon-scout/assembly/salimon-scout.blend` is the editable visual source.
- `export/salimon_phase0_ship.gltf` plus `.bin` is the matching DCC interchange.
- `export/salimon_phase0_ship.glb` is the self-contained Blender runtime export.
- `textures/salimon_floor_grip.png` is an original 16×16 procedural texture.
- `asset-manifest.json` records axes, scale, design dimensions and budgets.
- [Current exterior](previews/current-exterior.jpg) shows the shortened nose and
  balanced hull without the cargo section.
- [Current cockpit frame](previews/current-cockpit-frame.jpg) shows the slimmer
  canopy posts, top bar, and lower-pane trim.
- [Current console](previews/current-console.jpg) shows the three displays on one
  center console and the routes on both sides into the nose. The glass is hidden
  in this view to expose the controls.
- [Closed pressure door](previews/current-door-closed.jpg),
  [partway through the top-hinge motion](previews/current-door-opening.jpg), and
  [open doorway](previews/current-door-open.jpg) show the revised aft exit.
  [Exterior door face](previews/current-door-exterior-closed.jpg) shows the
  recessed backing and raised graphite plate without overlapping surfaces.
- [preview.jpg](preview.jpg) shows six source-geometry views, including standing
  cockpit and rear cabin sightlines.
- [Pilot station](previews/cockpit-station.png), [chair detail](previews/pilot-chair.png)
  and [seated inspection](previews/cockpit-seated.png) show the pilot geometry.
  Their cyan faces are source surfaces; live instrument artwork comes from the
  runtime and is verified in the native game.
- [Core close-up](previews/core-hero.png), [reverse detail](previews/core-detail.png)
  and [cabin context](previews/core-cabin.png) show the redesigned Energy Core.

```sh
python3 models/assets/ships/salimon-scout/export.py --blender /path/to/blender
python3 models/assets/ships/salimon-scout/validate.py
```

The older previews are historical design references. Inspect edits in Blender
and use native screenshots to inspect the exported runtime; asset previews do
not replace native runtime or reference hardware checks.

## Runtime contract

Units are meters; `+Y` is up, `+X` is forward, `-Z` is starboard. Meshes have baked
positions and identity transforms. `Exterior`, `Interior`, `Collision_Proxies`
and `Interaction_Markers` are stable hierarchy groups. Geometry in the linked Blender assembly
is authored in final meters. The historical Task 7
scaling is already baked into this source; do not apply it again. Edit the pilot
station directly in meters to retain human proportions independently of hull scale.

Metadata-only collision boxes describe the floor, walls, ceiling, central Core,
door, console, pilot chair, cockpit side hull, cabin traversal/exterior envelopes,
coarse exterior, wings and both thruster bodies/fins. They add no draw calls.
The adapter generates `client/character/src/spatial_contracts.rs` and the matching
`spatial-contracts.json` sidecar/GLB extras. Validation compares generated Rust
and sidecar to the export. The public character anchors keep their existing API.

The 20 authored boxes and four markers are enumerated in the scout manifest.
`COLLIDER_CabinTraversalEnvelope` bounds the clear cabin after window sills,
aft header and lowest ceiling lamps. It is a free-space volume, not a solid box.
`COLLIDER_CabinExteriorEnvelope` is the conservative cabin/nose contact envelope,
excluding wings/engines; `COLLIDER_ExteriorHull` remains the full broad-phase box.
`COLLIDER_PilotChair` and `COLLIDER_Cockpit{Port,Starboard}Hull` preserve the
previous conservative planar fixture footprints. `MARKER_DoorwayTransition`
marks the inner crossing plane separately from the exit interaction marker.
See [component ownership and geometry/policy mapping](../../../models/assets/ships/salimon-scout/README.md#geometry-and-gameplay-policy).

Character `layout.rs` applies player radius to those raw bounds and builds the
open/closed gate and nose-shoulder proxies. Sight and floor placement consume
the same derived geometry. The human is 1.80 m tall with a 1.75 m eye height;
these dimensions, the seated eye offset and permissions are Rust gameplay policy.
Keep spawn/cockpit exit in a clear aisle. Runtime Cargo builds need no Blender.

Only `Exit_Door` moves for the door state. Its sealed leaf rotates 110 degrees
outward and upward about the top hinge over 700 ms, leaving the 2.8 m doorway
clear; the hinge housing and lugs remain attached to the frame. Its exterior graphite plate
sits 25 mm proud of the teal backing to avoid depth flicker. Only the `Cockpit Glass` material uses
alpha blending. The renderer combines opaque geometry into one draw and all
window panes into a second draw; it preserves emissive color separately and
applies a warm fill to `Interior` descendants. There are no dynamic shadows,
post effects, additional light passes or per-frame geometry generation.

`Monitor_Center`, `Monitor_Port` and `Monitor_Starboard` are single quads, with
`TEXCOORD_0` `(0,0)` at the pilot-facing top-left and `(1,1)` at bottom-right.
All three surfaces sit on one center console and face the seated pilot. Node
extras retain `displayRole`, `uvOrigin`, and the shared-thruster-command power
source. Six triangles provide the complete live instrument surface without extra
ship draws. Keep the three stable monitor names because the renderer uses them.
`Port` and `Starboard` in those names identify the display positions on the
center console. The console collision proxy must fit the visible assembly while leaving
body-clear routes on both sides into the shortened nose. Exported bounds and
the portable character-controller footprints must agree.

`scenarios/cockpit-nose.json` walks the side route into and back out of the
nose. Its evidence variant captures the nose approach.

## Budgets and verification

Measured export counts and dimensions are recorded by `export-report.json`.
Hard caps: 6,000 triangles, 120 primitives, 13 materials, 512 KiB GLB; runtime ship
submission remains two draws. This is a deliberate increase from the initial
620-triangle greybox to allow the requested design detail, while remaining small.
The original texture is included for DCC interchange; the native ship shader
currently uses material colors and emission rather than sampling this texture.
No third-party models, textures or other external asset dependencies are used.

The validator checks exports agree, buffer/GLB structure, hierarchy, material
budgets, measured bounds, floor alignment, Core metadata and housing containment
inside its collider, shaped chair components, monitor planes/normals/UVs and
pilot-facing orientation, the unified console collider and body-clear routes
on both sides, unobstructed seated-eye rays over the three displays, seat height
and actual triangle ray intersections for forward seated/standing and side/rear
window sightlines, retained downward glazing, unoccluded recessed engine emitters,
and source-aligned thruster proxies in both exports and the controller. Rust
tests cover import/emission, cabin walking, Core containment, doorway
transitions, cockpit access and collision radius.

Native smoke validation should include walking around both sides of the Core,
viewing forward from behind the chair, inspecting side and rear windows,
opening/closing and crossing the aft doorway, boarding/leaving the seat, inspecting
all three live instruments on the center console while changing throttle/speed,
following the side route into the cockpit nose, inspecting seated downward views,
jumping, and walking outside to inspect both nozzles and the streamlined
silhouette without a cargo module. Also run
the normal Solar System/resize/minimize/relaunch checks from the root README.
The fixed 1920×1080 60 FPS acceptance check still requires the reference Apple
M1 iMac; asset complexity alone does not establish that result.
