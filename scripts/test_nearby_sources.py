"""Complete regional selection, exact IDs and resumable acquisition boundaries."""
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
import urllib.parse
import gzip
import hashlib
import json
from xml.sax.saxutils import escape
from acquire_gaia_sources import COLUMNS
from acquire_nearby_sources import freeze_gaia, prepare
from verify_nearby_sources import verify


def table(columns, rows):
    root = Path(__file__).resolve().parents[1]
    metadata = json.loads((root / 'catalogs/sources/gaia-524-v1/selection.json').read_text())['acquisitions'][0]['fields']
    attributes = {f['name']: f for f in metadata}
    fields = ''.join('<FIELD ' + ' '.join(k + '="' + escape(v, {'"': '&quot;'}) + '"' for k, v in attributes.get(c, dict(name=c)).items()) + '/>' for c in columns)
    data = ''.join('<TR>' + ''.join('<TD>' + escape(str(row.get(c, ''))) + '</TD>' for c in columns) + '</TR>' for row in rows)
    return (f'<VOTABLE><RESOURCE><INFO name="QUERY_STATUS" value="OK"/><TABLE>{fields}'
            f'<DATA><TABLEDATA>{data}</TABLEDATA></DATA></TABLE></RESOURCE></VOTABLE>').encode()


def server(url):
    query = urllib.parse.parse_qs(urllib.parse.urlparse(url).query)['QUERY'][0]
    if 'COUNT(*)' in query:
        return table(['n'], [dict(n=3)])
    if query.startswith('SELECT TOP 1 source_id '):
        return table(['source_id'], [])
    after = int(query.split('source_id > ')[1].split(')')[0]) if 'source_id > ' in query else 0
    ids = [i for i in [101, 102, 103] if i > after][:2]
    return table(COLUMNS, [dict(source_id=str(i), parallax='10.000000000000001', ref_epoch='2016.0') for i in ids])


class NearbySourcesTests(unittest.TestCase):
    def test_future_region_threshold_is_not_rounded_to_six_significant_digits(self):
        boundary = 10.123456789012345
        def precise(url):
            query = urllib.parse.parse_qs(urllib.parse.urlparse(url).query)['QUERY'][0]
            literal = query.split('WHERE parallax >= ')[1].split()[0]
            self.assertEqual(float(literal), boundary)
            return server(url).replace(b'10.000000000000001', b'20.0')
        with TemporaryDirectory() as tmp:
            result = freeze_gaia(Path(tmp) / 'snapshot', parallax_min_mas=boundary, batch_rows=2, fetcher=precise)
            self.assertEqual(result['scope']['parallax_min_mas'], boundary)

    def test_complete_keyset_region_is_not_a_row_limited_sample(self):
        with TemporaryDirectory() as tmp:
            output = Path(tmp) / 'snapshot'
            result = freeze_gaia(output, batch_rows=2, fetcher=server)
            self.assertTrue(result['selected_query_complete'])
            self.assertEqual(result['expected_rows'], 3)
            self.assertEqual([p['rows'] for p in result['pages']], [2, 1])
            self.assertEqual(result['scope']['nominal_parsecs'], 100)
            self.assertFalse(result['full_release_complete'])
            self.assertTrue((output / 'selection.json').exists())
            self.assertEqual(len(result['artifacts']), 7)

    def test_resume_reuses_verified_pages_and_preserves_exact_completion(self):
        with TemporaryDirectory() as tmp:
            output = Path(tmp) / 'snapshot'
            def interrupted(url):
                if 'source_id+%3E+102' in url:
                    raise TimeoutError('interrupted after durable first page')
                return server(url)
            with self.assertRaises(TimeoutError):
                freeze_gaia(output, batch_rows=2, fetcher=interrupted)
            self.assertTrue((output / 'progress.json').exists())
            self.assertFalse((output / 'selection.json').exists())
            result = freeze_gaia(output, batch_rows=2, resume=True, fetcher=server)
            self.assertEqual(sum(p['rows'] for p in result['pages']), 3)

    def test_budget_bad_count_changed_scope_and_partial_pages_fail_closed(self):
        for bad in ('budget', 'count', 'outside', 'duplicate', 'partial', 'overflow'):
            with self.subTest(bad=bad), TemporaryDirectory() as tmp:
                output = Path(tmp) / 'snapshot'
                def broken(url):
                    raw = server(url)
                    if bad == 'count' and 'COUNT' in url:
                        return table(['n'], [dict(n='not-a-count')])
                    if 'COUNT' not in url and not 'SELECT+TOP+1+source_id' in url:
                        if bad == 'outside':
                            raw = raw.replace(b'10.000000000000001', b'9.999999999999999')
                        elif bad == 'duplicate':
                            raw = raw.replace(b'<TD>102</TD>', b'<TD>101</TD>')
                        elif bad == 'partial':
                            return table(COLUMNS, [])
                        elif bad == 'overflow':
                            raw = raw.replace(b'value="OK"', b'value="OVERFLOW"')
                    return raw
                with self.assertRaises(ValueError):
                    freeze_gaia(output, batch_rows=2, budget=1 if bad == 'budget' else 1_000_000_000, fetcher=broken)
                self.assertFalse((output / 'selection.json').exists())

    def test_tampered_checkpoint_sources_and_parameter_changes_reject_resume(self):
        with TemporaryDirectory() as tmp:
            output = Path(tmp) / 'snapshot'
            def interrupted(url):
                if 'source_id+%3E+102' in url:
                    raise TimeoutError('interrupt')
                return server(url)
            with self.assertRaises(TimeoutError):
                freeze_gaia(output, batch_rows=2, fetcher=interrupted)
            with self.assertRaises(ValueError):
                freeze_gaia(output, parallax_min_mas=11, batch_rows=2, resume=True, fetcher=server)
            (output / 'gaia/0000.csv.gz').write_bytes(b'changed')
            with self.assertRaises(ValueError):
                freeze_gaia(output, batch_rows=2, resume=True, fetcher=server)


class NearbyVerificationTests(unittest.TestCase):
    def snapshot(self, output):
        def scientific_fixture(url):
            query = urllib.parse.parse_qs(urllib.parse.urlparse(url).query)['QUERY'][0]
            if 'COUNT(*)' in query:
                columns, data = ['n'], [dict(n=3)]
            elif query.startswith('SELECT TOP 1 source_id '):
                columns, data = ['source_id'], []
            else:
                columns = COLUMNS
                if 'source_id > ' not in query:
                    data = [dict(source_id='5853498713190525696', parallax='768.0665391873573'),
                            dict(source_id='4472832130942575872', parallax='546.975939730948')]
                else:
                    data = [dict(source_id='1000000000000000001', parallax='10.000000000000001')]
                data = [dict(r, ref_epoch='2016.0', parallax_error='0.05', ra='0', dec='0') for r in data]
            raw = table(columns, data)
            info = '<INFO name="QUERY" value="' + escape(query, {'"': '&quot;'}) + '"/>'
            return raw.replace(b'<TABLE>', info.encode() + b'<TABLE>')
        freeze_gaia(output, batch_rows=2, fetcher=scientific_fixture)
        prepare(output)

    def test_independent_originals_counts_exact_host_links_and_source_hashes(self):
        with TemporaryDirectory() as tmp:
            output = Path(tmp) / 'snapshot'
            self.snapshot(output)
            result = verify(output)
            self.assertEqual(result['status'], 'passed')
            self.assertEqual(result['gaia_input_rows'], 3)
            self.assertEqual(result['solar_system_targets'], 18)
            self.assertEqual(len(result['exact_golden_associations']), 2)
            self.assertFalse(result['full_release_complete'])

    def test_rehashed_but_changed_transcription_is_rejected_against_original_votable(self):
        with TemporaryDirectory() as tmp:
            output = Path(tmp) / 'snapshot'
            self.snapshot(output)
            name = 'gaia/0000.csv.gz'
            raw = gzip.decompress((output / name).read_bytes()).replace(b'768.0665391873573', b'767.9')
            changed = gzip.compress(raw, mtime=0)
            (output / name).write_bytes(changed)
            manifest = json.loads((output / 'manifest.json').read_text())
            for artifact in manifest['sources'][0]['artifacts']:
                if artifact['path'] == name:
                    artifact.update(bytes=len(changed), sha256=hashlib.sha256(changed).hexdigest())
            (output / 'manifest.json').write_text(json.dumps(manifest))
            with self.assertRaisesRegex(ValueError, 'transcription'):
                verify(output)

    def test_widening_scope_metadata_does_not_invent_downloaded_coverage(self):
        with TemporaryDirectory() as tmp:
            output = Path(tmp) / 'snapshot'
            self.snapshot(output)
            selection = json.loads((output / 'selection.json').read_text())
            selection['parameters']['parallax_min_mas'] = 5
            selection['scope'].update(parallax_min_mas=5, nominal_parsecs=200)
            (output / 'selection.json').write_text(json.dumps(selection))
            with self.assertRaisesRegex(ValueError, 'documented region'):
                verify(output)


if __name__ == '__main__':
    unittest.main()
