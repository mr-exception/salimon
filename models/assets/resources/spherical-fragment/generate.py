"""Original parametric flat-shaded sphere. Run with Blender --background --python."""
from pathlib import Path
import bpy

RADIUS = 0.48
bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.unit_settings.system = 'METRIC'
scene.unit_settings.scale_length = 1
root = bpy.data.objects.new('ASSET_spherical-fragment', None)
scene.collection.objects.link(root)
root['salimon'] = {'assetId': 'resource.spherical-fragment'}
group = bpy.data.objects.new('Visual', None)
scene.collection.objects.link(group)
group.parent = root
# 80 triangles; deterministic, no modifiers/textures/animation or seed variants.
bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=2, radius=RADIUS)
sphere = bpy.context.object
sphere.name = 'Spherical_Fragment'
sphere.parent = group
sphere['salimon'] = {'role': 'fragment-sphere-visual'}
for face in sphere.data.polygons:
    face.use_smooth = False
material = bpy.data.materials.new('Resource_Tint')
material.use_nodes = True
shader = material.node_tree.nodes.get('Principled BSDF')
shader.inputs['Base Color'].default_value = (1, 1, 1, 1)
shader.inputs['Roughness'].default_value = 0.85
sphere.data.materials.append(material)
bpy.context.preferences.filepaths.save_version = 0
bpy.ops.wm.save_as_mainfile(compress=True, filepath=str(Path(__file__).with_name('source.blend')))
