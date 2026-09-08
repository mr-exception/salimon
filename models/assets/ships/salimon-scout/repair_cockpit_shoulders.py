"""Deliberate #334 hull-source migration; preserve canopy and collision interfaces."""
from pathlib import Path
import bpy
import bmesh
from mathutils import Vector
ASSET=Path(__file__).resolve().parent
LIMIT=4.98  # inside the 4.94..5.06 m rear canopy frame, before glazing starts at 5

def trim(obj):
    old=obj.data;vertices=[];faces=[];slots=[];coords=[];seen=set()
    for poly in old.polygons:
        points=[obj.matrix_world@old.vertices[i].co for i in poly.vertices]
        for p in points:p.x=min(p.x,LIMIT)
        normal=Vector((0,0,0))
        for i in range(1,len(points)-1):normal+=(points[i]-points[0]).cross(points[i+1]-points[0])
        if normal.length<1e-8:continue
        key=tuple(sorted(tuple(round(c,6)for c in p)for p in points))
        if key in seen:continue
        seen.add(key);start=len(vertices)
        vertices.extend(tuple(obj.matrix_world.inverted()@p)for p in points)
        faces.append(tuple(range(start,start+len(points))));slots.append(poly.material_index)
        coords.extend(tuple(old.uv_layers.active.data[i].uv)for i in poly.loop_indices)
    mesh=bpy.data.meshes.new(obj.name+' / canopy termination');mesh.from_pydata(vertices,[],faces)
    for m in old.materials:mesh.materials.append(m)
    uv=mesh.uv_layers.new(name='UVMap')
    for v,t in zip(uv.data,coords):v.uv=t
    for p,m in zip(mesh.polygons,slots):p.material_index=m
    bm=bmesh.new();bm.from_mesh(mesh)
    bmesh.ops.remove_doubles(bm,verts=list(bm.verts),dist=1e-6)
    bmesh.ops.dissolve_degenerate(bm,edges=list(bm.edges),dist=1e-7)
    bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bm.to_mesh(mesh);bm.free();obj.data=mesh;mesh.update()

def repair():
    path=ASSET/'components/hull/source.blend';bpy.ops.wm.open_mainfile(filepath=str(path))
    for name in ['Hull_Port_Side','Hull_Starboard_Side','Hull_Roof']:trim(bpy.data.objects[name])
    bpy.context.preferences.filepaths.save_version=0;bpy.ops.wm.save_as_mainfile(filepath=str(path))
if __name__=='__main__':repair()
