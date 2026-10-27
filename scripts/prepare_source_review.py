#!/usr/bin/env python3
"""Freeze a source-review packet without approving rights or production inputs."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import tempfile

from import_universe_sources import ROOT, stage, verify

REVIEW_ISSUE = 'https://github.com/mr-exception/salimon/issues/659'


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')


def prepare(manifest, output):
    if output.exists():
        raise ValueError(f'output exists; choose a new destination: {output}')
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = Path(tempfile.mkdtemp(prefix='source-review-', dir=output.parent))
    try:
        # Existing staging validates paths/hashes and rechecks the copied bytes.
        # Derive every review field from that frozen copy, never a later live read.
        stage(manifest, temporary / 'inputs')
        frozen = temporary / 'inputs/manifest-v1.json'
        data, _ = verify(frozen)
        digest = hashlib.sha256(frozen.read_bytes()).hexdigest()
        sources = [{
            'id': source['id'], 'release': source['release'],
            'endpoint': source['endpoint'], 'acquisition': source['query'],
            'documentation': source['documentation'],
            'license_status': source['license']['status'],
            'terms_url': source['license']['policy_url'],
            'historical_attribution_guidance': source['license']['attribution'],
            'coverage': source['coverage'], 'uncertainty': source['uncertainty'],
            'artifacts': source['artifacts'],
            'full_current_terms_verified': False,
            'full_production_selection_verified': False,
        } for source in data['sources']]
        inventory = {
            'format': 'salimon.universe.source-review-packet', 'schema_version': 1,
            'review_issue': REVIEW_ISSUE, 'production_eligible': False,
            'manifest': 'inputs/manifest-v1.json', 'manifest_sha256': digest,
            'manifest_metadata_date': data['reviewed_on'], 'sources': sources,
        }
        pending = {
            'format': 'salimon.universe.production-review', 'schema_version': 1,
            'manifest_sha256': digest,
            'intended_use': 'Private development only',
            'distribution': 'No distribution approved',
            'reviewer': None, 'reviewed_on': None, 'approval_evidence': None,
            'sources': [{
                'id': source['id'], 'terms_url': source['terms_url'],
                'terms_checked_on': None, 'terms_evidence': None,
                'credit_text': None, 'references': None, 'decision': 'pending',
                'permission_record': None,
                'snapshot_scope': 'pending_full_production_selection',
                'acquisition': source['acquisition'], 'epoch': None, 'frame': None,
                'snapshot_approval_evidence': None,
            } for source in sources],
        }
        write_json(temporary / 'inventory.json', inventory)
        write_json(temporary / 'production-review.pending.json', pending)
        (temporary / 'README.md').write_text(
            '# Pending source review\n\n'
            f'Current review owner: [#659]({REVIEW_ISSUE}).\n\n'
            f'Frozen manifest SHA-256: `{digest}`.\n\n'
            'Inspect `inventory.json` and the exact raw bytes in `inputs/`. '
            'Attribution guidance and the manifest metadata date are historical; '
            'they are not a current terms review. No network requests are made.\n\n'
            '`production-review.pending.json` is deliberately incomplete and '
            'must fail the production validator. Do not submit it as approval. '
            'It records the existing private-development scope only.\n\n'
            'A human reviewer must inspect/archive full current provider terms, '
            'record approved use/distribution, exact credits/references and any '
            'required permission. Select and acquire the actual complete production '
            'inputs, freeze query/release/epoch/frame, sizes and hashes, then regenerate '
            'this packet against that new manifest before signing its exact digest. '
            'The development fixtures cannot become a full selection by filling '
            'out a form or changing status/origin flags.\n', encoding='utf-8')
        # Guard the exact input binding again before publishing the packet.
        verify(frozen)
        if hashlib.sha256(frozen.read_bytes()).hexdigest() != digest:
            raise ValueError('frozen manifest changed during review preparation')
        temporary.rename(output)
        return inventory
    finally:
        if temporary.exists():
            shutil.rmtree(temporary)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, default=ROOT / 'catalogs/sources/manifest-v1.json')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    try:
        result = prepare(args.manifest, args.output)
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(1, f'source-review preparation failed: {error}\n')
    print(json.dumps({'sources': len(result['sources']), 'manifest_sha256': result['manifest_sha256'],
                      'production_eligible': False, 'output': str(args.output)}))


if __name__ == '__main__':
    main()
