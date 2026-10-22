"""Validate human production-use evidence bound to one exact source manifest.

This validates a review record, not the truth or legal sufficiency of its claims.
"""
import hashlib
from datetime import date
from pathlib import Path

from import_universe_sources import load


def required_text(record, keys):
    if not isinstance(record, dict):
        raise ValueError('production review entry must be an object')
    for key in keys:
        if not isinstance(record.get(key), str) or not record[key].strip():
            raise ValueError(f'production review requires {key}')


def verify_review(manifest, data, review_path):
    if review_path is None:
        raise ValueError('production requires --production-review; status flags alone are not approval')
    review = load(review_path)
    if review.get('format') != 'salimon.universe.production-review' or review.get('schema_version') != 1:
        raise ValueError('unsupported production review')
    required_text(review, ('intended_use', 'distribution', 'reviewer', 'reviewed_on', 'approval_evidence'))
    date.fromisoformat(review['reviewed_on'])
    if review.get('manifest_sha256') != hashlib.sha256(manifest.read_bytes()).hexdigest():
        raise ValueError('production review does not bind this exact manifest')
    entries = review.get('sources')
    if not isinstance(entries, list) or len(entries) != len(data['sources']):
        raise ValueError('production review must cover every source exactly once')
    by_id = {}
    for source in entries:
        required_text(source, ('id', 'terms_url', 'terms_checked_on', 'terms_evidence',
                               'credit_text', 'references', 'snapshot_scope', 'acquisition',
                               'epoch', 'frame', 'snapshot_approval_evidence'))
        date.fromisoformat(source['terms_checked_on'])
        if not source['terms_url'].startswith('https://'):
            raise ValueError('production terms URL must use HTTPS')
        if source['id'] in by_id:
            raise ValueError('duplicate production review source')
        if source.get('decision') not in ('terms_permit_intended_use', 'permission_obtained'):
            raise ValueError('source-use decision remains pending or restricted')
        if source['decision'] == 'permission_obtained':
            required_text(source, ('permission_record',))
        if source['snapshot_scope'] != 'full_production_selection':
            raise ValueError('fixture snapshots are not full production selections')
        # Evidence identifies exact archived terms, reviewed scope and query. The
        # manifest digest binds the actual artifact sizes/hashes/origins as well.
        by_id[source['id']] = source
    if set(by_id) != {source['id'] for source in data['sources']}:
        raise ValueError('production review source identities differ from manifest')
    for source in data['sources']:
        if by_id[source['id']]['acquisition'] != source['query']:
            raise ValueError('review acquisition differs from manifest query')
    return review
