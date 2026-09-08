"""Editable #577 cabin mechanisms; exact named templates, Blender 4.5.3.

Separate templates share the same origin for extraction, never a composed scene.
All boxes use runtime metre dimensions, converted once by the existing Mesh owner.
"""
from pathlib import Path
import sys
import bpy
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'models/assets/ships/salimon-scout'))
from author_fabricator import Mesh, point
PARTS=[
    ('Table','Interior Light',[([0,0,.40],[1.0,.06,.80])]),
    ('ScreenPanel','Terracotta Upholstery',[([0,0,0],[.70,1.45,.035])]),
    ('Shutter','Hull Graphite',[([0,0,0],[2.3,.29,.04])]),
    ('ShutterHandle','Hull Graphite',[([0,0,0],[.16,.04,.06])]),
    ('LockerShell','Interior Light',[
        ([0,0,-.23],[.7,1.3,.04]),
        ([-.33,0,0],[.04,1.3,.42]),([.33,0,0],[.04,1.3,.42]),
        ([0,-.63,0],[.62,.04,.42]),([0,.63,0],[.62,.04,.42])]),
    ('LockerDoor','Hull Graphite',[([.35,0,0],[.7,1.3,.035])]),
    ('SeatBase','Hull Graphite',[([x,.20,z],[.045,.40,.045]) for x in [-.20,.20] for z in [-.20,.20]]+[([0,.40,.30],[.08,.80,.08])]),
    ('SeatPad','Terracotta Upholstery',[([0,-.05,-.25],[.5,.10,.5])]),
    ('SeatBack','Terracotta Upholstery',[([0,.43,0],[.5,.64,.06])]),
    ('Harness','Terracotta Upholstery',[
        ([-.28,.14,0],[.035,.28,.05]),([.28,.14,0],[.035,.28,.05]),
        ([0,.2975,0],[.595,.035,.05])]),
]
def author():
    if bpy.app.version[:3]!=(4,5,3):raise ValueError('Use Blender 4.5.3')
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.scene.unit_settings.system='METRIC';bpy.context.scene.unit_settings.scale_length=1
    root=bpy.data.objects.new('ASSET_scout-habitation-kit',None);bpy.context.collection.objects.link(root)
    root['salimon']={'assetId':'fixture.scout-habitation-kit','layoutVersion':1}
    group=bpy.data.objects.new('Visual',None);bpy.context.collection.objects.link(group);group.parent=root
    materials=['Hull Graphite','Interior Light','Terracotta Upholstery']
    with bpy.data.libraries.load(str(ROOT/'models/assets/ships/salimon-scout/assembly/shared.blend'),link=True,relative=True) as (_,dst):dst.materials=materials
    for name,material,boxes in PARTS:
        m=Mesh()
        if name=='LockerShell':
            m.vertices=[point((x,y,z)) for xlim,ylim,z in [(.35,.65,-.25),(.35,.65,.23),(.31,.61,-.21),(.31,.61,.23)] for x,y in [(-xlim,-ylim),(xlim,-ylim),(xlim,ylim),(-xlim,ylim)]]
            m.faces=[(3,2,1,0),(8,9,10,11)]
            for i in range(4):
                j=(i+1)%4
                m.faces.extend([(i,j,4+j,4+i),(8+i,12+i,12+j,8+j),(4+i,4+j,12+j,12+i)])
        elif name=='Harness':
            m.prism([(-.2975,0),(-.2625,0),(-.2625,.28),(.2625,.28),(.2625,0),(.2975,0),(.2975,.315),(-.2975,.315)],-.025,.025)
        else:
            for center,size in boxes:m.box(center,size)
        mesh=bpy.data.meshes.new(name);mesh.from_pydata(m.vertices,[],m.faces);mesh.update()
        uv=mesh.uv_layers.new(name='UVMap')
        for poly in mesh.polygons:
            for i in poly.loop_indices:
                v=mesh.vertices[mesh.loops[i].vertex_index].co;uv.data[i].uv=(v.x,v.z)
        mesh.materials.append(bpy.data.materials[material]);obj=bpy.data.objects.new(name,mesh)
        bpy.context.collection.objects.link(obj);obj.parent=group
    bpy.context.preferences.filepaths.save_version=0
    bpy.ops.wm.save_as_mainfile(filepath=str(Path(__file__).with_name('source.blend')))
if __name__=='__main__':author()
