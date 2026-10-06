"""Original angular iron ore deposit recipe for Blender 4.5 LTS."""
from pathlib import Path
import math
import bpy

bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete(use_global=False)
bpy.context.scene.unit_settings.system = 'METRIC'
bpy.context.scene.unit_settings.scale_length = 1
slug = Path(__file__).parent.name
root = bpy.data.objects.new('ASSET_' + slug, None)
bpy.context.collection.objects.link(root)
root['salimon'] = {'assetId': 'resource.' + slug}
visual = bpy.data.objects.new('Visual', None)
bpy.context.collection.objects.link(visual)
visual.parent = root
materials = []
for name, color in [('Iron_Ore', (.25, .22, .19, 1)), ('Iron_Inclusions', (.49, .47, .43, 1)), ('Oxide_Crust', (.75, .22, .06, 1))]:
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes['Principled BSDF']
    bsdf.inputs['Base Color'].default_value = color
    bsdf.inputs['Roughness'].default_value = .85 if name == 'Oxide_Crust' else .62
    bsdf.inputs['Metallic'].default_value = .65 if name == 'Iron_Inclusions' else .15
    materials.append(mat)
# Baked centers, radii, half-height and angle in Blender meters.
# Angular five-sided shoulders and skewed caps form broken oxide-rich ore.
rocks = [(0, 0, -0.01, 0.44, 0.4, 0.43, 0.12)]
vertices, faces = [], []
for cx, cy, cz, rx, ry, height, angle in rocks:
    start = len(vertices)
    for ring, (z, taper) in enumerate([(-height, .8), (height * .12, 1), (height, .67)]):
        for i in range(5):
            a = i * math.tau / 5 + angle
            irregular = (1, .82, .96, .88, .91)[i]
            vertices.append((cx + rx * taper * irregular * math.cos(a),
                             cy + ry * taper * irregular * math.sin(a),
                             cz + z + height * .06 * math.sin(i * 2 + ring)))
    faces.append(tuple(start + i for i in reversed(range(5))))
    for ring in range(2):
        for i in range(5):
            j = (i + 1) % 5
            faces.append((start + ring * 5 + i, start + ring * 5 + j,
                          start + (ring + 1) * 5 + j, start + (ring + 1) * 5 + i))
    faces.append(tuple(start + 10 + i for i in range(5)))
assert all(abs(axis) <= .48 for point in vertices for axis in point)
mesh = bpy.data.meshes.new('Iron_Deposit_Facets')
mesh.from_pydata(vertices, [], faces)
mesh.update()
obj = bpy.data.objects.new('Iron_Deposit', mesh)
bpy.context.collection.objects.link(obj)
obj.parent = visual
obj['salimon'] = {'role': 'iron-deposit-visual'}
for mat in materials:
    mesh.materials.append(mat)
for face in mesh.polygons:
    face.material_index = (2, 0, 2, 1, 2, 0, 2)[face.index % 7]
bpy.ops.wm.save_as_mainfile(filepath=str(Path(__file__).with_name('source.blend')), compress=True)
