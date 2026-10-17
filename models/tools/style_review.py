"""Advisory Option A measurements; generic validation owns hard contracts."""
import argparse
from collections import Counter
import json
import math
from pathlib import Path
import sys

from glb_costs import measure
from validate_asset import accessor_values, read_document

# Proposed screen-importance heuristics, not aesthetic acceptance gates.
PROFILES = {
    'ships': (6500, 13, .0001),
    'items': (3000, 4, .000001),
    'resources': (1200, 4, .00001),
    'props': (3000, 8, .00001),
    'structures': (12000, 16, .0001),
    'characters': (6000, 8, .000001),
    'vehicles': (6500, 13, .0001),
}
CODES = ('triangle-cost', 'material-cost', 'texture-cost', 'micro-geometry',
         'smooth-geology', 'missing-normals', 'lod-cost', 'mesh-reuse')


def report(document, binary, file_bytes, manifest):
    costs = measure(document, binary, file_bytes, manifest)
    profile = manifest.get('styleProfile')
    result = {'assetId': manifest['assetId'], 'profile': profile, 'costs': costs,
              'measurements': {}, 'warnings': [], 'exceptions': []}
    if profile is None:
        return result
    if profile['name'] != 'option-a-v1' or manifest['category'] not in PROFILES:
        raise ValueError('unsupported style profile/category')
    triangles, materials, micro_area = PROFILES[manifest['category']]
    reasons = {entry['code']: entry['reason'] for entry in profile.get('exceptions', [])}
    used_exceptions = set()

    def warn(code, **details):
        finding = {'code': code, **details}
        if code in reasons:
            finding['reason'] = reasons[code]
            used_exceptions.add(code)
            result['exceptions'].append(finding)
        else:
            result['warnings'].append(finding)

    for code, actual, threshold in (
        ('triangle-cost', costs['triangles'], triangles),
        ('material-cost', costs['materials'], materials),
        ('texture-cost', costs['textureMemoryBytes'], 1024*1024)):
        if actual > threshold:
            warn(code, actual=actual, advisoryMaximum=threshold)
    flat = smooth = missing = micro = valid = 0
    for mesh in document.get('meshes', []):
        for primitive in mesh['primitives']:
            points = accessor_values(document,binary,primitive['attributes']['POSITION'])
            normals = (accessor_values(document,binary,primitive['attributes']['NORMAL'])
                       if 'NORMAL' in primitive['attributes'] else None)
            ids = ([v[0] for v in accessor_values(document,binary,primitive['indices'])]
                   if 'indices' in primitive else list(range(len(points))))
            for start in range(0,len(ids),3):
                a,b,c = ids[start:start+3]
                u = [points[b][i]-points[a][i] for i in range(3)]
                v = [points[c][i]-points[a][i] for i in range(3)]
                n = (u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0])
                length = math.sqrt(sum(x*x for x in n))
                if length <= 1e-15:
                    continue  # topology failures belong to existing quality validator
                valid += 1
                micro += length/2 < micro_area
                if normals is None:
                    missing += 1
                elif all(sum(normals[j][i]*n[i]/length for i in range(3)) >= .999
                         for j in (a,b,c)):
                    flat += 1
                else:
                    smooth += 1
    refs = Counter(n['mesh'] for n in document.get('nodes',[]) if 'mesh' in n)
    # Exact duplicate mesh definitions are a conservative opportunity signal.
    signatures = []
    for mesh in document.get('meshes',[]):
        signatures.append(tuple((p.get('material'), tuple(
            (key,tuple(accessor_values(document,binary,index)))
            for key,index in sorted(p['attributes'].items())),
            tuple(accessor_values(document,binary,p['indices'])) if 'indices' in p else ())
            for p in mesh['primitives']))
    duplicates = len(signatures)-len(set(signatures))
    result['measurements'] = {'flatTriangles':flat, 'smoothTriangles':smooth,
        'missingNormalTriangles':missing, 'microTriangles':micro,
        'microAreaSquareMeters':micro_area, 'validTriangles':valid,
        'meshInstances':sum(refs.values()), 'referencedMeshes':len(refs),
        'reusedMeshReferences':sum(max(0,n-1) for n in refs.values()),
        'exactDuplicateMeshDefinitions':duplicates,
        'space':'mesh-local meters; instanced transforms/screen size not inferred'}
    if valid and micro/valid > .1:
        warn('micro-geometry', fraction=micro/valid)
    if missing:
        warn('missing-normals', triangles=missing)
    if manifest['category']=='resources' and smooth > flat:
        warn('smooth-geology', flatTriangles=flat, smoothTriangles=smooth)
    if duplicates:
        warn('mesh-reuse', duplicateDefinitions=duplicates)
    nodes = {n['name']:n for n in document['nodes']}
    def lod_triangles(name):
        node=nodes[name]
        children=[document['nodes'][i]['name'] for i in node.get('children',[])]
        own=0
        if 'mesh' in node:
            for p in document['meshes'][node['mesh']]['primitives']:
                own+=document['accessors'][p.get('indices',p['attributes']['POSITION'])]['count']//3
        return own+sum(lod_triangles(child) for child in children)
    levels=[lod_triangles(lod['node']) for lod in manifest.get('lods',[])]
    result['measurements']['lodTriangles']=levels
    if any(b>=a for a,b in zip(levels,levels[1:])):
        warn('lod-cost', triangles=levels)
    result['unusedExceptions']=sorted(set(reasons)-used_exceptions)
    return result


def compare(before, after):
    if before['assetId'] != after['assetId']:
        raise ValueError('comparison asset identity differs')
    return {'before':before,'after':after,'delta':{
        key: value-before['costs'][key] for key,value in after['costs'].items()
        if type(value) is int and type(before['costs'].get(key)) is int}}


def main():
    from export_asset import REPO, resolve_manifest
    from process_glb import registries
    from validate_asset import validate_manifest
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('asset')
    parser.add_argument('--before',type=Path,help='historical GLB; measured only, not current-contract validated')
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    try:
        _,manifest=resolve_manifest(args.asset,REPO)
        runtime=REPO/manifest['runtime']
        validate_manifest(manifest,runtime,REPO,**registries(manifest))
        after=report(*read_document(runtime),runtime.stat().st_size,manifest)
        result=after
        if args.before:
            before=report(*read_document(args.before),args.before.stat().st_size,manifest)
            result=compare(before,after)
            result['historicalContractValidation']='not performed; current contract may deliberately differ'
        destination=args.output.resolve()
        # Reports never overwrite tracked sources/assets or arbitrary input files.
        if not destination.is_relative_to(REPO/'artifacts'):
            raise ValueError('--output must be under artifacts/')
        destination.parent.mkdir(parents=True,exist_ok=True)
        destination.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    except (ValueError,OSError,KeyError) as exc:
        print(f'style review failed: {exc}',file=sys.stderr)
        return 1
    return 0


if __name__=='__main__':
    sys.exit(main())
