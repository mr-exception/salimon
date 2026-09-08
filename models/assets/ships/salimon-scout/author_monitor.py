"""Reauthor the single wide console display with Blender 4.5 LTS."""
from pathlib import Path
import bpy

ASSET = Path(__file__).resolve().parent
# Final runtime meters; source conversion is applied exactly once by export.
CORNERS = [(4.42, 1.2067, -.99), (4.42, .8735, -.99),
           (4.42, .8735, .99), (4.42, 1.2067, .99)]
METADATA = dict(interactive='cockpit-monitor', displayRole='multifunction',
                uvOrigin='top-left', powerSource='shared-thruster-command',
                pilotFacingTargetMeters=[2.76, 1.799032258064516, 0.0],
                pilotFacingYawDegrees=0.0)


def author():
    source = ASSET / 'components/cockpit-console/source.blend'
    bpy.ops.wm.open_mainfile(filepath=str(source))
    collection = bpy.data.collections['Scout_cockpit-console']
    parent = bpy.data.objects['Interior']
    display = bpy.data.objects.get('Monitor_Center') or bpy.data.objects['Monitor_Wide']
    display_material = display.data.materials[0]
    bezel_material = bpy.data.objects['Cockpit_Monitor_Bezels'].data.materials[0].name
    for obj in list(collection.objects):
        if obj.name.startswith('Monitor_Wide'):
            bpy.data.objects.remove(obj, do_unlink=True)
    for name in ('Monitor_Center', 'Monitor_Port', 'Monitor_Starboard',
                 'Cockpit_Center_Monitor_Bridge', 'Cockpit_Monitor_Housings',
                 'Cockpit_Monitor_Bezels'):
        obj = bpy.data.objects.get(name)
        if obj:
            bpy.data.objects.remove(obj, do_unlink=True)

    for data in list(bpy.data.meshes):
        if data.users == 0:
            bpy.data.meshes.remove(data)

    def mesh(name, vertices, faces, material):
        data = bpy.data.meshes.new(name)
        data.from_pydata([(x, -z, y) for x, y, z in vertices], [], faces)
        data.materials.append(material)
        data.update()
        obj = bpy.data.objects.new(name, data)
        collection.objects.link(obj)
        obj.parent = parent
        return obj

    screen = mesh('Monitor_Wide', CORNERS, [(0, 1, 2, 3)], display_material)
    screen['salimon'] = METADATA
    uv = screen.data.uv_layers.new(name='UVMap')
    for loop, coord in zip(screen.data.polygons[0].loop_indices,
                           [(0, 1), (0, 0), (1, 0), (1, 1)]):
        uv.data[loop].uv = coord

    def boxes(name, specs, material_name):
        vertices, faces = [], []
        for center, size in specs:
            start = len(vertices)
            vertices += [tuple(c+s*d/2 for c,s,d in zip(center, signs, size))
                         for signs in [(-1,-1,-1),(-1,-1,1),(-1,1,-1),(-1,1,1),
                                       (1,-1,-1),(1,-1,1),(1,1,-1),(1,1,1)]]
            faces += [tuple(start+i for i in face) for face in
                      [(0,4,6,2),(1,3,7,5),(0,1,5,4),(2,6,7,3),(0,2,3,1),(4,5,7,6)]]
        obj = mesh(name, vertices, faces, bpy.data.materials[material_name])
        uv = obj.data.uv_layers.new(name='UVMap')
        for polygon in obj.data.polygons:
            for loop, coord in zip(polygon.loop_indices, [(0,0),(1,0),(1,1),(0,1)]):
                uv.data[loop].uv = coord
        return obj

    # Recessed face with a continuous rim, rear enclosure and two short mounts.
    boxes('Cockpit_Monitor_Housings', [((4.49, 1.0401, 0), (.12, .3932, 2.10)),
          ((4.51, .78, -.68), (.14, .17, .14)),
          ((4.51, .78, .68), (.14, .17, .14))], 'Hull Graphite')
    boxes('Cockpit_Monitor_Bezels', [((4.425, y, 0), (.05, .028, 2.10))
          for y in [.8595, 1.2207]] +
          [((4.425, 1.0401, z), (.05, .3332, .06)) for z in [-1.02, 1.02]], bezel_material)
    bpy.ops.wm.save_as_mainfile(filepath=str(source))


if __name__ == '__main__':
    author()
