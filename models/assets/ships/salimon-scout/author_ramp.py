"""Reauthor the linked aft boarding ramp (run with Blender --background --python)."""
import math
from pathlib import Path
import bpy

ASSET = Path(__file__).resolve().parent
FLOOR = 0.24731182795698925
TOP = 2.516129032
LENGTH = 4.4
FOOT = -2.10752688172043
PIVOT = [-7.68, FLOOR, 0.0]


def mesh(name, boxes, materials):
    old = bpy.data.objects.get(name)
    parent = old.parent if old else bpy.data.objects['Interior']
    collection = bpy.data.collections['Scout_exit-door']
    if old:
        old_data = old.data
        bpy.data.objects.remove(old, do_unlink=True)
        if old_data.users == 0:
            bpy.data.meshes.remove(old_data)
    vertices, faces, slots = [], [], []
    for center, size, material in boxes:
        start = len(vertices)
        x, y, z = center
        a, b, c = (v / 2 for v in size)
        vertices.extend((x + sx*a, -(z + sz*c), y + sy*b) for sx, sy, sz in
                        [(-1,-1,-1),(-1,-1,1),(-1,1,-1),(-1,1,1),
                         (1,-1,-1),(1,-1,1),(1,1,-1),(1,1,1)])
        for face in [(0,4,6,2),(1,3,7,5),(0,1,5,4),(2,6,7,3),(0,2,3,1),(4,5,7,6)]:
            faces.append(tuple(start + i for i in face))
            slots.append(material)
    data = bpy.data.meshes.new(name)
    data.from_pydata(vertices, [], faces)
    data.update()
    for material in materials:
        data.materials.append(bpy.data.materials[material])
    uv = data.uv_layers.new(name='UVMap')
    for polygon, slot in zip(data.polygons, slots):
        polygon.material_index = slot
        for loop, coord in zip(polygon.loop_indices, [(0,0),(1,0),(1,1),(0,1)]):
            uv.data[loop].uv = coord
    obj = bpy.data.objects.new(name, data)
    collection.objects.link(obj)
    obj.parent = parent
    return obj


def author():
    bpy.ops.wm.open_mainfile(filepath=str(ASSET / 'components/exit-door/source.blend'))
    height = TOP - FLOOR
    # The inward face becomes the ramp's upper walking plane. Everything else
    # is behind that face, so details cannot protrude through the player's feet.
    boxes = [((-7.77, (FLOOR+TOP)/2, 0), (.18, height, 2.80), 0),
             ((-7.875, (FLOOR+TOP)/2, 0), (.01, height-.12, 2.60), 1)]
    for z in [-1.35, 1.35]:
        boxes.append(((-7.73, (FLOOR+TOP)/2, z), (.10, height, .10), 2))
    # Recessed anti-slip strips on the interior walking face.
    for i in range(10):
        boxes.append(((-7.686, FLOOR+.10+i*.22, 0), (.012, .035, 2.55), 2))
    door = mesh('Exit_Door', boxes, ['Hull Graphite', 'Petrol Teal', 'Copper Accent'])
    door['salimon'] = {'interactive': 'exit-door', 'pivot': PIVOT,
                       'openAngleRadians': math.pi/2 + math.asin((FLOOR-FOOT)/LENGTH),
                       'rampLengthMeters': LENGTH, 'extensionTravelMeters': LENGTH-height,
                       'extensionFaceOffsetMeters': .08, 'rampHalfWidthMeters': 1.4}
    # Nested tray is concealed within the leaf when sealed, then slides aft.
    tip = mesh('Exit_Ramp_Tip', [((-7.78, (FLOOR+TOP)/2, 0), (.04, height, 2.80), 0)],
               ['Hull Graphite'])
    tip['salimon'] = {'interactive': 'exit-ramp-tip'}
    # Flush threshold bridges the cabin to the hinge; bottom hinge and frame
    # bolts sit outside the sealed aperture instead of obstructing the walkway.
    mesh('Door_Threshold', [((-7.40, FLOOR-.04, 0), (.56, .08, 2.80), 0)], ['Hull Graphite'])
    frame_boxes = []
    for z in [-1.53, 1.53]:
        frame_boxes += [((-7.77, (FLOOR+TOP)/2, z), (.22, height+.18, .18), 0),
                        ((-7.87, FLOOR, z), (.18, .18, .24), 1)]
        for y in [FLOOR+.4, TOP-.4]:
            frame_boxes.append(((-7.90, y, z), (.10, .16, .13), 1))
    frame_boxes.append(((-7.77, TOP+.09, 0), (.22, .18, 3.24), 0))
    mesh('Door_Frame', frame_boxes, ['Hull Graphite', 'Copper Accent'])
    proxy = bpy.data.objects['COLLIDER_AftDoor']
    proxy.location = (-7.78, 0, (FLOOR+TOP)/2)
    proxy['salimon']['sizeMeters'] = [.20, height, 2.8]
    bpy.ops.wm.save_as_mainfile(filepath=str(ASSET / 'components/exit-door/source.blend'))


if __name__ == '__main__':
    author()
