#!/usr/bin/env python3
"""Export Blender scout visuals, preserving legacy metadata until #81–#83."""
import argparse
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile

ASSET = Path(__file__).resolve().parent
REPO = ASSET.parents[3]
sys.path.insert(0, str(REPO / 'models/tools'))
from export_asset import ExportError, contained_path, resolve_manifest
from validate_asset import read_document, require, validate_manifest


def encode_glb(document, binary):
    data = json.dumps(document, separators=(',', ':')).encode()
    data += b' ' * (-len(data) % 4)
    binary += b'\0' * (-len(binary) % 4)
    return (struct.pack('<4sII', b'glTF', 2, 28 + len(data) + len(binary)) +
            struct.pack('<II', len(data), 0x4E4F534A) + data +
            struct.pack('<II', len(binary), 0x004E4942) + binary)


def validate_preservation(context, data):
    """Keep runtime-significant hierarchy, transforms, extras and material roles."""
    baseline = json.loads((ASSET / 'preservation.json').read_text())
    doc = context['document']
    nodes = {n['name']: n for n in doc['nodes']}
    parents = {doc['nodes'][child]['name']: n['name'] for n in doc['nodes']
               for child in n.get('children', [])}
    require(nodes.keys() == {n['name'] for n in baseline['nodes']}, 'scout node set changed')
    for expected in baseline['nodes']:
        name = expected['name']
        actual = nodes[name]
        require(parents.get(name) == expected['parent'], f'{name}: parent changed')
        require(('mesh' in actual) == expected['visual'], f'{name}: visual role changed')
        require(actual.get('extras', {}) == expected.get('extras', {}), f'{name}: legacy extras changed')
        require(all(abs(a-b) <= 1e-6 for a, b in zip(actual.get('translation', [0, 0, 0]),
                                                    expected.get('translation', [0, 0, 0]))),
                f'{name}: spatial transform changed')
        require(actual.get('rotation', [0, 0, 0, 1]) == [0, 0, 0, 1] and
                actual.get('scale', [1, 1, 1]) == [1, 1, 1] and 'matrix' not in actual,
                f'{name}: legacy baked transform required')
    materials = {m['name']: m for m in doc['materials']}
    for expected in baseline['materials']:
        actual = materials[expected['name']]
        for key, default in (('alphaMode', 'OPAQUE'), ('doubleSided', False)):
            require(actual.get(key, default) == expected.get(key, default),
                    f'{expected["name"]}: {key} changed')
        for key, default in (('baseColorFactor', [1, 1, 1, 1]),
                             ('metallicFactor', 1), ('roughnessFactor', 1)):
            a = actual.get('pbrMetallicRoughness', {}).get(key, default)
            b = expected.get('pbrMetallicRoughness', {}).get(key, default)
            a, b = (a, b) if isinstance(a, list) else ([a], [b])
            require(all(abs(x-y) < 1e-6 for x, y in zip(a, b)), f'{expected["name"]}: {key} changed')
        require(all(abs(x-y) < 1e-6 for x, y in zip(actual.get('emissiveFactor', [0, 0, 0]),
                                                  expected.get('emissiveFactor', [0, 0, 0]))),
                f'{expected["name"]}: emission changed')


def export(blender=None):
    manifest_path, manifest = resolve_manifest('ship.salimon-scout', REPO)
    source = contained_path(REPO, manifest['source'], 'models/assets')
    output = contained_path(REPO, manifest['runtime'], 'client/assets')
    executable = shutil.which(blender or os.environ.get('BLENDER', 'blender'))
    if executable is None:
        raise ExportError('Blender not found; pass --blender or BLENDER')
    metadata = json.loads((ASSET / 'runtime-metadata.json').read_text())
    with tempfile.TemporaryDirectory(prefix='.scout-export-', dir=output.parent) as directory:
        staged = Path(directory)
        temporary = staged / output.name
        subprocess.run([executable, '--background', '--factory-startup', '--disable-autoexec',
                        str(source), '--python-exit-code', '1', '--python',
                        str(REPO / 'models/tools/blender_export.py'), '--',
                        str(manifest_path), str(temporary)], cwd=REPO, check=True)
        doc, binary = read_document(temporary)
        # Geometry/materials stay entirely Blender-owned. Only legacy nonvisual
        # asset metadata is carried forward; no generator is invoked here.
        doc['extras'] = copy.deepcopy(metadata['extras'])
        doc['asset']['extras'] = copy.deepcopy(metadata['assetExtras'])
        doc['asset']['extras']['salimon']['sourceWorkflow'] = 'Blender source + scout export adapter'
        doc['asset']['copyright'] = metadata['copyright']
        temporary.write_bytes(encode_glb(doc, binary))
        metrics = validate_manifest(manifest, temporary, REPO, extension_validators={
            ('ships', 'ship', 1): validate_preservation,
        })
        interchange = copy.deepcopy(doc)
        interchange['buffers'][0]['uri'] = output.with_suffix('.bin').name
        (staged / output.with_suffix('.gltf').name).write_text(json.dumps(interchange, indent=2) + '\n')
        (staged / output.with_suffix('.bin').name).write_bytes(binary)
        # Until #82 the old validator still owns detailed sightline/monitor and
        # generated-layout checks. Run those against the complete staged set.
        legacy_dir = REPO / 'client/assets/ship/source'
        sys.path.insert(0, str(legacy_dir))
        spec = importlib.util.spec_from_file_location('scout_legacy_validator', legacy_dir / 'validate_salimon_phase0_ship.py')
        validator = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(validator)
        validator.main(staged)
        report = dict(assetId=manifest['assetId'], sourceSha256=hashlib.sha256(source.read_bytes()).hexdigest(),
                      runtimeSha256=hashlib.sha256(temporary.read_bytes()).hexdigest(), metrics=metrics)
        # Every check completes before any destination is replaced. GLB is
        # replaced last; normal client builds consume only that file.
        for suffix in ('.bin', '.gltf', '.glb'):
            destination = output.with_suffix(suffix)
            os.replace(staged / destination.name, destination)
        (ASSET / 'export-report.json').write_text(json.dumps(report, indent=2) + '\n')
        return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--blender')
    args = parser.parse_args()
    try:
        print(json.dumps(export(args.blender), sort_keys=True))
    except (ExportError, OSError, subprocess.CalledProcessError, AssertionError) as exc:
        print(f'scout export failed: {exc}', file=sys.stderr)
        sys.exit(1)
