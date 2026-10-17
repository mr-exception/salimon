"""Author a Core-local status screen in the energy-core component (Blender 4.5)."""
from pathlib import Path
import bpy
import bmesh

ASSET = Path(__file__).resolve().parent
# Ship-local metres: starboard-facing screen, inside the existing Core collider.
CORNERS = [(-.35, 1.90, -.999), (-.35, 1.25, -.999),
           (-1.65, 1.25, -.999), (-1.65, 1.90, -.999)]
METADATA = dict(interactive='core-monitor', displayRole='energy-core', uvOrigin='top-left')


def author_frame():
    """An open bezel, closed rear housing and pedestal supports, in runtime metres.

    The original display is only 1 mm inside the collider's starboard face.
    Keep that face fixed and put enclosure depth behind it, not in the walkway.
    """
    def append_geometry(name, vertices, faces):
        # Keep the existing node/primitive budget. A named vertex group makes
        # the added parts selectable/editable and repeatable without touching
        # the original pedestal/armor/cradle vertices or material assignments.
        obj = bpy.data.objects[name]
        group = obj.vertex_groups.get('Core_Monitor_Frame')
        if group is None:
            group = obj.vertex_groups.new(name='Core_Monitor_Frame')
        bm = bmesh.new()
        bm.from_mesh(obj.data)
        deform = bm.verts.layers.deform.verify()
        previous = [v for v in bm.verts if v[deform].get(group.index, 0) > .5]
        bmesh.ops.delete(bm, geom=previous, context='VERTS')
        used = sorted({index for face in faces for index in face})
        added = {i: bm.verts.new((vertices[i][0], -vertices[i][2], vertices[i][1]))
                 for i in used}
        for vertex in added.values():
            vertex[deform][group.index] = 1
        for indices in faces:
            face = bm.faces.new([added[i] for i in indices])
            face.material_index = 0
            face.smooth = False
        bm.to_mesh(obj.data)
        bm.free()
        obj.data.update()

    def rectangle(x0, x1, y0, y1, z):
        # Clockwise viewed from the starboard approach (outward normal -Z).
        return [(x0, y0, z), (x0, y1, z), (x1, y1, z), (x1, y0, z)]

    # 3 mm aperture clearance preserves every display corner. The backing is
    # 24 mm behind the live face; the tapered side wall catches a broad highlight.
    vertices = (rectangle(-1.70, -.30, 1.20, 1.95, -1.0) +
                rectangle(-1.653, -.347, 1.247, 1.903, -1.0) +
                rectangle(-1.71, -.29, 1.19, 1.96, -.85) +
                rectangle(-1.653, -.347, 1.247, 1.903, -.975))
    faces, slots = [], []
    for i in range(4):
        j = (i + 1) % 4
        faces += [(i, j, 4+j, 4+i), (i, 8+i, 8+j, j),
                  (4+i, 4+j, 12+j, 12+i)]
        slots += [1, 0, 0]
    faces += [(11, 10, 9, 8), (12, 13, 14, 15)]
    slots += [0, 0]
    append_geometry('Core_Pedestal', vertices,
                    [face for face, slot in zip(faces, slots) if slot == 0])
    append_geometry('Core_Ceramic_Armor', vertices,
                    [face for face, slot in zip(faces, slots) if slot == 1])

    vertices, faces = [], []
    for x0, x1 in [(-1.65, -1.50), (-.50, -.35)]:
        start = len(vertices)
        vertices += (rectangle(x0, x1, .45, 1.195, -.97) +
                     rectangle(x0, x1, .45, 1.195, -.87))
        # Top caps are concealed inside the enclosure and deliberately absent.
        faces += [tuple(start+k for k in face) for face in
                  [(0, 1, 2, 3), (7, 6, 5, 4), (0, 4, 5, 1),
                   (3, 2, 6, 7), (0, 3, 7, 4)]]
    append_geometry('Core_Cradle', vertices, faces)


def author():
    source = ASSET / 'components/energy-core/source.blend'
    bpy.ops.wm.open_mainfile(filepath=str(source))
    collection = bpy.data.collections['Scout_energy-core']
    previous = bpy.data.objects.get('Core_Monitor')
    if previous:
        bpy.data.objects.remove(previous, do_unlink=True)
    for mesh in list(bpy.data.meshes):
        if mesh.users == 0:
            bpy.data.meshes.remove(mesh)
    data = bpy.data.meshes.new('Core_Monitor')
    data.from_pydata([(x, -z, y) for x, y, z in CORNERS], [], [(0, 1, 2, 3)])
    # Reuse the shared screen material without adding runtime materials/draws.
    material = bpy.data.materials.get('Cyan Display')
    if material is None:
        with bpy.data.libraries.load(str(ASSET / 'assembly/shared.blend'), link=True) as (src, dst):
            dst.materials = ['Cyan Display']
        material = dst.materials[0]
        material.library.filepath = '//../../assembly/shared.blend'
    data.materials.append(material)
    obj = bpy.data.objects.new('Core_Monitor', data)
    collection.objects.link(obj)
    obj.parent = bpy.data.objects['Interior']
    obj['salimon'] = METADATA
    uv = data.uv_layers.new(name='UVMap')
    for loop, coord in zip(data.polygons[0].loop_indices, [(0, 1), (0, 0), (1, 0), (1, 1)]):
        uv.data[loop].uv = coord
    author_frame()
    bpy.ops.wm.save_as_mainfile(filepath=str(source))


if __name__ == '__main__':
    author()
