"""Run with Blender 4.5: --background --python bootstrap.py [-- --verify].

One-time import of the legacy v11 scout; never exports or writes runtime assets.
--verify opens the committed source and checks its imported preservation contract.
"""
import hashlib
import json
from pathlib import Path
import struct
import sys

import bpy
from mathutils import Vector

ASSET = Path(__file__).resolve().parent
REPO = ASSET.parents[3]
LEGACY = REPO / 'client/assets/ship'


def plain(value):
    if hasattr(value, 'to_dict'):
        return {key: plain(item) for key, item in value.items()}
    if hasattr(value, 'to_list'):
        return value.to_list()
    return value


def verify(document):
    scene = bpy.context.scene
    assert scene.unit_settings.system == 'METRIC'
    assert scene.unit_settings.scale_length == 1
    objects = {obj.name: obj for obj in scene.objects}
    expected = {node['name']: node for node in document['nodes']}
    assert objects.keys() == expected.keys(), 'Missing/renamed/extra objects'
    binary = (LEGACY / 'export/salimon_phase0_ship.bin').read_bytes()

    def accessor(index):
        acc = document['accessors'][index]
        view = document['bufferViews'][acc['bufferView']]
        width = {'VEC3': 3, 'VEC2': 2, 'SCALAR': 1}[acc['type']]
        fmt = {5126: 'f', 5123: 'H', 5125: 'I'}[acc['componentType']] * width
        size = struct.calcsize('<' + fmt)
        start = view.get('byteOffset', 0) + acc.get('byteOffset', 0)
        return [struct.unpack_from('<' + fmt, binary, start + i * view.get('byteStride', size))
                for i in range(acc['count'])]

    parents = {document['nodes'][child]['name']: node['name']
               for node in document['nodes'] for child in node.get('children', [])}
    triangles = 0
    for name, node in expected.items():
        obj = objects[name]
        assert (obj.parent.name if obj.parent else None) == parents.get(name), name
        assert {key: plain(obj[key]) for key in node.get('extras', {})} == node.get('extras', {}), name
        # Blender's importer applies the inverse Y-up conversion once.
        position = obj.matrix_local.translation
        runtime_position = (position.x, position.z, -position.y)
        assert max(abs(a-b) for a, b in zip(runtime_position, node.get('translation', [0, 0, 0]))) < 1e-5, name
        assert max(abs(scale-1) for scale in obj.scale) < 1e-6, name
        if 'mesh' not in node:
            assert obj.type == 'EMPTY', name
            continue
        assert obj.type == 'MESH', name
        mesh = obj.data
        mesh.calc_loop_triangles()
        primitives = document['meshes'][node['mesh']]['primitives']
        count = sum(document['accessors'][p['indices']]['count']//3 for p in primitives)
        assert len(mesh.loop_triangles) == count, name
        triangles += count
        # Every imported vertex must match the interchange geometry in meters.
        positions = [v for p in primitives for v in accessor(p['attributes']['POSITION'])]
        for vertex in mesh.vertices:
            v = obj.matrix_local @ vertex.co
            runtime = Vector((v.x, v.z, -v.y))
            assert min((runtime-Vector(p)).length for p in positions) < 1e-5, name
        material_names = {document['materials'][p['material']]['name'] for p in primitives}
        assert {mat.name for mat in mesh.materials} == material_names, name
        for primitive in primitives:
            material = document['materials'][primitive['material']]['name']
            assert sum(mesh.materials[t.material_index].name == material for t in mesh.loop_triangles) == document['accessors'][primitive['indices']]['count']//3, name
        if name.startswith('Monitor_') and name in ('Monitor_Center', 'Monitor_Port', 'Monitor_Starboard'):
            assert count == 2 and len(mesh.uv_layers) == 1, name
            primitive = primitives[0]
            source_positions = accessor(primitive['attributes']['POSITION'])
            source_uvs = accessor(primitive['attributes']['TEXCOORD_0'])
            for loop in mesh.loops:
                v = obj.matrix_local @ mesh.vertices[loop.vertex_index].co
                runtime = Vector((v.x, v.z, -v.y))
                index = min(range(len(source_positions)), key=lambda i: (runtime-Vector(source_positions[i])).length)
                uv = mesh.uv_layers.active.data[loop.index].uv
                assert max(abs(uv.x-source_uvs[index][0]), abs(1-uv.y-source_uvs[index][1])) < 1e-6, name
    assert triangles == sum(document['accessors'][p['indices']]['count']//3
                            for mesh in document['meshes'] for p in mesh['primitives'])
    assert {mat.name for mat in bpy.data.materials} == {mat['name'] for mat in document['materials']}
    assert all(image.packed_file for image in bpy.data.images if image.type == 'IMAGE' and image.size[0]), 'Unpacked textures'
    print(json.dumps({'assetId': 'ship.salimon-scout', 'objects': len(objects),
                      'triangles': triangles, 'materials': len(bpy.data.materials),
                      'result': 'preserved'}, sort_keys=True))


def main():
    document = json.loads((LEGACY / 'export/salimon_phase0_ship.gltf').read_text())
    runtime = LEGACY / 'export/salimon_phase0_ship.glb'
    before = hashlib.sha256(runtime.read_bytes()).hexdigest()
    baseline = json.loads((ASSET / 'preservation.json').read_text())
    assert before == baseline['runtimeSha256'], 'Legacy GLB changed; review the migration baseline first'
    if '--verify' in sys.argv:
        bpy.ops.wm.open_mainfile(filepath=str(ASSET / 'source.blend'))
    else:
        if (ASSET / 'source.blend').exists():
            raise RuntimeError('Source already exists; use --verify to inspect it. Do not overwrite authored edits.')
        bpy.ops.wm.read_factory_settings(use_empty=True)
        bpy.ops.import_scene.gltf(filepath=str(runtime))
        bpy.context.scene.unit_settings.system = 'METRIC'
        bpy.context.scene.unit_settings.scale_length = 1
        bpy.ops.file.pack_all()
        # Metadata-only legacy colliders stay empties until issue #81.
        for obj in bpy.context.scene.objects:
            if obj.name.startswith('COLLIDER_'):
                obj.empty_display_type = 'CUBE'
                obj.empty_display_size = 0.25
        bpy.ops.wm.save_as_mainfile(filepath=str(ASSET / 'source.blend'))
        bpy.ops.wm.open_mainfile(filepath=str(ASSET / 'source.blend'))
    verify(document)
    assert hashlib.sha256(runtime.read_bytes()).hexdigest() == before, 'Runtime changed'


if __name__ == '__main__':
    main()
