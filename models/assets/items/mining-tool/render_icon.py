"""Render the derived toolbar icon from the loaded, saved mining-tool source."""
from pathlib import Path
import bpy
from mathutils import Vector

HERE = Path(__file__).resolve().parent
# Derived toolbar artwork: transparent orthographic three-quarter view of source.
# Authoring helpers are deliberately outside the asset hierarchy and not saved.
scene = bpy.context.scene
scene.render.engine = 'CYCLES'
scene.cycles.samples = 32
scene.render.resolution_x = scene.render.resolution_y = 128
scene.render.resolution_percentage = 100
scene.render.film_transparent = True
scene.view_settings.view_transform = 'Standard'
scene.world.color = (.18, .18, .18)
center = Vector((.090, 0, .018))
bpy.ops.object.camera_add(location=center + Vector((.13, -.80, .27)))
camera = bpy.context.object
camera.rotation_euler = (center - camera.location).to_track_quat('-Z', 'Y').to_euler()
camera.data.type = 'ORTHO'
camera.data.ortho_scale = .47
scene.camera = camera
for location, energy, size in [((.10, -.50, .65), 8, .7), ((.15, .40, .30), 5, .5)]:
    bpy.ops.object.light_add(type='AREA', location=location)
    lamp = bpy.context.object
    lamp.data.energy, lamp.data.shape, lamp.data.size = energy, 'DISK', size
    lamp.rotation_euler = (center - lamp.location).to_track_quat('-Z', 'Y').to_euler()
scene.render.image_settings.file_format = 'PNG'
scene.render.image_settings.color_mode = 'RGBA'
scene.render.filepath = str(HERE.parents[3] / 'client/assets/items/mining-tool/icon.png')
bpy.ops.render.render(write_still=True)
