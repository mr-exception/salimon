import csv
from pathlib import Path
import tempfile
import unittest

try:
    from PIL import Image
    from compare_terrain_style import compare
except ImportError:
    Image = None


@unittest.skipIf(Image is None, 'optional image comparisons require Pillow')
class TerrainComparisonTests(unittest.TestCase):
    def fixture(self, folder, color):
        folder.mkdir()
        with (folder / 'costs.csv').open('w', newline='') as stream:
            writer = csv.writer(stream)
            writer.writerow(['scene', 'draws', 'triangles', 'patch_gpu_bytes', 'uniform_upload_bytes'])
            for body in ('earth', 'moon', 'mars'):
                for distance in ('near', 'far', 'orbit'):
                    scene = f'{body}-{distance}'
                    writer.writerow([scene, 1, 512, 147620, 256])
                    image = Image.new('RGB', (512, 384), 'black')
                    image.paste(color, (10, 10, 200, 200))
                    image.save(folder / (scene + '.png'))

    def test_color_changes_allowed_but_missing_coverage_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.fixture(root / 'before', (50, 60, 70))
            self.fixture(root / 'after', (70, 80, 90))
            result = compare(root / 'before', root / 'after', root / 'report')
            self.assertTrue(all(row['changedCoveragePixels'] == 0 for row in result['scenes']))
            self.assertTrue(all(row['changedColorPixels'] > 0 for row in result['scenes']))
            path = root / 'after/earth-near.png'
            image = Image.open(path)
            image.putpixel((50, 50), (0, 0, 0))
            image.save(path)
            with self.assertRaisesRegex(ValueError, 'coverage'):
                compare(root / 'before', root / 'after', root / 'rejected')

    def test_extra_geometry_cost_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.fixture(root / 'before', (50, 60, 70))
            self.fixture(root / 'after', (50, 60, 70))
            path = root / 'after/costs.csv'
            path.write_text(path.read_text().replace(',512,', ',513,'))
            with self.assertRaisesRegex(ValueError, 'triangles'):
                compare(root / 'before', root / 'after', root / 'rejected')
