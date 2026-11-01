"""Scientific-source boundary regressions; no network access."""
from pathlib import Path
import unittest
from acquire_gaia_sources import decode
from verify_gaia_sources import verify

ROOT = Path(__file__).resolve().parents[1]


class GaiaSourcesTests(unittest.TestCase):
    def test_real_frozen_transcription_and_independent_exact_host_associations(self):
        result = verify(ROOT / 'catalogs/sources/gaia-524-v1', ROOT / 'catalogs/sources/observed-526-v1/ps.csv')
        self.assertEqual(result['input_rows'], 8192)
        self.assertEqual(len(result['exact_golden_cross_matches']), 2)
        self.assertFalse(result['full_release_complete'])

    def test_overflow_after_success_and_error_status_fail_closed(self):
        for status in ('OVERFLOW', 'ERROR'):
            xml = f'<VOTABLE><RESOURCE><INFO name="QUERY_STATUS" value="OK"/><TABLE><FIELD name="source_id"/><DATA><TABLEDATA><TR><TD>5853498713190525696</TD></TR></TABLEDATA></DATA></TABLE><INFO name="QUERY_STATUS" value="{status}"/></RESOURCE></VOTABLE>'
            with self.assertRaises(ValueError):
                decode(xml.encode())

    def test_exact_large_integer_missing_columns_and_units_are_not_reformatted(self):
        xml = b'<VOTABLE><RESOURCE><INFO name="QUERY_STATUS" value="OK"/><TABLE><FIELD name="source_id" datatype="long"/><FIELD name="parallax" unit="mas"/><DATA><TABLEDATA><TR><TD>5853498713190525696</TD><TD/></TR></TABLEDATA></DATA></TABLE></RESOURCE></VOTABLE>'
        fields, rows = decode(xml)
        self.assertEqual(rows, [{'source_id': '5853498713190525696', 'parallax': ''}])
        self.assertEqual(fields[1]['unit'], 'mas')

    def test_duplicate_fields_and_wrong_row_width_are_rejected(self):
        xml = '<VOTABLE><RESOURCE><INFO name="QUERY_STATUS" value="OK"/><TABLE>{}<DATA><TABLEDATA><TR><TD>1</TD></TR></TABLEDATA></DATA></TABLE></RESOURCE></VOTABLE>'
        for fields in ('<FIELD name="source_id"/><FIELD name="source_id"/>', '<FIELD name="source_id"/><FIELD name="ra"/>'):
            with self.assertRaises(ValueError):
                decode(xml.format(fields).encode())


if __name__ == '__main__':
    unittest.main()
