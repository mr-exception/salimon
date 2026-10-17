# Resource Option A pass (#277)

The twelve existing deposit silhouettes are retained: broken five-sided iron
shoulders, weathered seven-sided silicates, six-sided opaque ice spires. All use
explicit flat normals. Material assignments, vertex positions, IDs and inscribed
±0.48 m bounds remain unchanged. Iron oxide is quieter ochre/rust; silicates use
slate/olive; opaque ice uses restrained blue/frost. Linear colors live in
`models/tools/resource_style.py`; the art bible remains authoritative.

The live fragment is the existing neutral 80-triangle sphere. No legacy chunk
or shard exports are restyled or returned to gameplay. Resource identity is the
existing per-instance tint, mass still determines radius, and all sphere colors
share one indexed draw and immutable mesh. Visual simplicity never changes physics.

Reapply the saved-source pass with Blender 4.5.3:

```sh
blender -b --python-exit-code 1 --python models/tools/author_resources_option_a.py
python models/tools/export_asset.py resource.iron-deposit-nodule --blender /path/to/blender
```

Export all twelve `*-deposit-*` identities plus `item.mining-tool`; regenerate the
tool icon using its README. Reconstruction recipes create historical geometry;
after reconstructing, reapply this visual pass. Do not overwrite the sphere source.
The tool now uses graphite/slate broad atlas tiles, with molded grip ribs and
steel highlights. Status amber/teal and the camera-local grip transform are unchanged.

Use `models/references/option-a/render_resources.py` for matched GLB near/far
previews and `models/tools/style_review.py` for before/after costs. All resources
opt into `option-a-v1`; the tool also reports its retained close-range microgeometry.
Visual approval belongs to review; native sign-off remains #279/#229.
