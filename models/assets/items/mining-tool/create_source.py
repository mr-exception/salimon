"""Original Blender Python authoring recipe; source.blend remains editable."""
from pathlib import Path
import bpy

bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete(use_global=False)
bpy.context.scene.unit_settings.system = 'METRIC'
bpy.context.scene.unit_settings.scale_length = 1
root = bpy.data.objects.new('ASSET_mining-tool', None)
bpy.context.collection.objects.link(root)
root['salimon'] = {'assetId': 'item.mining-tool'}
visual = bpy.data.objects.new('Visual', None)
bpy.context.collection.objects.link(visual)
visual.parent = root
materials = []
for name, color in [('Tool_Housing', (.20,.25,.29,1)), ('Tool_Grip', (.065,.08,.10,1)), ('Tool_Metal', (.55,.62,.65,1)), ('Tool_Status', (.9,.6,.15,1))]:
    mat = bpy.data.materials.new(name)
    mat.diffuse_color = color
    mat.use_nodes = True
    mat.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value = color
    materials.append(mat)

def finish(obj, name, material):
    obj.name = name
    obj.parent = visual
    obj.data.materials.append(materials[material])
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    bevel = obj.modifiers.new('Machined edges', 'BEVEL')
    bevel.width = .006
    bevel.segments = 1
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.modifier_apply(modifier=bevel.name)
    obj['salimon'] = {'role': 'mining-tool-visual'}

def box(name, center, size, material):
    bpy.ops.mesh.primitive_cube_add(size=1, location=center)
    obj = bpy.context.object
    obj.dimensions = size
    finish(obj, name, material)

box('Housing', (.065,0,.065), (.24,.10,.105), 0)
box('Grip', (-.025,0,-.04), (.065,.075,.15), 1)
box('Battery', (-.065,0,.06), (.06,.115,.10), 1)
box('Top_Rail', (.06,0,.128), (.19,.07,.025), 2)
for i in range(4):
    box(f'Cooling_Fin_{i}', (.035+i*.027,0,.07), (.012,.116,.07), 2)
box('Status_Panel', (-.005,.057,.085), (.055,.014,.035), 3)
box('Status_Top', (-.01,0,.145), (.04,.045,.009), 3)
bpy.ops.mesh.primitive_cylinder_add(vertices=12, radius=.044, depth=.11, location=(.225,0,.065), rotation=(0,1.57079632679,0))
finish(bpy.context.object, 'Emitter_Collar', 2)
bpy.ops.mesh.primitive_cylinder_add(vertices=12, radius=.030, depth=.012, location=(.285,0,.065), rotation=(0,1.57079632679,0))
finish(bpy.context.object, 'Emitter_Lens', 3)
sockets = bpy.data.objects.new('Sockets', None)
bpy.context.collection.objects.link(sockets)
sockets.parent = root
grip = bpy.data.objects.new('SOCKET_Grip', None)
bpy.context.collection.objects.link(grip)
grip.parent = sockets
grip['salimon'] = {'role': 'first-person-grip'}
bpy.ops.wm.save_as_mainfile(filepath=str(Path(__file__).with_name('source.blend')))
