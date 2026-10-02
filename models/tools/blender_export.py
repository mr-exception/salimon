"""Executed only in a fresh headless Blender process by export_asset.py."""
import json
from pathlib import Path
import sys

import bpy


def export(manifest, output):
    scene = bpy.context.scene
    if scene.unit_settings.system != 'METRIC' or scene.unit_settings.scale_length != 1:
        raise ValueError(f'{manifest["assetId"]}: source must use metric units with unit scale 1')
    root = scene.objects.get(manifest['contracts']['root'])
    if root is None or root.parent is not None:
        raise ValueError('asset root must exist in the active scene and have no parent')
    objects = [root, *root.children_recursive]
    if any(obj.type not in {'EMPTY', 'MESH'} for obj in objects):
        raise ValueError('v1 supports static meshes/empties only; move authoring helpers outside the asset root')
    if any(obj.animation_data is not None for obj in objects):
        raise ValueError('animated assets need a future category export adapter')
    for obj in scene.objects:
        obj.select_set(False)
    # Proxy shapes remain editable in the source but export as transform/extras nodes.
    for obj in objects:
        if obj.name.startswith('COLLIDER_') and obj.type == 'MESH':
            proxy = bpy.data.objects.new(obj.name + '_export', None)
            scene.collection.objects.link(proxy)
            proxy.parent = obj.parent
            proxy.matrix_local = obj.matrix_local.copy()
            for key in obj.keys():
                proxy[key] = obj[key]
            proxy['salimonProxyDimensions'] = list(obj.dimensions)
            for child in list(obj.children):
                transform = child.matrix_local.copy()
                child.parent = proxy
                child.matrix_local = transform
            name = obj.name
            bpy.data.objects.remove(obj, do_unlink=True)
            proxy.name = name
            proxy.select_set(True)
        else:
            obj.hide_set(False)
            obj.hide_select = False
            obj.select_set(True)
    bpy.context.view_layer.objects.active = root
    result = bpy.ops.export_scene.gltf(
        filepath=str(output), export_format='GLB', check_existing=False,
        use_selection=True, export_extras=True, export_yup=True,
        export_cameras=False, export_lights=False, export_animations=False,
        export_apply=True, export_materials='EXPORT',
    )
    if result != {'FINISHED'}:
        raise RuntimeError(f'glTF exporter returned {result}')


if __name__ == '__main__':
    args = sys.argv[sys.argv.index('--') + 1:]
    export(json.loads(Path(args[0]).read_text()), Path(args[1]))
