"""Deliberate #333 source repair; not an export hook. Blender 4.5.3 LTS."""
from pathlib import Path
import bpy
ASSET=Path(__file__).resolve().parent
FLOOR=.24731182795698925
TOP=2.516129032

def repair():
    # The inset tray retains its exact support rectangle on its outward face.
    # Taper hidden side/end caps instead of leaving them coplanar with the leaf.
    path=ASSET/'components/exit-door/source.blend'
    bpy.ops.wm.open_mainfile(filepath=str(path))
    tip=bpy.data.objects['Exit_Ramp_Tip']
    for v in tip.data.vertices:
        world=tip.matrix_world@v.co
        if world.x < -7.78:
            world.y=world.y-(.01 if world.y>0 else -.01)
            world.z=world.z+(.01 if world.z<(FLOOR+TOP)/2 else -.01)
            v.co=tip.matrix_world.inverted()@world
    # Header has a deliberate 10 mm clearance above the moving leaf's end cap.
    frame=bpy.data.objects['Door_Frame']
    for v in frame.data.vertices:
        world=frame.matrix_world@v.co
        if abs(world.z-TOP)<1e-5:
            world.z+=.01;v.co=frame.matrix_world.inverted()@world
    # Remove only the tray support-face strip concealed by the leaf even at
    # full extension. Both same/different-material depth competition disappears;
    # leaf owns this overlap and the tray still owns every exposed support point.
    import sys
    sys.path.insert(0,str(ASSET.parents[3]/'models/tools'))
    from surface_partition import split
    from mathutils import Vector
    cutoff=TOP-float(bpy.data.objects['Exit_Door']['salimon']['extensionTravelMeters'])+.0001
    old=tip.data;verts=[];faces=[];uvs=[];slots=[]
    for poly in old.polygons:
        points=[tip.matrix_world@old.vertices[i].co for i in poly.vertices]
        tex=[tuple(old.uv_layers.active.data[i].uv) for i in poly.loop_indices]
        payload=[((p.y,p.z),uv,tuple(p)) for p,uv in zip(points,tex)]
        if all(abs(p.x+7.76)<1e-5 for p in points):
            payload,_=split(payload,(-2,cutoff),(2,cutoff))
        if len(payload)<3:continue
        for i in range(1,len(payload)-1):
            row=[payload[0],payload[i],payload[i+1]];start=len(verts)
            verts.extend(tuple(tip.matrix_world.inverted()@Vector(v[2])) for v in row)
            faces.append((start,start+1,start+2));uvs.extend(v[1] for v in row);slots.append(poly.material_index)
    mesh=bpy.data.meshes.new('Exit_Ramp_Tip / single support owner');mesh.from_pydata(verts,[],faces)
    for material in old.materials:mesh.materials.append(material)
    uv=mesh.uv_layers.new(name='UVMap')
    for loop,coord in zip(uv.data,uvs):loop.uv=coord
    for poly,slot in zip(mesh.polygons,slots):poly.material_index=slot
    tip.data=mesh
    tip.data.update();frame.data.update()
    bpy.context.preferences.filepaths.save_version=0
    bpy.ops.wm.save_as_mainfile(filepath=str(path))
    # Clear the pressure leaf's lateral/end caps from the stationary aperture.
    path=ASSET/'components/hull/source.blend'
    bpy.ops.wm.open_mainfile(filepath=str(path))
    cap=bpy.data.objects['Hull_Aft_Cap']
    for v in cap.data.vertices:
        world=cap.matrix_world@v.co
        if abs(abs(world.y)-1.4)<1e-5:world.y+=.01 if world.y>0 else -.01
        if abs(world.z-TOP)<1e-5:world.z+=.01
        if abs(world.z-FLOOR)<1e-5:world.z-=.01
        v.co=cap.matrix_world.inverted()@world
    cap.data.update();bpy.context.preferences.filepaths.save_version=0
    bpy.ops.wm.save_as_mainfile(filepath=str(path))
if __name__=='__main__':repair()
