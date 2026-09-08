#!/usr/bin/env python3
"""Optional offline compliance, exact byte optimization and budget reports."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile

from export_asset import REPO, contained_path, resolve_manifest
from glb_costs import measure, view_bytes
from validate_asset import read_document, validate_manifest

PROFILE = 'exact-v1'


def encode_glb(document, binary):
    data = json.dumps(document, separators=(',', ':'), sort_keys=True, allow_nan=False).encode()
    data += b' ' * (-len(data) % 4)
    binary += b'\0' * (-len(binary) % 4)
    return (struct.pack('<4sII', b'glTF', 2, 28 + len(data) + len(binary)) +
            struct.pack('<II', len(data), 0x4E4F534A) + data +
            struct.pack('<II', len(binary), 0x004E4942) + binary)


def exact_repack(document, binary):
    """Alias identical complete view payloads; retain all indices and view metadata.

    No graph rewrite, compression, quantization, material merge or vertex reorder.
    Four-byte alignment is sufficient for all supported ordinary glTF accessors.
    """
    if document.get('extensionsUsed') or document.get('extensionsRequired'):
        raise ValueError('exact-v1 does not rewrite extension-bearing assets')
    result = copy.deepcopy(document)
    packed = bytearray()
    offsets = {}
    for view in result.get('bufferViews', []):
        payload = view_bytes(document, binary, view)
        if payload not in offsets:
            packed.extend(b'\0' * (-len(packed) % 4))
            offsets[payload] = len(packed)
            packed.extend(payload)
        view['byteOffset'] = offsets[payload]
    result['buffers'][0]['byteLength'] = len(packed)
    return result, bytes(packed)


def semantic_snapshot(document, binary):
    """Preserve all JSON except storage offsets/length, plus every view's exact bytes.

    Includes all names, transforms, hierarchy, extras, material channels, accessor
    layout, indices, UVs and animation data. No whitelist can silently drop them.
    """
    result = copy.deepcopy(document)
    payloads = []
    for view in result.get('bufferViews', []):
        payloads.append(hashlib.sha256(view_bytes(document, binary, view)).hexdigest())
        view.pop('byteOffset', None)
    for buffer in result.get('buffers', []):
        buffer.pop('byteLength', None)
    return result, payloads


def compliance(path):
    try:
        result = subprocess.run(['node', str(REPO / 'models/tools/gltf/validate.cjs'), str(path)],
                                capture_output=True, text=True, timeout=120, check=True)
    except (OSError, subprocess.SubprocessError) as exc:
        raise ValueError(f'Khronos validator failed; run npm ci --prefix models/tools/gltf: {exc}') from exc
    report = json.loads(result.stdout)
    if report['issues']['truncated']:
        raise ValueError('Khronos validation report truncated')
    return report


def registries(manifest):
    if manifest['assetId'] != 'ship.salimon-scout':
        return {}
    # The registered ship adapter preserves monitor, collision, pivot and spatial
    # invariants on staged candidates as well as checked-in runtime files.
    import importlib.util
    path = REPO / 'models/assets/ships/salimon-scout/validate.py'
    sys.path.insert(0, str(path.parent))
    spec = importlib.util.spec_from_file_location('pipeline_scout_validation', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return {'extension_validators': module.REGISTRIES}


def process(manifest, source, destination, profile='none', repo=REPO, **validators):
    """Stage only: caller owns publication after success. No changes to source."""
    if source.resolve() == destination.resolve():
        raise ValueError('candidate must not overwrite source')
    if profile not in ('none', PROFILE):
        raise ValueError(f'unknown optimization profile {profile}')
    doc, binary = read_document(source)
    before = measure(doc, binary, source.stat().st_size, manifest)
    before_compliance = compliance(source)
    if before_compliance['issues']['numErrors']:
        raise ValueError(f'{manifest["assetId"]}: input Khronos compliance errors: '
                         + json.dumps(before_compliance['issues'], sort_keys=True))
    validate_manifest(manifest, source, repo, **validators)
    destination.parent.mkdir(parents=True, exist_ok=True)
    # Write final candidate only after both compliance and all contract gates pass.
    with tempfile.TemporaryDirectory(prefix='.glb-process-', dir=destination.parent) as directory:
        staged = Path(directory) / 'candidate.glb'
        if profile == PROFILE:
            candidate = encode_glb(*exact_repack(doc, binary))
            # A size regression is never accepted, even if canonical JSON grew.
            staged.write_bytes(candidate if len(candidate) < source.stat().st_size else source.read_bytes())
        else:
            staged.write_bytes(source.read_bytes())
        after_doc, after_bin = read_document(staged)
        if semantic_snapshot(doc, binary) != semantic_snapshot(after_doc, after_bin):
            raise ValueError('optimization changed asset semantics')
        after_compliance = compliance(staged)
        if after_compliance['issues']['numErrors']:
            raise ValueError('candidate failed Khronos compliance: ' + json.dumps(after_compliance['issues']))
        if after_compliance['issues']['numWarnings'] > before_compliance['issues']['numWarnings']:
            raise ValueError('optimization introduced compliance warnings')
        contract_metrics = validate_manifest(manifest, staged, repo, **validators)
        after = measure(after_doc, after_bin, staged.stat().st_size, manifest)
        report = dict(schemaVersion=1, assetId=manifest['assetId'], profile=profile,
                      visualStyle=manifest.get('visualBudget', {}).get('style', 'unspecified'),
                      budgets=manifest['budgets'], visualBudget=manifest.get('visualBudget'),
                      before=before, after=after,
                      delta={k: after[k] - v for k, v in before.items() if isinstance(v, int)},
                      sourceSha256=hashlib.sha256(source.read_bytes()).hexdigest(),
                      candidateSha256=hashlib.sha256(staged.read_bytes()).hexdigest(),
                      semanticEquality=True, contractMetrics=contract_metrics,
                      complianceBefore=before_compliance, complianceAfter=after_compliance)
        if 'styleProfile' in manifest:
            from style_review import report as style_report
            report['styleReview'] = {
                'before': style_report(doc, binary, source.stat().st_size, manifest),
                'after': style_report(after_doc, after_bin, staged.stat().st_size, manifest)}
        staged.replace(destination)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('asset', help='asset ID or all')
    parser.add_argument('--profile', choices=['none', PROFILE], default='none')
    parser.add_argument('--output', type=Path, required=True, help='staged candidates and JSON reports')
    args = parser.parse_args()
    try:
        assets = [json.loads(p.read_text())['assetId'] for p in sorted((REPO / 'models/assets').rglob('manifest.json'))] if args.asset == 'all' else [args.asset]
        reports = []
        for asset in assets:
            _, manifest = resolve_manifest(asset, REPO)
            source = contained_path(REPO, manifest['runtime'], 'client/assets')
            candidate = args.output / manifest['assetId'] / 'model.glb'
            # The offline CLI cannot publish over any runtime asset directory.
            if candidate.resolve().is_relative_to((REPO / 'client/assets').resolve()):
                raise ValueError('--output must be outside client/assets')
            report = process(manifest, source, candidate, args.profile, **registries(manifest))
            (candidate.parent / 'report.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
            reports.append(report)
            print(f'{asset}: {report["before"]["glbBytes"]} -> {report["after"]["glbBytes"]} bytes')
        args.output.mkdir(parents=True, exist_ok=True)
        (args.output / 'summary.json').write_text(json.dumps(reports, indent=2, sort_keys=True) + '\n')
    except (ValueError, OSError, KeyError) as exc:
        print(f'GLB processing failed: {exc}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
