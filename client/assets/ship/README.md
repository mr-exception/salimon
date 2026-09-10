# Salimon Phase 0 Scout

This directory contains the custom spaceship asset produced for Task 7. It is a
scope-limited polished greybox: a 13.4 m scout with a tapered ceramic nose,
graphite lifting body, copper edge accents, paired engine pods, cockpit canopy,
and a deliberately readable wing silhouette.

The interior has a 3.1 m-wide walkable deck, cockpit and pilot seat, center and
side monitors, a small aft cabin/corridor, storage and bench forms, ceiling
lights, and an interactive rear exit door. Task 8 owns character movement,
physics, door behavior, and runtime integration.

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

The runtime export contains 620 triangles, 41 single-primitive meshes, nine
reused PBR materials, one 16×16 texture, and no animation, skin, morph target, or
external dependency. The GLB remains below the 256 KiB Task 7 budget. The model
uses opaque materials and simple geometry suited to the Phase 0 60-FPS target;
Task 11 owns measured 1920×1080 performance evidence on the reference M1 MacBook
Air after runtime integration.

## Visual and import checks

The validator checks GLB chunk structure, buffer ranges, required hierarchy and
gameplay markers, unique node names, matching glTF/GLB contents, primitive
attributes, PNG signature, and all declared budgets. After geometry edits, also
import the glTF or GLB into the target DCC/runtime and visually check the outer
silhouette, central interior clearance, cockpit readability, rear door, normals,
and material assignments.
