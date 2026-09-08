"""#540 base habitation: preserve fixture IDs, replace only the bunk panels.

Run with Blender 4.5.3. Source/runtime axes use the existing category adapter.
The 0.85 m berth folds around its wall hinge; the generated #538 collision
reserve already contains its full motion. No shell, anchors or library links move.
"""
from pathlib import Path
import sys
import bpy
sys.path.insert(0,str(Path(__file__).resolve().parent))
from author_fabricator import Mesh
from habitation_geometry import definition
ASSET=Path(__file__).resolve().parent

def author():
    if bpy.app.version[:3] != (4,5,3):
        raise ValueError('Use the supported Blender 4.5.3 authoring toolchain')
    source=ASSET/'components/cabin-interior/source.blend'
    bpy.ops.wm.open_mainfile(filepath=str(source))
    collection=bpy.data.collections['Scout_cabin-interior']
    d=definition(); x,y,z=d['bunkHingeMeters']
    center_y=y+d['frameSizeMeters'][1]/2
    for name,material,center,size in [
        ('Starter_BunkFrame','Hull Graphite',[x,center_y,z],d['frameSizeMeters']),
        ('Starter_BunkCushion','Terracotta Upholstery',[x,center_y,z+d['cushionOffsetMeters']],d['cushionSizeMeters']),
    ]:
        old=bpy.data.objects[name]
        if old.library is not None: raise ValueError('Expected local cabin-owned fixture')
        mesh=Mesh();mesh.box(center,size)
        replacement=mesh.publish('HabitationTemporary',material,collection)
        old.data=replacement.data
        bpy.data.objects.remove(replacement,do_unlink=True)
        if 'salimon' in old: del old['salimon']
    bpy.ops.wm.save_as_mainfile(filepath=str(source))
if __name__=='__main__':author()
