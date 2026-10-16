"""Shared sphere export/material and mass-independent unit radius contract."""
import math
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
from validate_asset import validate_asset, read_document, accessor_values

ROOT = Path(__file__).resolve().parents[2]


class SphereTests(unittest.TestCase):
    def test_shared_sphere_schema_radius_flat_facets_and_material_role(self):
        metrics = validate_asset('resource.spherical-fragment')
        self.assertEqual(metrics['maxTriangles'], 80)
        self.assertEqual(metrics['maxTextureBytes'], 0)
        doc, binary = read_document(ROOT/'client/assets/resources/spherical-fragment/model.glb')
        self.assertEqual(len(doc['meshes']), 1)
        self.assertEqual(len(doc['materials']), 1)
        self.assertEqual(doc['materials'][0]['name'], 'Resource_Tint')
        self.assertEqual(doc['materials'][0]['pbrMetallicRoughness'].get('baseColorFactor', [1, 1, 1, 1]), [1, 1, 1, 1])
        primitive = doc['meshes'][0]['primitives'][0]
        positions = accessor_values(doc, binary, primitive['attributes']['POSITION'])
        normals = accessor_values(doc, binary, primitive['attributes']['NORMAL'])
        self.assertTrue(all(abs(math.sqrt(sum(v*v for v in p))-0.48) < 1e-6 for p in positions))
        self.assertTrue(all(abs(math.sqrt(sum(v*v for v in n))-1) < 1e-5 for n in normals))
        self.assertNotIn('animations', doc)
        self.assertNotIn('textures', doc)
