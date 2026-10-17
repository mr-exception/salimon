"""Blender 4.5.3 CPU reference authoring; never writes production sources.

Run from repo root: blender -b --python-exit-code 1 --python <this file>
 -- --output artifacts/option-a [--production]
"""
import argparse
import json
import math
from pathlib import Path
import sys

import bpy
from mathutils import Vector

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO / 'models/tools'))
from style_palette import apply


def setup():
    s = bpy.context.scene
    s.unit_settings.system = 'METRIC'
    s.unit_settings.scale_length = 1
    s.render.engine = 'CYCLES'
    s.cycles.device = 'CPU'
    s.cycles.samples = 8
    s.cycles.seed = 273
    s.render.threads_mode = 'FIXED'
    s.render.threads = 4
    s.render.resolution_x, s.render.resolution_y = 640, 480
    s.render.resolution_percentage = 100
    s.render.image_settings.file_format = 'PNG'
    s.view_settings.view_transform = 'Standard'
    s.view_settings.look = 'None'
    s.view_settings.exposure = 0
    s.world = bpy.data.worlds.new('ReferenceWorld')
    s.world.use_nodes = True
    s.world.node_tree.nodes['Background'].inputs['Color'].default_value = (.12,.12,.12,1)
    s.world.node_tree.nodes['Background'].inputs['Strength'].default_value = .5
    return s


def light(name, location, target, energy, size):
    data = bpy.data.lights.new(name, 'AREA')
    data.energy, data.shape, data.size = energy, 'DISK', size
    obj = bpy.data.objects.new(name, data)
    bpy.context.scene.collection.objects.link(obj)
    obj.location = location
    obj.rotation_euler = (Vector(target)-obj.location).to_track_quat('-Z','Y').to_euler()


def camera(location, target, scale=None):
    data = bpy.data.cameras.new('ReferenceCamera')
    obj = bpy.data.objects.new('ReferenceCamera',data)
    bpy.context.scene.collection.objects.link(obj)
    obj.location = location
    obj.rotation_euler = (Vector(target)-obj.location).to_track_quat('-Z','Y').to_euler()
    if scale:
        data.type, data.ortho_scale = 'ORTHO', scale
    else:
        data.lens = 19
    data.clip_start = .01
    bpy.context.scene.camera = obj


def costs():
    meshes = {o.data for o in bpy.context.scene.objects if o.type == 'MESH' and not o.hide_render}
    for mesh in meshes:
        mesh.calc_loop_triangles()
    return {'uniqueMeshTriangles':sum(len(m.loop_triangles) for m in meshes),
            'meshObjects':sum(o.type == 'MESH' and not o.hide_render for o in bpy.context.scene.objects),
            'materials':len({m for mesh in meshes for m in mesh.materials if m})}


def save_render(output, name):
    s = bpy.context.scene
    # Portable editable snapshot: localize only the disposable reference scene.
    bpy.ops.wm.save_as_mainfile(filepath=str(output / (name+'.blend')), relative_remap=True)
    s.render.filepath = str(output / (name+'.png'))
    bpy.ops.render.render(write_still=True)
    return {'name':name, **costs(), 'cameraSourceMeters':list(s.camera.location),
            'cameraRotationEuler':list(s.camera.rotation_euler)}


def ship(output, production):
    results = []
    for view, loc, target, scale in [
        ('exterior',(26,-30,21),(0,0,1),34),
        ('exterior-far',(26,-30,21),(0,0,1),68),
        ('interior',(2.76,0,1.8),(4.42,0,1.04),None),
        ('core',(-1,2.5,1.8),(-1,0,1.5),None)]:
        bpy.ops.wm.open_mainfile(filepath=str(REPO/'models/assets/ships/salimon-scout/assembly/salimon-scout.blend'))
        # Make the reference self-contained without modifying linked production libraries.
        bpy.ops.object.select_all(action='SELECT')
        bpy.ops.object.make_local(type='ALL')
        setup()
        for obj in bpy.context.scene.objects:
            if obj.type == 'EMPTY':
                obj.hide_render = True
        light('Key',(10,-15,23),(0,0,0),4500,14)
        light('Fill',(-10,12,10),(0,0,0),2400,12)
        light('Cabin',(0,0,2.8),(3,0,0),55,2)
        camera(loc,target,scale)
        if production:
            results.append(save_render(output,'production-'+view))
        else:
            results.append(save_render(output,'current-'+view))
            apply(bpy.data.materials)
            results.append(save_render(output,'proposed-'+view))
    return results


def material(name,color):
    m=bpy.data.materials.new(name)
    m.diffuse_color=(*color,1)
    m.use_nodes=True
    m.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=(*color,1)
    m.node_tree.nodes['Principled BSDF'].inputs['Roughness'].default_value=.85
    return m


def resources(output):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    setup()
    # Deterministic visual sample only, not a second terrain generation policy.
    points=[(x,y,.32*math.sin(x*.8)*math.cos(y*.6)) for y in range(-4,5) for x in range(-4,5)]
    faces=[]
    for y in range(8):
        for x in range(8):
            a=y*9+x
            faces.extend([(a,a+1,a+10),(a,a+10,a+9)])
    mesh=bpy.data.meshes.new('FacetedTerrain')
    mesh.from_pydata(points,[],faces)
    patch=bpy.data.objects.new('TerrainReference',mesh)
    bpy.context.scene.collection.objects.link(patch)
    mesh.materials.append(material('TerrainSlate',(.24,.30,.37)))
    # Original parametric proposal, no third-party geometry or gameplay meaning.
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=1,radius=1,location=(0,0,1))
    deposit=bpy.context.object
    deposit.name='DepositReference'
    deposit.scale=(1.4,1,1.2)
    deposit.data.materials.append(material('DepositIron',(.42,.25,.14)))
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=2,radius=.32,location=(1.8,-.5,.5))
    bpy.context.object.name='SphereFragmentReference'
    bpy.context.object.data.materials.append(material('FragmentIron',(.42,.25,.14)))
    light('Key',(3,-4,8),(0,0,0),650,5)
    camera((8,-10,7),(0,0,.3),10)
    results=[save_render(output,'terrain-resources')]
    bpy.context.scene.camera.data.ortho_scale=5
    results.append(save_render(output,'resource-closeup'))
    return results


def tool(output):
    bpy.ops.wm.open_mainfile(filepath=str(REPO/'models/assets/items/mining-tool/source.blend'))
    setup()
    light('Key',(1,-2,3),(0,0,0),150,2)
    camera((1,-1,.7),(0,0,0),1.3)
    results=[save_render(output,'tool-near')]
    bpy.context.scene.camera.data.ortho_scale=2.6
    results.append(save_render(output,'tool-far'))
    return results


def main():
    p=argparse.ArgumentParser()
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--production',action='store_true')
    args=p.parse_args(sys.argv[sys.argv.index('--')+1:])
    if bpy.app.version[:3] != (4,5,3):
        raise ValueError('reference source pinned to Blender 4.5.3')
    output=args.output.resolve()
    if not output.is_relative_to(REPO/'artifacts'):
        raise ValueError('reference output must be under artifacts/')
    output.mkdir(parents=True,exist_ok=True)
    results=ship(output,args.production)
    if not args.production:
        results+=resources(output)+tool(output)
    (output/'reference-costs.json').write_text(json.dumps({'blender':bpy.app.version_string,
        'seed':273,'samples':8,'renderer':'Cycles CPU; not native shader evidence',
        'views':results},indent=2)+'\n')


if __name__=='__main__':
    main()
