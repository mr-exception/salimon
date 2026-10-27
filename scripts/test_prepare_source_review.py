import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from import_universe_sources import ROOT, stage, verify
from prepare_source_review import prepare
from production_source_review import verify_review


class SourceReviewPacketTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        stage(ROOT / 'catalogs/sources/manifest-v1.json', self.root / 'source')
        self.manifest = self.root / 'source/manifest-v1.json'

    def test_packet_binds_and_freezes_exact_bytes_without_promoting_sources(self):
        output = self.root / 'review'
        inventory = prepare(self.manifest, output)
        self.assertFalse(inventory['production_eligible'])
        self.assertEqual(inventory['manifest_sha256'], hashlib.sha256(self.manifest.read_bytes()).hexdigest())
        frozen = output / inventory['manifest']
        data, _ = verify(frozen)
        for source in data['sources']:
            for artifact in source['artifacts']:
                original = self.manifest.parent / artifact['path']
                self.assertEqual((frozen.parent / artifact['path']).read_bytes(), original.read_bytes())
        self.assertEqual([s['license_status'] for s in inventory['sources']],
                         ['restricted', 'unreviewed', 'unreviewed'])
        pending = output / 'production-review.pending.json'
        with self.assertRaisesRegex(ValueError, 'reviewer'):
            verify_review(frozen, data, pending)
        with self.assertRaisesRegex(ValueError, 'not approved'):
            verify(frozen, True, pending)
        # Changing the live source after preparation cannot mutate frozen review bytes.
        self.manifest.write_text('{}')
        self.assertEqual(inventory['manifest_sha256'], hashlib.sha256(frozen.read_bytes()).hexdigest())

    def test_corrupt_input_never_publishes_a_partial_packet(self):
        artifact = self.manifest.parent / 'fixtures/gaia-synthetic.csv'
        artifact.write_bytes(b'corrupted')
        with self.assertRaisesRegex(ValueError, 'checksum/size mismatch'):
            prepare(self.manifest, self.root / 'review')
        self.assertFalse((self.root / 'review').exists())
        self.assertFalse(list(self.root.glob('source-review-*')))

    def test_existing_review_is_preserved_and_generation_is_deterministic(self):
        prepare(self.manifest, self.root / 'a')
        before = (self.root / 'a/inventory.json').read_bytes()
        with self.assertRaisesRegex(ValueError, 'output exists'):
            prepare(self.manifest, self.root / 'a')
        self.assertEqual((self.root / 'a/inventory.json').read_bytes(), before)
        prepare(self.manifest, self.root / 'b')
        for name in ['inventory.json', 'production-review.pending.json', 'README.md']:
            self.assertEqual((self.root / 'a' / name).read_bytes(), (self.root / 'b' / name).read_bytes())
        pending = json.loads((self.root / 'a/production-review.pending.json').read_text())
        self.assertTrue(all(s['decision'] == 'pending' for s in pending['sources']))


if __name__ == '__main__':
    unittest.main()
