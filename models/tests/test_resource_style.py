"""Production Option A exports retain flat geology and shared sphere costs."""
import json
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
from style_review import report
from validate_asset import read_document, validate_asset, accessor_values

REPO = Path(__file__).resolve().parents[2]


class ResourceStyleTests(unittest.TestCase):
    def test_geology_exports_have_face_aligned_normals_and_no_texture_cost(self):
        manifests = sorted((REPO / 'models/assets/resources').glob('*-deposit-*/manifest.json'))
        manifests += [REPO / 'models/assets/resources/spherical-fragment/manifest.json']
        self.assertEqual(len(manifests), 13)
        for path in manifests:
            manifest = json.loads(path.read_text())
            with self.subTest(asset=manifest['assetId']):
                validate_asset(manifest['assetId'])
                runtime = REPO / manifest['runtime']
                result = report(*read_document(runtime), runtime.stat().st_size, manifest)
                doc, binary = read_document(runtime)
                # Nonplanar authored n-gons retain a shared polygon normal after
                # triangulation; require hard faces, not a changed triangle split.
                for mesh in doc['meshes']:
                    for primitive in mesh['primitives']:
                        normals = accessor_values(doc, binary, primitive['attributes']['NORMAL'])
                        indices = accessor_values(doc, binary, primitive['indices'])
                        for i in range(0, len(indices), 3):
                            a, b, c = (normals[j[0]] for j in indices[i:i+3])
                            self.assertEqual(a, b)
                            self.assertEqual(a, c)
                self.assertEqual(result['costs']['textureMemoryBytes'], 0)
                self.assertEqual(result['warnings'], [])
                if manifest['assetId'] == 'resource.spherical-fragment':
                    self.assertEqual(result['costs']['triangles'], 80)
                    self.assertEqual(result['costs']['materials'], 1)
