"""Isolated authored Core previews; Blender CPU rendering, not native game evidence.

blender -b --python reports/issue-294/render_preview.py -- before
"""
from pathlib import Path
import sys
import bpy
from mathutils import Vector

REPO = Path(__file__).resolve().parents[2]
label = sys.argv[sys.argv.index('--') + 1]
dark = label.endswith('dark')
bpy.ops.wm.open_mainfile(filepath=str(REPO / 'models/assets/ships/salimon-scout/components/energy-core/source.blend'))
scene = bpy.context.scene
# Keep only Core-local meshes in the preview; omit the long floor cable.
for obj in scene.objects:
    obj.hide_render = obj.type != 'MESH' or obj.name == 'Core_Power_Conduits'
scene.render.engine = 'CYCLES'
scene.cycles.device = 'CPU'
scene.cycles.samples = 24
scene.render.resolution_x = 1000
scene.render.resolution_y = 800
scene.render.resolution_percentage = 100
scene.view_settings.view_transform = 'Standard'
scene.view_settings.look = 'None'
scene.world = bpy.data.worlds.new('Preview world')
scene.world.color = (.01, .01, .01) if dark else (.07, .07, .07)

def source(p):
    return Vector((p[0], -p[2], p[1]))

def aim(obj, target):
    obj.rotation_euler = (source(target) - obj.location).to_track_quat('-Z', 'Y').to_euler()

bpy.ops.object.camera_add(location=source((1.3, 2.6, -4.6)))
scene.camera = bpy.context.object
scene.camera.data.type = 'ORTHO'
scene.camera.data.ortho_scale = 2.9
aim(scene.camera, (-1, 1.15, 0))
for pos, watts, size in [((1, 5, -4), 700, 4), ((-4, 3, -1), 400, 3)]:
    bpy.ops.object.light_add(type='AREA', location=source(pos))
    lamp = bpy.context.object
    lamp.data.energy = watts * (.12 if dark else 1)
    lamp.data.shape = 'DISK'
    lamp.data.size = size
    aim(lamp, (-1, 1.2, 0))
scene.render.filepath = str(Path(__file__).parent / f'{label}.png')
bpy.ops.render.render(write_still=True)
