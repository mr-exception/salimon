"""Check linked ownership and component edit propagation in a disposable copy."""
import json
from pathlib import Path
import shutil
import tempfile

import bpy

ASSET = Path(__file__).resolve().parent
COMPONENTS = ('hull', 'cockpit', 'wings', 'engines', 'exit-door', 'pilot-seat',
              'cockpit-console', 'energy-core', 'cabin-interior')


def verify(asset):
    assembly = asset / 'assembly/salimon-scout.blend'
    bpy.ops.wm.open_mainfile(filepath=str(assembly))
    assert len(bpy.context.scene.collection.children) == 10
    assert all(lib.filepath.startswith('//') for lib in bpy.data.libraries), 'Absolute library path'
    assert all(Path(bpy.path.abspath(lib.filepath)).is_file() for lib in bpy.data.libraries)
    baseline = {o.name: [tuple(v.co) for v in o.data.vertices]
                for o in bpy.context.scene.objects if o.type == 'MESH'}
    for component in COMPONENTS:
        collection = bpy.data.collections['Scout_' + component]
        assert collection.library is not None, component
        assert Path(bpy.path.abspath(collection.library.filepath)).resolve() == asset / 'components' / component / 'source.blend'
        assert all(o.library == collection.library for o in collection.objects), component
    source = asset / 'components/hull/source.blend'
    bpy.ops.wm.open_mainfile(filepath=str(source))
    hull = bpy.context.scene.objects['Hull_Belly']
    assert hull.library is None and hull.data.library is None
    assert hull.parent.library is not None
    assert all(m.library is not None for m in hull.data.materials)
    hull.data.vertices[0].co.x += 0.125
    bpy.ops.wm.save_as_mainfile(filepath=str(source))
    bpy.ops.wm.open_mainfile(filepath=str(assembly))
    for name, vertices in baseline.items():
        current = [tuple(v.co) for v in bpy.context.scene.objects[name].data.vertices]
        if name == 'Hull_Belly':
            assert abs(current[0][0] - vertices[0][0] - 0.125) < 1e-6
            assert current[1:] == vertices[1:]
        else:
            assert current == vertices, name
    print(json.dumps({'components': len(COMPONENTS), 'editPropagation': 'passed'}))


if __name__ == '__main__':
    with tempfile.TemporaryDirectory(prefix='scout-modular-') as directory:
        asset = Path(directory) / 'salimon-scout'
        shutil.copytree(ASSET, asset)
        verify(asset)
