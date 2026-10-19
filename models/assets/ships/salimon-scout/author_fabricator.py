"""Reviewed parametric wall module; run in Blender 4.5.3, then scout export.py.

Edits only cabin-interior/source.blend. Runtime meters are converted once to
Blender axes; no shared library is made local and no assembly link is replaced.
"""
import math
from pathlib import Path
import bpy

ASSET = Path(__file__).resolve().parent
SOURCE = ASSET / 'components/cabin-interior/source.blend'

def point(p):
    return (p[0], -p[2], p[1])

class Mesh:
    def __init__(self):
        self.vertices, self.faces = [], []
    def prism(self, outline, front, back):
        base = len(self.vertices)
        self.vertices += [point((x,y,z)) for z in (front,back) for x,y in outline]
        n=len(outline)
        self.faces += [tuple(base+i for i in reversed(range(n))),tuple(base+n+i for i in range(n))]
        self.faces += [(base+i,base+(i+1)%n,base+n+(i+1)%n,base+n+i) for i in range(n)]
    def box(self, center, size):
        x,y,z=center; a,b,c=(v/2 for v in size)
        self.prism([(x-a,y-b),(x+a,y-b),(x+a,y+b),(x-a,y+b)],z-c,z+c)
    def ring(self,x,y,inner,outer,front,back):
        # Eight separated radial panels form one open throat; no overlaid caps.
        base=len(self.vertices)
        for z,r in [(front,outer),(front,inner),(back,outer),(back,inner)]:
            self.vertices += [point((x+r*math.cos(i*math.tau/8),y+r*math.sin(i*math.tau/8),z)) for i in range(8)]
        for i in range(8):
            j=(i+1)%8
            self.faces += [tuple(base+k for k in f) for f in [(i,j,8+j,8+i),
                (i,16+i,16+j,j),(8+j,24+j,24+i,8+i)]]
    def publish(self,name,material,collection):
        mesh=bpy.data.meshes.new(name);mesh.from_pydata(self.vertices,[],self.faces);mesh.update()
        # Every face remains flat, with a deterministic UV layer for the legacy shader contract.
        uv=mesh.uv_layers.new(name='UVMap')
        for polygon in mesh.polygons:
            for loop in polygon.loop_indices:
                vertex=mesh.vertices[mesh.loops[loop].vertex_index].co
                uv.data[loop].uv=(vertex.x,vertex.z)
        obj=bpy.data.objects.new(name,mesh);collection.objects.link(obj)
        obj.parent=bpy.data.objects['Interior'];mesh.materials.append(bpy.data.materials[material])
        return obj

def author():
    bpy.ops.wm.open_mainfile(filepath=str(SOURCE))
    for obj in list(bpy.data.objects):
        if obj.library is None and ('Fabricator' in obj.name):bpy.data.objects.remove(obj,do_unlink=True)
    shared=str(ASSET/'assembly/shared.blend')
    for name in ['Petrol Teal','Cyan Display']:
        if name not in bpy.data.materials:
            with bpy.data.libraries.load(shared,link=True,relative=True) as (src,dst):dst.materials=[name]
    collection=bpy.data.collections['Scout_cabin-interior']
    body=Mesh();body.prism([(-4.25,.5),(-2.05,.5),(-1.95,.6),(-1.95,2.25),(-2.1,2.4),(-4.2,2.4),(-4.35,2.25),(-4.35,.6)],3.76,4.40)
    body.publish('Fabricator_Chassis','Hull Graphite',collection)
    panels=Mesh();panels.box((-3.15,2.25,3.735),(1.85,.16,.025))
    panels.publish('Fabricator_Panels','Petrol Teal',collection)
    frame=Mesh();frame.ring(-3.0,1.65,.265,.34,3.55,3.70)
    frame.box((-3.8,2.0,3.73),(.88,.62,.04))
    frame.publish('Fabricator_Frame','Copper Accent',collection)
    throat=Mesh();throat.vertices=[point(p) for p in [(-3.265,1.915,3.73),(-3.265,1.385,3.73),(-2.735,1.385,3.73),(-2.735,1.915,3.73)]];throat.faces=[(0,2,1),(0,3,2)];throat.publish('Fabricator_Input','Interior Dark',collection)
    monitor=Mesh()
    # Two triangles, top-left UV origin. The machine screen artwork is owned by #343.
    monitor.vertices=[point(p) for p in [(-4.175,2.245,3.70),(-4.175,1.755,3.70),(-3.425,1.755,3.70),(-3.425,2.245,3.70)]]
    monitor.faces=[(0,2,1),(0,3,2)]
    obj=monitor.publish('Fabricator_Monitor','Cyan Display',collection)
    uv=obj.data.uv_layers.active
    for poly in obj.data.polygons:
        for loop in poly.loop_indices:
            uv.data[loop].uv=[(0,1),(0,0),(1,0),(1,1)][obj.data.loops[loop].vertex_index]
    obj['salimon']={'displayRole':'fabricator','uvOrigin':'top-left','purpose':'fabricator-monitor'}
    tray=Mesh();tray.box((-2.55,.94,3.47),(1.0,.1,.64))
    tray.box((-3.08,1.035,3.47),(.05,.16,.64));tray.box((-2.02,1.035,3.47),(.05,.16,.64))
    tray.box((-2.55,1.035,3.12),(1.0,.16,.04));tray.publish('Fabricator_Tray','Interior Dark',collection)
    product=Mesh();product.box((-2.55,1.066,3.4),(.4,.15,.35))
    product.publish('Fabricator_Output','Hull Ceramic',collection)['salimon']={'purpose':'fabricator-output','visibility':'tray-occupied'}
    def empty(name,pos,parent,extras):
        o=bpy.data.objects.new(name,None);collection.objects.link(o);o.parent=bpy.data.objects[parent];o.location=point(pos);o['salimon']=extras
    empty('COLLIDER_Fabricator',[-3.15,1.45,3.75],'Collision_Proxies',{'collisionShape':'box','sizeMeters':[2.4,1.9,1.3],'purpose':'fabricator-wall-module'})
    for role,pos in [('Input',[-3,1.65,3.55]),('Monitor',[-3.8,2.0,3.69]),('Tray',[-2.55,.991,3.40])]:
        empty('MARKER_Fabricator'+role,pos,'Interaction_Markers',{'purpose':'fabricator-'+role.lower(),'facingDirection':[0,0,-1]})
    bpy.ops.wm.save_as_mainfile(filepath=str(SOURCE))

if __name__=='__main__':author()
