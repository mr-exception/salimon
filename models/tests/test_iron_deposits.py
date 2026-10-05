"""Iron deposits must be distinct opaque meshes inside the gameplay-safe cube."""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
from validate_asset import validate_asset, read_document, accessor_values

REPO = Path(__file__).resolve().parents[2]
VARIANTS = ('nodule', 'vein', 'ledge', 'rubble')


class IronDepositTests(unittest.TestCase):
    def test_exports_validate_and_fit_spherical_scaling_rule(self):
        signatures = []
        for variant in VARIANTS:
            asset = 'iron-deposit-' + variant
            with self.subTest(variant=variant):
                metrics = validate_asset('resource.' + asset)
                self.assertLessEqual(metrics['maxTriangles'], 240)
                self.assertEqual(metrics['maxTextureBytes'], 0)
                document, binary = read_document(
                    REPO / 'client/assets/resources' / asset / 'model.glb')
                for node in document['nodes']:
                    self.assertNotIn('matrix', node)
                    self.assertEqual(node.get('translation', [0, 0, 0]), [0, 0, 0])
                    self.assertEqual(node.get('rotation', [0, 0, 0, 1]), [0, 0, 0, 1])
                    self.assertEqual(node.get('scale', [1, 1, 1]), [1, 1, 1])
                self.assertEqual(len(document['materials']), 3)
                for material in document['materials']:
                    self.assertEqual(material.get('alphaMode', 'OPAQUE'), 'OPAQUE')
                    self.assertEqual(material['pbrMetallicRoughness']['baseColorFactor'][3], 1)
                self.assertEqual({m['name'] for m in document['materials']},
                                 {'Iron_Ore', 'Iron_Inclusions', 'Oxide_Crust'})
                for material in document['materials']:
                    self.assertAlmostEqual(
                        material['pbrMetallicRoughness']['roughnessFactor'],
                        .85 if material['name'] == 'Oxide_Crust' else .62, places=6)
                positions = []
                for mesh in document['meshes']:
                    for primitive in mesh['primitives']:
                        self.assertEqual(primitive.get('mode', 4), 4)
                        positions.extend(accessor_values(
                            document, binary, primitive['attributes']['POSITION']))
                self.assertTrue(positions)
                self.assertTrue(all(abs(axis) <= .48 for point in positions for axis in point))
                # Exact runtime rule, checked on exported vertex data at several radii.
                for radius in (.01, .2, 3):
                    scale = radius / (.48 * 3 ** .5)
                    self.assertTrue(all(sum((axis * scale) ** 2 for axis in point)
                                        <= radius ** 2 for point in positions))
                self.assertNotIn(positions, signatures)
                signatures.append(positions)
