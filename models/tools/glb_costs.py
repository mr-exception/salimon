"""Deterministic storage/visual cost estimates, separate from spatial policy."""
import struct


def view_bytes(document, binary, view):
    start = view.get('byteOffset', 0)
    return binary[start:start + view['byteLength']]


def measure(document, binary, file_bytes, manifest):
    meshes = document.get('meshes', [])
    primitives = [p for m in meshes for p in m['primitives']]
    triangles = sum(document['accessors'][p['indices'] if 'indices' in p else
                    p['attributes']['POSITION']]['count'] // 3 for p in primitives)
    image_bytes = 0
    texture_memory = 0
    for image in document.get('images', []):
        payload = view_bytes(document, binary, document['bufferViews'][image['bufferView']])
        image_bytes += len(payload)
        if image.get('mimeType') != 'image/png' or payload[:8] != b'\x89PNG\r\n\x1a\n' or len(payload) < 24:
            raise ValueError('texture memory estimate supports embedded PNG only')
        width, height = struct.unpack_from('>II', payload, 16)
        if not width or not height:
            raise ValueError('invalid PNG dimensions')
        # RGBA8, full mip chain, per image; no sampler/driver allocation overhead.
        while True:
            texture_memory += width * height * 4
            if width == height == 1:
                break
            width, height = max(1, width // 2), max(1, height // 2)
    materials = document.get('materials', [])
    if manifest['assetId'] == 'ship.salimon-scout':
        used = [document['meshes'][n['mesh']]['primitives'] for n in document['nodes'] if 'mesh' in n]
        glass = [materials[p['material']].get('name') == 'Cockpit Glass' for ps in used for p in ps]
        draws, policy = int(any(glass)) + int(any(not x for x in glass)), 'ship opaque + glass batches'
    elif manifest['category'] == 'resources':
        draws, policy = 1, 'one indexed instanced draw per visible variant'
    elif manifest['assetId'] == 'item.mining-tool':
        draws, policy = 1, 'one held-item batch'
    else:
        draws, policy = None, 'no runtime consumer; primitive references are not actual draws'
    return dict(triangles=triangles, nodes=len(document.get('nodes', [])),
                primitives=len(primitives), materials=len(materials),
                encodedTextureBytes=image_bytes, textureMemoryBytes=texture_memory,
                glbBytes=file_bytes, drawCalls=draws, drawPolicy=policy,
                primitiveReferences=sum(len(meshes[n['mesh']]['primitives'])
                                        for n in document.get('nodes', []) if 'mesh' in n))


def check_visual_budget(metrics, manifest):
    profile = manifest.get('visualBudget')
    if not profile:
        return
    for limit, metric in (('maxNodes', 'nodes'), ('maxTextureMemoryBytes', 'textureMemoryBytes'),
                          ('maxDrawCalls', 'drawCalls')):
        if limit in profile:
            value = metrics[metric]
            if value is None or value > profile[limit]:
                raise ValueError(f'visualBudget.{limit}: measured {value} exceeds/unsupported {profile[limit]}')
