import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from import_universe_sources import ROOT, stage, verify


class OfflineSourceImport(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.manifest = ROOT / 'catalogs/sources/manifest-v1.json'

    def test_offline_import_is_byte_reproducible_and_never_overwrites(self):
        with patch('socket.socket', side_effect=AssertionError('network forbidden')):
            for n in ['a', 'b']:
                self.assertEqual(stage(self.manifest, self.root / n), 3)
            for p in (self.root / 'a').rglob('*'):
                if p.is_file():
                    self.assertEqual(p.read_bytes(), (self.root / 'b' / p.relative_to(self.root / 'a')).read_bytes())
            with self.assertRaises(ValueError):
                stage(self.manifest, self.root / 'a')

    def test_tampered_bytes_and_unsafe_paths_fail_before_publication(self):
        stage(self.manifest, self.root / 'source')
        manifest = self.root / 'source/manifest-v1.json'
        data = json.loads(manifest.read_text())
        path = manifest.parent / data['sources'][0]['artifacts'][0]['path']
        path.write_bytes(b'bad')
        with self.assertRaisesRegex(ValueError, 'checksum'):
            stage(manifest, self.root / 'out')
        self.assertFalse((self.root / 'out').exists())
        data['sources'][0]['artifacts'][0]['path'] = '../secret'
        manifest.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, 'unsafe'):
            verify(manifest)

    def test_production_requires_reuse_review_and_actual_source_snapshots(self):
        with self.assertRaisesRegex(ValueError, 'reuse not approved'):
            stage(self.manifest, self.root / 'out', True)
        stage(self.manifest, self.root / 'source')
        manifest = self.root / 'source/manifest-v1.json'
        data = json.loads(manifest.read_text())
        for s in data['sources']:
            s['license']['status'] = 'approved'
        manifest.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, 'synthetic'):
            stage(manifest, self.root / 'out', True)

    def test_manifest_unknown_schema_and_duplicate_json_fields_reject(self):
        p = self.root / 'manifest.json'
        p.write_text('{"schema_version":1,"schema_version":2}')
        with self.assertRaisesRegex(ValueError, 'duplicate JSON'):
            verify(p)
        data = json.loads(self.manifest.read_text())
        data['schema_version'] = 2
        p.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, 'unsupported'):
            verify(p)


if __name__ == '__main__':
    unittest.main()
