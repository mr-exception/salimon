"""Reference capture coverage must retain the same camera pairs and real power transition."""
import json
from pathlib import Path
import unittest
from model_visual_tests import expanded_scenario

ROOT = Path(__file__).resolve().parents[1]


class ScoutReferences(unittest.TestCase):
    def test_capture_pairs_and_offline_camera_inventory(self):
        cameras = json.loads((ROOT/'scenarios/visual/scout-cameras.json').read_text())
        names = {v['name'] for v in cameras}
        self.assertEqual(names, {'cockpit', 'core', 'fabricator', 'aft-door'})
        fixture = json.loads((ROOT/'scenarios/visual/scout-fixtures.json').read_text())['scout-lighting']
        scenario, images = expanded_scenario(fixture)
        self.assertEqual(len(images), len(names)*2*3)
        self.assertEqual(scenario['setup']['scenario'], 'core-restart')
        for name in names:
            for state in ('blackout', 'powered'):
                self.assertIn(f'{name}-{state}-repeat-00.png', images)
        transitions = [s['assert']['equals'] for s in scenario['steps']
                       if s.get('assert', {}).get('path') == 'ship.power_available']
        self.assertEqual(transitions, [False, True])
        self.assertEqual(scenario['steps'][-1]['action'], {'op':'scout_reference', 'view':'clear'})
