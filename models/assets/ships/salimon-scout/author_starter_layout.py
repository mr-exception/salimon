"""Author the #538 starter envelope in the linked cabin library (Blender 4.5.3).

Runtime meters; no hull scaling or existing anchor migration. Cargo door operation
and bin intake belong to #539; habitation interactions belong to #540.
"""
from pathlib import Path
import sys
import bpy
sys.path.insert(0, str(Path(__file__).resolve().parent))
from author_fabricator import Mesh, point

ASSET = Path(__file__).resolve().parent
SOURCE = ASSET / 'components/cabin-interior/source.blend'
FLOOR = 0.24731182795698925
TOP = 2.8064515590667725
OPENING_TOP = 2.486548387096774


def author():
    bpy.ops.wm.open_mainfile(filepath=str(SOURCE))
    collection = bpy.data.collections['Scout_cabin-interior']
    for obj in list(bpy.data.objects):
        if obj.library is None and obj.name.startswith(('Starter_', 'COLLIDER_Starter_', 'MARKER_Starter_')):
            bpy.data.objects.remove(obj, do_unlink=True)

    def boxes(name, material, entries):
        mesh = Mesh()
        for center, size in entries:
            mesh.box(center, size)
        return mesh.publish('Starter_' + name, material, collection)

    def empty(name, position, group, metadata):
        obj = bpy.data.objects.new(name, None)
        collection.objects.link(obj)
        obj.parent = bpy.data.objects[group]
        obj.location = point(position)
        obj['salimon'] = metadata

    def collider(name, center, size):
        empty('COLLIDER_Starter_' + name, center, 'Collision_Proxies',
              {'collisionShape': 'box', 'sizeMeters': size, 'purpose': 'starter-' + name.lower()})

    # Split the existing ceiling fixtures around the new structural wall.
    # Keep their names, materials, heights and all unaffected extents. These
    # source bounds predate #538; never infer boxes from already split output.
    fixtures = {
        'Ceiling_Warm_Lights': ('Warm Lamp', [(-6.6,-2.1,2.693548,2.709677),(-.6,4.,2.693548,2.709677)], [(3.35,3.57),(-3.57,-3.35)]),
        'Ceiling_Lamp_Housings': ('Copper Accent', [(-6.72,-1.98,2.698925,2.806452),(-.72,4.12,2.698925,2.806452)], [(3.22,3.7),(-3.7,-3.22)]),
        'Ceiling_Teal_Inset_Panels': ('Hull Ceramic', [(-6.54,-3.7,2.793548,2.805376),(-3.1,.5,2.793548,2.805376),(1.1,4.1,2.793548,2.805376)], [(-1.14,1.14)]),
    }
    for name, (material, segments, widths) in fixtures.items():
        mesh = Mesh()
        for x0,x1,y0,y1 in segments:
            intervals = [(x0,-4.76),(-4.54,x1)] if x0 < -4.65 < x1 else [(x0,x1)]
            if name == 'Ceiling_Warm_Lights' and len(intervals) == 2:
                intervals = [(x0,-4.765),(-4.535,x1)]
            for a,b in intervals:
                for z0,z1 in widths:
                    mesh.box([(a+b)/2,(y0+y1)/2,(z0+z1)/2], [b-a,y1-y0,z1-z0])
        if name == 'Ceiling_Lamp_Housings':
            mesh.faces = [f for f in mesh.faces if not all(abs(mesh.vertices[i][2]-2.806452)<1e-6 for i in f)]
        old = bpy.data.objects[name]
        replacement = mesh.publish('Starter_TemporaryFixture', material, collection)
        old.data = replacement.data
        bpy.data.objects.remove(replacement, do_unlink=True)

    # Full-height wall with an unobstructed 3 m opening; leave 0.24 m between
    # the forward face and the existing Fabricator. No hull/glass intersections.
    height = TOP - FLOOR + .005
    wall = []
    for side in (-1, 1):
        center = [-4.65, FLOOR - .005 + height / 2, side * 2.85]
        size = [.12, height, 2.7]
        wall.append((center, size))
        collider('Bulkhead' + ('Port' if side > 0 else 'Starboard'), center, size)
    # One U-shaped extrusion has no shared side/header caps.
    bulkhead = Mesh()
    bulkhead.prism([(-4.195, FLOOR-.005), (-1.5, FLOOR-.005),
                    (-1.5, OPENING_TOP+.055), (1.5, OPENING_TOP+.055),
                    (1.5, FLOOR-.005), (4.195, FLOOR-.005),
                    (4.195, TOP), (-4.195, TOP)], -4.71, -4.59)
    bulkhead.vertices = [(-y, -x, z) for x,y,z in bulkhead.vertices]
    bulkhead.faces = [tuple(reversed(face)) for face in bulkhead.faces]
    # The ceiling owns the hidden top contact surface.
    bulkhead.faces = [f for f in bulkhead.faces if not all(abs(bulkhead.vertices[i][2]-TOP)<1e-6 for i in f)]
    bulkhead.publish('Starter_CargoBulkhead', 'Interior Light', collection)
    boxes('DoorFrame', 'Hull Graphite', [
        ([-4.65, FLOOR + 1.09, side * 1.56], [.20, 2.2, .10]) for side in (-1, 1)
    ] + [([-4.65, OPENING_TOP + .03, 0], [.20, .06, 3.02])])
    collider('Header', [-4.65, (TOP+OPENING_TOP)/2, 0], [.20, TOP-OPENING_TOP, 3.02])

    # Open physical storage shells. The central ramp/carrying lane is 5.5 m wide.
    bins = Mesh()
    for side in (-1, 1):
        z = side * 3.45
        base = len(bins.vertices)
        for x0, x1, z0, z1, y in [
            (-6.9, -5.3, z-.6, z+.6, FLOOR-.005),
            (-6.9, -5.3, z-.6, z+.6, FLOOR+.84),
            (-6.86, -5.34, z-.56, z+.56, FLOOR+.84),
            (-6.86, -5.34, z-.56, z+.56, FLOOR+.12),
        ]:
            bins.vertices += [point(p) for p in [(x0,y,z0),(x1,y,z0),(x1,y,z1),(x0,y,z1)]]
        bins.faces += [tuple(base+i for i in (3,2,1,0)), tuple(base+i for i in (12,13,14,15))]
        for i in range(4):
            j = (i+1)%4
            bins.faces += [tuple(base+k for k in f) for f in
                           [(i,j,4+j,4+i), (4+i,4+j,8+j,8+i), (8+i,8+j,12+j,12+i)]]
        collider('Bin' + ('Port' if side > 0 else 'Starboard'),
                 [-6.1, FLOOR + .42, z], [1.6, .84, 1.2])
    bins.publish('Starter_CargoBins', 'Hull Graphite', collection)

    # Fold-away bunk is stowed against the starboard wall; its fixture envelope
    # also reserves the deployed 0.85 m berth for later #540 interaction.
    boxes('BunkFrame', 'Hull Graphite', [([-.8, 1.25, -3.9], [2.2, 1.7, .18])])
    boxes('BunkCushion', 'Terracotta Upholstery', [([-.8, 1.25, -3.785], [2.0, 1.48, .06])])
    collider('Bunk', [-.8, 1.12, -3.65], [2.2, 1.75, .85])

    boxes('Galley', 'Interior Dark', [([.65, .76, 3.7], [1.6, 1.02, .7])])
    boxes('GalleyWorktop', 'Honey Wood', [([.65, 1.295, 3.7], [1.7, .06, .76])])
    boxes('GalleyBasin', 'Hull Graphite', [([.22, 1.337, 3.7], [.50, .014, .48])])
    boxes('GalleyTap', 'Copper Accent', [([.22, 1.43, 3.98], [.06, .20, .06])])
    collider('Galley', [.65, .79, 3.7], [1.7, 1.08, .76])

    boxes('Hygiene', 'Interior Light', [([1.5, .63, -3.65], [.8, .76, .8]),
                                           ([1.5, 1.195, -3.95], [.74, .38, .16])])
    boxes('HygieneSeat', 'Interior Dark', [([1.5, 1.032, -3.65], [.64, .06, .64])])
    collider('Hygiene', [1.5, .82, -3.65], [.8, 1.14, .8])

    for name, position in [('CargoDoor', [-4.65, 1.35, 0]),
                           ('AftExpansion', [-7.68, FLOOR, 0]),
                           ('UpperExpansion', [0, TOP, 0])]:
        empty('MARKER_Starter_' + name, position, 'Interaction_Markers',
              {'purpose': 'starter-' + name.lower()})
    bpy.ops.wm.save_as_mainfile(filepath=str(SOURCE))


if __name__ == '__main__':
    author()
