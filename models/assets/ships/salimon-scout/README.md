# Salimon scout — Blender visual source

`assembly/salimon-scout.blend` is the editable Blender source for the current scout. It contains
the authored visual geometry, metadata-only colliders and interaction empties.
The original floor texture is packed. This is original Salimon geometry under
the repository license, with no third-party model dependencies. The current
revision removes the protruding port cargo module, cabin bench and storage,
shortens the cockpit nose and glazing, and places one wide multifunction monitor on the
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
| hull | Hull panels, spine, roof details, broad hull/cabin envelope, fixed landing legs/feet and their proxies |
| cockpit | Cockpit and cabin glazing, window frame, nose-floor collider and cockpit side-hull proxies |
| wings | Both wings, armor/accents and their colliders |
| engines | Both pods, fins, intake/nozzle details and engine colliders |
| exit-door | Moving door, frame/threshold/light, door collider, exit marker and doorway transition marker |
| pilot-seat | Seat, chair collider and cockpit-seat marker |
| cockpit-console | Console, monitors, housings, controls and console collider |
| energy-core | Core, pedestal, details and core collider |
| cabin-interior | Deck, walls, ceiling, trim/lights, wall Fabricator, cabin colliders, traversal envelope and authored markers |

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
| Door | `Exit_Door` rotates down/out about its bottom edge; `Exit_Ramp_Tip` then slides out to form the 4.4 m boarding ramp. Preserve authored runtime-axis pivot/angle/travel metadata; the exporter generates character and renderer ramp dimensions |
| Glass | `Cockpit Glass` is the only alpha-blended, double-sided material; all glazing retains its exterior-visibility metadata and sightlines |
| Rendering | One combined opaque draw and one glass draw; preserve base colors, metallic/roughness, alpha, double-sided and emissive factors in the frozen material definitions |
| Monitor | One `Monitor_Wide` multifunction quad on the centered console; two triangles, top-left (0,0) UVs, bottom-right (1,1); old monitor nodes are retired |
| Monitor metadata | Preserve the renderer-consumed `displayRole`, `thrusterSide`, `uvOrigin=top-left` and shared-thruster-command metadata; update the authored pilot-facing target/yaw with the new positions |
| Monitor semantics | Runtime artwork additionally shows Core energy on port and nearby-body navigation on starboard; these UI roles do not replace the legacy extras values |
| Pilot/Core | `Pilot_Seat_Back`, `Core_Pedestal`, `Energy_Core` retain their interactive/purpose/design extras; energy metadata is still visual-only, not simulation policy |
| Markers | `MARKER_CockpitSeat`, `MARKER_ExitDoor`, `MARKER_PlayerStart` retain transforms and purpose/facing metadata |
| Budgets | 6,500 triangles, 124 primitives, 13 materials, 544 KiB GLB; current measured values are in `export-report.json` |

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
The seven markers remain empties. Empty cube display glyphs help find colliders
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
records the single authored wide monitor quad independently of DCC vertex ordering;
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
28-box/four-marker inventory as the sidecar and embedded GLB contract. Rustfmt
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

## Landing support (#168)

Four fixed deployed supports belong to `components/hull/source.blend`, centered
at runtime (X,Z) = (-5.6, ±3.6) and (4.8, ±3.6) m. Each has a reinforced mount,
telescopic column, copper compression collars and a 1.35 × 1.10 m broad sole.
The feet lie 2 m below the lowest non-gear mesh. They remain deployed in flight;
no animations, new materials, runtime nodes per frame or extra draws are added.
Eight finite boxes cover feet and columns, consumed by movement and sight/placement.

The exporter measures hull/foot minimum Y, rejects a clearance other than 2 m,
and generates `client/ship/src/landing_geometry.rs`. Both Earth spawn and assisted
landing use the negative foot Y as their origin altitude. The existing coarse
hull proxy retains its non-gear bounds; it must not become a solid box spanning
all the space below the hull. Catalog sphere curvature over the support footprint
and f64 anchor quantization stay below 1 mm; terrain adaptation is future work.

The aft threshold top is local Y = 0.275 m, therefore 2.382527 m above
nominal ground at rest (the deck is 2.354839 m above ground). Reserve the centered aft gate
and its outward approach for #169's bottom-hinged ramp. The #169 ramp now supplies continuous sloping contact through that corridor,
with closed-door gating and an occupied-sweep interlock. Under-hull walking remains
free away from the solid supports; the legs do not obstruct cabin routes/glazing.

## Boarding ramp (#169)

The authoritative exit-door component has a flush cabin threshold at Y
0.24731182796 m, bottom pivot X -7.68 m, 2.268817 m sealed leaf, and nested
2.131183 m travel tray with 0.137635 m overlap at full deployment. Its 4.4 m
support plane descends 2.354839 m at 32.36 degrees below horizontal to the fixed
landing foot plane. Full leaf rotation is 122.36 degrees, not the old 110-degree
upward swing. Closed tray geometry is concealed within the pressure leaf.
The inner faces align at full extension; trim/treads stay behind the walking plane.
The 2.8 m width retains 1.16 m center clearance on either side for a 0.24 m body.

`author_ramp.py` is the repeatable Blender editing recipe for this component.
Run it with `blender --background --python-exit-code 1 --python
models/assets/ships/salimon-scout/author_ramp.py`, then the normal scout exporter.
`boardingRamp` in the spatial sidecar comes from `Exit_Door.salimon`; the exporter
writes both character and renderer `ramp_geometry.rs`. Neither consumer hard-codes
a second set of dimensions. Validation checks tip contact, walking-face alignment,
leaf/tray overlap, flush threshold, pivot and the unchanged draw policy.

A 700 ms reversible pose parameter lowers the leaf during the first 60% and
extends the tray during the final 40%, using smoothstep in each phase. Reversal
retracts the tray before raising the leaf. The gate is walkable only at 100%.
Nominal spherical surfaces differ from the tangent foot plane by micrometers at
this footprint. Terrain-aware ramp endpoints remain dependent on future terrain
support; this contract targets the current nominal spherical bodies.

## Wide cockpit monitor (#171)

`author_monitor.py` records the deliberate console geometry edit (Blender 4.5
background script); the saved `components/cockpit-console/source.blend` remains
editable and authoritative. The continuous rim and rear housing surround one
1.98 × 0.3332 m pilot-facing quad. Existing controls, vents, fasteners and mounts
stay inside the unchanged console collision proxy; both side lanes and forward/
lower glazing remain clear. `monitor-contract.json` freezes the four final
runtime corners. Preservation, manifest and runtime metadata use only
`Monitor_Wide` with multifunction semantics. Export then run source/modular,
asset, renderer and character checks as above; inspect native screenshots in
[issue #171 evidence](../../../../reports/issue-171/README.md).

## Core-local monitor (#194)

`author_core_monitor.py` edits the energy-core component in Blender 4.5.3.
`Core_Monitor` is a 1.30 × 0.65 m starboard-facing UV quad inside the existing
Core collider, reusing `Cyan Display`; its dark margin is part of the live atlas.
Export/verify source and modular ownership using the commands above. The
validator checks exact corners, facing, UV fit and a two-triangle budget. The
runtime renders live energy/consumption/conversion/depletion/warnings here,
independently of the cockpit console. It blanks at outage; mechanical intake
stays usable. Apple M1 visual signoff belongs to the separate native work.

### Physical Core display frame (#294)

The same recipe now adds a 150 mm deep graphite enclosure with a slate rim,
24 mm clearance behind the unchanged live quad, and two ochre supports seated
into the pedestal. Everything stays inside the existing Core collision box;
the forward service/feeding face remains clear. The opening leaves 3 mm around
the display and the rim stays 1 mm ahead of its original starboard plane.

`Core_Monitor_Frame` vertex groups select the editable additions in
`Core_Pedestal` (housing), `Core_Ceramic_Armor` (rim), and `Core_Cradle` (supports).
Reauthoring replaces only those grouped vertices. Reusing the owners' materials
keeps 140 nodes, 116 primitives, 13 materials and one opaque/one glass draw;
the enclosure adds 48 triangles (6,329 total). No preservation, collider, marker
or renderer interface changes are required. Category validation traces display
corners from three approaches and probes the physical frame/back/supports.
See [authored previews and verification](../../../../reports/issue-294/README.md).

## Surface remediation (#244)

Different-material coplanar panels now have one surface owner; lower glazing is
trimmed against opaque framing. Six owning libraries were edited with assembly
links intact. All spatial contracts and monitor UVs are preserved.
Current export: 6,281 triangles, 116 primitives, 13 materials, 525,356 bytes.
The [named-pair report](../../../../reports/issue-244/README.md) explains retained
internal joins and articulated contact caps; native M1 signoff remains #229.

## Option A pass (#276)

The [art bible](../../../../docs/visual-style.md) owns direction. Shared materials
now use graphite/slate, ochre mechanical accents and warm neutral interior panels.
Broad roof/ceiling cyan panels reuse Hull Ceramic; live status/display/glow, glass
and floor-texture roles remain unchanged. Eight owning libraries explicitly use
flat opaque panel shading while soft seat padding retains its functional normal
policy. No vertices, UVs, dimensions, pivots, metadata or collision changed.
Existing broad hull/wing/engine/gear silhouettes already meet the primary-form
policy and are intentionally retained; no extra bevels or micro geometry added.

Reapply `author_option_a.py` with Blender 4.5.3, then export through the normal
adapter and run source/modular/category checks. The recipe is visually idempotent;
Blender save metadata/file hashes can change on another save. `styleProfile`
opts this asset into #275 advisory measurements; all previous hard caps remain.
The export report records measured costs and source hashes. [Before/after images
and costs](../../../../reports/issue-276/README.md) are review candidates, not
approved GPU baselines. Approve the #274 reference and this production pass in
order; native M1 evidence remains #279 (coordinated with #229).

## Fabricator wall module (#342)

`cabin-interior` owns the Fabricator (original project geometry/license). Its
2.4 × 1.9 × 1.3 m conservative proxy sits at ship-local (-3.15, 1.45, 3.75).
The port, monitor and one-item tray use `MARKER_FabricatorInput`,
`MARKER_FabricatorMonitor` and `MARKER_FabricatorTray`; generated Rust constants
are the runtime authority. A generic opaque parcel is gated by tray occupancy
in the existing ship draw; it appears without animation at completion. Native
UI/artwork and inventory are separate #343/#345 integrations.

Reproduce the reviewed parametric edit with Blender 4.5.3:

```sh
blender -b --python-exit-code 1 --python models/assets/ships/salimon-scout/author_fabricator.py
python models/assets/ships/salimon-scout/export.py --blender /path/to/blender
blender -b --python-exit-code 1 --python models/assets/ships/salimon-scout/preview_fabricator.py
```

The author script only replaces named Fabricator objects in the cabin library;
shared materials/hierarchy and assembly links stay linked. Preview cameras and
lights are read-only and never saved into source. The preview shows an occupied
tray; the game gates the same parcel by actual domain state.

The node budget deliberately grows 140→151 for seven visual nodes, three
markers and one proxy. Existing hard triangle/primitive/material/GLB limits
stay 6500 / 124 / 13 / 557056 bytes; export measures 6493 / 123 / 13 / 549060.
The collision and output-support checks, overlap checks, named interfaces and
source comparison are mandatory; no new overlap exemption was added.


## Articulated aft seam repair (#333)

The moving leaf no longer shares coplanar cap surfaces with the stationary hull
aperture/header or inset tray. A 10 mm hidden aperture clearance and tapered
tray caps replace the old closed-contact exemptions. The leaf owns the support
overlap at full extension; the tray's concealed front strip is omitted with a
0.1 mm numerical separation at the seam. The 4.4 m ramp, support rectangle,
hinge, marker and collider contracts remain unchanged.
`seam_checks.py` validates closed/opening/extension/reversal samples and rejects
competing materials in deployed support. `repair_door_seams.py` records the
deliberate one-time Blender source migration; do not run it during export or
reapply it to already repaired sources. `preview_seams.py` creates read-only CPU
reference views using the runtime's hinge/extension phases.


## Canopy shoulder junction (#334)

Both opaque side-hull forward ends and the roof now terminate at X=4.98 m,
inside the rear canopy frame (4.94..5.06 m), before glazing starts at X=5 m.
This removes the old 0.22 m shoulder intrusion and 0.18 m roof overhang without
changing glass alpha/materials, window frames, nose/floor, monitors or walking
lanes. The owning hull library and exports are updated together. The overall
broad envelope and conservative pane/traversal proxies are unchanged: they do
not represent these decorative shoulder end caps. Generated spatial/ramp/landing
contracts remain byte-identical.
The category validator tests 24 additional near-corner rays from seated and
standing eyes on both sides, plus existing lower-window/monitor visibility.
A deliberately extended shoulder fails those rays. `repair_cockpit_shoulders.py`
records the one-time source migration; `preview_seams.py -- corners <output>`
uses the actual seated eye and creates read-only interior/exterior references.
