"""Deterministic low-poly silicate slab recipe for Blender 4.5 LTS."""
from pathlib import Path
import bpy

RUNTIME_VERTICES = [
    (-.43,-.22,-.32),(.38,-.20,-.28),(.44,-.18,.24),(-.34,-.21,.36),
    (-.28,.18,-.30),(.24,.26,-.22),(.30,.21,.26),(-.22,.16,.32),(0,.36,-.05),
]
FACES = [
    (0,1,5),(0,5,4),(1,2,6),(1,6,5),(2,3,7),(2,7,6),(3,0,4),(3,4,7),
    (4,5,8),(5,6,8),(6,7,8),(7,4,8),(0,3,2),(0,2,1),
]

bpy.ops.object.select_all(action="SELECT")
bpy.ops.object.delete(use_global=False)
scene = bpy.context.scene
scene.unit_settings.system = "METRIC"
scene.unit_settings.scale_length = 1
slug = Path(__file__).parent.name
root = bpy.data.objects.new("ASSET_" + slug, None)
scene.collection.objects.link(root)
root["salimon"] = {"assetId": "resource." + slug}
visual = bpy.data.objects.new("Visual", None)
scene.collection.objects.link(visual)
visual.parent = root

materials = []
for name, color in [("Silicate_Dark", (.34,.39,.28,1)), ("Silicate_Light", (.62,.64,.52,1))]:
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = color
    bsdf.inputs["Roughness"].default_value = .82
    materials.append(mat)

# Runtime is +Y up; Blender source is +Z up. Inverse of runtime x,z,-y conversion.
vertices = [(x, -z, y) for x, y, z in RUNTIME_VERTICES]
mesh = bpy.data.meshes.new("Silicate_Faceted")
mesh.from_pydata(vertices, [], FACES)
mesh.update()
obj = bpy.data.objects.new("Silicate_Fragment", mesh)
scene.collection.objects.link(obj)
obj.parent = visual
obj["salimon"] = {"role": "silicate-fragment-visual"}
for mat in materials:
    mesh.materials.append(mat)
for face in mesh.polygons:
    face.material_index = face.index % 2
bpy.ops.wm.save_as_mainfile(filepath=str(Path(__file__).with_name("source.blend")))
