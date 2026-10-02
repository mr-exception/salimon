# Authored asset manifest v1

[asset-manifest.schema.json](asset-manifest.schema.json) is the JSON Schema
Draft 2020-12 contract for `models/assets/<category>/<slug>/manifest.json`.
The [ship](examples/ship.manifest.json),
[resource](examples/resource.manifest.json), and
[item](examples/item.manifest.json) examples are design fixtures, **not migrated
assets**. Their future `.blend` sources do not exist yet. Do not discover assets
by scanning `models/examples/`, or copy an example without adapting its contracts.
The legacy `client/assets/ship/asset-manifest.json` remains unchanged and uses its
own format until the scheduled migration.

## Fields

Paths inside the manifest are repository-root-relative POSIX paths, independent
of the working directory or manifest location. `$schema` alone is a URI reference
resolved relative to the manifest (normally `../../../asset-manifest.schema.json`
for the standard asset layout). Absolute paths, backslashes, traversal components,
and empty path components are forbidden. Runtime outputs must be GLBs under
`client/assets/`; source files must be `.blend` files under `models/assets/`
(the latter containment is also checked by future tooling after path resolution).

| Field | Required | Meaning |
| --- | --- | --- |
| `schemaVersion` | Yes | Integer `1`; incompatible shared changes require a new version |
| `assetId` | Yes | Stable logical ID, e.g. `resource.iron-fragment`; never derive it from paths |
| `category` | Yes | Plural category such as `ships`, `resources`, `items`, `props`, `structures`, `characters`, `vehicles`; new categories use the same envelope |
| `source` / `runtime` | Yes | Saved Blender source / checked-in runtime GLB destination |
| `coordinates` | Yes | Meter units, source/runtime axes, conversion, and human-readable pivot description |
| `profile` | Yes | `standard-v1` or the narrowly scoped `legacy-scout-v1` compatibility profile |
| `budgets` | Yes | Positive integer hard caps: triangles, primitives, materials, texture bytes, GLB bytes |
| `collision` | Yes | Explicit policy and its configuration, described below |
| `contracts` | Yes | Root, visual groups, groups, nodes, materials, sockets, markers, colliders, and required extras; empty arrays mean no requirement |
| `lods` | No | Nonempty array of explicit visual levels, nodes, triangle budgets, and selection contracts |
| `extensions` | No | Namespaced, versioned category-specific contracts |
| `$schema` | No | Editor/schema reference; readers use `schemaVersion`, not remote schema fetching |

Unrecognized envelope fields are rejected to catch misspellings. Extension `data`
is deliberately open: it is validated by the category's versioned validator,
not silently treated as gameplay policy by the generic exporter. IDs need not
match a directory slug, nor be rewritten when a source/output moves. IDs and
runtime destinations must be unique across actual asset manifests.

## Coordinates and budgets

Version 1 uses the shared meter frame: Blender metric unit scale 1, `+Z` up,
`+X` forward; runtime `+Y` up, `+X` forward. `conversion: "x,z,-y"` records the
single source-to-runtime mapping. Importing the legacy runtime scout into Blender
must apply its inverse before a later re-export. Do not apply this conversion
again in the renderer. The pivot description must specify the asset-local origin;
world positions never belong in a model manifest.

Triangle and primitive caps count the whole exported GLB, including all visual
LODs. Material caps count its material definitions; `maxTextureBytes` caps the
sum of encoded image payload bytes (embedded or referenced), not decoded GPU
memory. `maxGlbBytes` caps the GLB file itself. No implicit per-category defaults
or draw budgets exist. Category extensions may impose stricter limits such as
the scout's two runtime draws.

## Collision and named contracts

- `none`: no `shapes`, `generator`, or required colliders.
- `authored-proxies`: nonempty `shapes` and `contracts.colliders`, no generator.
  Shape names are `box`, `sphere`, `capsule`, or `convex-hull`; listing a shape
  does not implement runtime support. The category/consumer must support it.
- `generated`: a nonempty `generator` identifies a documented, versioned offline
  representation algorithm; no authored `shapes` list. Tooling must reject unknown
  generators. This policy never implicitly selects the visible mesh as collision.

`contracts.root` identifies one exported node. Every entry is an exact,
case-sensitive required name. `groups` and `visualGroups` refer to exported
parented nodes, not Blender-only collections. All arrays are individually unique.
A visual group may also appear in `groups`; a socket/marker/collider may also
appear in `nodes`. These declarations refer to the **same** unique exported node,
not permission to duplicate it. Sockets, markers, and colliders must not overlap
with one another. Material names inhabit the material namespace.

`requiredExtras` entries pair a node name with a dot-separated glTF extras path,
e.g. `{ "node": "Monitor_Center", "path": "salimon.displayRole" }`.
`salimon` alone requires the namespace object. Exporters preserve custom metadata;
validators check presence and category-specific value/transform semantics.

The standard profile requires the `ASSET_<slug>` root, `Visual` group, and shared
prefix conventions in [contracts.md](contracts.md). `legacy-scout-v1` is limited
to `ship.salimon-scout` in category `ships`, rooted at `Salimon_Phase0_Scout`.
It preserves `Exterior`/`Interior`, door/monitor names, all current collision and
marker names, material roles, and required `salimon` metadata. This explicit
compatibility boundary must not become the default for other ships.

## LODs and extensions

Omitting `lods` means one visual representation with no LOD selection contract.
If present, require consecutive unique levels starting at 0; `LOD0` is the most
detailed. Each entry declares `level`, exported `node`, `maxTriangles`, and a
`selection` description specifying the consumer's measurable switch criteria.
The node must exist below the visual hierarchy; all levels share the pivot and
coordinate frame. Collision, sockets, and markers are shared outside visual LOD
subtrees. Total asset budgets still apply.

An extension has a namespace key and `{ "version": 1, "data": { ... } }`.
For example `extensions.item.data.gripSocket` describes an attachment transform;
`extensions.ship.data` describes monitor UV/material and legacy preservation
contracts. The ship example references its complete legacy contract rather than
copying all spatial/material expectations into the generic schema. Resource type,
mining yield, carrying limits, and other gameplay rules remain in Rust domains.
Unknown extension namespaces/versions must fail clearly until a registered
category validator supports them; they must never be silently ignored.

## Schema checks and tooling

Schema checks validate structure and scalar constraints. They do **not** prove
source existence, symlink containment, unique IDs across assets, valid exported
names/hierarchy, finite transforms, applied scale, measured budgets, supported
collision algorithms, LOD ordering, or category semantics. The
[generic command](tools/README.md) implements export; the validator implements
generic semantic validation and category extension dispatch. Keep
these authoring dependencies outside Cargo and normal game builds.

Run the schema regression checks in an optional Python environment:

```sh
python3 -m venv /tmp/salimon-manifest-check
/tmp/salimon-manifest-check/bin/pip install jsonschema==4.26.0
/tmp/salimon-manifest-check/bin/python -m unittest discover -s models/tests -v
```

This checks the schema itself, all three examples, and malformed manifest cases;
it neither invokes Blender nor writes runtime assets.
