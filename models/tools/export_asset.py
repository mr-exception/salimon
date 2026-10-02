#!/usr/bin/env python3
"""Opt-in, manifest-driven Blender export; never invoked by Cargo."""
import argparse
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile

REPO = Path(__file__).resolve().parents[2]


class ExportError(ValueError):
    pass


def read_manifest(path, repo):
    try:
        from jsonschema import Draft202012Validator
    except ImportError as exc:
        raise ExportError('install models/tools/requirements.txt in your authoring Python environment') from exc
    try:
        manifest = json.loads(path.read_text())
        schema = json.loads((repo / 'models/asset-manifest.schema.json').read_text())
        errors = list(Draft202012Validator(schema).iter_errors(manifest))
        if errors:
            error = errors[0]
            raise ExportError(f'{path}: {".".join(map(str, error.absolute_path))}: {error.message}')
        return manifest
    except (OSError, json.JSONDecodeError) as exc:
        raise ExportError(f'cannot read manifest {path}: {exc}') from exc


def resolve_manifest(asset, repo):
    base = (repo / 'models/assets').resolve()
    candidate = Path(asset)
    if not candidate.is_absolute():
        candidate = repo / candidate
    if candidate.is_dir():
        candidate /= 'manifest.json'
    explicit = candidate.is_file()
    if explicit and not candidate.resolve().is_relative_to(base):
        raise ExportError('manifest must be under models/assets (examples are not assets)')
    matches = []
    identities, destinations = set(), set()
    paths = set(base.rglob('manifest.json'))
    if explicit:
        paths.add(candidate.resolve())
    for path in sorted(paths):
        if not path.resolve().is_relative_to(base):
            raise ExportError(f'manifest escapes models/assets: {path}')
        manifest = read_manifest(path, repo)
        for value, seen, label in ((manifest['assetId'], identities, 'assetId'),
                                   (manifest['runtime'], destinations, 'runtime destination')):
            if value in seen:
                raise ExportError(f'duplicate {label}: {value}')
            seen.add(value)
        if (explicit and path.resolve() == candidate.resolve()) or (not explicit and manifest['assetId'] == asset):
            matches.append((path.resolve(), manifest))
    if not matches:
        raise ExportError(f'no asset manifest for {asset!r} under models/assets')
    return matches[0]


def contained_path(repo, value, area):
    path = (repo / value).resolve()
    base = (repo / area).resolve()
    if not base.is_relative_to(repo.resolve()) or not path.is_relative_to(base):
        raise ExportError(f'path escapes {area}: {value}')
    return path


def read_glb(path):
    data = path.read_bytes()
    if len(data) < 20:
        raise ExportError('Blender did not produce a GLB')
    magic, version, size = struct.unpack_from('<4sII', data)
    length, kind = struct.unpack_from('<II', data, 12)
    if magic != b'glTF' or version != 2 or size != len(data) or kind != 0x4E4F534A or 20 + length > size:
        raise ExportError('Blender produced an invalid GLB header/JSON chunk')
    try:
        return json.loads(data[20:20 + length])
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise ExportError(f'invalid GLB JSON: {exc}') from exc


def check_export(path, manifest):
    """Verify export preservation; semantic/budget checks are layered on by validate_asset."""
    document = read_glb(path)
    nodes = {}
    for node in document.get('nodes', []):
        name = node.get('name')
        if name in nodes:
            raise ExportError(f'duplicate exported node: {name}')
        nodes[name] = node
    contracts = manifest['contracts']
    required = {contracts['root']}
    for key in ('visualGroups', 'groups', 'nodes', 'sockets', 'markers', 'colliders'):
        required.update(contracts[key])
    required.update(lod['node'] for lod in manifest.get('lods', []))
    missing = required - nodes.keys()
    if missing:
        raise ExportError(f'missing exported nodes: {sorted(missing)}')
    materials = {m.get('name') for m in document.get('materials', [])}
    if set(contracts['materials']) - materials:
        raise ExportError('missing exported material names')
    for extra in contracts['requiredExtras']:
        value = nodes.get(extra['node'], {}).get('extras', {})
        for key in extra['path'].split('.'):
            if not isinstance(value, dict) or key not in value:
                raise ExportError(f'missing exported extras: {extra["node"]}.{extra["path"]}')
            value = value[key]
    if any('uri' in buffer for buffer in document.get('buffers', [])) or any('uri' in image for image in document.get('images', [])):
        raise ExportError('runtime GLB must embed buffers and images')


def export_asset(asset, blender=None, repo=REPO, **registries):
    manifest_path, manifest = resolve_manifest(asset, repo)
    identity = manifest['assetId']
    try:
        source = contained_path(repo, manifest['source'], 'models/assets')
        output = contained_path(repo, manifest['runtime'], 'client/assets')
        if not source.is_file():
            raise ExportError(f'missing Blender source: {source}')
        executable = shutil.which(blender or os.environ.get('BLENDER', 'blender'))
        if not executable:
            raise ExportError('Blender executable not found; use --blender or BLENDER')
        output.parent.mkdir(parents=True, exist_ok=True)
        # Work beside the destination: os.replace is atomic and failures keep the old GLB.
        with tempfile.TemporaryDirectory(prefix='.salimon-export-', dir=output.parent) as directory:
            temporary = Path(directory) / 'model.glb'
            command = [executable, '--background', '--factory-startup', '--disable-autoexec',
                       str(source), '--python-exit-code', '1', '--python',
                       str(REPO / 'models/tools/blender_export.py'), '--',
                       str(manifest_path), str(temporary)]
            result = subprocess.run(command, cwd=repo, check=False)
            if result.returncode:
                raise ExportError(f'Blender failed with exit code {result.returncode}')
            if not temporary.is_file():
                raise ExportError('Blender finished without producing the runtime GLB')
            from validate_asset import validate_manifest
            validate_manifest(manifest, temporary, repo, **registries)
            os.replace(temporary, output)
        return output
    except (OSError, ExportError) as exc:
        raise ExportError(f'{identity}: {exc}') from exc


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('asset', help='logical asset ID, asset directory, or manifest path under models/assets')
    parser.add_argument('--blender', help='Blender executable (defaults to BLENDER or PATH)')
    args = parser.parse_args()
    try:
        output = export_asset(args.asset, args.blender)
    except ExportError as exc:
        print(f'export failed: {exc}', file=sys.stderr)
        return 1
    print(f'Exported {output.relative_to(REPO)}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
