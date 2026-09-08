"""Reviewed visual-only migration of saved sources; export with export_asset.py.

Blender 4.5.3 --background --python-exit-code 1 --python <this file>.
No vertex, polygon, UV, material-slot, socket or transform changes.
"""
from pathlib import Path
import sys
import bpy

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / 'models/tools'))
from resource_style import RESOURCE_COLORS, tool_pixel


def main():
    if bpy.app.version[:3] != (4, 5, 3):
        raise ValueError('authoring requires Blender 4.5.3')
    bpy.context.preferences.filepaths.save_version = 0
    sources = sorted((REPO / 'models/assets/resources').glob('*-deposit-*/source.blend'))
    sources.append(REPO / 'models/assets/items/mining-tool/source.blend')
    for source in sources:
        bpy.ops.wm.open_mainfile(filepath=str(source))
        for material in bpy.data.materials:
            if material.name in RESOURCE_COLORS:
                color = RESOURCE_COLORS[material.name]
                material.diffuse_color = color
                material.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value = color
        if source.parent.name == 'mining-tool':
            image = bpy.data.images['Tool_Surface']
            image.pixels = [c for y in range(256) for x in range(256) for c in tool_pixel(x, y)]
            image.filepath_raw = str(source.with_name('surface.png'))
            image.file_format = 'PNG'
            image.save()
            image.pack()
        for obj in bpy.data.objects:
            if obj.type == 'MESH':
                for face in obj.data.polygons:
                    face.use_smooth = False
        bpy.ops.wm.save_as_mainfile(filepath=str(source), compress=True)


if __name__ == '__main__':
    main()
