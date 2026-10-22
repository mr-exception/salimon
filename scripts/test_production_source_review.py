import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from production_source_review import verify_review


class ProductionReview(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.manifest = self.root / 'manifest.json'
        self.data = {'sources': [{'id': 'gaia:dr3', 'query': 'exact query'}]}
        self.manifest.write_text(json.dumps(self.data))
        self.path = self.root / 'review.json'
        self.source = dict(id='gaia:dr3', terms_url='https://example.test/terms',
                           terms_checked_on='2026-10-22', terms_evidence='archived terms hash/link',
                           credit_text='exact approved credit', references='release papers',
                           snapshot_scope='full_production_selection', acquisition='exact query',
                           epoch='J2016.0', frame='ICRS', snapshot_approval_evidence='review link',
                           decision='permission_obtained', permission_record='permission for stated use')
        self.review = dict(format='salimon.universe.production-review', schema_version=1,
                           intended_use='test use', distribution='test distribution', reviewer='test reviewer',
                           reviewed_on='2026-10-22', approval_evidence='synthetic test approval',
                           manifest_sha256=hashlib.sha256(self.manifest.read_bytes()).hexdigest(),
                           sources=[self.source])

    def verify(self):
        self.path.write_text(json.dumps(self.review))
        return verify_review(self.manifest, self.data, self.path)

    def test_exact_manifest_binding_and_complete_review(self):
        self.verify()
        self.manifest.write_text(self.manifest.read_text() + '\n')
        with self.assertRaisesRegex(ValueError, 'exact manifest'):
            self.verify()

    def test_pending_permissions_and_fixture_scope_never_promote(self):
        for field, value in [('decision', 'pending'), ('permission_record', ''),
                             ('snapshot_scope', 'fixture'), ('credit_text', ''),
                             ('terms_evidence', ''), ('snapshot_approval_evidence', '')]:
            previous = self.source[field]
            self.source[field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.verify()
            self.source[field] = previous

    def test_missing_duplicate_or_wrong_source_reviews_fail(self):
        for entries in [[], [self.source, self.source], [{**self.source, 'id': 'other:source'}]]:
            self.review['sources'] = entries
            with self.assertRaises(ValueError):
                self.verify()
        with self.assertRaisesRegex(ValueError, '--production-review'):
            verify_review(self.manifest, self.data, None)

    def test_staging_preserves_review_binding_and_rechecks_before_publish(self):
        from import_universe_sources import ROOT, stage
        stage(ROOT / 'catalogs/sources/manifest-v1.json', self.root / 'input')
        self.manifest = self.root / 'input/manifest-v1.json'
        self.data = json.loads(self.manifest.read_text())
        # Test-only approval for an existing observed fixture exercises successful
        # staging; no repository source decision or full-data approval is changed.
        self.data['sources'] = [self.data['sources'][1]]
        self.data['sources'][0]['license']['status'] = 'approved'
        self.manifest.write_text(json.dumps(self.data))
        self.review['manifest_sha256'] = hashlib.sha256(self.manifest.read_bytes()).hexdigest()
        self.source['id'] = self.data['sources'][0]['id']
        self.source['acquisition'] = self.data['sources'][0]['query']
        self.verify()
        stage(self.manifest, self.root / 'out', True, self.path)
        self.assertEqual((self.root / 'out/manifest-v1.json').read_bytes(), self.manifest.read_bytes())
        self.assertEqual((self.root / 'out/production-review.json').read_bytes(), self.path.read_bytes())
        with self.assertRaises(ValueError):
            stage(self.manifest, self.root / 'out', True, self.path)
