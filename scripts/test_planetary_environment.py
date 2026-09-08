"""The combined matrix must fail on incomplete, forged or unavailable evidence."""
import copy
import json
from pathlib import Path
import tempfile
import unittest

from planetary_environment import ROOT, distribution, review_native
from scenario_suites import PLANETARY_ENVIRONMENT
from salimon_test import parse_scenario

try:
    from PIL import Image
except ImportError:
    Image = None


class DistributionTests(unittest.TestCase):
    def test_empty_gpu_samples_and_spikes_are_preserved(self):
        self.assertIsNone(distribution([]))
        self.assertEqual(distribution([2, 8])["median"], 5)
        self.assertEqual(distribution([1] * 19 + [400]),
                         {"count": 20, "median": 1, "p95": 1, "max": 400})
        for value in [float('nan'), float('inf'), -1, True]:
            with self.assertRaises(ValueError):
                distribution([value])


@unittest.skipIf(Image is None, "requires Pillow")
class MatrixTests(unittest.TestCase):
    def test_missing_routes_failed_steps_and_forged_capture_reject(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            image = root / 'capture.png'
            Image.new('RGB', (20, 10), (40, 60, 80)).save(image)
            summary = {"status": "passed", "results": []}
            for index, name in enumerate(PLANETARY_ENVIRONMENT):
                route = root / str(index)
                route.mkdir()
                scenario = parse_scenario(ROOT / 'scenarios/evidence' / name)
                (route / 'scenario.json').write_text(json.dumps(scenario))
                source = {"status": "passed", "steps": [{
                    "status": "passed", "step": {"screenshot": "fixture"},
                    "screenshot": {"status": "captured", "source": "GPU color surface", "path": str(image)},
                    "state": {"visual_context": {"viewport_pixels": [20, 10]},
                              "performance": {"samples": 1, "raw_frames": [[1, 2, 3, None, 5, 6]]}}}]}
                template = source["steps"][0]
                source["steps"] = [{**copy.deepcopy(template), "step": step} for step in scenario["steps"]]
                (route / 'result.json').write_text(json.dumps(source))
                summary['results'].append({'result_file': str(route / 'result.json')})
            self.assertEqual(len(review_native(summary)), len(PLANETARY_ENVIRONMENT))
            incomplete = copy.deepcopy(summary)
            incomplete['results'].pop()
            with self.assertRaisesRegex(ValueError, 'incomplete'):
                review_native(incomplete)
            first = Path(summary['results'][0]['result_file'])
            original = json.loads(first.read_text())
            for mutation in ['status', 'step', 'capture', 'source', 'raw', 'dimensions']:
                altered = copy.deepcopy(original)
                step = next(s for s in altered['steps'] if 'screenshot' in s['step'])
                if mutation == 'status':
                    altered['status'] = 'failed'
                elif mutation == 'step':
                    step['status'] = 'failed'
                elif mutation == 'capture':
                    step['screenshot']['status'] = 'unavailable'
                elif mutation == 'source':
                    step['screenshot']['source'] = 'OS desktop'
                elif mutation == 'raw':
                    step['state']['performance']['raw_frames'] = []
                else:
                    step['state']['visual_context']['viewport_pixels'] = [1, 1]
                first.write_text(json.dumps(altered))
                with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                    review_native(summary)


class AudioReviewTests(unittest.TestCase):
    def test_vacuum_leak_and_reversed_transmission_fail(self):
        import wave
        import struct
        from planetary_environment import review_audio
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            def clip(name, value, frames=96000):
                with wave.open(str(root / (name + '.wav')), 'wb') as output:
                    output.setparams((2, 2, 48000, frames, 'NONE', 'not compressed'))
                    output.writeframes(struct.pack('<hh', value, -value) * frames)
            for name, value in [('vacuum-quiet', 0), ('suit-local-weather', 30),
                                ('scout-ramp-open', 100), ('scout-ramp-closed', 10)]:
                clip(name, value)
            self.assertEqual(review_audio(root)['rms_pcm16']['vacuum-quiet'], 0)
            clip('vacuum-quiet', 1)
            with self.assertRaises(ValueError):
                review_audio(root)
            clip('vacuum-quiet', 0)
            clip('scout-ramp-closed', 200)
            with self.assertRaises(ValueError):
                review_audio(root)
            clip('scout-ramp-closed', 10, frames=10)
            with self.assertRaises(ValueError):
                review_audio(root)
