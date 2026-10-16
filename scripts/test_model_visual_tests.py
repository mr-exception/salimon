import tempfile
import unittest
from pathlib import Path

from model_visual_tests import compare, evaluate

try:
    from PIL import Image
except ImportError:
    Image = None


@unittest.skipIf(Image is None, 'optional visual tests require Pillow')
class VisualTests(unittest.TestCase):
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
