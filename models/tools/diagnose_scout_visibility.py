#!/usr/bin/env python3
"""Read-only scout baseline probe; candidates are evidence, not validation failures.

Coordinates are runtime ship-local meters, closed door, identity ship pose.
Only the scout's baked identity-transform profile is supported. No Blender/GPU.
"""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
import math
import struct
from pathlib import Path

from validate_asset import REPO, accessor_values, read_document

SCOUT = REPO / 'models/assets/ships/salimon-scout'
PLANE_EPSILON = 1e-5  # meters; narrower than a visible 0.1 mm authored separation
AREA_EPSILON = 1e-8  # projected square meters; excludes shared edges


def sub(a, b):
    return tuple(x-y for x, y in zip(a, b))


def dot(a, b):
    return sum(x*y for x, y in zip(a, b))


def cross(a, b):
    return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])


def signed_area(poly):
    return sum(a[0]*b[1]-a[1]*b[0] for a, b in zip(poly, poly[1:]+poly[:1])) / 2


def overlap_polygon(a, b):
    """Convex triangle intersection in 2D; shared edges have zero area."""
    polygon = list(a)
    if signed_area(b) < 0:
        b = list(reversed(b))
    for start, end in zip(b, b[1:]+b[:1]):
        if not polygon:
            break
        def side(p):
            return (end[0]-start[0])*(p[1]-start[1]) - (end[1]-start[1])*(p[0]-start[0])
        output = []
        for p, q in zip(polygon, polygon[1:]+polygon[:1]):
            sp, sq = side(p), side(q)
            if sp >= 0:
                output.append(p)
            if (sp < 0) != (sq < 0):
                t = sp / (sp-sq)
                output.append(tuple(x+t*(y-x) for x, y in zip(p, q)))
        polygon = output
    return polygon


def triangles(document, binary):
    result, bounds, normal_counts = [], {}, Counter()
    parents = {child: parent for parent, n in enumerate(document['nodes']) for child in n.get('children', [])}
    for ni, node in enumerate(document['nodes']):
        if (node.get('rotation', [0, 0, 0, 1]) != [0, 0, 0, 1]
                or node.get('scale', [1, 1, 1]) != [1, 1, 1] or 'matrix' in node):
            raise ValueError(f"{node['name']}: diagnostic supports baked scout transforms only")
        if 'mesh' not in node:
            continue
        # Runtime loader applies only the node-local transform; verify that all
        # ancestors of visual nodes are identity rather than hiding that limitation.
        ancestor = parents.get(ni)
        while ancestor is not None:
            if document['nodes'][ancestor].get('translation', [0, 0, 0]) != [0, 0, 0]:
                raise ValueError('nonidentity visual ancestor: runtime world transform needs review')
            ancestor = parents.get(ancestor)
        points = []
        for pi, primitive in enumerate(document['meshes'][node['mesh']]['primitives']):
            if primitive.get('mode', 4) != 4:
                raise ValueError('triangles required')
            positions = [tuple(x+y for x, y in zip(p, node.get('translation', [0, 0, 0])))
                         for p in accessor_values(document, binary, primitive['attributes']['POSITION'])]
            normals = accessor_values(document, binary, primitive['attributes']['NORMAL'])
            points.extend(positions)
            indices = ([v[0] for v in accessor_values(document, binary, primitive['indices'])]
                       if 'indices' in primitive else list(range(len(positions))))
            for fi in range(0, len(indices), 3):
                ids = indices[fi:fi+3]
                vertices = [positions[i] for i in ids]
                n = cross(sub(vertices[1], vertices[0]), sub(vertices[2], vertices[0]))
                length = math.sqrt(dot(n, n))
                if length < 1e-12:
                    normal_counts['degenerateTriangles'] += 1
                    continue
                n = tuple(v/length for v in n)
                for i in ids:
                    norm = math.sqrt(dot(normals[i], normals[i]))
                    if not math.isfinite(norm) or abs(norm-1) > 1e-4:
                        normal_counts['nonUnitCornerNormals'] += 1
                    if dot(n, normals[i]) < -1e-5:
                        normal_counts['opposedWindingCornerNormals'] += 1
                if next(v for v in n if abs(v) > 1e-8) < 0:
                    n = tuple(-v for v in n)
                result.append(dict(node=node['name'], primitive=pi, face=fi//3,
                                   material=document['materials'][primitive['material']]['name'],
                                   vertices=vertices, normal=n, plane=dot(n, vertices[0])))
        bounds[node['name']] = dict(min=[min(p[i] for p in points) for i in range(3)],
                                    max=[max(p[i] for p in points) for i in range(3)])
    return result, bounds, normal_counts


def coplanar_candidates(faces):
    groups = defaultdict(list)
    for face in faces:
        groups[tuple(round(v, 5) for v in face['normal'])].append(face)
    pairs = defaultdict(lambda: dict(trianglePairs=0, projectedOverlapAreaM2=0, witness=None))
    for group in groups.values():
        group.sort(key=lambda f: f['plane'])
        for i, a in enumerate(group):
            axis = max(range(3), key=lambda k: abs(a['normal'][k]))
            keep = [k for k in range(3) if k != axis]
            pa = [tuple(p[k] for k in keep) for p in a['vertices']]
            for b in group[i+1:]:
                if b['plane']-a['plane'] > PLANE_EPSILON:
                    break
                if max(abs(dot(a['normal'], p)-a['plane']) for p in b['vertices']) > PLANE_EPSILON:
                    continue
                pb = [tuple(p[k] for k in keep) for p in b['vertices']]
                poly = overlap_polygon(pa, pb)
                area = abs(signed_area(poly))
                if area <= AREA_EPSILON:
                    continue
                key = tuple(sorted((a['node'], b['node'])))
                item = pairs[key]
                item['trianglePairs'] += 1
                item['projectedOverlapAreaM2'] += area
                if item['witness'] is None:
                    center = [0.0]*3
                    for j, k in enumerate(keep):
                        center[k] = sum(p[j] for p in poly)/len(poly)
                    center[axis] = (a['plane']-sum(a['normal'][k]*center[k] for k in keep))/a['normal'][axis]
                    item['witness'] = dict(positionMeters=center, normal=a['normal'],
                                           faces=[{k: f[k] for k in ('node', 'primitive', 'face', 'material')} for f in (a, b)])
    return [dict(nodes=list(key), **value) for key, value in sorted(pairs.items())]


def ray_hit(origin, direction, face):
    """Double-sided Moller-Trumbore, matching the ship's no-cull pipeline."""
    a, b, c = face['vertices']
    e1, e2 = sub(b, a), sub(c, a)
    h = cross(direction, e2)
    determinant = dot(e1, h)
    if abs(determinant) < 1e-12:
        return None
    relative = sub(origin, a)
    u = dot(relative, h)/determinant
    q = cross(relative, e1)
    v = dot(direction, q)/determinant
    distance = dot(e2, q)/determinant
    if u < -1e-8 or v < -1e-8 or u+v > 1+1e-8 or distance <= 1e-6:
        return None
    return distance


def witness_views(candidate, faces):
    witness = candidate['witness']
    point, normal = witness['positionMeters'], witness['normal']
    views = []
    for side in (-1, 1):
        eye = [p+side*n for p, n in zip(point, normal)]
        direction = [-side*n for n in normal]
        hits = [(distance, f) for f in faces
                if f['material'] != 'Cockpit Glass'
                and (distance := ray_hit(eye, direction, f)) is not None]
        nearest = min((d for d, _ in hits), default=None)
        front = [dict(node=f['node'], primitive=f['primitive'], face=f['face'], distanceMeters=d)
                 for d, f in hits if nearest is not None and d <= nearest+PLANE_EPSILON]
        views.append(dict(eyeMeters=eye, targetMeters=point,
                          up=[0, 0, 1] if abs(normal[1]) > 0.9 else [0, 1, 0],
                          nearestOpaqueHits=front,
                          witnessExposed=nearest is not None and abs(nearest-1) <= PLANE_EPSILON))
    return views


def diagnose(views_path=None):
    manifest = json.loads((SCOUT / 'manifest.json').read_text())
    path = REPO / manifest['runtime']
    document, binary = read_document(path)
    faces, bounds, normals = triangles(document, binary)
    images = []
    for image in document.get('images', []):
        view = document['bufferViews'][image['bufferView']]
        start = view.get('byteOffset', 0)
        payload = binary[start:start+view['byteLength']]
        entry = dict(name=image.get('name'), embeddedBytes=len(payload))
        if payload[:8] == b'\x89PNG\r\n\x1a\n':
            width, height = struct.unpack_from('>II', payload, 16)
            entry.update(width=width, height=height, baseMipRgba8Bytes=width*height*4)
        images.append(entry)
    lower = [min(b['min'][i] for b in bounds.values()) for i in range(3)]
    upper = [max(b['max'][i] for b in bounds.values()) for i in range(3)]
    export = json.loads((SCOUT / 'export-report.json').read_text())
    digest = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    fixtures = json.loads(views_path.read_text()) if views_path else {'views': []}
    ray_traces = {}
    for view in fixtures['views']:
        delta = sub(view['targetMeters'], view['eyeMeters'])
        length = math.sqrt(dot(delta, delta))
        direction = tuple(v/length for v in delta)
        ray_traces[view['id']] = sorted(
            [dict(distanceMeters=d, **{k: f[k] for k in ('node', 'primitive', 'face', 'material')})
             for f in faces if (d := ray_hit(view['eyeMeters'], direction, f)) is not None],
            key=lambda h: h['distanceMeters'])
    candidates = coplanar_candidates(faces)
    for candidate in candidates:
        candidate['views'] = witness_views(candidate, faces)
    return dict(assetId=manifest['assetId'], runtimeSha256=digest(path),
                exportRuntimeHashMatches=digest(path) == export['runtimeSha256'],
                authoringHashMatches={p: digest(SCOUT / p) == h for p, h in export['authoringSha256'].items()},
                budgets=manifest['budgets'],
                measured=dict(triangles=len(faces)+normals['degenerateTriangles'],
                              primitives=sum(len(document['meshes'][n['mesh']]['primitives']) for n in document['nodes'] if 'mesh' in n),
                              materials=len(document['materials']), glbBytes=path.stat().st_size,
                              embeddedImageBytes=sum(document['bufferViews'][im['bufferView']]['byteLength'] for im in document.get('images', []))),
                boundsMeters=dict(min=lower, max=upper, dimensions=sub(upper, lower)),
                nodeBoundsMeters=bounds, normalDiagnostics={k: normals[k] for k in ('degenerateTriangles', 'nonUnitCornerNormals', 'opposedWindingCornerNormals')},
                materialContracts=document['materials'], embeddedImages=images,
                diagnosticPose=dict(doorOpenFraction=0, shipOrientationXyzw=[0, 0, 0, 1]),
                thresholds=dict(planeDistanceMeters=PLANE_EPSILON, projectedAreaM2=AREA_EPSILON),
                coplanarCandidates=candidates, fixtureCenterRayHits=ray_traces)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, help='JSON destination; stdout by default')
    parser.add_argument('--views', type=Path, help='optional ship-local diagnostic camera JSON')
    args = parser.parse_args()
    report = json.dumps(diagnose(args.views), indent=2, sort_keys=True) + '\n'
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(report)
    else:
        print(report, end='')


if __name__ == '__main__':
    main()
