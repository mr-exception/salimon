import tempfile
import unittest
import json
from pathlib import Path

from model_visual_tests import compare, evaluate, expanded_scenario

try:
    from PIL import Image
except ImportError:
    Image = None


@unittest.skipIf(Image is None, 'optional visual tests require Pillow')
class VisualTests(unittest.TestCase):
    def test_complete_fixture_required_even_without_baselines(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            scenario = root/'scenario.json'
            scenario.write_text(json.dumps({'steps': [
                {'screenshot': 'front'}, {'action': {'op': 'look', 'dx': 8, 'dy': 2}},
                {'screenshot': 'side'}]}))
            fixture = {'mode': 'moving', 'repeat': 3, 'scenario': str(scenario)}
            expanded, names = expanded_scenario(fixture)
            self.assertEqual(len(expanded['steps']), 7)
            captures = root/'captures'
            captures.mkdir()
            for name in names:
                Image.new('RGB', (8, 8)).save(captures/name)
            self.assertTrue(evaluate(captures, root/'unapproved', root/'diff',
                                     fixture, temporal_only=True)['passed'])
            # Losing one repeat or a whole camera pose used to pass temporal CI.
            for missing in [('side-repeat-02.png',),
                            tuple(name for name in names if name.startswith('side'))]:
                for name in missing:
                    (captures/name).unlink()
                with self.assertRaisesRegex(ValueError, 'incomplete fixture evidence: missing='):
                    evaluate(captures, root/'unapproved', root/'diff', fixture, True)
                for name in missing:
                    Image.new('RGB', (8, 8)).save(captures/name)
            Image.new('RGB', (8, 8)).save(captures/'stale.png')
            with self.assertRaisesRegex(ValueError, "unexpected=\\['stale.png'\\]"):
                evaluate(captures, root/'unapproved', root/'diff', fixture, True)

    def test_fixture_contract_rejects_ambiguous_or_unrepeatable_sequences(self):
        with tempfile.TemporaryDirectory() as tmp:
            scenario = Path(tmp)/'scenario.json'
            for names, mode, repeat in [(['same', 'same'], 'static', 1),
                                        (['../escape'], 'static', 2),
                                        ([], 'static', 2),
                                        (['one'], 'moving', 1),
                                        (['one'], 'static', 1),
                                        (['one'], 'static', 0),
                                        (['one'], 'static', True),
                                        (['one'], 'unknown', 2)]:
                with self.subTest(names=names, mode=mode, repeat=repeat):
                    scenario.write_text(json.dumps({'steps': [{'screenshot': n} for n in names]}))
                    with self.assertRaises(ValueError):
                        expanded_scenario({'scenario': str(scenario), 'mode': mode, 'repeat': repeat})

    def test_production_fixture_sequences_are_complete_and_repeatable(self):
        fixtures = json.loads(Path('scenarios/visual/fixtures.json').read_text())
        for name, fixture in fixtures.items():
            with self.subTest(fixture=name):
                _, names = expanded_scenario(fixture)
                self.assertGreaterEqual(len(names), 2)

    def test_injected_change_masks_tolerance_and_artifacts(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            a, b = root/'a.png', root/'b.png'
            Image.new('RGB', (10, 10), (40, 40, 40)).save(a)
            image = Image.open(a)
            image.putpixel((3, 3), (240, 40, 40))
            image.save(b)
            result = compare(a, b, root/'diff')
            self.assertFalse(result['passed'])
            self.assertEqual(result['changed_pixels'], 1)
            self.assertTrue((root/'diff/highlight.png').is_file())
            self.assertTrue(compare(a, b, root/'mask', masks=[[3, 3, 4, 4]])['passed'])
            self.assertTrue(compare(a, b, root/'noise', channel_tolerance=200)['passed'])
            with self.assertRaises(ValueError):
                compare(a, b, root/'empty', masks=[[0, 0, 10, 10]])

    def test_static_flicker_and_corresponding_moving_frames(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for directory in ['baseline', 'capture']:
                (root/directory).mkdir()
                for index, color in enumerate(['black', 'white']):
                    Image.new('RGB', (8, 8), color).save(root/directory/f'{index}.png')
            for mode, passed in [('moving', True), ('static', False)]:
                self.assertEqual(evaluate(root/'capture', root/'baseline', root/mode,
                                          {'mode': mode})['passed'], passed)
            (root/'baseline/0.png').unlink()
            with self.assertRaisesRegex(ValueError, 'unapproved'):
                evaluate(root/'capture', root/'baseline', root/'missing', {'mode': 'moving'})

    def test_missing_capture_size_mismatch_and_single_frame_cannot_pass(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root/'baseline').mkdir()
            (root/'capture').mkdir()
            Image.new('RGB', (8, 8)).save(root/'baseline/0.png')
            Image.new('RGB', (8, 8)).save(root/'baseline/1.png')
            Image.new('RGB', (9, 8)).save(root/'capture/0.png')
            with self.assertRaisesRegex(ValueError, 'dimensions'):
                compare(root/'baseline/0.png', root/'capture/0.png', root/'diff')
            with self.assertRaisesRegex(ValueError, 'missing captured'):
                evaluate(root/'capture', root/'baseline', root/'diff', {'mode': 'moving'})
            with self.assertRaisesRegex(ValueError, 'no repeated'):
                evaluate(root/'capture', root/'baseline', root/'diff', {'mode': 'moving'}, True)
