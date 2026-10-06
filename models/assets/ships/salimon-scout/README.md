# Salimon scout — Blender visual source

`assembly/salimon-scout.blend` is the editable Blender source for the current scout. It contains
the authored visual geometry, metadata-only colliders and interaction empties.
The original floor texture is packed. This is original Salimon geometry under
the repository license, with no third-party model dependencies. The current
revision removes the protruding port cargo module, cabin bench and storage,
shortens the cockpit nose and glazing, and places three monitors on one
center console. Body-clear routes on both sides reach the nose from the cabin.

`assembly/salimon-scout.blend` is authoritative for visual geometry and materials (#80).
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

Open `assembly/salimon-scout.blend` in Blender to inspect the complete ship.
Edit geometry in the owning `components/<component>/source.blend` instead; the
assembly uses direct linked Collections, so save the component and reload its
library (or reopen the assembly) to see the change. Do not append/make local or
create collection instances in the saved assembly: that would break ownership or
introduce extra runtime nodes. Each component opens with its editable local
objects and linked shared hierarchy/materials. All use the same ship-local frame.

| Component | Ownership |
| --- | --- |
| hull | Hull panels, spine, roof details, broad hull and cabin exterior envelope |
| cockpit | Cockpit and cabin glazing, window frame, nose-floor collider and cockpit side-hull proxies |
| wings | Both wings, armor/accents and their colliders |
| engines | Both pods, fins, intake/nozzle details and engine colliders |
| exit-door | Moving door, frame/threshold/light, door collider, exit marker and doorway transition marker |
| pilot-seat | Seat, chair collider and cockpit-seat marker |
| cockpit-console | Console, monitors, housings, controls and console collider |
| energy-core | Core, pedestal, details and core collider |
| cabin-interior | Deck, walls, ceiling, trim/lights, cabin colliders, traversal envelope and spawn marker |

`assembly/shared.blend` owns the root/group empties and the 13 shared materials
(including the packed floor image). Components link these dependencies through
relative paths, so each parent/material has one identity in the assembly.
Open that file to deliberately edit shared materials; preserve their contracts.
Keep the entire asset folder together when moving/copying it. Existing node
parenting is independent of Collection organization and is preserved verbatim.
New objects must belong to their owner's `Scout_<component>` Collection and
parent to the appropriate shared runtime group. Keep small details with their
owner. The runtime still loads one GLB; no runtime modular loading is added.

The export report hashes every component, shared library and assembly as well as
the runtime GLB. After any source edit, export and run the checks below. Test
portable library paths and independent edit propagation without changing saved
sources with:

```sh
blender --background --python-exit-code 1 \
  --python models/assets/ships/salimon-scout/verify_modular.py
```

The check copies the asset to a temporary folder, changes one hull vertex,
saves only that component, then verifies the assembly sees the change and every
other mesh remains unchanged. The sources use Blender 4.5.3 LTS.

Mesh objects retain separate materials and
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
The four markers remain empties. Empty cube display glyphs help find colliders
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

See [migration evidence](../../../../reports/issue-80/README.md) for the complete
triangle comparison and recorded validation limits.
Original bootstrap GLB SHA-256:
`926c93570a48dae8c66b1721a9183d70c69c3f3d3b15ff92cbbae9d2fe3eb312`.

## Geometry and gameplay policy

The scout adapter emits one `client/character/src/spatial_contracts.rs`, replacing
`ship_anchors.rs` and `thruster_collision.rs`. Raw bounds use
`[forward min, forward max, up min, up max, port min, port max]` in runtime meters.
Anchors and selected traversal boxes plus engine/wing arrays come from the same
20-box/four-marker inventory as the sidecar and embedded GLB contract. Rustfmt
must be on PATH for offline export/validation of the generated Rust module.
Normal runtime builds consume checked-in artifacts without Blender or Python.

| Authored source | Character consumer |
| --- | --- |
| InteriorFloor upper Y / forward maximum | Floor height / broad deck end |
| CockpitNoseFloor bounds | Narrow floor end/width; derived shoulder footprints |
| CabinTraversalEnvelope | Aft inner stopping plane, sill-limited width, lowest lamp clearance |
| CabinExteriorEnvelope | Cabin/nose exterior contact, height overlap and sight shells; excludes appendages |
| AftDoor upper Y / side width | Lintel height / gate aperture |
| DoorwayTransition marker X | Inner doorway crossing plane |
| EnergyCore, PilotChair, CockpitCenterConsole | Body-expanded planar fixture footprints |
| CockpitPortHull / CockpitStarboardHull | Conservative solid side-hull footprints |
| Engine/wing boxes | Exterior body contact and sight obstruction |
| Seat/start/exit markers and engine centers | Existing public anchors |

Traversal and exterior envelopes are intentional coarse authored geometry,
not automatically extracted from the full visual mesh. They preserve the prior
walking/doorway limits, including projecting sills and low fixtures. Keep them
aligned when editing those visuals. The full `ExteriorHull` contains appendages
and roof details and cannot replace the cabin-only envelope. Nose/cabin floors
must remain level; widths used by centered traversal must stay symmetric.
Required names, positive finite sizes, finite transforms/bounds, nonvisual roles,
identity parent frames and axis-aligned applied transforms are checked at export.

`client/character/src/layout.rs` owns radius expansion/inset, conservative planar
projection, gate splitting, shoulder composition and ceiling-height clamping.
Player dimensions, seated eye offset, view pitch/sensitivity, gravity, timing,
door permissions, movement/carrying rules and interaction ranges remain Rust
policy. Existing fixture sight occlusion still spans floor to ceiling; the raw
fixture Y bounds do not change that conservative query rule in this migration.
