import copy
import json
from pathlib import Path
import tempfile
import subprocess
import sys
import unittest
from unittest.mock import patch

from visual_corpus import candidate, reviewed_status, write_json
from model_visual_tests import capture, evaluate
try:
    from PIL import Image, ImageDraw
except ImportError:
    Image = None


@unittest.skipIf(Image is None, 'optional visual tests require Pillow')
class CorpusTests(unittest.TestCase):
    def test_capture_diagnostics_do_not_contaminate_corpus_inventory(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fixtures, context = self.fixture(root)

            def native_run(scenario_path, binary, artifacts, tags):
                artifacts.mkdir(parents=True, exist_ok=True)
                scenario = json.loads(scenario_path.read_text())
                steps = []
                for step in scenario['steps']:
                    image = artifacts/(step['screenshot'] + '.png')
                    Image.new('RGB', (32, 32), 'black').save(image)
                    steps.append({'step': step, 'screenshot': {'path': str(image)},
                                  'state': {'visual_context': context}})
                return {'status': 'passed', 'steps': steps}

            with patch('salimon_test.run', side_effect=native_run):
                result = capture(fixtures['ship'], root/'binary', root/'fresh/ship')
            metadata = candidate(root/'fresh', fixtures, result['visual_context'], 'a'*40, 'b'*64)
            self.assertEqual(len(metadata['images']), 2)
            self.assertEqual(len(list((root/'fresh-runs/ship').glob('*.png'))), 2)
            self.assertFalse((root/'fresh/ship/runs').exists())
            # The fix must not weaken rejection of genuinely stray corpus PNGs.
            Image.new('RGB', (32, 32)).save(root/'fresh/stray.png')
            with self.assertRaisesRegex(ValueError, 'unexpected image outside fixture contract'):
                candidate(root/'fresh', fixtures, result['visual_context'], 'a'*40, 'b'*64)

    def fixture(self, root):
        scenario = root/'scenario.json'
        scenario.write_text(json.dumps({'setup': {'seed': 42}, 'steps': [{'screenshot': 'pose'}]}))
        fixtures = {'ship': {'scenario': str(scenario), 'mode': 'moving', 'repeat': 2,
                             'tolerances': {'channel_tolerance': 8, 'changed_fraction': 0.001, 'masks': []}}}
        context = dict(backend='Vulkan', adapter='test software', driver='test-driver-v1',
                       platform='test-linux', viewport_pixels=[32, 32], scale_factor=1.0)
        for parent in ('captures', 'baseline'):
            (root/parent/'ship').mkdir(parents=True)
            for index in range(2):
                image = Image.new('RGB', (32, 32), 'black')
                ImageDraw.Draw(image).rectangle((8, 8, 23, 23), fill=(120, 120, 120))
                image.save(root/parent/'ship'/f'pose-repeat-{index:02}.png')
            meta = candidate(root/parent, fixtures, context, 'a'*40, 'b'*64)
            if parent == 'baseline':
                meta.update(status='approved', review={'reviewer': 'test reviewer',
                    'reviewed_at': '2026-10-18', 'evidence': 'test fixture review',
                    'masks_and_tolerances_reviewed': True})
            write_json(root/parent/'corpus.json', meta)
        return fixtures, context

    def test_pending_missing_and_unapproved_are_not_appearance_passes(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fixtures, _ = self.fixture(root)
            self.assertEqual(reviewed_status(root/'captures', root/'absent', fixtures)['status'], 'missing')
            for status in ('pending', 'candidate'):
                write_json(root/'baseline/corpus.json', {'schema': 1, 'status': status})
                result = reviewed_status(root/'captures', root/'baseline', fixtures)
                self.assertEqual(result['status'], status)
                self.assertFalse(result['passed'])

    def test_backend_adapter_viewport_dpi_and_driver_require_independent_review(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fixtures, _ = self.fixture(root)
            original = json.loads((root/'captures/corpus.json').read_text())
            self.assertEqual(reviewed_status(root/'captures', root/'baseline', fixtures)['status'], 'approved')
            for field, value in [('backend', 'Metal'), ('adapter', 'Apple M1'), ('driver', 'v2'),
                                 ('platform', 'macOS'), ('viewport_pixels', [64, 32]), ('scale_factor', 2.0)]:
                changed = copy.deepcopy(original)
                changed['context'][field] = value
                write_json(root/'captures/corpus.json', changed)
                with self.subTest(field=field), self.assertRaisesRegex(ValueError, 'incompatible'):
                    reviewed_status(root/'captures', root/'baseline', fixtures)
            write_json(root/'captures/corpus.json', original)
            approved = json.loads((root/'baseline/corpus.json').read_text())
            approved['review']['masks_and_tolerances_reviewed'] = False
            write_json(root/'baseline/corpus.json', approved)
            with self.assertRaisesRegex(ValueError, 'explicit image/mask/tolerance review'):
                reviewed_status(root/'captures', root/'baseline', fixtures)

    def test_tampered_images_changed_actions_seeds_masks_or_missing_frames_fail(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fixtures, _ = self.fixture(root)
            for alteration in ('mask', 'seed', 'action'):
                changed = copy.deepcopy(fixtures)
                source = Path(fixtures['ship']['scenario'])
                original = source.read_text()
                if alteration == 'mask':
                    changed['ship']['tolerances']['masks'] = [[1, 1, 2, 2]]
                else:
                    scenario = json.loads(original)
                    if alteration == 'seed':
                        scenario['setup']['seed'] += 1
                    else:
                        scenario['steps'].insert(0, {'action': {'op': 'look', 'dx': 5, 'dy': 0}})
                    source.write_text(json.dumps(scenario))
                with self.subTest(alteration=alteration), self.assertRaisesRegex(ValueError, 'actions/seeds/tolerances'):
                    reviewed_status(root/'captures', root/'baseline', changed)
                source.write_text(original)
            image = root/'baseline/ship/pose-repeat-00.png'
            Image.new('RGB', (32, 32), 'red').save(image)
            with self.assertRaisesRegex(ValueError, 'hashes'):
                reviewed_status(root/'captures', root/'baseline', fixtures)
            image.unlink()
            with self.assertRaisesRegex(ValueError, 'incomplete fixture'):
                reviewed_status(root/'captures', root/'baseline', fixtures)

    def test_reviewed_cli_exit_codes_and_capture_never_overwrites_approval(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fixtures, _ = self.fixture(root)
            config = root/'fixtures.json'
            write_json(config, fixtures)
            command = [sys.executable, 'scripts/model_visual_tests.py', 'compare', '--reviewed',
                       '--fixtures', str(config), '--captures', str(root/'captures'),
                       '--output', str(root/'diff')]
            pending = subprocess.run(command + ['--baselines', str(root/'missing')], capture_output=True, text=True)
            self.assertEqual(pending.returncode, 3, pending.stderr)
            self.assertEqual(json.loads((root/'diff/appearance.json').read_text())['status'], 'missing')
            approved = subprocess.run(command + ['--baselines', str(root/'baseline')], capture_output=True, text=True)
            self.assertEqual(approved.returncode, 0, approved.stderr)
            original = (root/'baseline/corpus.json').read_bytes()
            overwrite = subprocess.run([sys.executable, 'scripts/model_visual_tests.py', 'capture',
                '--fixtures', str(config), '--captures', str(root/'baseline'), '--binary', 'unused'],
                capture_output=True, text=True)
            self.assertEqual(overwrite.returncode, 1)
            self.assertIn('cannot overwrite', overwrite.stderr)
            self.assertEqual((root/'baseline/corpus.json').read_bytes(), original)
            Image.new('RGB', (32, 32), 'white').save(root/'captures/ship/pose-repeat-00.png')
            unstable = subprocess.run(command + ['--baselines', str(root/'missing')], capture_output=True, text=True)
            self.assertEqual(unstable.returncode, 1, unstable.stderr)
            self.assertEqual(json.loads((root/'diff/appearance.json').read_text())['status'], 'failed')

    def test_consistent_silhouette_material_and_layout_regressions_fail_appearance(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fixtures, context = self.fixture(root)
            for change in ('silhouette', 'material', 'layout'):
                image = Image.new('RGB', (32, 32), 'black')
                rectangle = (8, 8, 23, 23)
                color = (120, 120, 120)
                if change == 'silhouette':
                    rectangle = (8, 8, 18, 23)
                elif change == 'material':
                    color = (180, 60, 50)
                else:
                    rectangle = (12, 8, 27, 23)
                ImageDraw.Draw(image).rectangle(rectangle, fill=color)
                for index in range(2):
                    image.save(root/'captures/ship'/f'pose-repeat-{index:02}.png')
                write_json(root/'captures/corpus.json', candidate(root/'captures', fixtures, context, 'c'*40, 'd'*64))
                reviewed_status(root/'captures', root/'baseline', fixtures)
                result = evaluate(root/'captures/ship', root/'baseline/ship', root/change, fixtures['ship'])
                with self.subTest(change=change):
                    self.assertFalse(result['passed'])
                    self.assertTrue(all(r['passed'] for r in result['results'] if r['kind'] == 'temporal'))
