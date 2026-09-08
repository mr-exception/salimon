"""Flight acceptance rejects missing routes, changed contracts and fake captures."""
import copy
import json
from pathlib import Path
import tempfile
import unittest

from salimon_test import parse_scenario
from scenario_suites import UNIVERSE_FLIGHT
from planetary_environment import ROOT, review_native
from PIL import Image


class FlightEvidenceTests(unittest.TestCase):
    def test_complete_matrix_and_evidence_failures(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            image = root / 'original.png'
            Image.new('RGB', (20, 10), (40, 60, 80)).save(image)
            summary = {'status': 'passed', 'results': []}
            for index, name in enumerate(UNIVERSE_FLIGHT):
                route = root / str(index)
                route.mkdir()
                contract = parse_scenario(ROOT / 'scenarios/evidence' / name)
                (route / 'scenario.json').write_text(json.dumps(contract))
                source = {'status': 'passed', 'steps': [
                    {'status': 'passed', 'step': step,
                     'screenshot': {'status': 'captured', 'source': 'GPU color surface', 'path': str(image)},
                     'state': {'visual_context': {'viewport_pixels': [20, 10]},
                               'performance': {'samples': 1, 'raw_frames': [[1, 2, 3, None, 5, 6]]}}}
                    for step in contract['steps']]}
                (route / 'result.json').write_text(json.dumps(source))
                summary['results'].append({'result_file': str(route / 'result.json')})
            self.assertEqual(len(review_native(summary, UNIVERSE_FLIGHT)), 9)
            incomplete = copy.deepcopy(summary)
            incomplete['results'].pop()
            with self.assertRaisesRegex(ValueError, 'incomplete'):
                review_native(incomplete, UNIVERSE_FLIGHT)
            first = Path(summary['results'][0]['result_file'])
            original = json.loads(first.read_text())
            for mutation in ('assertion', 'truncated', 'capture', 'metrics'):
                changed = copy.deepcopy(original)
                if mutation == 'assertion':
                    changed['steps'][0]['status'] = 'failed'
                elif mutation == 'truncated':
                    changed['steps'].pop()
                else:
                    step = next(s for s in changed['steps'] if 'screenshot' in s['step'])
                    if mutation == 'capture':
                        step['screenshot']['source'] = 'CPU fixture'
                    else:
                        step['state']['performance']['raw_frames'] = []
                first.write_text(json.dumps(changed))
                with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                    review_native(summary, UNIVERSE_FLIGHT)
