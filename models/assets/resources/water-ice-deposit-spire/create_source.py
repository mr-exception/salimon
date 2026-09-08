"""Original opaque faceted ice deposit recipe for Blender 4.5 LTS."""
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
for name, color in [('Ice_Blue', (.13, .51, .77, 1)), ('Ice_Frost', (.64, .96, 1, 1))]:
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes['Principled BSDF']
    bsdf.inputs['Base Color'].default_value = color
    bsdf.inputs['Roughness'].default_value = .38
    materials.append(mat)
# Baked coordinates keep every crystal in the centered +/-0.48 meter cube.
# Each tuple is center x/y/z, horizontal radius, half-height, and facet angle.
crystals = [(0, 0, 0, 0.29, 0.46, 0.2)]
vertices, faces = [], []
for cx, cy, cz, radius, height, angle in crystals:
    start = len(vertices)
    for z, taper, shift in [(-height, .70, 0), (height * .45, 1, 0), (height, .22, .03)]:
        for i in range(6):
            a = i * math.tau / 6 + angle
            vertices.append((cx + radius * taper * math.cos(a) + shift,
                             cy + radius * taper * math.sin(a), cz + z))
    faces.append(tuple(start + i for i in reversed(range(6))))
    for ring in range(2):
        for i in range(6):
            j = (i + 1) % 6
            faces.append((start + ring * 6 + i, start + ring * 6 + j,
                          start + (ring + 1) * 6 + j, start + (ring + 1) * 6 + i))
    faces.append(tuple(start + 12 + i for i in range(6)))
mesh = bpy.data.meshes.new('Ice_Deposit_Facets')
mesh.from_pydata(vertices, [], faces)
mesh.update()
obj = bpy.data.objects.new('Ice_Deposit', mesh)
bpy.context.collection.objects.link(obj)
obj.parent = visual
obj['salimon'] = {'role': 'water-ice-deposit-visual'}
for mat in materials:
    mesh.materials.append(mat)
for face in mesh.polygons:
    face.material_index = int(face.index % 4 == 0)
bpy.ops.wm.save_as_mainfile(filepath=str(Path(__file__).with_name('source.blend')), compress=True)
