# Silicate deposit: ridge

Original Salimon geometry/materials; no third-party content or textures.
Editable `source.blend` is created by `create_source.py` with Blender 4.5.3 LTS.
The four variants are a rounded faceted boulder, two broad layered slabs,
a three-rock ridge and a five-rock scree cluster. Opaque muted olive/grey
materials match silicate fragments and distinguish stone from rusty iron and
blue/frost ice. Roughness is 0.82; no transparency or textures are required.

Pivot: authoritative deposit center. Baked coordinates fit ±0.48 m with
identity transforms. Blender +Z up converts once to runtime +Y. Runtime scales
uniformly by `bounds_radius_meters / (0.48 * sqrt(3))`, inscribing the visual
cube in the existing spherical gameplay bound at every body/latitude.
Placement remains axis-independent and centered at the surface anchor.
Local deposit ID modulo four selects boulder/slab/ridge/scree in that order,
independent of mass, camera, streaming or query order. Depleted deposits emit
no visual. World generation, mining, mass and session persistence are unchanged.
Collision policy is none: the visual never defines the authoritative bound.

Regenerate exports from the saved source (normal Cargo builds need no Blender):

```sh
python models/tools/export_asset.py resource.silicate-deposit-ridge --blender /path/to/blender
python models/tools/validate_asset.py resource.silicate-deposit-ridge
```

Recreate source only deliberately:

```sh
blender --background --python-exit-code 1 --python models/assets/resources/silicate-deposit-ridge/create_source.py
```

Budget: 240 triangles, two primitives/materials, no textures and 32 KiB.
See [comparison evidence](../../../../reports/issue-87/README.md).
