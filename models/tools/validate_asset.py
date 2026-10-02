#!/usr/bin/env python3
"""Validate generic authored asset contracts without loading Blender or game code."""
import argparse
from collections import Counter
import json
import math
from pathlib import Path
import struct
import sys

from export_asset import REPO, ExportError, check_export, contained_path, resolve_manifest


class ValidationError(ExportError):
    pass


def require(condition, message):
    if not condition:
        raise ValidationError(message)


def vector(value, length, label):
    require(isinstance(value, list) and len(value) == length and
            all(type(x) in (int, float) and math.isfinite(x) for x in value),
            f'{label}: expected {length} finite numbers')
    return value


def index(value, sequence, label):
    require(type(value) is int and 0 <= value < len(sequence), f'{label}: invalid index {value!r}')
    return sequence[value]


def read_document(path):
    """Read the complete GLB envelope, including embedded buffer ranges."""
    data = path.read_bytes()
    require(len(data) >= 20, 'GLB header is truncated')
    magic, version, size = struct.unpack_from('<4sII', data)
    require(magic == b'glTF' and version == 2 and size == len(data), 'invalid GLB header')
    chunks = []
    offset = 12
    while offset < size:
        require(offset + 8 <= size, 'truncated GLB chunk header')
        length, kind = struct.unpack_from('<II', data, offset)
        offset += 8
        require(length % 4 == 0 and offset + length <= size, 'invalid GLB chunk length')
        chunks.append((kind, data[offset:offset + length]))
        offset += length
    require([kind for kind, _ in chunks] in ([0x4E4F534A], [0x4E4F534A, 0x004E4942]),
            'GLB must contain JSON followed by at most one BIN chunk')
    document = json.loads(chunks[0][1])
    require(isinstance(document, dict), 'GLB JSON must be an object')
    require(document.get('asset', {}).get('version') == '2.0', 'glTF asset.version must be 2.0')
    binary = chunks[1][1] if len(chunks) == 2 else b''
    buffers = document.get('buffers', [])
    require(len(buffers) <= 1 and all('uri' not in b for b in buffers), 'buffers must be embedded')
    declared = buffers[0]['byteLength'] if buffers else 0
    require(type(declared) is int and 0 <= declared <= len(binary) <= declared + 3,
            'embedded buffer length does not match BIN chunk')
    views = document.get('bufferViews', [])
    for i, view in enumerate(views):
        index(view['buffer'], buffers, f'bufferView {i}.buffer')
        start, length = view.get('byteOffset', 0), view['byteLength']
        require(type(start) is int and type(length) is int and start >= 0 and length > 0 and
                start + length <= declared, f'bufferView {i}: range exceeds buffer')
    return document, binary


def accessor_values(document, binary, reference):
    """Decode validated embedded accessors, honoring offsets and byte strides."""
    accessor = document['accessors'][reference]
    view = document['bufferViews'][accessor['bufferView']]
    width = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4}[accessor['type']]
    kind = {5120: 'b', 5121: 'B', 5122: 'h', 5123: 'H', 5125: 'I', 5126: 'f'}[accessor['componentType']]
    layout = '<' + kind * width
    start = view.get('byteOffset', 0) + accessor.get('byteOffset', 0)
    stride = view.get('byteStride', struct.calcsize(layout))
    return [struct.unpack_from(layout, binary, start + row * stride)
            for row in range(accessor['count'])]


def position_bounds(document, binary, reference):
    """Measure real vertices instead of trusting declared accessor min/max."""
    values = accessor_values(document, binary, reference)
    return ([min(p[a] for p in values) for a in range(3)],
            [max(p[a] for p in values) for a in range(3)])


def validate_document(path, manifest, extension_validators=None, collision_validators=None):
    """Run shared checks first; registered callbacks may add, never bypass, checks."""
    doc, binary = read_document(path)
    check_export(path, manifest)
    require(not doc.get('extensionsRequired'), 'required glTF extensions need a future supported decoder')
    require(not doc.get('animations') and not doc.get('skins'), 'v1 only supports static assets')
    nodes = doc.get('nodes', [])
    names = {node.get('name'): i for i, node in enumerate(nodes)}
    contracts = manifest['contracts']
    parents = {}
    for i, node in enumerate(nodes):
        label = f'node {node.get("name", i)}'
        require('matrix' not in node or not any(k in node for k in ('translation', 'rotation', 'scale')),
                f'{label}: matrix and TRS cannot coexist')
        if 'matrix' in node:
            m = vector(node['matrix'], 16, label + '.matrix')
            require(all(abs(m[j] - expected) < 1e-6 for j, expected in ((3, 0), (7, 0), (11, 0), (15, 1))),
                    f'{label}: matrix must be affine')
            columns = [m[j:j + 3] for j in (0, 4, 8)]
            require(all(abs(sum(x*x for x in c) - 1) < 1e-5 for c in columns) and
                    all(abs(sum(a*b for a, b in zip(columns[x], columns[y]))) < 1e-5
                        for x, y in ((0, 1), (0, 2), (1, 2))), f'{label}: apply scale/shear')
            a, b, c = columns
            determinant = a[0]*(b[1]*c[2]-b[2]*c[1])-b[0]*(a[1]*c[2]-a[2]*c[1])+c[0]*(a[1]*b[2]-a[2]*b[1])
            require(determinant > 0, f'{label}: reflected transform')
        else:
            vector(node.get('translation', [0, 0, 0]), 3, label + '.translation')
            scale = vector(node.get('scale', [1, 1, 1]), 3, label + '.scale')
            require(all(abs(x - 1) < 1e-6 for x in scale), f'{label}: apply scale (meter frame requires unit scale)')
            rotation = vector(node.get('rotation', [0, 0, 0, 1]), 4, label + '.rotation')
            require(abs(sum(x*x for x in rotation) - 1) < 1e-5, f'{label}: rotation must be a unit quaternion')
        for child in node.get('children', []):
            index(child, nodes, label + '.children')
            require(child not in parents, f'node {child}: duplicate child or multiple parents')
            parents[child] = i
        require('camera' not in node and 'skin' not in node, f'{label}: authoring helper/rig in runtime asset')
    root = names[contracts['root']]
    require(root not in parents, 'contracts.root must have no parent')
    scene = index(doc.get('scene', 0), doc.get('scenes', []), 'scene')
    require(scene.get('nodes') == [root] and len(doc['scenes']) == 1, 'scene must contain exactly the declared asset root')
    descendants = {}
    def walk(node, active):
        require(node not in active, f'node {node}: hierarchy cycle')
        result = {node}
        for child in nodes[node].get('children', []):
            result.update(walk(child, active | {node}))
        descendants[node] = result
        return result
    require(walk(root, set()) == set(range(len(nodes))), 'all exported nodes must belong to the asset root')
    roles = [contracts[key] for key in ('sockets', 'markers', 'colliders')]
    require(len(set(sum(roles, []))) == len(sum(roles, [])), 'socket/marker/collider roles must not overlap')
    visual = set().union(*(descendants[names[n]] for n in contracts['visualGroups']))
    require(root not in visual, 'visual groups must be below asset root')
    for name in contracts['groups'] + contracts['visualGroups']:
        require('mesh' not in nodes[names[name]], f'group {name}: expected a hierarchy node, not geometry')
    if manifest['profile'] == 'standard-v1':
        require(contracts['root'].startswith('ASSET_') and len(contracts['root']) > 6,
                'standard-v1 root must use ASSET_<slug>')
        require('Visual' in contracts['visualGroups'], 'standard-v1 requires Visual group')
        for key, prefix, group in (('sockets', 'SOCKET_', 'Sockets'), ('markers', 'MARKER_', 'Interaction_Markers'),
                                   ('colliders', 'COLLIDER_', 'Collision_Proxies')):
            for name in contracts[key]:
                require(name.startswith(prefix) and len(name) > len(prefix), f'{key}: invalid name {name}')
                require(group in names and names[name] in descendants[names[group]] and name != group,
                        f'{name}: must be below {group}')
    for name in sum(roles, []):
        node = nodes[names[name]]
        require('mesh' not in node and names[name] not in visual, f'{name}: spatial contracts must be nonvisual nodes')
    if manifest['collision']['policy'] == 'none':
        require(not any(n and n.startswith('COLLIDER_') for n in names), 'collision none: unexpected proxy')
    for name in contracts['colliders']:
        node = nodes[names[name]]
        extras = node.get('extras', {})
        metadata = extras.get('salimon', {})
        # The explicit migration profile preserves metadata-only legacy boxes
        # until #81 moves them into authored proxies. Standard assets still
        # require source-frame proxy dimensions and shape metadata.
        legacy = manifest['profile'] == 'legacy-scout-v1'
        shape = metadata.get('collisionShape' if legacy else 'shape')
        require(shape in manifest['collision'].get('shapes', []),
                f'{name}: shape is not declared in collision.shapes')
        dimensions = vector(metadata.get('sizeMeters') if legacy else extras.get('salimonProxyDimensions'),
                            3, name + ('.sizeMeters' if legacy else '.salimonProxyDimensions'))
        require(all(x > 0 for x in dimensions), f'{name}: proxy dimensions must be positive meters')
    materials = doc.get('materials', [])
    duplicates = [n for n, count in Counter(m.get('name') for m in materials if m.get('name')).items() if count > 1]
    require(not duplicates, f'duplicate material names: {duplicates}')
    accessors = doc.get('accessors', [])
    widths = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4}
    component_sizes = {5120: 1, 5121: 1, 5122: 2, 5123: 2, 5125: 4, 5126: 4}
    for i, acc in enumerate(accessors):
        require('sparse' not in acc, f'accessor {i}: sparse data is unsupported')
        require(acc.get('type') in widths and acc.get('componentType') in component_sizes,
                f'accessor {i}: unsupported component/type')
        require(type(acc.get('count')) is int and acc['count'] > 0, f'accessor {i}: invalid count')
        view = index(acc.get('bufferView'), doc.get('bufferViews', []), f'accessor {i}.bufferView')
        width = widths[acc['type']] * component_sizes[acc['componentType']]
        stride = view.get('byteStride', width)
        offset = acc.get('byteOffset', 0)
        require(type(stride) is int and stride >= width and type(offset) is int and offset >= 0 and
                offset + (acc['count'] - 1)*stride + width <= view['byteLength'], f'accessor {i}: range exceeds bufferView')
        if acc['componentType'] == 5126:
            for row in range(acc['count']):
                values = struct.unpack_from('<' + 'f'*widths[acc['type']], binary,
                                            view.get('byteOffset', 0) + offset + row*stride)
                require(all(math.isfinite(x) for x in values), f'accessor {i}: nonfinite geometry')
    mesh_triangles = []
    primitives = 0
    for mi, mesh in enumerate(doc.get('meshes', [])):
        triangles = 0
        for primitive in mesh['primitives']:
            require(primitive.get('mode', 4) == 4, f'mesh {mi}: only triangle lists are supported')
            position = index(primitive.get('attributes', {}).get('POSITION'), accessors, f'mesh {mi}.POSITION')
            require(position['type'] == 'VEC3' and position['componentType'] == 5126,
                    f'mesh {mi}: POSITION must contain float VEC3 data')
            for semantic, reference in primitive['attributes'].items():
                attribute = index(reference, accessors, f'mesh {mi}.{semantic}')
                require(attribute['count'] == position['count'], f'mesh {mi}.{semantic}: attribute count mismatch')
            if 'indices' in primitive:
                acc = index(primitive['indices'], accessors, f'mesh {mi}.indices')
                require(acc['type'] == 'SCALAR' and acc['componentType'] in (5121, 5123, 5125), f'mesh {mi}: invalid indices')
                view = doc['bufferViews'][acc['bufferView']]
                size = component_sizes[acc['componentType']]
                for row in range(acc['count']):
                    value = struct.unpack_from('<' + {5121: 'B', 5123: 'H', 5125: 'I'}[acc['componentType']], binary,
                                               view.get('byteOffset', 0) + acc.get('byteOffset', 0) + row*view.get('byteStride', size))[0]
                    require(value < position['count'], f'mesh {mi}: index exceeds POSITION count')
                count = acc['count']
            else:
                count = position['count']
            require(count % 3 == 0, f'mesh {mi}: triangle count is not divisible by three')
            if 'material' in primitive:
                index(primitive['material'], materials, f'mesh {mi}.material')
            triangles += count // 3
            primitives += 1
        mesh_triangles.append(triangles)
    for i, node in enumerate(nodes):
        if 'mesh' in node:
            index(node['mesh'], mesh_triangles, f'node {node.get("name", i)}.mesh')
            require(i in visual, f'node {node.get("name", i)}: geometry outside visual groups')
    require(mesh_triangles and sum(mesh_triangles) > 0, 'asset must contain visible triangles')
    textures = 0
    for image in doc.get('images', []):
        require('uri' not in image, 'images must be embedded')
        textures += index(image.get('bufferView'), doc.get('bufferViews', []), 'image.bufferView')['byteLength']
    metrics = dict(maxTriangles=sum(mesh_triangles), maxPrimitives=primitives, maxMaterials=len(materials),
                   maxTextureBytes=textures, maxGlbBytes=path.stat().st_size)
    for key, measured in metrics.items():
        require(measured <= manifest['budgets'][key], f'budgets.{key}: measured {measured} exceeds {manifest["budgets"][key]}')
    lods = manifest.get('lods', [])
    require([l['level'] for l in lods] == list(range(len(lods))), 'lods: levels must be ordered consecutively from zero')
    require(len({l['node'] for l in lods}) == len(lods), 'lods: duplicate node')
    covered = set()
    for lod in lods:
        subtree = descendants[names[lod['node']]]
        require(subtree <= visual and not covered.intersection(subtree), f'LOD {lod["level"]}: must be disjoint beneath visual hierarchy')
        require(lod['node'] == f'LOD{lod["level"]}' and nodes[names[lod['node']]].get('translation', [0, 0, 0]) == [0, 0, 0] and
                nodes[names[lod['node']]].get('rotation', [0, 0, 0, 1]) == [0, 0, 0, 1] and 'matrix' not in nodes[names[lod['node']]],
                f'LOD {lod["level"]}: expected named level with shared identity pivot')
        used = {nodes[i]['mesh'] for i in subtree if 'mesh' in nodes[i]}
        measured = sum(mesh_triangles[i] for i in used)
        require(measured > 0 and measured <= lod['maxTriangles'], f'LOD {lod["level"]}: triangle budget exceeded or empty')
        covered.update(subtree)
    context = {'manifest': manifest, 'document': doc, 'binary': binary, 'runtime': path, 'metrics': metrics}
    if manifest['collision']['policy'] == 'generated':
        generator = manifest['collision']['generator']
        callback = (collision_validators or {}).get(generator)
        require(callback is not None, f'collision.generator: unsupported {generator!r}')
        callback(context)
    for namespace, extension in manifest.get('extensions', {}).items():
        callback = (extension_validators or {}).get((manifest['category'], namespace, extension['version']))
        require(callback is not None, f'extensions.{namespace}: unsupported {manifest["category"]} version {extension["version"]}')
        callback(context, extension['data'])
    return metrics


def validate_manifest(manifest, runtime, repo=REPO, **registries):
    identity = manifest['assetId']
    try:
        source = contained_path(repo, manifest['source'], 'models/assets')
        contained_path(repo, manifest['runtime'], 'client/assets')
        require(source.is_file(), f'source: missing Blender file {source}')
        return validate_document(runtime, manifest, **registries)
    except (OSError, ValueError, KeyError, TypeError, IndexError, AttributeError, RecursionError, struct.error) as exc:
        raise ValidationError(f'{identity}: {exc}') from exc


def validate_asset(asset, repo=REPO, **registries):
    _, manifest = resolve_manifest(asset, repo)
    runtime = contained_path(repo, manifest['runtime'], 'client/assets')
    return validate_manifest(manifest, runtime, repo, **registries)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('asset', help='logical ID, asset directory, or manifest path')
    args = parser.parse_args()
    try:
        metrics = validate_asset(args.asset)
    except (OSError, ExportError) as exc:
        print(f'validation failed: {args.asset}: {exc}', file=sys.stderr)
        return 1
    print(json.dumps({'asset': args.asset, 'status': 'passed', 'metrics': metrics}, sort_keys=True))
    return 0


if __name__ == '__main__':
    sys.exit(main())
