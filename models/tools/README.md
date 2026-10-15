# Generic Blender export

This command is an optional offline authoring tool. Cargo and native build scripts
consume checked-in runtime assets and never invoke it. Install Blender 4.5 LTS
with its bundled glTF exporter and an authoring Python environment:

```sh
python3 -m venv /tmp/salimon-models
/tmp/salimon-models/bin/pip install -r models/tools/requirements.txt
/tmp/salimon-models/bin/python models/tools/export_asset.py resource.iron-fragment
```

Pass `--blender /path/to/blender` (on macOS, the executable inside Blender.app),
or set `BLENDER`; otherwise `blender` is resolved from PATH. An asset directory
or manifest path under `models/assets/` can replace the logical ID. Relative
paths are repository-root-relative regardless of the calling working directory.
Logical IDs are discovered only from real `models/assets/**/manifest.json`
files; `models/examples/` remains schema documentation, not runnable content.
The [iron fragment](../assets/resources/iron-fragment/README.md) is a committed,
runnable resource asset; the [cargo container](../assets/items/cargo-container/README.md)
is a runnable item asset (`item.cargo-container`).
The [scout](../assets/ships/salimon-scout/README.md) uses a ship adapter over the
same headless exporter, shared validator and scout category extension:

```sh
python3 models/assets/ships/salimon-scout/export.py --blender /path/to/blender
```

Use this adapter for `ship.salimon-scout`; the generic CLI has no automatically
registered ship extension. The adapter preserves legacy asset-level metadata
and matching interchange output. Detailed checks read the authored GLB directly.

The command checks the local v1 schema, source/output containment (including
symlinks), and missing dependencies before invoking a fresh headless Blender
process. It disables source auto-execution, uses the saved active scene, and
selects the declared root plus its descendants, independent of saved UI
selection. Put reference art, preview meshes, cameras, and lights **outside** the
root. V1 supports static meshes/empties; animated objects/rigs need a future
category adapter and fail instead of silently losing animation.

Sources must use metric units at scale 1. The bundled glTF exporter applies
`export_yup=True` exactly once, mapping `(x,y,z)` to `(x,z,-y)`. Export preserves
object/material names and custom properties as extras, applies visual mesh
modifiers, embeds images/buffers, and includes hidden required content. Save
portable/packed textures in the `.blend`; workstation-specific texture paths
are not a reproducible source.

`COLLIDER_` meshes export as empty nodes with the same name, transform, and
custom metadata plus `extras.salimonProxyDimensions` (source-frame dimensions
in meters). They do not contribute visible mesh geometry. Other meshes under
the root are visual output. Declare shape semantics in the proxy's `salimon`
properties and manifest; this command adds no collision generator or runtime
consumer. Blender edits occur only in the throwaway export process, never in
the saved source.

The output is written beside the destination into a temporary GLB. Before atomic
replacement, checks verify the GLB header, required node/material/LOD names,
required extras paths, unique node names, and embedded dependencies. A failed
Blender process, invalid output, or missing contract preserves the previous
runtime file and returns a nonzero exit with the asset ID and error. Blender
logs remain visible. Review output diffs before committing.

The generic semantic validator also runs on the temporary GLB before replacement.
Category-specific metadata values and gameplay integration require registered
category validators and their own consumer tests. The scout adapter registers
its complete category validator; no procedural ship generator remains.

## Regression checks

```sh
/tmp/salimon-models/bin/python -m unittest discover -s models/tests -v
BLENDER=/path/to/blender /tmp/salimon-models/bin/python -m unittest discover -s models/tests -v
```

Without Blender, the real export integration test is explicitly skipped. With
Blender it creates temporary `.blend` fixtures for resource and item categories,
exports using the same command, verifies stable names/nested extras, axis mapping,
proxy exclusion and helper exclusion, then verifies a missing contract cannot
overwrite a valid artifact. Fixtures never replace the game's ship or create
committed example assets.

## Generic validation

Validate a checked-in authored asset without Blender:

```sh
python models/tools/validate_asset.py resource.iron-fragment
```

IDs, directories, and manifest paths use the same discovery/schema/path checks as
export. Success emits JSON containing the asset and measured budgets; failure
returns nonzero with the asset ID and broken contract. Examples are not real
assets and are not discovered. Validate the scout with its registered category adapter:

```sh
python models/assets/ships/salimon-scout/validate.py
```

The old `client/assets/ship/source/validate_salimon_phase0_ship.py` command
forwards to this adapter. Both verify matching interchange and generated spatial
outputs, in addition to the GLB checks used during staged export.

Shared checks cover source existence, resolved source/runtime containment,
unique IDs/destinations, embedded GLB buffer ranges, finite geometry/transforms,
unit scale, normalized rotations, affine unscaled matrices, root ownership,
required names/extras, duplicate names, spatial role overlap, standard naming
and parent groups, nonvisual proxies/sockets/markers, declared proxy shapes and
positive meter dimensions (the explicit `legacy-scout-v1` profile checks retained
`collisionShape`/`sizeMeters` boxes), consecutive/disjoint LODs with shared pivots, and all
five measured budgets. Triangle/primitive counts sum mesh definitions across
all LODs; instancing/draw-call policy belongs to a category extension. Texture
bytes count encoded embedded image payloads. V1 accepts static triangle lists
and ordinary embedded accessors; unsupported sparse data, rigs, animation, and
required glTF extensions fail rather than bypassing checks.

The standalone validator checks manifest meter/axis declarations and runtime
scale. It does not parse `.blend` contents: the headless exporter separately
checks the actual saved scene's metric unit scale and applies the axis mapping.
Intentional source dimensions, articulated behavior, monitor UVs/materials, and
other category semantics remain category checks.

### Extension API

Import `validate_asset.validate_asset(asset, repo=..., extension_validators=...,
collision_validators=...)`, or pass those same keyword registries to
`export_asset.export_asset`. The CLI has empty registries by default; it never
loads arbitrary plugins from a manifest.

`extension_validators` maps `(category, namespace, version)` to
`callback(context, data)`. `collision_validators` maps a documented versioned
generator ID to `callback(context)`. Context supplies `manifest`, parsed
`document`, decoded `binary` bytes, runtime `Path`, and measured `metrics`.
`accessor_values(document, binary, reference)` honors buffer/accessor offsets,
strides and all supported scalar widths; `position_bounds` measures actual
vertex data rather than trusting accessor min/max. A callback raises
`ValidationError` with its broken contract on failure. Common checks always run
first. Unknown namespaces/versions/generators fail clearly, including the item
and ship extension examples until their category adapters are implemented.
No category semantics or collision algorithm is guessed by generic tooling.

`validate_manifest(manifest, runtime, repo=..., **registries)` is the exporter
entry point for a temporary GLB. Its manifest must already pass schema and
workspace identity checks through `resolve_manifest`; `validate_asset` is the
public entry point that performs both stages. Export runs this before atomic
replacement, so semantic failures preserve existing output.
# Read-only scout quality diagnosis

`python models/tools/diagnose_scout.py` prints actual GLB bounds, provenance
hash checks, normals/degenerate-face findings and coplanar triangle-overlap
candidates without Blender or asset mutations. It is an investigative probe for
the baked scout profile, not a general validation gate. See the
[#224 baseline](../../reports/issue-224/README.md) for tolerances, camera fixtures,
confirmed geometry/material findings and unverified graphical symptoms.


For supplemental ray visibility/occlusion, PNG memory estimates and explicit
camera fixtures, use the separate read-only scout probe:

```sh
python models/tools/diagnose_scout_visibility.py --views reports/issue-224/visibility-views.json --output artifacts/scout-visibility.json
```

It retains the original baseline tool. See [visibility findings](../../reports/issue-224/VISIBILITY.md)
for exposed vs buried overlap witnesses, render-order diagnosis and limitations.

## Category-aware geometry quality (#226)

`python models/tools/validate_quality.py <asset-id>` emits a complete sorted JSON
report and exits nonzero on policy violations. Generic export/validation invokes
the same quality gate for every real manifest; category-specific scout spatial,
monitor, pivot, collider, aisle/headroom checks still run in its adapter.

Manifest `quality` is separate from presentation `budgets`: meter tolerance
(default 10 µm), area cutoff (1e-10 m²), per-node runtime bounds, root/socket/marker
origins, optional closed topology nodes and empty clearance boxes with explicit
obstacle ownership. Triangle points/normals use cumulative parent transforms.
Normals/winding, degenerate and duplicate triangles and positive-area near-plane
overlap fail by default. Shared edges, flat normals and disconnected/open panels
pass; only `closedNodes` demand manifold welded edges. Do not declare an aisle's
floor/walls as obstacles; this checks box interiors and is conservative for
non-box meshes. Existing scout detailed clearance/UV/hinge semantics remain its
category adapter's responsibility, independently of style.

Overlap candidates use sweep-and-prune on the longest asset axis, tolerant AABBs,
normal/plane checks and projected polygon clipping. Unlike plane quantization,
this catches pairs across bucket boundaries. Reports identify object, primitive /
triangle index, meter location, tolerance, competing face and area. Worst case
is quadratic for a dense overlapping pile; this is not general self-intersection
or visible-flicker proof. Bounds are measured, never read from accessor min/max.

Intentional intersections/layered glass require a named object-pair exemption,
a reason and a maximum pair count; exceeding the count fails. Existing scout,
cargo-container and mining-tool candidates have **temporary** counted exemptions
linked to [#244](https://github.com/mr-exception/salimon/issues/244), not artistic
approval. Mining tool's 32 existing degenerate triangles are capped warnings;
new warnings beyond that count fail. All resource GLBs pass without exemptions.
The gate does not repair assets or change physics dimensions. Category budgets
remain independently enforced; changing style does not loosen spatial policy.

Synthetic fixtures run in ordinary model CI and cover inverted normals,
degenerate/duplicate/near-coplanar faces, shared edges, world transforms, bounds,
anchors, closed-node topology, clearances and 2,000 spatially separated triangles.
An Option A/Option B fixture swaps one triangle for three facets while keeping
bounds/anchors identical. Node-only spatial metadata (including colliders) is
independent of visual tessellation; scout category tests preserve its richer
contract. No GPU/native hardware is required by this gate.
