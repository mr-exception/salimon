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
There are no committed Blender sources until #79/#84/#85.

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

These are export-preservation checks, **not** the semantic validation framework
in #78: measured budgets, category extensions, transform/LOD semantics, collision
algorithm dispatch, and metadata values still require that validator. Export
success alone is not authorization to accept an asset into gameplay. Keep the
legacy scout's generator/validator until its migration issues complete.

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
