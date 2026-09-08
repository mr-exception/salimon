"""Original opaque faceted silicate deposit recipe for Blender 4.5 LTS."""
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
for name, color in [('Silicate_Dark', (.34, .39, .28, 1)), ('Silicate_Light', (.62, .64, .52, 1))]:
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes['Principled BSDF']
    bsdf.inputs['Base Color'].default_value = color
    bsdf.inputs['Roughness'].default_value = .82
    materials.append(mat)
# Baked source-frame centers, horizontal radii, half-height and facet angle.
# Uneven seven-sided rings form weathered stone, without pointed ice crystals.
rocks = [(0, 0, -0.14, 0.45, 0.38, 0.22, 0.12), (0.01, 0.01, 0.13, 0.37, 0.31, 0.17, 0.4)]
vertices, faces = [], []
for cx, cy, cz, rx, ry, height, angle in rocks:
    start = len(vertices)
    for ring, (z, taper) in enumerate([(-height, .78), (height * .3, 1), (height, .56)]):
        for i in range(7):
            a = i * math.tau / 7 + angle
            irregular = (1, .86, .94, .81, .98, .88, .92)[i]
            vertices.append((cx + rx * taper * irregular * math.cos(a),
                             cy + ry * taper * irregular * math.sin(a),
                             cz + z + height * .06 * math.sin(i * 2 + ring)))
    faces.append(tuple(start + i for i in reversed(range(7))))
    for ring in range(2):
        for i in range(7):
            j = (i + 1) % 7
            faces.append((start + ring * 7 + i, start + ring * 7 + j,
                          start + (ring + 1) * 7 + j, start + (ring + 1) * 7 + i))
    faces.append(tuple(start + 14 + i for i in range(7)))
assert all(abs(axis) <= .48 for point in vertices for axis in point)
mesh = bpy.data.meshes.new('Silicate_Deposit_Facets')
mesh.from_pydata(vertices, [], faces)
mesh.update()
obj = bpy.data.objects.new('Silicate_Deposit', mesh)
bpy.context.collection.objects.link(obj)
obj.parent = visual
obj['salimon'] = {'role': 'silicate-deposit-visual'}
for mat in materials:
    mesh.materials.append(mat)
for face in mesh.polygons:
    face.material_index = int(face.index % 5 in (0, 1))
bpy.ops.wm.save_as_mainfile(filepath=str(Path(__file__).with_name('source.blend')), compress=True)
