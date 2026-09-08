"""Allowlisted operations, run in a disposable Blender 4.5.3 process."""
import json
from pathlib import Path
import sys

import bpy
from mathutils import Vector

PALETTE = [(0.09, 0.12, 0.16, 1), (0.24, 0.30, 0.37, 1),
           (0.14, 0.65, 0.72, 1), (0.83, 0.53, 0.18, 1)]


def inspect(root):
    return [{'name': obj.name, 'type': obj.type,
             'library': obj.library.filepath if obj.library else None,
             'matrixSource': [list(row) for row in obj.matrix_world],
             'dimensionsSourceMeters': list(obj.dimensions),
             'boundsRuntimeMeters': [[v.x, v.z, -v.y] for v in
                  (obj.matrix_world @ Vector(corner) for corner in obj.bound_box)]
                 if obj.type == 'MESH' else [],
             'materials': [slot.material.name if slot.material else None for slot in obj.material_slots]}
            for obj in sorted([root, *root.children_recursive], key=lambda obj: obj.name)]


def render(root, output):
    scene = bpy.context.scene
    scene.render.engine = 'CYCLES'
    scene.cycles.device = 'CPU'
    scene.cycles.samples = 8
    scene.render.resolution_x = scene.render.resolution_y = 384
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = 'PNG'
    if scene.world is None:
        scene.world = bpy.data.worlds.new('Preview_World')
    scene.view_settings.view_transform = 'Standard'
    scene.view_settings.exposure = 0
    scene.world.color = (0.12, 0.12, 0.12)
    points = [obj.matrix_world @ Vector(corner) for obj in [root, *root.children_recursive]
              if obj.type == 'MESH' and not obj.name.startswith('COLLIDER_') for corner in obj.bound_box]
    center = sum(points, Vector()) / len(points)
    radius = max((p-center).length for p in points)
    light = bpy.data.lights.new('Preview_Key', 'AREA')
    light.energy = 400 * radius * radius
    light.shape = 'DISK'
    light.size = radius * 3
    lamp = bpy.data.objects.new('Preview_Key', light)
    scene.collection.objects.link(lamp)
    lamp.location = center + Vector((2, -3, 4)) * radius
    lamp.rotation_euler = (center-lamp.location).to_track_quat('-Z', 'Y').to_euler()
    camera = bpy.data.objects.new('Preview_Camera', bpy.data.cameras.new('Preview_Camera'))
    scene.collection.objects.link(camera)
    scene.camera = camera
    camera.data.type = 'ORTHO'
    camera.data.ortho_scale = radius * 2.4
    for name, direction in [('front', (1, 0, 0.2)), ('side', (0, -1, 0.2)),
                             ('three-quarter', (1, -1, 0.8))]:
        camera.location = center + Vector(direction).normalized() * radius * 4
        camera.rotation_euler = (center-camera.location).to_track_quat('-Z', 'Y').to_euler()
        scene.render.filepath = str(output / (name + '.png'))
        bpy.ops.render.render(write_still=True)


def main(request):
    if bpy.app.version[:3] != (4, 5, 3):
        raise ValueError('authoring bridge v1 is pinned to Blender 4.5.3')
    scene = bpy.context.scene
    if scene.unit_settings.system != 'METRIC' or scene.unit_settings.scale_length != 1:
        raise ValueError('expected metric source, unit scale 1')
    manifest = request['manifest']
    root = scene.objects[manifest['contracts']['root']]
    output = Path(request['output'])
    before = inspect(root)
    operation = request['operation']
    if operation == 'sample_palette':
        if manifest['assetId'] != 'item.cargo-container':
            raise ValueError('sample edit is restricted to item.cargo-container')
        # Material-only vertical slice; geometry/anchors remain authoritative.
        materials = sorted({slot.material for obj in root.children_recursive
                            for slot in obj.material_slots if slot.material}, key=lambda m: m.name)
        for material, color in zip(materials, PALETTE):
            material.diffuse_color = color
            if material.use_nodes:
                bsdf = material.node_tree.nodes.get('Principled BSDF')
                if bsdf:
                    bsdf.inputs['Base Color'].default_value = color
                    bsdf.inputs['Metallic'].default_value = 0
                    bsdf.inputs['Roughness'].default_value = 0.8
        bpy.ops.wm.save_as_mainfile(filepath=str(output / 'candidate.blend'))
    if operation in {'preview_asset', 'sample_palette'}:
        render(root, output)
    after = inspect(root)
    if before != after:
        raise ValueError('operation changed geometry or named interfaces')
    (output / 'inspection.json').write_text(json.dumps({
        'assetId': manifest['assetId'], 'blender': bpy.app.version_string,
        'operation': operation, 'objects': after}, indent=2, sort_keys=True))


if __name__ == '__main__':
    main(json.loads(Path(sys.argv[sys.argv.index('--') + 1]).read_text()))
