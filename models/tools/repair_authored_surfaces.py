"""Blender-only, deliberate source migration for #244; never runs during export.

Partition coplanar primitive panels rather than moving them or changing scale.
Opposing closed-solid caps remain intentional construction; different-material
same-facing surfaces have one owner. Glass is trimmed against opaque frames.
Articulated contact surfaces are retained because their runtime pose changes.
Run from the repository root with Blender 4.5.3 --background --python this_file.
"""
from pathlib import Path
import json
import math
import sys

import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from surface_partition import area, subtract


def dot(a, b):
    return sum(x*y for x, y in zip(a, b))


def sub(a, b):
    return tuple(x-y for x, y in zip(a, b))


REPO = Path(__file__).resolve().parents[2]
EPS = 1e-5  # meters, same as the exported quality policy
AREA = 1e-10
MOVING = {'Exit_Door', 'Exit_Ramp_Tip'}


def priority(face):
    material = face['material']
    rank = {'Cockpit Glass': -10, 'Hull Graphite': 0, 'Hull Ceramic': 1,
            'Floor Grip': 2, 'Interior Light': 2, 'Interior Dark': 2,
            'Cargo_Panel': 0, 'Cargo_Frame': 1, 'Cargo_Trim': 2,
            'Copper Accent': 3, 'Engine Glow': 4}.get(material, 3)
    return rank, face['name'], face['index']


def faces_for(objects):
    faces = []
    for obj in objects:
        mesh = obj.data
        mesh.calc_loop_triangles()
        uv = mesh.uv_layers.active
        # These sources are baked identity static meshes; moving meshes retain
        # translations. Work in shared source coordinates without touching poses.
        for tri in mesh.loop_triangles:
            points = [obj.matrix_world @ mesh.vertices[i].co for i in tri.vertices]
            points = [tuple(p) for p in points]
            a, b = sub(points[1], points[0]), sub(points[2], points[0])
            normal = (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])
            length = math.sqrt(dot(normal, normal))
            if length <= AREA*2:
                continue
            normal = tuple(v/length for v in normal)
            faces.append(dict(obj=obj, name=obj.name, index=len(faces), points=points,
                normal=normal, material=mesh.materials[tri.material_index].name,
                slot=tri.material_index, normals=[tuple(mesh.corner_normals[i].vector) for i in tri.loops],
                uv=[tuple(uv.data[i].uv) if uv else (0, 0) for i in tri.loops],
                low=[min(p[i] for p in points) for i in range(3)],
                high=[max(p[i] for p in points) for i in range(3)]))
    return faces


def repair(objects, label):
    faces = faces_for(objects)
    cutters = [[] for _ in faces]
    active = []
    for b in sorted(faces, key=lambda f: f['low'][0]):
        active = [a for a in active if a['high'][0] >= b['low'][0]-EPS]
        for a in active:
            if a['name'] != b['name'] and {a['name'], b['name']} & MOVING:
                continue
            if any(a['high'][i] < b['low'][i]-EPS or b['high'][i] < a['low'][i]-EPS for i in range(3)):
                continue
            alignment = dot(a['normal'], b['normal'])
            if abs(alignment) < 1-1e-8 or any(abs(dot(a['normal'], sub(p,a['points'][0]))) > EPS for p in b['points']):
                continue
            if 'Cockpit Glass' in (a['material'], b['material']):
                loser, winner = (a, b) if a['material'] == 'Cockpit Glass' else (b, a)
                cutters[loser['index']].append(winner)
            elif alignment < 0:
                # Flush caps of joined solids remain editable construction faces.
                continue
            elif a['material'] == b['material']:
                continue
            else:
                loser, winner = sorted((a, b), key=priority)
                cutters[loser['index']].append(winner)
        active.append(b)
    output = {o.name: [] for o in objects}
    changed = {o.name for o in objects if sum(1 for f in faces if f['name']==o.name) < len(o.data.loop_triangles)}
    for face, others in zip(faces, cutters):
        axis = max(range(3), key=lambda i: abs(face['normal'][i]))
        keep = [i for i in range(3) if i != axis]
        project = lambda p: tuple(p[i] for i in keep)
        pieces = [[(project(p), uv, tuple(p)) for p, uv in zip(face['points'], face['uv'])]]
        original = pieces[0]
        for other in others:
            pieces = [part for poly in pieces for part in subtract(poly, [project(p) for p in other['points']])]
        if pieces != [original]:
            changed.add(face['name'])
        for piece in pieces:
            # Fan triangulation is valid: clipping convex triangles yields convex pieces.
            for i in range(1, len(piece)-1):
                tri = [piece[0], piece[i], piece[i+1]]
                if abs(area([v[0] for v in tri])) <= AREA:
                    continue
                verts, uvs = [], []
                for xy, uv, position in tri:
                    verts.append(tuple(face['obj'].matrix_world.inverted() @ Vector(position)))
                    uvs.append(uv)
                output[face['name']].append((verts, uvs, face['slot'], face['normals']))
    summary = {}
    for obj in objects:
        if obj.name not in changed:
            continue
        rows = output[obj.name]
        mesh = bpy.data.meshes.new(obj.data.name+'_partitioned')
        vertices, indices, by_position = [], [], {}
        for points, _, _, _ in rows:
            face_indices = []
            for point in points:
                if point not in by_position:
                    by_position[point] = len(vertices)
                    vertices.append(point)
                face_indices.append(by_position[point])
            indices.append(face_indices)
        mesh.from_pydata(vertices, [], indices)
        for mat in obj.data.materials:
            mesh.materials.append(mat)
        uv = mesh.uv_layers.new(name=obj.data.uv_layers.active.name) if obj.data.uv_layers.active else None
        for polygon, (_, coords, slot, _) in zip(mesh.polygons, rows):
            polygon.material_index = slot
            polygon.use_smooth = True
            if uv:
                for loop, coord in zip(polygon.loop_indices, coords):
                    uv.data[loop].uv = coord
        summary[obj.name] = {'before': sum(1 for f in faces if f['name']==obj.name), 'after': len(rows)}
        mesh.normals_split_custom_set([n for _, _, _, normals in rows for n in normals])
        obj.data = mesh
    print('SURFACE_REPAIR', label, json.dumps(summary, sort_keys=True))
    return changed


def main():
    if bpy.app.version[:3] != (4, 5, 3):
        raise ValueError('source remediation requires Blender 4.5.3')
    scout = REPO/'models/assets/ships/salimon-scout'
    assembly = scout/'assembly/salimon-scout.blend'
    bpy.ops.wm.open_mainfile(filepath=str(assembly))
    objects = [o for o in bpy.context.scene.objects if o.type == 'MESH' and not o.name.startswith('COLLIDER_')]
    # Freeze evaluated assembly data before opening any owning component.
    owners = {o.name: bpy.path.abspath(o.library.filepath) for o in objects}
    changed = repair(objects, 'scout')
    objects = [o for o in objects if o.name in changed]
    owners = {name: path for name, path in owners.items() if name in changed}
    # Carry numeric mesh payloads across file opens; loading a scratch .blend
    # would also bring linked materials and their relative library dependencies.
    repaired = {}
    for obj in objects:
        mesh = obj.data
        repaired[obj.name] = dict(
            vertices=[tuple(v.co) for v in mesh.vertices],
            faces=[tuple(p.vertices) for p in mesh.polygons],
            slots=[p.material_index for p in mesh.polygons],
            normals=[tuple(n.vector) for n in mesh.corner_normals],
            uv=[tuple(u.uv) for u in mesh.uv_layers.active.data] if mesh.uv_layers.active else None,
            uv_name=mesh.uv_layers.active.name if mesh.uv_layers.active else None)
    for path in sorted(set(owners.values())):
        bpy.ops.wm.open_mainfile(filepath=path)
        for name, owner in owners.items():
            if owner != path:
                continue
            obj, data = bpy.data.objects[name], repaired[name]
            mesh = bpy.data.meshes.new(obj.data.name+'_repaired')
            mesh.from_pydata(data['vertices'], [], data['faces'])
            for material in obj.data.materials:
                mesh.materials.append(material)
            for polygon, slot in zip(mesh.polygons, data['slots']):
                polygon.material_index = slot
                polygon.use_smooth = True
            if data['uv']:
                uv = mesh.uv_layers.new(name=data['uv_name'])
                for loop, coord in zip(uv.data, data['uv']):
                    loop.uv = coord
            mesh.normals_split_custom_set(data['normals'])
            old = obj.data
            original_name = old.name
            obj.data = mesh
            if old.users == 0:
                bpy.data.meshes.remove(old)
            mesh.name = original_name
        bpy.ops.wm.save_as_mainfile(filepath=path)
    for slug in ('cargo-container', 'mining-tool'):
        path = REPO/f'models/assets/items/{slug}/source.blend'
        bpy.ops.wm.open_mainfile(filepath=str(path))
        repair([o for o in bpy.context.scene.objects if o.type=='MESH'], slug)
        bpy.ops.wm.save_as_mainfile(filepath=str(path))


if __name__ == '__main__':
    main()
