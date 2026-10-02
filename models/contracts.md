# Shared 3D asset contracts

These conventions apply to newly authored assets. The
[manifest schema v1](manifest.md) records these contracts; the exporter (#77) and
validators (#78) will enforce them on assets. Existing assets retain their
documented contracts during migration; no runtime reader currently
implements the new generic conventions.

## Units, axes, pivot, and transforms

- Runtime coordinates are right-handed meters, with `+Y` up and `+X` forward
  for oriented assets. The remaining lateral axis is `+Z` to the left/port;
  `-Z` is right/starboard. Resource and prop orientation has no gameplay meaning
  unless the category contract gives it one.
- Blender sources use metric units with unit scale 1, `+Z` up, `+X` forward,
  and `+Y` left. Convert once on export: source `(x, y, z)` becomes runtime
  `(x, z, -y)`. Do not rotate exported meshes again in the client. The manifest
  records source and runtime axes explicitly.
- Keep geometry near the asset's local origin. Document its pivot (for example,
  a prop's bottom center or a vehicle's reference frame); world coordinates and
  planetary positions belong to game state, not the model.
- Static visual meshes have applied scale/rotation and finite transforms.
  Bake static geometry consistently; sockets and markers retain meaningful
  local transforms. Animated or articulated nodes keep the transforms required
  by their category contract. Avoid negative/nonuniform scale on spatial
  contracts; do not apply transforms blindly to rigs or moving assemblies.
- All geometry, proxies, sockets, and markers share the same asset-local frame
  and unit conversion. Report exported dimensions in meters and compare them
  to the asset's declared bounds.

## Hierarchy and stable names

Use one exported asset root named `ASSET_<slug>`. Blender collections organize
editing; explicit parented objects/empties define the exported node hierarchy.
An exporter must not assume collection names alone become glTF nodes.

| Contract | Convention | Requirement |
| --- | --- | --- |
| Visible geometry | `Visual` group; meaningful mesh names | Required for a visible asset |
| Collision proxies | `Collision_Proxies` group; `COLLIDER_<role>` objects | Required only when collision policy declares authored proxies |
| Attachment transforms | `Sockets` group; `SOCKET_<role>` empties | Optional; category declares any required sockets |
| Interaction/spatial anchors | `Interaction_Markers` group; `MARKER_<role>` empties | Optional; category declares any required markers |
| Optional visual LODs | `LOD0`, `LOD1`, … beneath `Visual` | `LOD0` is the most detailed; add lower levels only with an explicit selection/budget contract |

Names are case-sensitive and unique within an exported asset. Do not rely on
automatic Blender suffixes such as `.001` to resolve duplicated contract names.
The manifest enumerates required nodes and their roles. Export must preserve
those names and their relevant custom properties as glTF node `extras`; use a
`salimon` namespace for project metadata. Authoring helpers, reference images,
cameras, lights, and preview meshes are excluded from runtime output unless an
explicit contract requires them.

## Visuals, collisions, sockets, and markers

Visuals use exportable glTF materials and resolved textures. Blender-only shader
nodes are not a runtime material contract. Declare per-asset triangle, primitive,
material, texture, and file-size budgets; there is no shared ship-sized budget
for all assets. Render-pass and material restrictions belong to the consumer or
category extension. LODs must retain the same origin, orientation, and spatial
anchors; do not duplicate gameplay proxies/markers for each visual LOD.

Collision policy is explicit: none, authored proxies, or a documented generated
representation. A visual mesh must not silently become the gameplay collider.
Authored proxy objects are visible while editing but excluded from rendered
geometry. Their shape, dimensions, and transforms become metadata. Boxes and
other supported shape types must be declared by the manifest/validator; adding
a shape needs consumer support. No backend automatically interprets a new
proxy just because it is named correctly.

A socket is an attachment position and orientation (for example `SOCKET_Grip`
on a held tool). A marker is a spatial reference or interaction location (for
example `MARKER_Use` or a vehicle entry point). Neither encodes reach distance,
permission, carrying limits, mining yield, health, inventory rules, or other
gameplay policy. Category contracts define orientation and consumer semantics;
an optional marker is not implicitly an active interaction.

## Category extensions

Every category uses the same identity, paths, coordinates, naming, budgets, and
collision conventions. A versioned category extension may add required names,
metadata, dimensions, material/UV rules, or validation checks. It must not relax
the shared invariants or force its requirements on unrelated categories.

| Category | Examples of additional contracts |
| --- | --- |
| Ship | Interior/exterior groups, moving door, seat/entry/engine/cargo anchors, monitor faces and UVs |
| Resource | Deposit/fragment visual roles, declared collision bounds; resource identity/yield stays in the resource domain |
| Item | Optional grip/mount sockets or use marker; carrying policy stays in the item/character domain |
| Structure | Footprint, entry and placement anchors |
| Prop | Static visual and optional collision policy; no mandatory interactions |
| Character | Skeleton, bind pose, animation and attachment conventions |
| Vehicle | Occupant/entry anchors and articulated parts without assuming a ship cockpit |

Character rigs and animated vehicle parts require their own extension before
being supported by tooling; listing a category does not implement animation.
Common validation runs first, then the category-specific validator. Both must
identify the logical asset and the broken contract. Renames or changed spatial
semantics require updating consumers and regression checks together; treat them
as contract changes, not cosmetic edits.

## Existing scout compatibility

The scout's [current contract](../client/assets/ship/README.md#runtime-contract)
takes precedence during migration. Preserve `Exterior`, `Interior`,
`Collision_Proxies`, `Interaction_Markers`, `Exit_Door`, `Monitor_Center`,
`Monitor_Port`, `Monitor_Starboard`, collision names, material roles, and
`extras.salimon` metadata. Its baked coordinates and identity transforms,
pilot-facing monitor normals/top-left UVs, glazing behavior, cargo layout, and
generated controller bounds must survive the later migration.

`Exit_Door` and monitor names are consumed directly by
`client/renderer/src/ship_mesh.rs`; renaming them would change behavior. Do not
wrap or rename the current hierarchy just to match `ASSET_<slug>`/`Visual`.
The ship category adapter must express the legacy hierarchy as an explicit
compatibility contract. Detailed preservation evidence belongs to #79 and the
runtime switch to #80, rather than a generic requirement for resource/item art.
