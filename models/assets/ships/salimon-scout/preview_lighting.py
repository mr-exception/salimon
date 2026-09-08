"""Read-only mood references; same metre cameras as native scout_reference.

Blender --background --python-exit-code 1 --python this_file -- OUTPUT
Cycles is an offline reference, never evidence of native shader fidelity.
"""
from pathlib import Path
import json
import sys

import bpy
from mathutils import Vector

ASSET = Path(__file__).resolve().parent
REPO = ASSET.parents[3]


def point(value):
    # Same inverse glTF axis mapping as preview_fabricator.py.
    return Vector((value[0], -value[2], value[1]))


def render(output):
    output = Path(output).resolve()
    if output == ASSET or ASSET in output.parents:
        raise ValueError('preview output must be outside authored asset sources')
    output.mkdir(parents=True, exist_ok=True)
    cameras = json.loads((REPO / 'scenarios/visual/scout-cameras.json').read_text())
    for powered in (False, True):
        bpy.ops.wm.open_mainfile(filepath=str(ASSET / 'assembly/salimon-scout.blend'))
        scene = bpy.context.scene
        for obj in scene.objects:
            if obj.type == 'MESH':
                obj.hide_render = obj.name.startswith('COLLIDER_')
        # This file never saves a .blend or exports geometry.
        for mat in bpy.data.materials:
            if mat.use_nodes:
                for node in mat.node_tree.nodes:
                    if node.type == 'BSDF_PRINCIPLED' and not powered:
                        node.inputs['Emission Strength'].default_value = 0.0
        scene.render.engine = 'CYCLES'
        scene.cycles.device = 'CPU'
        scene.cycles.samples = 16
        scene.render.resolution_x = 1280
        scene.render.resolution_y = 800
        scene.render.resolution_percentage = 100
        scene.render.image_settings.file_format = 'PNG'
        scene.world = bpy.data.worlds.new('Scout reference cool spill')
        scene.world.use_nodes = True
        scene.world.node_tree.nodes['Background'].inputs['Color'].default_value = (.10, .14, .20, 1)
        scene.world.node_tree.nodes['Background'].inputs['Strength'].default_value = .35
        scene.view_settings.view_transform = 'Standard'
        scene.view_settings.look = 'None'
        if powered:
            for x in (4.0, -3.0):
                bpy.ops.object.light_add(type='AREA', location=point((x, 2.6, 0)))
                lamp = bpy.context.object
                lamp.data.energy = 160
                lamp.data.color = (1.0, .73, .47)
                lamp.data.size = 4.0
                lamp.rotation_euler = (point((x, .25, 0)) - lamp.location).to_track_quat('-Z', 'Y').to_euler()
        bpy.ops.object.camera_add()
        scene.camera = camera = bpy.context.object
        camera.data.type = 'PERSP'
        camera.data.sensor_fit = 'VERTICAL'
        camera.data.angle_y = __import__('math').radians(70)
        camera.data.clip_start = .05
        for view in cameras:
            camera.location = point(view['eye'])
            camera.rotation_euler = (point(view['target']) - camera.location).to_track_quat('-Z', 'Y').to_euler()
            scene.render.filepath = str(output / (view['name'] + ('-powered.png' if powered else '-blackout.png')))
            bpy.ops.render.render(write_still=True)


if __name__ == '__main__':
    args = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
    if len(args) != 1:
        raise SystemExit('one output directory required')
    render(args[0])
