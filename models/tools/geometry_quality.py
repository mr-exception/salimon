"""Deterministic meter-space diagnostics; independent of Blender and shading style.

Sweep-and-prune triangle candidates avoid global all-pairs scans. Deliberately
flat shading and disconnected visual panels are valid unless a node is declared
closed. Spatial policy is separate from manifest presentation budgets.
"""
from collections import Counter
import math

from diagnose_scout import cross, dot, overlap_area, sub


def matrix(node):
    if 'matrix' in node:
        a = node['matrix']
        return [[a[c*4+r] for c in range(4)] for r in range(4)]
    x, y, z, w = node.get('rotation', [0, 0, 0, 1])
    t = node.get('translation', [0, 0, 0])
    return [[1-2*(y*y+z*z), 2*(x*y-z*w), 2*(x*z+y*w), t[0]],
            [2*(x*y+z*w), 1-2*(x*x+z*z), 2*(y*z-x*w), t[1]],
            [2*(x*z-y*w), 2*(y*z+x*w), 1-2*(x*x+y*y), t[2]], [0, 0, 0, 1]]


def multiply(a, b):
    return [[sum(a[r][k]*b[k][c] for k in range(4)) for c in range(4)] for r in range(4)]


def transform(m, p, direction=False):
    return tuple(sum(m[r][k]*p[k] for k in range(3)) + (0 if direction else m[r][3]) for r in range(3))


def diagnose(document, binary, policy):
    # Late import avoids a validator/diagnostic import cycle.
    from validate_asset import accessor_values
    tolerance = policy.get('toleranceMeters', 1e-5)
    area_tolerance = policy.get('areaToleranceSquareMeters', 1e-10)
    parents = {child: i for i, n in enumerate(document['nodes']) for child in n.get('children', [])}
    matrices = {}
    def world(i):
        if i not in matrices:
            local = matrix(document['nodes'][i])
            matrices[i] = multiply(world(parents[i]), local) if i in parents else local
        return matrices[i]
    findings, faces, bounds, origins = [], [], {}, {}
    def report(code, node, face, location, **details):
        findings.append(dict(code=code, node=node, face=face, locationMeters=list(location),
                             toleranceMeters=tolerance, **details))
    for ni, node in enumerate(document['nodes']):
        m, name = world(ni), node['name']
        origins[name] = transform(m, (0, 0, 0))
        if 'mesh' not in node:
            continue
        all_points, edges = [], Counter()
        for pi, primitive in enumerate(document['meshes'][node['mesh']]['primitives']):
            points = [transform(m, p) for p in accessor_values(document, binary, primitive['attributes']['POSITION'])]
            all_points.extend(points)
            normals = [transform(m, n, True) for n in accessor_values(document, binary, primitive['attributes']['NORMAL'])] if 'NORMAL' in primitive['attributes'] else None
            indices = [v[0] for v in accessor_values(document, binary, primitive['indices'])] if 'indices' in primitive else list(range(len(points)))
            for offset in range(0, len(indices), 3):
                ids = indices[offset:offset+3]
                vertices = [points[i] for i in ids]
                label = [pi, offset//3]
                center = tuple(sum(v[a] for v in vertices)/3 for a in range(3))
                n = cross(sub(vertices[1], vertices[0]), sub(vertices[2], vertices[0]))
                length = math.sqrt(dot(n, n))
                if length <= area_tolerance*2:
                    report('degenerate', name, label, center)
                    continue
                normal = tuple(v/length for v in n)
                if normals and any(abs(dot(normals[i], normals[i])-1) > 1e-4 or dot(normal, normals[i]) < -1e-5 for i in ids):
                    report('normal-winding', name, label, center)
                welded = [tuple(round(v/tolerance) for v in p) for p in vertices]
                for a, b in zip(welded, welded[1:]+welded[:1]):
                    edges[tuple(sorted((a, b)))] += 1
                faces.append(dict(node=name, face=label, vertices=vertices, normal=normal, center=center,
                                  low=[min(p[a] for p in vertices) for a in range(3)],
                                  high=[max(p[a] for p in vertices) for a in range(3)]))
        if all_points:
            bounds[name] = [[min(p[a] for p in all_points) for a in range(3)],
                            [max(p[a] for p in all_points) for a in range(3)]]
        if name in policy.get('closedNodes', []) and any(count != 2 for count in edges.values()):
            report('non-manifold', name, None, origins[name], badEdges=sum(c != 2 for c in edges.values()))
    # Sweep along the longest global axis, with tolerant AABB rejection on all axes.
    axis = max(range(3), key=lambda a: max((f['high'][a] for f in faces), default=0)-min((f['low'][a] for f in faces), default=0))
    active, candidates = [], 0
    exemptions = {tuple(sorted(e['nodes'])): e for e in policy.get('overlapExemptions', [])}
    exempt_counts = Counter()
    for b in sorted(faces, key=lambda f: (f['low'][axis], f['node'], f['face'])):
        active = [a for a in active if a['high'][axis] >= b['low'][axis]-tolerance]
        for a in active:
            if any(a['high'][i] < b['low'][i]-tolerance or b['high'][i] < a['low'][i]-tolerance for i in range(3)):
                continue
            candidates += 1
            if abs(dot(a['normal'], b['normal'])) < 1-1e-8:
                continue
            if any(abs(dot(a['normal'], sub(p, a['vertices'][0]))) > tolerance for p in b['vertices']):
                continue
            area = overlap_area(a['vertices'], b['vertices'], a['normal'])
            if area <= area_tolerance:
                continue
            duplicate = all(any(math.dist(p, q) <= tolerance for q in b['vertices']) for p in a['vertices'])
            pair = tuple(sorted((a['node'], b['node'])))
            exemption = exemptions.get(pair)
            exempt_counts[pair] += 1
            reason = exemption['reason'] if exemption and exempt_counts[pair] <= exemption['maxPairs'] else None
            report('duplicate-face' if duplicate else 'surface-overlap', b['node'], b['face'], b['center'],
                   otherNode=a['node'], otherFace=a['face'], overlapSquareMeters=area,
                   exemption=reason)
        active.append(b)
    for name, expected in sorted(policy.get('boundsMeters', {}).items()):
        actual = bounds.get(name)
        if actual is None or any(abs(x-y) > tolerance for row, target in zip(actual, expected) for x, y in zip(row, target)):
            report('bounds', name, None, origins.get(name, (0, 0, 0)), expected=expected, actual=actual)
    for name, expected in sorted(policy.get('anchorsMeters', {}).items()):
        if name not in origins or math.dist(origins[name], expected) > tolerance:
            report('anchor', name, None, origins.get(name, (0, 0, 0)), expected=expected)
    for clearance in policy.get('clearanceBoxes', []):
        low, high = clearance['boundsMeters']
        for name in clearance['obstacles']:
            if name not in bounds:
                report('clearance-missing-obstacle', name, None, low)
            elif all(bounds[name][1][a] > low[a]+tolerance and bounds[name][0][a] < high[a]-tolerance for a in range(3)):
                report('clearance', name, None, origins[name], clearance=clearance['name'])
    warning_counts = Counter()
    for row in findings:
        row['severity'] = 'exempt' if row.get('exemption') else policy.get('severity', {}).get(row['code'], 'error')
        if row['severity'] == 'warning':
            warning_counts[row['code']] += 1
            if warning_counts[row['code']] > policy.get('maxWarnings', {}).get(row['code'], 0):
                row['severity'] = 'error'
    findings.sort(key=lambda f: (f['node'], f['code'], str(f['face']), str(f.get('otherFace'))))
    return {'findings': findings, 'triangles': len(faces), 'candidatePairs': candidates,
            'boundsMeters': bounds, 'originsMeters': origins,
            'status': 'failed' if any(f['severity'] == 'error' for f in findings) else 'passed'}
