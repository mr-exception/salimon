#!/usr/bin/env python3
"""Read-only scout geometry evidence, not a visual-quality acceptance gate.

Run from any directory; JSON goes to stdout. Only the baked, identity-transform
scout profile is supported. Coplanar tests use projected triangle intersection,
not bounding-box overlap. Near planes are bucketed at 10 micrometers; findings
are candidates for inspection, not proof of visible flicker or completeness.
"""
from collections import defaultdict
import hashlib
import itertools
import json
import math

from export_asset import REPO
from validate_asset import accessor_values, read_document

ASSET = REPO / 'models/assets/ships/salimon-scout'


def sub(a, b):
    return tuple(x-y for x, y in zip(a, b))


def dot(a, b):
    return sum(x*y for x, y in zip(a, b))


def cross(a, b):
    return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])


def side(a, b, p):
    return (b[0]-a[0])*(p[1]-a[1])-(b[1]-a[1])*(p[0]-a[0])


def overlap_area(a, b, normal):
    """Sutherland-Hodgman clip in the dominant plane, then recover m²."""
    drop = max(range(3), key=lambda i: abs(normal[i]))
    axes = [i for i in range(3) if i != drop]
    project = lambda p: tuple(p[i] for i in axes)
    polygon, clip = list(map(project, a)), list(map(project, b))
    if side(*clip) < 0:
        clip.reverse()
    for start, end in zip(clip, clip[1:] + clip[:1]):
        source, polygon = polygon, []
        for p, q in zip(source, source[1:] + source[:1]):
            sp, sq = side(start, end, p), side(start, end, q)
            if sp >= 0:
                polygon.append(p)
            if (sp >= 0) != (sq >= 0):
                t = sp / (sp-sq)
                polygon.append(tuple(x+t*(y-x) for x, y in zip(p, q)))
        if not polygon:
            return 0.0
    area = abs(sum(p[0]*q[1]-p[1]*q[0]
                   for p, q in zip(polygon, polygon[1:] + polygon[:1]))) / 2
    return area / abs(normal[drop])


def diagnose(document, binary):
    # Runtime imports local node transforms only. Reject rather than silently
    # ignore a future hierarchy change, rotation, or rescale in this probe.
    for node in document['nodes']:
        for key, identity in (('translation', [0, 0, 0]), ('rotation', [0, 0, 0, 1]),
                              ('scale', [1, 1, 1])):
            if node.get(key, identity) != identity and ('mesh' in node or node.get('children')):
                raise ValueError(f"{node['name']}: probe requires baked identity transforms")
        if 'matrix' in node:
            raise ValueError(f"{node['name']}: probe does not support matrix nodes")
    planes, nodes, degenerate, normal_issues = defaultdict(list), {}, [], []
    primitives = triangles = 0
    for node in document['nodes']:
        if 'mesh' not in node:
            continue
        name = node['name']
        points = []
        for pi, primitive in enumerate(document['meshes'][node['mesh']]['primitives']):
            if primitive.get('mode', 4) != 4:
                raise ValueError('probe requires triangle primitives')
            primitives += 1
            positions = accessor_values(document, binary, primitive['attributes']['POSITION'])
            normals = accessor_values(document, binary, primitive['attributes']['NORMAL'])
            points.extend(positions)
            indices = ([v[0] for v in accessor_values(document, binary, primitive['indices'])]
                       if 'indices' in primitive else list(range(len(positions))))
            material = document['materials'][primitive['material']]
            for ti in range(0, len(indices), 3):
                ids = indices[ti:ti+3]
                vertices = [positions[i] for i in ids]
                face = cross(sub(vertices[1], vertices[0]), sub(vertices[2], vertices[0]))
                length = math.sqrt(dot(face, face))
                label = {'node': name, 'primitive': pi, 'triangle': ti//3}
                triangles += 1
                if length <= 1e-10:
                    degenerate.append(label)
                    continue
                normal = tuple(v/length for v in face)
                if any(not math.isfinite(dot(normals[i], normals[i])) or
                       abs(dot(normals[i], normals[i])-1) > 1e-4 or
                       dot(normal, normals[i]) < -1e-4 for i in ids):
                    normal_issues.append(label)
                # Canonical sign groups opposite-winding faces as well.
                axis = max(range(3), key=lambda i: abs(normal[i]))
                if normal[axis] < 0:
                    normal = tuple(-v for v in normal)
                plane = tuple(round(v, 5) for v in (*normal, dot(normal, vertices[0])))
                planes[plane].append((label, vertices, normal, material['name']))
        nodes[name] = {'minMeters': [min(p[i] for p in points) for i in range(3)],
                       'maxMeters': [max(p[i] for p in points) for i in range(3)]}
    overlaps = {}
    for group in planes.values():
        for a, b in itertools.combinations(group, 2):
            # Bucket equality is only a prefilter; verify all B points against A.
            if any(abs(dot(a[2], sub(p, a[1][0]))) > 1e-5 for p in b[1]):
                continue
            if any(max(p[i] for p in a[1]) < min(p[i] for p in b[1]) or
                   max(p[i] for p in b[1]) < min(p[i] for p in a[1]) for i in range(3)):
                continue
            area = overlap_area(a[1], b[1], a[2])
            if area <= 1e-8:
                continue  # Shared edges and numerical dust are not surfaces.
            key = tuple(sorted((a[0]['node'], b[0]['node'])))
            row = overlaps.setdefault(key, {'nodes': key, 'trianglePairs': 0,
                                            'summedOverlapSquareMeters': 0,
                                            'example': {'a': a[0], 'b': b[0],
                                                        'aVerticesMeters': a[1],
                                                        'bVerticesMeters': b[1],
                                                        'materials': [a[3], b[3]]}})
            row['trianglePairs'] += 1
            row['summedOverlapSquareMeters'] += area
    lower = [min(n['minMeters'][i] for n in nodes.values()) for i in range(3)]
    upper = [max(n['maxMeters'][i] for n in nodes.values()) for i in range(3)]
    return {'triangles': triangles, 'primitives': primitives,
            'materials': len(document['materials']), 'boundsMeters': [lower, upper],
            'dimensionsMeters': sub(upper, lower), 'nodeBounds': nodes,
            'degenerateTriangles': degenerate, 'normalIssueTriangles': normal_issues,
            'coplanarOverlapCandidates': [overlaps[k] for k in sorted(overlaps)],
            'blendedMaterials': [m['name'] for m in document['materials'] if m.get('alphaMode') == 'BLEND']}


def main():
    manifest = json.loads((ASSET / 'manifest.json').read_text())
    path = REPO / manifest['runtime']
    document, binary = read_document(path)
    result = diagnose(document, binary)
    result.update(runtimeSha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                  glbBytes=path.stat().st_size, budgets=manifest['budgets'])
    report = json.loads((ASSET / 'export-report.json').read_text())
    result['authoringHashMatches'] = {
        name: hashlib.sha256((ASSET / name).read_bytes()).hexdigest() == expected
        for name, expected in report['authoringSha256'].items()}
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == '__main__':
    main()
