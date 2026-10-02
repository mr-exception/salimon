# Salimon scout — Blender visual source

`source.blend` is an editable Blender 4.5.3 LTS import of the checked-in v11
scout GLB. All 150 named objects, 120 visual meshes/primitives, 5,938 triangles,
13 materials, metadata-only colliders and interaction empties are retained.
The original floor texture is packed. This is original Salimon geometry under
the repository license, with no third-party model dependencies.

`source.blend` is now authoritative for visual geometry and materials (#80).
The runtime GLB remains at its existing destination. Export through the scout
adapter, which uses the shared headless Blender exporter, validates a temporary
GLB with shared budgets and the preserved ship contracts, then publishes the
GLB and matching interchange files:

```sh
python3 models/assets/ships/salimon-scout/export.py --blender /path/to/blender
```

Install `models/tools/requirements.txt` in the authoring Python environment.
`export-report.json` records source/runtime SHA-256 and measured budgets. All
checks finish before replacing runtime files; a failed export keeps the prior
GLB, interchange and report. Blender is optional for normal Cargo builds.

## Open, edit, and verify

Open `source.blend` in Blender. Mesh objects retain separate materials and
editable vertices; parented empties retain the export hierarchy. The saved scene
uses metric units with scale 1, +Z up and +X forward. Blender's glTF importer
applies the inverse runtime mapping `(x,y,z) -> (x,-z,y)`; a later Y-up export
maps source `(x,y,z) -> (x,z,-y)` exactly once. Do not add another axis rotation.
The origin is the original ship-local reference frame, not a planetary position.

Inspect the original import after saving/reopening:

```sh
blender --background --python-exit-code 1 \
  --python models/assets/ships/salimon-scout/bootstrap.py -- --verify
python3 client/assets/ship/source/validate_salimon_phase0_ship.py
python3 -m unittest discover -s models/tests -v
```

The Blender check compares the saved source against the current interchange: names, hierarchy, exact node extras, translated
anchors, mesh geometry in meters, triangle/material assignments, monitor UVs
and packed images with the legacy interchange. Verify after exporting intentional edits. Bootstrap without `--verify` is retired
after this migration and cannot overwrite an existing source.
Blender is never required by Cargo or normal client builds.

## Preservation contract

[preservation.json](preservation.json) freezes **every node name**, parent,
transform, custom extra, asset metadata and material definition from the current import.
`manifest.json` uses the generic v1 schema and the explicit `legacy-scout-v1`
profile. Do not rename this ship to the new `ASSET_`/`Visual` conventions.

| Contract | Preserve during migration |
| --- | --- |
| Root/hierarchy | `Salimon_Phase0_Scout` with `Exterior`, `Interior`, `Collision_Proxies`, `Interaction_Markers`; interior ancestry controls warm renderer fill |
| Dimensions/pivot | Runtime +Y up, +X forward, -Z starboard; 20.9 × 4 × 21 m envelope; lowest local Y -0.1075268817 m; deck Y 0.247311828 m |
| Door | `Exit_Door` is the only door-state moving mesh; preserve `salimon.interactive=exit-door` and runtime-frame `pivot` metadata; do not bake a new door pivot |
| Glass | `Cockpit Glass` is the only alpha-blended, double-sided material; all glazing retains its exterior-visibility metadata and sightlines |
| Rendering | One combined opaque draw and one glass draw; preserve base colors, metallic/roughness, alpha, double-sided and emissive factors in the frozen material definitions |
| Monitors | `Monitor_Center`, `Monitor_Port`, `Monitor_Starboard`; each remains a two-triangle quad with TEXCOORD_0 at pilot-facing top-left (0,0), bottom-right (1,1) |
| Monitor metadata | Preserve actual legacy `displayRole`: center `speed`, sides `thruster-power`; `thrusterSide`: none/port/starboard; `uvOrigin=top-left`; side `powerSource=shared-thruster-command`; exact target/yaw in preservation.json |
| Monitor semantics | Runtime artwork additionally shows Core energy on port and nearby-body navigation on starboard; these UI roles do not replace the legacy extras values |
| Pilot/Core | `Pilot_Seat_Back`, `Core_Pedestal`, `Energy_Core` retain their interactive/purpose/design extras; energy metadata is still visual-only, not simulation policy |
| Markers | `MARKER_CockpitSeat`, `MARKER_ExitDoor`, `MARKER_PlayerStart` retain transforms and purpose/facing metadata |
| Budgets | 6,000 triangles, 120 primitives, 13 materials, 512 KiB GLB; imported runtime measures 5,938 / 120 / 13 / 513,824 bytes |

The seated eye is `[2.76, 1.7990322581, 0]` in runtime meters. Preserve screen
planes/normals, seatward monitor offsets, console collision, all unobstructed
monitor rays and lower cockpit/window sightlines checked by the legacy validator.
Blender UV coordinates flip V relative to glTF; the verification checks the
corresponding values, so do not visually flip the screens to compensate twice.

Nested custom `salimon` properties are preserved verbatim. Numbers inside extras
(door pivot, pilot-facing target, proxy sizes) stay in **runtime axes**: they are
metadata values, not Blender transforms, and the importer does not convert them.
Asset-level version/axes/source-workflow metadata is frozen in preservation.json;
it is not imported as object properties, so the later category export must
explicitly retain or update it. Node translations themselves are converted into
Blender axes. Keep that
distinction when editing and exporting.

## Spatial contracts and remaining migration

All 22 `COLLIDER_` names, translations, `collisionShape=box`, `sizeMeters` and
`purpose` values are enumerated in the manifest and preservation file. They are
legacy metadata-only empties, not newly authored solid proxies. The groups cover
cabin floor/walls/ceiling, furniture/Core, aft door, coarse hull, seven cargo
boundaries, four engine bounds and the center console. The three markers remain
empties. Empty cube display glyphs help find colliders but **do not describe
their actual box dimensions**; use their saved size metadata.

`client/character/src/cargo_layout.rs` and `thruster_collision.rs` remain
generator-owned; other gameplay constants and policy stay in Rust. The legacy
manifest also owns cargo volume/passage, instrument assemblies and sightline
samples. Read [the complete legacy runtime contract](../../../../client/assets/ship/README.md#runtime-contract)
and its validator before altering them.

The `legacy-scout-v1` profile explicitly validates legacy metadata boxes using
`collisionShape`/`sizeMeters`; standard assets still require
`shape`/`salimonProxyDimensions`. This preserves the old spatial contract without
pretending these empties are newly authored proxy meshes. The scout adapter
registers the ship extension and checks frozen names, hierarchy, transforms,
extras, material roles and factors. Standalone generic CLI export/validation
still requires an explicit category adapter; use the scout command above.

`runtime-metadata.json` preserves asset-level nonvisual metadata not imported
into Blender. It supplies ship dimensions, instrument/window policy and legacy
metrics; the adapter never generates visual geometry or replaces Blender
materials. Detailed legacy validation remains in place until #82. The old
Python CLI accepts only `--legacy-layout-only` and writes the two retained Rust
spatial layouts; it cannot replace Blender visual exports.

#81 moves spatial contracts into authored proxies/anchors, #82 removes
validation's procedural definitions, and #83 retires the generator. Do not
change frozen spatial/material contracts as part of a visual edit.

## Migration comparison

All 150 node names/hierarchy roles, 120 primitives, 5,938 oriented triangles,
13 material roles, monitor UVs, door metadata and spatial extras are preserved.
Blender changes node/material/accessor ordering and merges equivalent vertices;
normal encoding has minor DCC precision differences. No geometry, silhouette,
UV, budget or gameplay change is intentional. The GLB shrinks from 513,824 to
485,332 bytes. The interchange now includes the same embedded image buffer as
the GLB, with its binary externally referenced for editing.

See [migration evidence](../../../../docs/issue-80/README.md) for the complete
triangle comparison and recorded validation limits.
Original bootstrap GLB SHA-256:
`926c93570a48dae8c66b1721a9183d70c69c3f3d3b15ff92cbbae9d2fe3eb312`.
