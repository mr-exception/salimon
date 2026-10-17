"""Matched GLB review views, independent of saved-source material preview.

blender -b --python-exit-code 1 --python <this file> --
 --assets client/assets --output artifacts/issue-277/after-renders
Before exports may use --assets artifacts/issue-277/before with the same tree.
"""
import argparse
from pathlib import Path
import sys
import bpy
from mathutils import Vector

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(Path(__file__).parent))
from build_reference import setup, light, camera


def render(assets, output, subject, far):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = setup()
    if subject == 'tool':
        paths = [assets / 'items/mining-tool/model.glb']
        offsets = [0]
        scale, target = .60, (0, 0, .02)
    else:
        paths = [assets / 'resources' / name / 'model.glb' for name in (
            'iron-deposit-nodule', 'silicate-deposit-boulder',
            'water-ice-deposit-spire', 'spherical-fragment')]
        offsets = [-1.5, -.5, .5, 1.5]
        scale, target = 4.5, (0, 0, 0)
    for path, offset in zip(paths, offsets):
        bpy.ops.object.select_all(action='DESELECT')
        bpy.ops.import_scene.gltf(filepath=str(path))
        for obj in bpy.context.selected_objects:
            if obj.parent is None:
                obj.location.x += offset
    light('Key', (2, -4, 6), target, 400, 5)
    light('Fill', (-3, 1, 2), target, 160, 4)
    camera((2, -5, 3), target, scale * (2 if far else 1))
    name = subject + ('-far' if far else '-near')
    output.mkdir(parents=True, exist_ok=True)
    scene.render.filepath = str(output / (name + '.png'))
    bpy.ops.render.render(write_still=True)


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--assets', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args(sys.argv[sys.argv.index('--')+1:])
    for subject in ('tool', 'resources'):
        for far in (False, True):
            render(a.assets.resolve(), a.output.resolve(), subject, far)
