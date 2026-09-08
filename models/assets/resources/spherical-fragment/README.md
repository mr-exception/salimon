# Shared spherical fragment

Original Salimon parametric icosphere, no external assets, links, textures,
add-ons or runtime Blender dependency. `generate.py` is editable source logic;
`source.blend` is the saved Blender scene (generated/exported with Blender 4.0.2;
opens in the supported 4.5.3 authoring environment). Asset identity is
`resource.spherical-fragment`; resource and fragment IDs are unchanged.

```
blender --background --python-exit-code 1 --python models/assets/resources/spherical-fragment/generate.py
python models/tools/export_asset.py resource.spherical-fragment --blender /path/to/blender
python models/tools/validate_asset.py resource.spherical-fragment
python -m unittest discover -s models/tests -p test_spherical_fragment.py -v
```

80 flat-shaded triangles, one opaque neutral `Resource_Tint` material/primitive,
no texture, identity node transforms, origin at center, every vertex 0.48 m
from origin. Runtime uses radius/0.48 scale and established linear resource
colors per instance, independent of fragment ID. One resident mesh and one
fragment batch serve every resource; old variants remain solely for legacy
contact proxies and explicit geometry evaluation, not new fragment rendering.
