"""Read-only CPU seam views. Run Blender --background --python this_file -- topic output."""
from pathlib import Path
import math
import sys
import bpy
from mathutils import Vector, Matrix
ASSET=Path(__file__).resolve().parent

def render(topic,output):
    bpy.ops.wm.open_mainfile(filepath=str(ASSET/'assembly/salimon-scout.blend'))
    scene=bpy.context.scene
    for o in scene.objects:
        if o.type=='MESH':o.hide_render=o.name.startswith('COLLIDER_')
    scene.render.engine='CYCLES';scene.cycles.device='CPU';scene.cycles.samples=12
    scene.render.threads_mode='FIXED';scene.render.threads=4
    scene.render.resolution_x=800;scene.render.resolution_y=600;scene.render.resolution_percentage=100
    scene.render.image_settings.file_format='JPEG';scene.render.image_settings.quality=88
    scene.world=bpy.data.worlds.new('Seam preview');scene.world.color=(.16,.16,.16)
    scene.view_settings.view_transform='Standard';scene.view_settings.look='None'
    for p,power,size in [((-4,0,2.2),600,4),((-10,-4,5),1200,5),((5,0,2.5),800,4),((11,4,6),1600,6)]:
        bpy.ops.object.light_add(type='AREA',location=p);o=bpy.context.object;o.data.energy=power;o.data.size=size
        o.rotation_euler=(Vector((-2,0,1))-o.location).to_track_quat('-Z','Y').to_euler()
    bpy.ops.object.camera_add();camera=bpy.context.object;camera.data.lens=30;scene.camera=camera
    door=bpy.data.objects['Exit_Door'];tip=bpy.data.objects['Exit_Ramp_Tip'];original={o.name:o.matrix_world.copy()for o in [door,tip]}
    ramp=door['salimon'];pivot=Vector((-7.68,0,float(ramp['pivot'][1])))
    output=Path(output);output.mkdir(parents=True,exist_ok=True)
    if topic=='door':
        views=[('inside-closed',(-4.8,0,1.65),(-7.8,0,1.35),0),('inside-oblique',(-5.2,-1.5,1.65),(-7.8,0,1.35),0),
          ('opening',(-5.2,-1.5,1.65),(-8.2,0,.8),.35),('extending',(-5.2,-1.5,1.65),(-8.5,0,.3),.75),
          ('deployed',(-5.2,-1.5,1.65),(-9,0,0),1),('outside-closed',(-11,-3,2),(-7.8,0,1.3),0)]
    else:
        views=[]
        for side in [-1,1]:
            views += [(f'seated-{side}',(6.1,0,1.65),(5.12,side*4.8,1.65),0),
              (f'standing-{side}',(4.3,0,1.7),(5.12,side*4.8,1.65),0),
              (f'exterior-{side}',(10,side*9,4),(5.1,side*4.7,1.6),0)]
    for name,eye,target,f in views:
        h=min(f/.6,1);h=h*h*(3-2*h);e=max(0,min(1,(f-.6)/.4));e=e*e*(3-2*e)
        rotation=Matrix.Translation(pivot)@Matrix.Rotation(-float(ramp['openAngleRadians'])*h,4,'Y')@Matrix.Translation(-pivot)
        door.matrix_world=rotation@original[door.name]
        tip.matrix_world=rotation@Matrix.Translation((float(ramp['extensionFaceOffsetMeters'])*e,0,float(ramp['extensionTravelMeters'])*e))@original[tip.name]
        camera.location=eye;camera.rotation_euler=(Vector(target)-camera.location).to_track_quat('-Z','Y').to_euler()
        scene.render.filepath=str(output/(name+'.jpg'));bpy.ops.render.render(write_still=True)
if __name__=='__main__':
    args=sys.argv[sys.argv.index('--')+1:];render(*args)
