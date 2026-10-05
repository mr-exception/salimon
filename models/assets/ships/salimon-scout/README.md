# Salimon scout — Blender visual source

`source.blend` is the editable Blender source for the current scout. It contains
the authored visual geometry, metadata-only colliders and interaction empties.
The original floor texture is packed. This is original Salimon geometry under
the repository license, with no third-party model dependencies. The current
revision removes the protruding port cargo module, cabin bench and storage,
shortens the cockpit nose and glazing, and places three monitors on one
center console. Body-clear routes on both sides reach the nose from the cabin.

`source.blend` is authoritative for visual geometry and materials (#80).
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

Compare the saved source against the current interchange after exporting edits:

```sh
blender --background --python-exit-code 1 \
  --python models/assets/ships/salimon-scout/verify_source.py
python3 models/assets/ships/salimon-scout/validate.py
python3 -m unittest discover -s models/tests -v
```

The Blender check compares the saved source against the current interchange: names, hierarchy, exact node extras, translated
anchors, mesh geometry in meters, triangle/material assignments, monitor UVs
and packed images with the matching interchange. This optional verification
helper is read-only; the one-time bootstrap importer is retired.
Blender is never required by Cargo or normal client builds.

## Preservation contract

[preservation.json](preservation.json) records the named hierarchy, transforms,
extras, asset metadata and material definitions that the exporter validates.
Update it deliberately when changing authored geometry or spatial contracts;
retired cargo-room nodes and old console positions must not be carried forward.
`manifest.json` uses the generic v1 schema and the explicit `legacy-scout-v1`
profile. Do not rename this ship to the new `ASSET_`/`Visual` conventions.

| Contract | Current requirement |
| --- | --- |
| Root/hierarchy | `Salimon_Phase0_Scout` with `Exterior`, `Interior`, `Collision_Proxies`, `Interaction_Markers`; interior ancestry controls warm renderer fill |
| Dimensions/pivot | Runtime +Y up, +X forward, -Z starboard; current envelope, lowest point and deck height are recorded in the manifests and spatial contracts |
| Door | `Exit_Door` is the only door-state moving mesh; it rotates 110 degrees outward and upward about its top hinge. Preserve `salimon.interactive=exit-door` and runtime-frame `pivot` metadata; do not bake a new door pivot |
| Glass | `Cockpit Glass` is the only alpha-blended, double-sided material; all glazing retains its exterior-visibility metadata and sightlines |
| Rendering | One combined opaque draw and one glass draw; preserve base colors, metallic/roughness, alpha, double-sided and emissive factors in the frozen material definitions |
| Monitors | `Monitor_Center`, `Monitor_Port`, `Monitor_Starboard` share one center console; their stable names denote display roles rather than new physical sides; each remains a two-triangle quad with TEXCOORD_0 at pilot-facing top-left (0,0), bottom-right (1,1) |
| Monitor metadata | Preserve the renderer-consumed `displayRole`, `thrusterSide`, `uvOrigin=top-left` and shared-thruster-command metadata; update the authored pilot-facing target/yaw with the new positions |
| Monitor semantics | Runtime artwork additionally shows Core energy on port and nearby-body navigation on starboard; these UI roles do not replace the legacy extras values |
| Pilot/Core | `Pilot_Seat_Back`, `Core_Pedestal`, `Energy_Core` retain their interactive/purpose/design extras; energy metadata is still visual-only, not simulation policy |
| Markers | `MARKER_CockpitSeat`, `MARKER_ExitDoor`, `MARKER_PlayerStart` retain transforms and purpose/facing metadata |
| Budgets | 6,000 triangles, 120 primitives, 13 materials, 512 KiB GLB; current measured values are in `export-report.json` |

Preserve pilot-facing screen planes/normals, clear monitor rays, the lower
cockpit/window sightlines and the walking routes on both sides checked by the scout
category validator. Keep the console collider fitted to the visible assembly
and the player body clear of it, the floor and the shortened nose glazing.
Keep `Cockpit_Window_Frame` narrow while its edges follow the glazing aperture.
Blender UV coordinates flip V relative to glTF; the verification checks the
corresponding values, so do not visually flip the screens to compensate twice.

Nested custom `salimon` properties are preserved verbatim. Numbers inside extras
(door pivot, pilot-facing target, proxy sizes) stay in **runtime axes**: they are
metadata values, not Blender transforms, and the importer does not convert them.
Asset-level version/axes/source-workflow metadata is recorded in
`preservation.json`; it is not imported as object properties, so the category
export must explicitly retain or update it. Node translations themselves are
converted into Blender axes. Keep that distinction when editing and exporting.

## Spatial contracts

All current `COLLIDER_` names, translations, `collisionShape=box`, `sizeMeters` and
`purpose` values are enumerated in the manifest and preservation file. They are
legacy metadata-only empties, not newly authored solid proxies. The groups cover
cabin floor/walls/ceiling, Core, aft door, coarse hull, engine bounds
and the unified center console. The retired cargo-room boundaries are absent.
The three markers remain empties. Empty cube display glyphs help find colliders
but **do not describe their actual box dimensions**; use their saved size metadata.

The adapter generates portable spatial Rust layouts from Blender-authored
proxy/marker nodes. `client/assets/ship/spatial-contracts.json` is the matching
runtime sidecar. Seat, exit, spawn and engine anchors are sourced from authored
spatial transforms while gameplay rules remain in Rust. The legacy manifest
owns instrument assemblies and sightline samples, with no dedicated cargo-room
volume. Read [the complete runtime contract](../../../../client/assets/ship/README.md#runtime-contract)
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
materials. Detailed validation lives in `validate.py`, registered as the
`ships/ship/v1` extension during export. It reuses shared envelope, accessor,
hierarchy and budget checks and measures exported vertices for dimensions,
monitor planes/UVs, collision envelopes and sightlines. `monitor-contract.json`
records the three authored monitor quads independently of DCC vertex ordering;
update it deliberately when the display faces change. No validator imports
procedural geometry. The compatibility command in `client/assets/ship/source/`
forwards here. Spatial Rust layouts are generated by the scout adapter.

The migration (#79–#83) is complete. Blender source, shared export/validation,
and this category adapter are the supported ship-authoring path. When a visual
revision changes spatial contracts, update their consumers and tests together.

## Migration comparison

The #80 migration preserved 150 node names/hierarchy roles, 120 primitives,
5,890 oriented triangles, 13 material roles, monitor UVs, door metadata and
spatial extras at the time of migration. That migration changed
node/material/accessor ordering and merged equivalent vertices, with minor DCC
normal-encoding differences. It intentionally kept the old geometry, silhouette,
UVs and gameplay behavior while shrinking the GLB from 513,824 to 485,332 bytes.
The current design revision changes that historical geometry and removes the
dedicated cargo module.

See [migration evidence](../../../../docs/issue-80/README.md) for the complete
triangle comparison and recorded validation limits.
Original bootstrap GLB SHA-256:
`926c93570a48dae8c66b1721a9183d70c69c3f3d3b15ff92cbbae9d2fe3eb312`.
