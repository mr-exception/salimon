"""Checked-in silicate exports must honor the physical fragment presentation cube."""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from validate_asset import validate_asset, read_document, accessor_values

REPO = Path(__file__).resolve().parents[2]


class SilicateFragmentTests(unittest.TestCase):
    def test_variants_validate_and_fit_authoritative_unit_cube(self):
        signatures = []
        for variant in ("slab", "ridge"):
            asset = "silicate-fragment-" + variant
            with self.subTest(variant=variant):
                metrics = validate_asset("resource." + asset)
                self.assertLessEqual(metrics["maxTriangles"], 64)
                self.assertEqual(metrics["maxTextureBytes"], 0)
                document, binary = read_document(
                    REPO / "client/assets/resources" / asset / "model.glb"
                )
                positions = []
                for mesh in document["meshes"]:
                    for primitive in mesh["primitives"]:
                        positions.extend(
                            accessor_values(
                                document,
                                binary,
                                primitive["attributes"]["POSITION"],
                            )
                        )
                self.assertTrue(positions)
                self.assertTrue(
                    all(abs(axis) <= 0.48 for point in positions for axis in point)
                )
                signatures.append(positions)
        self.assertNotEqual(*signatures)


if __name__ == "__main__":
    unittest.main()
