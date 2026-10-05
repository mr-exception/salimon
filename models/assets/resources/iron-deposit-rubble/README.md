# Iron ore deposit: rubble

Original Salimon geometry/materials; no third-party art, textures or add-ons.
Editable `source.blend` is authored with Blender 4.5.3 LTS. The four variants
are a massive angular nodule, a split ore vein, a stepped ledge and a five-rock
rubble cluster. Oxide-orange crust preserves iron readability; dark ore and
metallic inclusions match the authored iron fragments. All materials are opaque.
Runtime uses authored base colors and local facet shading rather than PBR lighting.

Pivot: authoritative deposit center. Baked coordinates fit ±0.48 m with identity
transforms. Blender +Z up converts once to runtime +Y. Runtime scales uniformly
by `bounds_radius_meters / (0.48 * sqrt(3))`, inscribing the entire visual cube
in the existing spherical gameplay bound at every body/latitude. Placement stays
axis-independent at the surface anchor. Local deposit ID modulo four selects
nodule/vein/ledge/rubble in that order, independent of remaining mass, camera,
query order or streaming. Depletion hides the visual. Collision policy is `none`:
world targeting, mining, mass and session persistence remain authoritative.

Export the saved source (normal Cargo builds require no Blender):

```sh
python models/tools/export_asset.py resource.iron-deposit-rubble --blender /path/to/blender
python models/tools/validate_asset.py resource.iron-deposit-rubble
```

Recreate source only deliberately; this overwrites manual Blender edits:

```sh
blender --background --python-exit-code 1 --python models/assets/resources/iron-deposit-rubble/create_source.py
```

Budget: 240 triangles, three primitives/materials, no textures and 32 KiB.
See [comparison and validation evidence](../../../../reports/issue-86/README.md).
