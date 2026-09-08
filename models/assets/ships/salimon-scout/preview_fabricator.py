"""Read-only neutral CPU previews; no changes to saved Blender sources."""
from pathlib import Path
import sys
import bpy
from mathutils import Vector

ASSET=Path(__file__).resolve().parent
REPO=ASSET.parents[3]

def point(p):return Vector((p[0],-p[2],p[1]))

def render(output):
    bpy.ops.wm.open_mainfile(filepath=str(ASSET/'assembly/salimon-scout.blend'))
    scene=bpy.context.scene
    for obj in scene.objects:
        if obj.type=='MESH':
            obj.hide_render=not (obj.name.startswith(('Fabricator_','Core_','Energy_Core','Deck_'))
                or obj.name in ['Wall_Port_Inner','Window_Port_Sill','Cabin_Woven_Runners'])
    scene.render.image_settings.file_format='JPEG';scene.render.image_settings.quality=90
    scene.render.engine='CYCLES';scene.cycles.device='CPU';scene.cycles.samples=32
    scene.render.resolution_x=1000;scene.render.resolution_y=750;scene.render.resolution_percentage=100
    scene.world=bpy.data.worlds.new('Fabricator preview world');scene.world.color=(.18,.18,.18)
    scene.view_settings.view_transform='Standard';scene.view_settings.look='None';scene.view_settings.exposure=0
    target=point((-3.1,1.4,3.75))
    for pos,power,size in [((-4,5,0),500,5),((0,3,1),250,4)]:
        bpy.ops.object.light_add(type='AREA',location=point(pos));light=bpy.context.object
        light.data.energy=power;light.data.shape='DISK';light.data.size=size
        light.rotation_euler=(target-light.location).to_track_quat('-Z','Y').to_euler()
    bpy.ops.object.camera_add();camera=bpy.context.object;scene.camera=camera;camera.data.type='ORTHO'
    output=Path(output);output.mkdir(parents=True,exist_ok=True)
    for name,pos,aim,scale in [('front',(-3.15,1.5,0),(-3.15,1.45,3.75),3.2),
        ('angle',(-5.6,2.6,.7),(-3.15,1.4,3.75),3.7),
        ('layout',(-6.0,4.0,-1.0),(-2.2,1.4,1.7),8.0)]:
        camera.location=point(pos);camera.rotation_euler=(point(aim)-camera.location).to_track_quat('-Z','Y').to_euler()
        camera.data.ortho_scale=scale;scene.render.filepath=str(output/(name+'.jpg'))
        bpy.ops.render.render(write_still=True)

if __name__=='__main__':
    args=sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else []
    render(args[0] if args else REPO/'artifacts/issue-342/previews')
