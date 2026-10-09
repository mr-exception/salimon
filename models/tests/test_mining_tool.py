"""The handheld's surface atlas must reach its runtime UV/material consumers."""
from pathlib import Path
import struct
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
from validate_asset import validate_asset, read_document, accessor_values

REPO = Path(__file__).resolve().parents[2]


class MiningToolTests(unittest.TestCase):
    def test_texture_geometry_and_material_contract(self):
        metrics = validate_asset('item.mining-tool')
        self.assertGreater(metrics['maxTriangles'], 640)
        self.assertLessEqual(metrics['maxTriangles'], 3000)
        self.assertEqual(metrics['maxPrimitives'], 15)
        self.assertGreater(metrics['maxTextureBytes'], 0)
        doc, binary = read_document(REPO / 'client/assets/items/mining-tool/model.glb')
        self.assertEqual(len(doc['images']), 1)
        image = doc['images'][0]
        self.assertEqual(image['mimeType'], 'image/png')
        view = doc['bufferViews'][image['bufferView']]
        png = binary[view['byteOffset']:view['byteOffset'] + view['byteLength']]
        self.assertEqual(png[:8], b'\x89PNG\r\n\x1a\n')
        self.assertEqual(struct.unpack_from('>II', png, 16), (256, 256))
        # Test the actual export, not just the presence of source textures.
        quadrants = {'Tool_Housing': (0, 1), 'Tool_Grip': (1, 1),
                     'Tool_Metal': (0, 0), 'Tool_Status': (1, 0)}
        for mesh in doc['meshes']:
            for primitive in mesh['primitives']:
                material = doc['materials'][primitive['material']]
                pbr = material['pbrMetallicRoughness']
                self.assertEqual(doc['textures'][pbr['baseColorTexture']['index']]['source'], 0)
                self.assertNotIn('metallicRoughnessTexture', pbr)
                u, v = quadrants[material['name']]
                coords = accessor_values(doc, binary, primitive['attributes']['TEXCOORD_0'])
                self.assertTrue(all(u / 2 < x < (u + 1) / 2 and v / 2 < y < (v + 1) / 2
                                    for x, y in coords), material['name'])
        for node in doc['nodes']:
            self.assertNotIn('matrix', node)
            self.assertEqual(node.get('translation', [0, 0, 0]), [0, 0, 0])
            self.assertEqual(node.get('rotation', [0, 0, 0, 1]), [0, 0, 0, 1])
            self.assertEqual(node.get('scale', [1, 1, 1]), [1, 1, 1])
