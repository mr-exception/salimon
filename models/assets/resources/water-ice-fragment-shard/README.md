# Water-ice fragment: shard

Original geometry and materials created for Salimon; no third-party content,
textures or external licenses are required. This records provenance without
introducing a new repository license. Editable source:
`source.blend`, created with Blender 4.5.3 LTS by `create_source.py`.
The shard is a single tapered hexagonal crystal; the cluster combines three
unequal crystals. Both use two opaque flat-color materials, no textures,
transparency, transmission, sockets or authored collision.

The origin is the center of a one-meter authoritative bounding cube. Applied
mesh coordinates stay within ±0.48 m on every axis, leaving a 2% safety margin.
The exporter converts Blender +Z up to runtime +Y up once. All exported nodes
have identity transforms. The runtime scales uniformly by the fragment's
mass-derived physical side, centered at its authoritative absolute position;
no physical mass, volume, contact radius or carrying policy comes from this mesh.
Even fragment IDs select the shard; odd IDs select the cluster. Selection does
not depend on position, camera, streaming, material mass or carry state.

Regenerate from the saved source (normal builds need no Blender):

```sh
python models/tools/export_asset.py resource.water-ice-fragment-shard --blender /path/to/blender
python models/tools/validate_asset.py resource.water-ice-fragment-shard
```

Rebuild the editable source only deliberately:

```sh
blender --background --python-exit-code 1 --python models/assets/resources/water-ice-fragment-shard/create_source.py
```

The renderer validates triangles, identity transforms, opaque materials and
actual vertex bounds when loading checked-in GLBs. Python and Rust regression
tests validate both exports and their size mapping. The generic manifest caps
this asset at 200 triangles, 2 primitives/materials, 0 textures and 32 KiB.
See the [comparison preview](../../../../reports/issue-91/ice-export-preview.png).
