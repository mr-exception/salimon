"""Reviewed #575 reusable opaque library templates, Blender 4.5.3 only.

Nodes are separate templates at the same origin, not a combined scene. Renderer
selects a named template; authority supplies physical identity/pose/condition.
"""
from pathlib import Path
import sys
import bpy
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'models/assets/ships/salimon-scout'))
from author_fabricator import Mesh

def author():
    if bpy.app.version[:3]!=(4,5,3):raise ValueError('Use Blender 4.5.3')
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.scene.unit_settings.system='METRIC';bpy.context.scene.unit_settings.scale_length=1
    root=bpy.data.objects.new('ASSET_scout-interior-kit',None);bpy.context.collection.objects.link(root)
    root['salimon']={'assetId':'fixture.scout-interior-kit','layoutVersion':1}
    group=bpy.data.objects.new('Visual',None);bpy.context.collection.objects.link(group);group.parent=root
    materials=['Hull Graphite','Interior Light','Terracotta Upholstery']
    with bpy.data.libraries.load(str(ROOT/'models/assets/ships/salimon-scout/assembly/shared.blend'),link=True,relative=True) as (_,dst):dst.materials=materials
    for name,material,boxes in [
        ('Shelf','Hull Graphite',[([0,0,0],[1.0,.06,.35])]),
        ('Seat','Terracotta Upholstery',[([0,.05,0],[.5,.10,.5]),([0,.43,-.28],[.5,.64,.06])]),
        ('Panel','Interior Light',[([0,0,0],[.8,.8,.06])]),
        ('PanelDamaged','Interior Light',[([0,0,0],[.8,.8,.06])]),
    ]:
        m=Mesh()
        for center,size in boxes:m.box(center,size)
        if name=='PanelDamaged':
            m.vertices=[(x,y,z+(.12 if x>0 and z>0 else 0)) for x,y,z in m.vertices]
        mesh=bpy.data.meshes.new(name);mesh.from_pydata(m.vertices,[],m.faces);mesh.update()
        uv=mesh.uv_layers.new(name='UVMap')
        for poly in mesh.polygons:
            for i in poly.loop_indices:
                v=mesh.vertices[mesh.loops[i].vertex_index].co;uv.data[i].uv=(v.x,v.z)
        mesh.materials.append(bpy.data.materials[material]);obj=bpy.data.objects.new(name,mesh)
        bpy.context.collection.objects.link(obj);obj.parent=group
    bpy.ops.wm.save_as_mainfile(filepath=str(Path(__file__).with_name('source.blend')))
if __name__=='__main__':author()
