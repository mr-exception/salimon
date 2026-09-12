# Salimon Phase 0 Scout

A custom, editable 20.30 × 4.00 × 20.00 m scout (length × height × width)
with a broad living cabin, ivory armor, graphite structure, copper seams,
layered wing plates and paired aft ion thrusters. The cabin deck is 9.20 m wide,
up from 6.20 m, with 2.56 m floor-to-ceiling clearance. Deep window sills and
furniture narrow the actual walking lanes; the character controller accounts
for the player's 0.24 m radius.

Three tall window bays on each side and two aft observation panes surround the
living area. The forward canopy has no central sightline mullion. Its low console
and 1.505 m seat back leave clear forward rays from both the seated eye
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
ship. Its representation is visual only in Phase 0; no energy, damage, fuel, or
repair simulation is added.

The aft engines have hollow flared nozzles, recessed emitters, stepped liners,
armor rings, cooling fins and running lights. Their forms use 8–12 sided
cross-sections instead of dense smoothing or displacement.

## Source and regeneration

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
- [Core close-up](previews/core-hero.png), [reverse detail](previews/core-detail.png)
  and [cabin context](previews/core-cabin.png) show the redesigned Energy Core.

```sh
python3 client/assets/ship/source/generate_salimon_phase0_ship.py
python3 client/assets/ship/source/validate_salimon_phase0_ship.py
python3 client/assets/ship/source/render_salimon_phase0_ship_preview.py /tmp/salimon-ship-preview.jpg
python3 client/assets/ship/source/render_salimon_phase0_ship_preview.py --core client/assets/ship/previews
```

The optional preview requires Pillow. It renders the actual source geometry;
asset previews do not replace native runtime or reference hardware checks.

## Runtime contract

Units are meters; `+Y` is up, `+X` is forward, `-Z` is starboard. Meshes have baked
positions and identity transforms. `Exterior`, `Interior`, `Collision_Proxies`
and `Interaction_Markers` are stable hierarchy groups. Geometry is authored at
Task 7 scale then converted by `[2.0, 4.0 / 3.72, 2.0]`; this revision reshapes the
cabin and wings, so its dimensions are no longer a pure scale of Task 7.

Metadata-only collision boxes describe the floor, walls, ceiling, central Core,
door and coarse exterior. They add no draw calls. The gameplay controller still
owns matching portable constants; changing these dimensions requires updating
character/runtime contract tests together. The visible deck and collision floor
now agree at local Y `0.2473118 m`. The human is 1.80 m tall with 1.75 m eye height.
Spawn and cockpit exit use the clear starboard aisle `[0.50, 1.9973, -2.20]`.
The landed ship rests at its lowest mesh point (`-0.1075269 m` local Y), and its
conservative exterior collision radius is 15 m.

Only `Exit_Door` moves for the door state. Only the `Cockpit Glass` material uses
alpha blending. The renderer combines opaque geometry into one draw and all
window panes into a second draw; it preserves emissive color separately and
applies a warm fill to `Interior` descendants. There are no dynamic shadows,
post effects, additional light passes or per-frame geometry generation.

## Budgets and verification

Current export: **4,476 triangles, 91 primitives, 13 materials**.
Hard caps: 4,500 triangles, 100 primitives, 13 materials, 512 KiB GLB; runtime ship
submission remains two draws. This is a deliberate increase from the initial
620-triangle greybox to allow the requested design detail, while remaining small.
The original texture is included for DCC interchange; the native ship shader
currently uses material colors and emission rather than sampling this texture.
No third-party models, textures or other external asset dependencies are used.

The validator checks exports agree, buffer/GLB structure, hierarchy, material
budgets, measured bounds, floor alignment, Core metadata and housing containment
inside its collider, seat height and actual
triangle ray intersections for forward seated/standing and side/rear window
sightlines, plus unoccluded recessed engine emitters. Rust tests cover import/emission, widened walking, core/furniture
containment, doorway transitions, cockpit access and collision radius.

Native smoke validation should include walking around both sides of the Core,
viewing forward from behind the chair, inspecting side and rear windows,
opening/closing and crossing the aft doorway, boarding/leaving the seat, jumping,
and walking outside to inspect both nozzles and the wider silhouette. Also run
the normal Solar System/resize/minimize/relaunch checks from the root README.
The fixed 1920×1080 60 FPS acceptance check still requires the reference
MacBook Air M1; asset complexity alone does not establish that result.
