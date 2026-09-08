# Water-ice deposit: shelf

Original Salimon geometry/materials; no third-party content or textures.
Editable `source.blend` is created by `create_source.py` with Blender 4.5.3 LTS.
The four variants are a single tall spire, four-crystal crown, three-crystal ridge,
and broad shelf with a raised crystal. Two opaque blue/frost materials provide
faceted ice without transparency, transmission or post-processing.

The pivot is the authoritative deposit center. Exported coordinates fit inside
±0.48 m on every axis with identity transforms. Runtime scales uniformly by
`bounds_radius_meters / (0.48 * sqrt(3))`, inscribing that cube in the world-owned
spherical bound. This is safe at every body/latitude without introducing a
surface orientation or new collider. Blender +Z up converts once to runtime +Y.
The source center lies at the surface anchor; only the exposed geometry is seen.
Deposit local ID modulo four selects spire/crown/ridge/shelf in that order,
independent of camera, query order, remaining mass or streaming. Depleted deposits
emit no visual. Mining, mass, session persistence and streaming remain world-owned.

Regenerate exports from the saved source (normal Cargo builds need no Blender):

```sh
python models/tools/export_asset.py resource.water-ice-deposit-shelf --blender /path/to/blender
python models/tools/validate_asset.py resource.water-ice-deposit-shelf
```

Recreate source only deliberately:

```sh
blender --background --python-exit-code 1 --python models/assets/resources/water-ice-deposit-shelf/create_source.py
```

Budget: 240 triangles, two primitives/materials, no textures and 32 KiB.
See [comparison evidence](../../../../reports/issue-88/README.md).
