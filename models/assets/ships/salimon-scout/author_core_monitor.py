"""Author a Core-local status screen in the energy-core component (Blender 4.5)."""
from pathlib import Path
import bpy

ASSET = Path(__file__).resolve().parent
# Ship-local metres: starboard-facing screen, inside the existing Core collider.
CORNERS = [(-.35, 1.90, -.999), (-.35, 1.25, -.999),
           (-1.65, 1.25, -.999), (-1.65, 1.90, -.999)]
METADATA = dict(interactive='core-monitor', displayRole='energy-core', uvOrigin='top-left')


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
    bpy.ops.wm.save_as_mainfile(filepath=str(source))


if __name__ == '__main__':
    author()
