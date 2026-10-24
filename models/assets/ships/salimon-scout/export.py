#!/usr/bin/env python3
"""Export Blender scout visuals, preserving runtime metadata and validating authored contracts."""
import argparse
import copy
import hashlib
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
sys.path.insert(0, str(ASSET))
sys.path.insert(0, str(REPO / 'models/tools'))
from export_asset import ExportError, contained_path, resolve_manifest
from validate_asset import read_document, validate_manifest
from validate import REGISTRIES, validate_preservation
from habitation_geometry import rust_source as habitation_rust_source
from spatial_contracts import (
    build_spatial_contracts,
    landing_rust_source,
    ramp_rust_source,
    sidecar_json,
    spatial_rust_source,
)


def authoring_hashes(asset=ASSET):
    """Use portable manifest-style keys even when exporting on Windows."""
    return {p.relative_to(asset).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(asset.rglob('*.blend'))}


def encode_glb(document, binary):
    data = json.dumps(document, separators=(',', ':')).encode()
    data += b' ' * (-len(data) % 4)
    binary += b'\0' * (-len(binary) % 4)
    return (struct.pack('<4sII', b'glTF', 2, 28 + len(data) + len(binary)) +
            struct.pack('<II', len(data), 0x4E4F534A) + data +
            struct.pack('<II', len(binary), 0x004E4942) + binary)

def export(blender=None, glb_profile=None):
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
        preservation = json.loads((ASSET / 'preservation.json').read_text())
        spatial = build_spatial_contracts(doc, preservation, binary)
        support = spatial["landingSupport"]
        # Geometry/materials stay entirely Blender-owned. Only legacy nonvisual
        # asset metadata is carried forward; no generator is invoked here.
        doc['extras'] = copy.deepcopy(metadata['extras'])
        doc['extras']['salimon']['spatialContracts'] = copy.deepcopy(spatial)
        doc['asset']['extras'] = copy.deepcopy(metadata['assetExtras'])
        doc['asset']['extras']['salimon']['sourceWorkflow'] = 'Blender source + scout export adapter'
        doc['asset']['copyright'] = metadata['copyright']
        temporary.write_bytes(encode_glb(doc, binary))
        processing = None
        if glb_profile is not None:
            from process_glb import process
            optimized = staged / 'processed.glb'
            processing = process(manifest, temporary, optimized, glb_profile,
                                 extension_validators=REGISTRIES)
            optimized.replace(temporary)
            doc, binary = read_document(temporary)
        metrics = validate_manifest(manifest, temporary, REPO, extension_validators=REGISTRIES)
        interchange = copy.deepcopy(doc)
        interchange['buffers'][0]['uri'] = output.with_suffix('.bin').name
        (staged / output.with_suffix('.gltf').name).write_text(json.dumps(interchange, indent=2) + '\n')
        (staged / output.with_suffix('.bin').name).write_bytes(binary)
        (staged / 'spatial-contracts.json').write_text(sidecar_json(spatial))
        (staged / 'spatial_contracts.rs').write_text(spatial_rust_source(spatial))
        report = dict(assetId=manifest['assetId'], sourceSha256=hashlib.sha256(source.read_bytes()).hexdigest(),
                      runtimeSha256=hashlib.sha256(temporary.read_bytes()).hexdigest(), metrics=metrics,
                      authoringSha256=authoring_hashes())
        if processing is not None:
            report['processing'] = processing
        # Every check completes before any destination is replaced. GLB is
        # replaced last; normal client builds consume only that file.
        os.replace(staged / 'spatial-contracts.json', REPO / 'client/assets/ship/spatial-contracts.json')
        os.replace(staged / 'spatial_contracts.rs', REPO / 'client/character/src/spatial_contracts.rs')
        for suffix in ('.bin', '.gltf', '.glb'):
            destination = output.with_suffix(suffix)
            os.replace(staged / destination.name, destination)
        (REPO / 'client/ship/src/landing_geometry.rs').write_text(landing_rust_source(support))
        for crate in ('character', 'renderer'):
            (REPO / f'client/{crate}/src/habitation_geometry.rs').write_text(habitation_rust_source())
            (REPO / f'client/{crate}/src/ramp_geometry.rs').write_text(ramp_rust_source(spatial['boardingRamp']))
        (ASSET / 'export-report.json').write_text(json.dumps(report, indent=2) + '\n')
        return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--blender')
    parser.add_argument('--glb-profile', choices=['none', 'exact-v1'], default=None)
    args = parser.parse_args()
    try:
        print(json.dumps(export(args.blender, args.glb_profile), sort_keys=True))
    except (ExportError, OSError, subprocess.CalledProcessError, AssertionError, ValueError) as exc:
        print(f'scout export failed: {exc}', file=sys.stderr)
        sys.exit(1)
