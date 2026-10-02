# Salimon Phase 0 Scout

A custom, editable 20.90 × 4.00 × 21.00 m scout (length × height × width)
with a broad living cabin, ivory armor, graphite structure, copper seams,
layered wing plates and paired aft ion thrusters. The cabin deck is 9.20 m wide,
up from 6.20 m, with 2.56 m floor-to-ceiling clearance. Deep window sills and
furniture narrow the actual walking lanes; the character controller accounts
for the player's 0.24 m radius.

Three tall window bays on each side and two aft observation panes surround the
living area. The forward canopy has no central sightline mullion. Its low console
and 1.59 m seat back with a compact 1.73 m headrest leave clear forward rays from both the seated eye
`[2.76, 1.799, 0.0]` and standing eye `[1.30, 1.997, 0.0]`. All panes share
inexpensive, lightly tinted, double-sided glass. The aft hull contains a real
door aperture; opening the door does not leave a solid cap across the exit.

The interior uses warm ivory lining, copper lamp housings, terracotta cushions
and woven runners, petrol-teal cabinetry and honey-colored worktops. Amber
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

The pilot station has a faceted dashboard, cast teal side pods, recessed monitor
housings, ivory bezels, copper fasteners, tactile keys, service vents and deck
pedals. Three flat monitor surfaces receive live runtime instrument artwork:
the center displays speed and common thrust, the port panel shows Core energy,
and the starboard panel shows nearby-body distance/radial state. Both sides retain
the shared thruster power percentage. Both thrusters currently use the same
flight command.
Asset version 9 scales every monitor assembly to 70% of its version 8 size and
moves it 0.40 m toward the pilot seat. The center screen is 0.882 × 0.3332 m
(49% of version 7); side screens are 0.63 × 0.315 m. Housings, bezels, fasteners,
keys, indicators and mounting stems move with their screens. A low support bridge
connects the center stem to the unchanged dashboard; the side stems remain on
their consoles.
The pilot chair is a contoured bucket with a reclined tapered back, split
terracotta cushions, side bolsters, compact headrest, armrests, a short stick and
throttle-shaped hand control, plus a suspension pedestal on floor rails. The
hand controls are visual props; the existing flight input remains authoritative.
Eight-corner profile rings and shared materials provide shape without dense
smoothing or subdivision.

The aft engines have hollow flared nozzles, recessed emitters, stepped liners,
armor rings, cooling fins and running lights. Their forms use 8–12 sided
cross-sections instead of dense smoothing or displacement.

## Dedicated cargo module (#34)

Asset version 10 adds a port cargo room with a 5.4 × 5.5 m clear floor
(29.7 m²) and 2.56 m structural height. It has a level deck, enclosing walls,
ceiling lamps and painted loading lanes. The 1.8 m passage at local X
0.4–2.2 m connects directly to the cabin's port aisle, beyond the Core. Its
clear height is 2.253 m; player-body expansion leaves 1.32 m usable passage
width. The cabin and cargo room use the same ship-local movement frame and
internal gravity. The controller retains its conservative lamp/head clearance.

The port wing plates stop behind the module, and the port engine moves 0.60 m
aft to clear its wall. Cockpit, spawn, door anchors, floor height and existing
starboard walking routes remain aligned. The 16 m flight sphere contains every
exterior vertex; the familiar 15 m takeoff lift/touchdown stage stays unchanged.

`interior.cargoRoom` in the manifest and `extras.salimon.cargoRoom` in the
exports describe the real physical volume. The generator writes
`client/character/src/cargo_layout.rs` from room geometry and passage metadata;
validation checks this contract and every room collision proxy against both
exports. Triangle rays verify the doorway is free of old glazing/sills/hull and
the cargo deck is not covered by a wing. The room does not implement storage,
item transfer or cargo counting; those belong to dependent issues #47/#49.

`scenarios/cargo-room.json` uses real movement and interactions to exit/re-enter
the ship, reach the room, test its walls/partition, return to the airlock and
cockpit, take off, and revisit the room in flight. Its evidence variant captures
the exterior, cockpit, and room while landed and flying. See
[issue #34 evidence](../../../docs/issue-34/README.md).

## Lower cockpit glazing (#38)

Asset version 11 opens the lower nose, its cheek panes, and two structural glass
floor shoulders beside the center console. The deck and belly have matching
apertures. Copper rims identify the floor panes; the original deck collision
continues to support their load-bearing glass. The pilot station, monitor faces,
chair, controls, interaction markers and main canopy retain their transforms.

`Cockpit_Lower_Glazing` uses the existing lightly tinted, double-sided glass draw,
with alpha blending and no depth writes. Its exported outward normals include
upward floor panes and downward nose glazing. The seated eye can see through
both sides at 12/18 degrees down and through the floor shoulders at 26/30 degrees
down. The manifest records the validated pitch/yaw samples. Exported-triangle
rays must hit lower glazing and no opaque mesh; they cover the actual deck,
belly, consoles, trim and hull rather than only pane bounds. The asset remains
within 6,000 triangles, 120 primitives, 13 materials and the 512 KiB GLB budget.

The required native E2E visual scenario follows real free-look, assisted landing,
low-altitude approach, landed inspection, takeoff and cockpit exit. See
[issue #38 evidence](../../../docs/issue-38/README.md) for matching before/after
native screenshots and authoritative state/logs.

## Source and regeneration

The root [models workspace](../../../models/README.md) defines the future generic
Blender authoring boundary. This ship remains on the procedural source below
until #79–#83 migrate it in stages. Its runtime path, generator, validator, and
metadata contracts are unchanged; do not apply the new naming conventions by
renaming current ship nodes.

- `source/generate_salimon_phase0_ship.py` is the deterministic editable source,
  using only Python's standard library. Dimensions and named components remain
  outside the Rust renderer.
- `export/salimon_phase0_ship.gltf` plus `.bin` is the Blender-importable glTF 2.0
  interchange asset.
- `export/salimon_phase0_ship.glb` is the self-contained runtime export.
- `textures/salimon_floor_grip.png` is an original 16×16 procedural texture.
- `asset-manifest.json` records axes, scale, design dimensions and budgets.
- [preview.jpg](preview.jpg) shows six source-geometry views, including standing
  cockpit and rear cabin sightlines.
- [Pilot station](previews/cockpit-station.png), [chair detail](previews/pilot-chair.png)
  and [seated inspection](previews/cockpit-seated.png) show the pilot geometry.
  Their cyan faces are source surfaces; live instrument artwork comes from the
  runtime and is verified in the native game.
- [Core close-up](previews/core-hero.png), [reverse detail](previews/core-detail.png)
  and [cabin context](previews/core-cabin.png) show the redesigned Energy Core.

```sh
python3 client/assets/ship/source/generate_salimon_phase0_ship.py
python3 client/assets/ship/source/validate_salimon_phase0_ship.py
python3 client/assets/ship/source/render_salimon_phase0_ship_preview.py /tmp/salimon-ship-preview.jpg
python3 client/assets/ship/source/render_salimon_phase0_ship_preview.py --core client/assets/ship/previews
python3 client/assets/ship/source/render_salimon_phase0_ship_preview.py --cockpit client/assets/ship/previews
```

The optional preview requires Pillow. It renders the actual source geometry;
asset previews do not replace native runtime or reference hardware checks.

## Runtime contract

Units are meters; `+Y` is up, `+X` is forward, `-Z` is starboard. Meshes have baked
positions and identity transforms. `Exterior`, `Interior`, `Collision_Proxies`
and `Interaction_Markers` are stable hierarchy groups. Geometry is authored at
Task 7 scale then converted by `[2.0, 4.0 / 3.72, 2.0]`; this revision reshapes the
cabin and wings, so its dimensions are no longer a pure scale of Task 7. The
updated pilot station is authored directly in final meters by
`cockpit_components()` to retain human proportions independently of hull scale.

Metadata-only collision boxes describe the floor, walls, ceiling, central Core,
door, center console/monitor assembly, coarse exterior, and both thruster bodies
and swept fins. They add no draw calls. The four thruster boxes are measured from
the editable engine meshes at export time; regeneration also writes
`client/character/src/thruster_collision.rs`, and validation compares that
portable controller contract with both exports. Other gameplay bounds still
have matching portable constants; changing those dimensions requires updating
character/runtime contract tests together. The visible deck and collision floor
now agree at local Y `0.2473118 m`. The human is 1.80 m tall with 1.75 m eye height.
Spawn and cockpit exit use the clear starboard aisle `[0.50, 1.9973, -2.20]`.
The landed ship rests at its lowest mesh point (`-0.1075269 m` local Y), and its
conservative exterior collision radius is 16 m.

Only `Exit_Door` moves for the door state. Only the `Cockpit Glass` material uses
alpha blending. The renderer combines opaque geometry into one draw and all
window panes into a second draw; it preserves emissive color separately and
applies a warm fill to `Interior` descendants. There are no dynamic shadows,
post effects, additional light passes or per-frame geometry generation.

`Monitor_Center`, `Monitor_Port` and `Monitor_Starboard` are single quads, with
`TEXCOORD_0` `(0,0)` at the pilot-facing top-left and `(1,1)` at bottom-right.
The center normal faces `-X`; each side display and its complete physical assembly
yaws about 60.3 degrees toward the seated eye `[2.76, 1.799, 0.0]`. Node extras
record the exact pilot-facing target/yaw alongside `displayRole`, `uvOrigin`, and
the `shared-thruster-command` power source. Six triangles provide the complete
live instrument surface without extra ship draws.

`COLLIDER_CockpitCenterConsole` combines the unchanged dashboard with the smaller
monitor: its physical bounds are `[3.94662, 0.25, -1.06]` to
`[6.12, 1.23855, 1.06]`. Character collision expands this footprint by the player's
radius. `cockpitInstruments.centerAssembly` records the original and reduced
monitor bounds, uniform scale and fixed pivot in both exports and the manifest.

## Budgets and verification

Current export: **5,720 triangles, 119 primitives, 13 materials**, 497,628-byte GLB.
Hard caps: 6,000 triangles, 120 primitives, 13 materials, 512 KiB GLB; runtime ship
submission remains two draws. This is a deliberate increase from the initial
620-triangle greybox to allow the requested design detail, while remaining small.
The original texture is included for DCC interchange; the native ship shader
currently uses material colors and emission rather than sampling this texture.
No third-party models, textures or other external asset dependencies are used.

The validator checks exports agree, buffer/GLB structure, hierarchy, material
budgets, measured bounds, floor alignment, Core metadata and housing containment
inside its collider, shaped chair components, exact monitor planes/normals/UVs,
side-display yaw toward the authored pilot viewpoint,
the exact 0.49 transform and 0.40 m seatward translation of every center-monitor
assembly vertex, fitted side display geometry, unchanged dashboard geometry,
and the matching combined console collider,
fifteen unobstructed seated-eye rays over the three displays, seat height and actual
triangle ray intersections for forward seated/standing and side/rear window
sightlines, unoccluded recessed engine emitters, and source-aligned thruster proxies
in both exports and the controller. Rust tests cover import/emission, widened walking, core/furniture
containment, doorway transitions, cockpit access and collision radius.

Native smoke validation should include walking around both sides of the Core,
viewing forward from behind the chair, inspecting side and rear windows,
opening/closing and crossing the aft doorway, boarding/leaving the seat, inspecting
all three live instruments while changing throttle/speed, jumping,
and walking outside to inspect both nozzles and the wider silhouette. Also run
the normal Solar System/resize/minimize/relaunch checks from the root README.
The fixed 1920×1080 60 FPS acceptance check still requires the reference Apple
M1 iMac; asset complexity alone does not establish that result.
