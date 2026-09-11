# Salimon Phase 0 Scout

This directory contains the custom spaceship asset produced for Task 7 and
uniformly enlarged by Task 10. It is a scope-limited polished greybox: a
20.30 m-long, 7.44 m-tall, 16.60 m-wide scout with a tapered ceramic nose,
graphite lifting body, copper edge accents, paired engine pods, framed cockpit
glazing, and a deliberately readable wing silhouette.

The Task 7 mesh bounds were 10.15 × 3.72 × 8.30 m (length × height × width).
Task 10 applies an exact 2.0 uniform linear scale to all render geometry and
collision proxies. The enlarged interior has a 6.20 m-wide walkable deck and
4.66 m of vertical clearance. Player eye height remains 1.62 m rather than
scaling with the ship; the regenerated player, seat, door, and camera anchors
are placed against the scaled floor and fixtures so the interior stays usable.

The interior contains a cockpit and pilot seat, center and
side monitors, a small aft cabin/corridor, storage and bench forms, ceiling
lights, and an interactive rear exit door. The solid nose stays below the
console and the open-backed, double-sided canopy gives the seated eye at
`[2.76, 2.77, 0.0]` and a standing eye at `[1.30, 2.08, 0.0]` clear forward and
side sightlines. Task 11 owns character movement, physics, door behavior, and
runtime integration.

## Files and regeneration

- `source/generate_salimon_phase0_ship.py` is the deterministic editable source.
  It contains named component dimensions and uses no external assets or Python
  packages. Edit it and run `python3` to regenerate every checked-in export.
- `export/salimon_phase0_ship.gltf` plus its `.bin` is the readable/editable glTF
  2.0 interchange source and can be imported into Blender.
- `export/salimon_phase0_ship.glb` is the self-contained runtime export.
- `textures/salimon_floor_grip.png` is an original 16×16 procedural texture.
- `asset-manifest.json` fixes units, axes, ownership, and shipping budgets.

From the repository root:

```sh
python3 client/assets/ship/source/generate_salimon_phase0_ship.py
python3 client/assets/ship/source/validate_salimon_phase0_ship.py
```

The optional preview renderer needs Pillow and writes only the requested image:

```sh
python3 client/assets/ship/source/render_salimon_phase0_ship_preview.py /tmp/salimon-ship-preview.jpg
```

## Runtime contract

The asset uses meters, `+Y` up, `+X` ship-forward, and `-Z` starboard. Mesh nodes
have baked geometry and identity transforms. Stable hierarchy groups separate
`Exterior`, `Interior`, `Collision_Proxies`, and `Interaction_Markers`.

Collision nodes are metadata-only boxes so they add no draw calls. They define
the interior floor, side walls, ceiling, aft door, and one coarse exterior hull.
Interaction markers identify the cockpit seat, exit door, and player start.
Their `extras.salimon` metadata is the handoff contract for later import code.
The runtime places the landed ship so its scaled lowest mesh point (`-0.20 m`
local Y) rests on Earth's nominal surface; the identity orientation keeps local
`+Y` aligned to the surface normal at the starting point.

The runtime export remains below the fixed triangle, primitive, material, and
256 KiB GLB budgets. Its only transparent surface is the `Cockpit Glass`
material: a double-sided alpha-blended pane rendered in one additional ship draw
without shadows or post effects. All other materials remain opaque. The scale
pass adds no primitives, triangles, materials, textures, or draw calls, so its
GPU workload is unchanged apart from projected pixel coverage. The fixed
1920×1080 smoke check below remains required on the reference M1 MacBook Air for
the Phase 0 60 FPS acceptance target.

## Visual and import checks

The validator checks GLB chunk structure, buffer ranges, required hierarchy and
gameplay markers, unique node names, matching glTF/GLB contents, primitive
attributes, PNG signature, and all declared budgets. After geometry edits, also
import the glTF or GLB into the target DCC/runtime and visually check the outer
silhouette, central interior clearance, seated and standing cockpit sightlines,
rear door, normals, and material assignments.

For the Task 10 native smoke check, run the release client at a fixed 1920×1080
window, enable F3, and walk the cockpit, corridor, and open doorway. Confirm the
camera does not clip severely at the 0.05 m near plane; the cockpit/door prompts
activate at their enlarged fixtures; the player can exit and re-enter; the
lowest exterior point rests on Earth without a visible gap or penetration; and
the overlay never reports below 60 FPS during the interior and exterior passes.
