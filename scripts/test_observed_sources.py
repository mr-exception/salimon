import json
import hashlib
from pathlib import Path
import tempfile
import unittest
import zlib

from acquire_observed_sources import freeze, queries, TARGETS, count_query
from verify_observed_sources import catalog_records, read_blob, number_equal

ROOT = Path(__file__).resolve().parents[1]
SNAPSHOT = ROOT / 'catalogs/sources/observed-526-v1'


class ObservedSources(unittest.TestCase):
    def test_raw_and_v2_compressed_blobs_return_identical_original_bytes(self):
        raw = b'{"missing_error":null,"source_id":"5853498713190525696"}' * 100
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'blobs').mkdir()
            for stored, extra in [(raw, {}), (zlib.compress(raw), dict(zlib_raw_bytes=len(raw)))]:
                sha = hashlib.sha256(stored).hexdigest()
                (root / 'blobs' / sha).write_bytes(stored)
                self.assertEqual(read_blob(root, dict(sha256=sha, bytes=len(stored), **extra)), raw)

    def test_rehashed_truncated_trailing_or_false_length_zlib_blobs_fail_closed(self):
        raw = b'original source fields and uncertainties' * 100
        compressed = zlib.compress(raw)
        cases = [(compressed[:-1], len(raw)), (compressed + b'x', len(raw)),
                 (compressed + compressed, len(raw)), (compressed, len(raw) - 1),
                 (compressed, len(raw) + 1), (compressed, 256 * 1024 * 1024 + 1)]
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'blobs').mkdir()
            for stored, size in cases:
                sha = hashlib.sha256(stored).hexdigest()
                (root / 'blobs' / sha).write_bytes(stored)
                with self.assertRaises((ValueError, zlib.error)):
                    read_blob(root, dict(sha256=sha, bytes=len(stored), zlib_raw_bytes=size))

    def test_full_query_and_exact_target_freeze(self):
        ps, jpl = queries()
        self.assertIn('default_flag%3D1+order+by+hostname%2Cpl_name', ps)
        self.assertNotIn('top+', ps.lower())
        self.assertEqual(len(jpl), len(TARGETS))
        paths = {count_query(): 'ps-count.csv', ps: 'ps.csv', **{url: f'jpl-{code}.json' for code, url in jpl.items()}}
        with tempfile.TemporaryDirectory() as temp:
            output = Path(temp) / 'snapshot'
            freeze(output, lambda url: (SNAPSHOT / paths[url]).read_bytes())
            request = json.loads((output / 'build.json').read_text())
            for h in request['cells'][0]['observed']['horizons']:
                self.assertEqual(h['target'], 'jpl:' + h['artifact'][4:-5])
            self.assertEqual((output / 'ps.csv').read_bytes(), (SNAPSHOT / 'ps.csv').read_bytes())
            with self.assertRaises(FileExistsError):
                freeze(output, lambda _: b'')

    def test_service_error_truncated_and_nondefault_snapshots_fail(self):
        for data in [b'', b'pl_name,default_flag\nX,0\n', b'{"error":"unavailable"}', b'x' * (8 * 1024 * 1024 + 1)]:
            with self.subTest(data=data[:50]), tempfile.TemporaryDirectory() as temp:
                with self.assertRaises(ValueError):
                    freeze(Path(temp) / 'snapshot', lambda _: data)

    def test_corrupt_blob_bad_digest_and_nonfinite_comparisons_reject(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            with self.assertRaises(ValueError):
                read_blob(root, dict(sha256='../escape', bytes=1))
            (root / 'blobs').mkdir()
            (root / 'blobs' / ('a' * 64)).write_bytes(b'corrupt')
            with self.assertRaises(ValueError):
                read_blob(root, dict(sha256='a' * 64, bytes=7))
        for value in [float('nan'), float('inf'), 1.001]:
            with self.assertRaises(ValueError):
                number_equal(value, 1., 'test corruption')
        with self.assertRaises(ValueError):
            catalog_records(b'not a catalog')

    def test_failed_acquisition_preserves_bytes_without_completed_manifest(self):
        # Acquisition must not rewrite raw scientific values to make import pass.
        ps, jpl = queries()
        paths = {count_query(): 'ps-count.csv', ps: 'ps.csv', **{url: f'jpl-{code}.json' for code, url in jpl.items()}}
        original = (SNAPSHOT / 'ps.csv').read_bytes()
        # A receiver error after the PS freeze retains its original bytes and
        # never exposes a completed manifest/build request.
        with tempfile.TemporaryDirectory() as temp:
            output = Path(temp) / 'snapshot'
            def fail_after_ps(url):
                if url == count_query():
                    return (SNAPSHOT / 'ps-count.csv').read_bytes()
                if url == ps:
                    return original
                raise OSError('service failed')
            with self.assertRaises(OSError):
                freeze(output, fail_after_ps)
            self.assertEqual((output / 'ps.csv').read_bytes(), original)
            self.assertFalse((output / 'manifest.json').exists())
            self.assertFalse((output / 'build.json').exists())
