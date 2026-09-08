"""Source repair geometry and production regression witnesses for #244."""
import json
from pathlib import Path
import sys
import unittest

TOOLS = Path(__file__).resolve().parents[1] / 'tools'
sys.path.insert(0, str(TOOLS))
from surface_partition import area, subtract
from geometry_quality import diagnose
from validate_asset import REPO, read_document


def panel(points):
    return [(p, p, (p[0], p[1], 0)) for p in points]


class SurfacePartitionTests(unittest.TestCase):
    def test_shared_edge_is_unchanged(self):
        original = panel([(0, 0), (1, 0), (0, 1)])
        self.assertEqual(subtract(original, [(1, 0), (1, 1), (0, 1)]), [original])

    def test_partial_cut_preserves_area_orientation_uv_and_position(self):
        original = panel([(0, 0), (2, 0), (0, 2)])
        parts = subtract(original, [(0, 0), (1, 0), (0, 1)])
        self.assertAlmostEqual(sum(area([v[0] for v in part]) for part in parts), 1.5)
        for part in parts:
            self.assertGreater(area([v[0] for v in part]), 0)
            for xy, uv, xyz in part:
                self.assertEqual(xy, uv)
                self.assertEqual(xy, xyz[:2])
                self.assertEqual(xyz[2], 0)
        self.assertEqual(subtract(original, [(0, 0), (0, 2), (2, 0)]), [])

    def test_production_repairs_and_retained_contacts(self):
        removed = {
            'ships/salimon-scout': {
                ('Aft_Bulkhead_Header', 'Aft_Bulkhead_Header'),
                ('Cockpit_Monitor_Bezels', 'Cockpit_Monitor_Housings'),
                ('Cockpit_Lower_Glazing', 'Cockpit_Window_Frame'),
                ('Cockpit_Lower_Glazing', 'Hull_Nose'),
                ('Engine_Port_Cooling_Fins', 'Engine_Port_Running_Light'),
                ('Engine_Starboard_Cooling_Fins', 'Engine_Starboard_Running_Light'),
                ('Wall_Port_Inner', 'Window_Port_Sill'),
                ('Wall_Starboard_Inner', 'Window_Starboard_Sill'),
            },
            'items/cargo-container': set(),
            'items/mining-tool': set(),
        }
        for asset, forbidden in removed.items():
            with self.subTest(asset=asset):
                manifest = json.loads((REPO/'models/assets'/asset/'manifest.json').read_text())
                policy = manifest['quality']
                self.assertNotIn('degenerate', policy.get('severity', {}))
                self.assertTrue(all('Temporary' not in e['reason'] for e in policy['overlapExemptions']))
                result = diagnose(*read_document(REPO/manifest['runtime']), policy)
                self.assertEqual(result['status'], 'passed')
                self.assertFalse(any(f['code']=='degenerate' for f in result['findings']))
                pairs = {tuple(sorted((f['node'], f['otherNode']))) for f in result['findings'] if 'otherNode' in f}
                self.assertFalse(pairs & forbidden, pairs & forbidden)
