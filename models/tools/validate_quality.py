#!/usr/bin/env python3
"""Emit complete deterministic quality JSON even when the policy fails."""
import argparse
import json
import sys

from export_asset import REPO, resolve_manifest
from validate_asset import read_document, validate_asset, ValidationError
from geometry_quality import diagnose


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('asset')
    args = parser.parse_args()
    try:
        _, manifest = resolve_manifest(args.asset, REPO)
        # Structural/accessor safety validation first, without quality/category dispatch.
        from validate_asset import validate_document
        shared = {k: v for k, v in manifest.items() if k not in {'quality', 'extensions'}}
        validate_document(REPO / manifest['runtime'], shared)
        document, binary = read_document(REPO / manifest['runtime'])
        result = diagnose(document, binary, manifest.get('quality', {}))
        result['asset'] = manifest['assetId']
        print(json.dumps(result, indent=2, sort_keys=True))
        return int(result['status'] == 'failed')
    except (ValueError, OSError, KeyError) as error:
        print(json.dumps({'asset': args.asset, 'status': 'failed', 'error': str(error)}, sort_keys=True))
        return 1


if __name__ == '__main__':
    sys.exit(main())
