#!/usr/bin/env python3
"""Verify and stage pinned source bytes offline; science conversion is U07/U08."""
import argparse
import hashlib
import json
import shutil
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f'duplicate JSON field: {key}')
        result[key] = value
    return result


def load(path):
    if path.stat().st_size > 1024 * 1024:
        raise ValueError('manifest exceeds 1 MiB')
    return json.loads(path.read_text(encoding='utf-8'), object_pairs_hook=unique_object)


def verify(manifest, production=False, production_review=None):
    data = load(manifest)
    if set(data) != {'format', 'schema_version', 'reviewed_on', 'sources'} or data['format'] != 'salimon.universe.sources' or data['schema_version'] != 1:
        raise ValueError('unsupported source manifest')
    if not data['sources']:
        raise ValueError('empty sources')
    ids, paths, entries = set(), set(), []
    root = manifest.parent.resolve()
    for source in data['sources']:
        if set(source) != {'id', 'release', 'endpoint', 'query', 'documentation', 'license', 'coverage', 'uncertainty', 'artifacts'}:
            raise ValueError('unknown/missing source fields')
        if source['id'] in ids or not source['artifacts']:
            raise ValueError('duplicate source or empty artifacts')
        ids.add(source['id'])
        if source['license']['status'] not in {'restricted', 'unreviewed', 'approved'}:
            raise ValueError('unknown reuse status')
        if production and source['license']['status'] != 'approved':
            raise ValueError(f"{source['id']}: production reuse not approved; see {source['license']['review_issue']}")
        for artifact in source['artifacts']:
            if set(artifact) != {'path', 'sha256', 'bytes', 'origin', 'encoding'}:
                raise ValueError('unknown/missing artifact fields')
            name = artifact['path']
            if name in {'manifest-v1.json', 'production-review.json'} or name in paths or not name or '\\' in name or ':' in name or name.startswith('/') or any(p in {'', '.', '..'} for p in name.split('/')):
                raise ValueError(f'unsafe/duplicate artifact path: {name}')
            paths.add(name)
            path = (root / name).resolve()
            if not path.is_relative_to(root):
                raise ValueError(f'artifact symlink escapes source root: {name}')
            if artifact['origin'] not in {'synthetic', 'observed_snapshot'} or artifact['encoding'] not in {'csv', 'json'}:
                raise ValueError('unknown artifact origin/encoding')
            if production and artifact['origin'] == 'synthetic':
                raise ValueError('synthetic fixture is not production data')
            digest, size = hashlib.sha256(), 0
            with path.open('rb') as stream:
                for chunk in iter(lambda: stream.read(1024 * 1024), b''):
                    digest.update(chunk)
                    size += len(chunk)
            if size != artifact['bytes'] or digest.hexdigest() != artifact['sha256']:
                raise ValueError(f'{name}: checksum/size mismatch')
            entries.append((path, artifact))
    if production:
        from production_source_review import verify_review
        verify_review(manifest, data, production_review)
    return data, entries


def stage(manifest, output, production=False, production_review=None):
    data, entries = verify(manifest, production, production_review)
    if output.exists():
        raise ValueError(f'output exists; choose a new destination: {output}')
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = Path(tempfile.mkdtemp(prefix='universe-sources-', dir=output.parent))
    try:
        for path, artifact in entries:
            dest = temporary / artifact['path']
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, dest)
        # Reverify staged bytes: copying concurrent modified input must fail.
        frozen = temporary / 'manifest-v1.json'
        frozen.write_bytes(manifest.read_bytes())
        frozen_review = None
        if production:
            frozen_review = temporary / 'production-review.json'
            shutil.copyfile(production_review, frozen_review)
        verify(frozen, production, frozen_review)
        temporary.rename(output)
    finally:
        if temporary.exists():
            shutil.rmtree(temporary)
    return len(entries)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, default=ROOT / 'catalogs/sources/manifest-v1.json')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--production', action='store_true', help='requires reviewed reuse and forbids synthetic fixtures')
    parser.add_argument('--production-review', type=Path,
                        help='human use/terms/full-snapshot review bound to the exact manifest digest')
    args = parser.parse_args()
    try:
        if args.production_review and not args.production:
            raise ValueError('--production-review requires --production')
        count = stage(args.manifest, args.output, args.production, args.production_review)
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(1, f'source import failed: {error}\n')
    print(f'Verified and staged {count} source artifacts; no network or scientific conversion.')


if __name__ == '__main__':
    main()
