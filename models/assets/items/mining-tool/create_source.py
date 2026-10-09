"""Original, deterministic authoring recipe. Overwrites source and icon; export separately."""
from pathlib import Path
import math
import random
import bpy
from mathutils import Vector

HERE = Path(__file__).resolve().parent
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

# Four padded atlas tiles: coated polymer, molded rubber, brushed metal, clean status.
# Original generated imagery, packed in the editable source and embedded by glTF.
rng = random.Random(142)
image = bpy.data.images.new('Tool_Surface', width=256, height=256, alpha=False)
pixels = []
for y in range(256):
    for x in range(256):
        tile = (x // 128) + 2 * (y // 128)
        u, v = x % 128, y % 128
        grain = rng.uniform(-.025, .025)
        if tile == 0:
            value = .82 + grain
            if (v in (14, 15, 111, 112) and 18 < u < 109) or (u in (14, 15, 111, 112) and 18 < v < 109):
                value = .43  # recessed panel seam
            if 84 < u < 106 and 24 < v < 35:
                value = .32 if (u + v) % 12 < 5 else .92  # small hazard label
            if v % 29 == 0 and 18 < u < 45:
                value = .96  # restrained edge scuff
        elif tile == 1:
            value = .78 + grain - (.20 if v % 12 < 3 else 0)
        elif tile == 2:
            value = .82 + rng.uniform(-.08, .08) + .035 * math.sin(v * 2.5)
            if v in (25, 76) and 20 < u < 80:
                value = .97
        else:
            value = 1.0
        color = [(.72, .43, .20), (.24, .28, .31), (.68, .74, .79), (.9, .6, .15)][tile]
        pixels.extend((*[c * value for c in color], 1.0))
image.pixels = pixels
image.filepath_raw = str(HERE / 'surface.png')
image.file_format = 'PNG'
image.save()
image.pack()
materials = []
for name, color, metallic, roughness in [
    ('Tool_Housing', (.48, .20, .055, 1), .05, .65),
    ('Tool_Grip', (.055, .068, .075, 1), 0, .88),
    ('Tool_Metal', (.55, .61, .65, 1), .85, .32),
    ('Tool_Status', (.9, .6, .15, 1), .0, .3),
]:
    mat = bpy.data.materials.new(name)
    mat.diffuse_color = color
    mat.use_nodes = True
    shader = mat.node_tree.nodes['Principled BSDF']
    shader.inputs['Metallic'].default_value = metallic
    shader.inputs['Roughness'].default_value = roughness
    tex = mat.node_tree.nodes.new('ShaderNodeTexImage')
    tex.image = image
    mat.node_tree.links.new(tex.outputs['Color'], shader.inputs['Base Color'])
    materials.append(mat)


def finish(obj, name, material, bevel=.002):
    obj.name = name
    obj.parent = visual
    obj.data.materials.append(materials[material])
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    if bevel:
        modifier = obj.modifiers.new('Machined edges', 'BEVEL')
        modifier.width = bevel
        modifier.segments = 1
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.modifier_apply(modifier=modifier.name)
    # Per-face metric planar mapping into a padded material tile. UVs are baked,
    # never inferred from the model in the renderer. Keep seams out of bevel edges.
    for layer in list(obj.data.uv_layers):
        obj.data.uv_layers.remove(layer)
    uv = obj.data.uv_layers.new(name='SurfaceUV')
    for face in obj.data.polygons:
        axis = max(range(3), key=lambda i: abs(face.normal[i]))
        a, b = [(1, 2), (0, 2), (0, 1)][axis]
        for loop in face.loop_indices:
            point = obj.data.vertices[obj.data.loops[loop].vertex_index].co
            u = (point[a] * 7) % 1
            v = (point[b] * 7) % 1
            uv.data[loop].uv = ((material % 2 + .08 + .84 * u) / 2,
                                (material // 2 + .08 + .84 * v) / 2)
    obj['salimon'] = {'role': 'mining-tool-visual'}
    return obj


def box(name, center, size, material, bevel=.002):
    bpy.ops.mesh.primitive_cube_add(size=1, location=center)
    obj = bpy.context.object
    obj.dimensions = size
    return finish(obj, name, material, bevel)


def cylinder(name, center, radius, depth, material, axis='X', vertices=12, bevel=.001):
    rotation = {'X': (0, math.pi / 2, 0), 'Y': (math.pi / 2, 0, 0), 'Z': (0, 0, 0)}[axis]
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius, depth=depth,
                                     location=center, rotation=rotation)
    return finish(bpy.context.object, name, material, bevel)

box('Housing', (.065, 0, .065), (.24, .10, .105), 0, .006)
box('Grip', (-.025, 0, -.04), (.065, .075, .15), 1, .004)
box('Battery', (-.065, 0, .06), (.06, .115, .10), 1, .004)
box('Top_Rail', (.06, 0, .128), (.19, .07, .025), 2)
for i in range(4):
    box(f'Cooling_Fin_{i}', (.066 + i * .023, 0, .07), (.006, .116, .074), 2, .001)
box('Status_Panel', (-.005, .057, .085), (.043, .014, .025), 3, .001)
box('Status_Top', (-.01, 0, .145), (.04, .045, .009), 3, .001)
cylinder('Emitter_Collar', (.225, 0, .065), .044, .11, 2)
cylinder('Emitter_Lens', (.285, 0, .065), .027, .012, 3)
# Keep stable original node names; group detailed subassemblies to bound primitives.
for side in (-1, 1):
    box(f'Panel_{side}', (.005, side * .052, .066), (.083, .006, .073), 0, .003)
    for x in (-.028, .033):
        for z in (.042, .092):
            cylinder(f'Fastener_{side}_{x}_{z}', (x, side * .057, z), .0045, .004, 2, 'Y', 6, 0)
            box(f'Screw_Slot_{side}_{x}_{z}', (x, side * .060, z), (.005, .001, .0015), 1, 0)
    for i in range(5):
        box(f'Vent_{side}_{i}', (.060 + i * .017, side * .059, .071), (.008, .003, .042), 1, 0)
for i in range(5):
    box(f'Grip_Rib_{i}', (-.025, 0, -.085 + i * .019), (.070, .079, .006), 1, .001)
box('Grip_Heel', (-.025, 0, -.112), (.076, .082, .012), 2)
box('Trigger', (.013, 0, -.012), (.013, .028, .021), 0)
box('Trigger_Guard_Base', (.020, 0, -.045), (.022, .018, .009), 2)
box('Trigger_Guard_Front', (.036, 0, -.018), (.009, .018, .060), 2)
box('Battery_Back_Panel', (-.097, 0, .06), (.005, .080, .055), 0, .001)
for y in (-.033, .033):
    for z in (.041, .079):
        cylinder(f'Battery_Fastener_{y}_{z}', (-.101, y, z), .0035, .003, 2, vertices=6, bevel=0)
box('Battery_Back_Label', (-.101, 0, .06), (.002, .038, .014), 2, 0)
box('Battery_Latch', (-.083, -.060, .065), (.020, .012, .028), 0)
for i in range(3):
    box(f'Battery_Contact_{i}', (-.055 + i * .011, -.061, .027), (.006, .009, .012), 2, 0)
for x in (.183, .252):
    cylinder(f'Emitter_Ring_{x}', (x, 0, .065), .048, .012, 1)
for i in range(6):
    angle = i * math.tau / 6
    cylinder(f'Emitter_Bolt_{i}', (.275, .037 * math.sin(angle), .065 + .037 * math.cos(angle)),
             .003, .004, 1, vertices=6, bevel=0)
box('Power_Switch', (-.058, 0, .117), (.019, .029, .008), 0, .001)

# Preserve original editable names; join added parts into four meaningful groups.
original = {'Housing', 'Grip', 'Battery', 'Top_Rail', 'Status_Panel', 'Status_Top',
            'Emitter_Collar', 'Emitter_Lens', *(f'Cooling_Fin_{i}' for i in range(4))}
for material, name in enumerate(('Housing_Details', 'Rubber_Details', 'Metal_Details', 'Status_Details')):
    extras = [o for o in visual.children if o.name not in original and o.type == 'MESH'
              and o.data.materials[0] == materials[material]]
    if extras:
        bpy.ops.object.select_all(action='DESELECT')
        for obj in extras:
            obj.select_set(True)
        bpy.context.view_layer.objects.active = extras[0]
        bpy.ops.object.join()
        extras[0].name = name
sockets = bpy.data.objects.new('Sockets', None)
bpy.context.collection.objects.link(sockets)
sockets.parent = root
grip = bpy.data.objects.new('SOCKET_Grip', None)
bpy.context.collection.objects.link(grip)
grip.parent = sockets
grip['salimon'] = {'role': 'first-person-grip'}
bpy.ops.wm.save_as_mainfile(filepath=str(HERE / 'source.blend'))
