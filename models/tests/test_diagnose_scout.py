"""Portable geometric witnesses for the read-only quality probe."""
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
from diagnose_scout import diagnose, overlap_area


class CoplanarProbeTests(unittest.TestCase):
    def test_overlapping_faces_including_opposite_winding(self):
        face = [(0, 0, 0), (2, 0, 0), (0, 2, 0)]
        inside = [(0, 0, 0), (1, 0, 0), (0, 1, 0)]
        self.assertAlmostEqual(overlap_area(face, inside, (0, 0, 1)), .5)
        self.assertAlmostEqual(overlap_area(face, inside[::-1], (0, 0, 1)), .5)
        # Test the projection onto XZ as well as XY.
        xz = lambda ps: [(x, z, y) for x, y, z in ps]
        self.assertAlmostEqual(overlap_area(xz(face), xz(inside), (0, 1, 0)), .5)

    def test_shared_edges_and_disjoint_triangles_are_not_surfaces(self):
        face = [(0, 0, 0), (1, 0, 0), (0, 1, 0)]
        adjacent = [(1, 0, 0), (1, 1, 0), (0, 1, 0)]
        disjoint = [(1, 1, 0), (2, 1, 0), (1, 2, 0)]
        for other in (adjacent, disjoint):
            self.assertEqual(overlap_area(face, other, (0, 0, 1)), 0)

    def test_aabb_overlap_does_not_imply_triangle_overlap(self):
        face = [(0, 0, 0), (2, 0, 0), (0, 2, 0)]
        other = [(1.5, 1.5, 0), (2, 1.5, 0), (1.5, 2, 0)]
        self.assertEqual(overlap_area(face, other, (0, 0, 1)), 0)

    def test_future_parent_transform_is_not_silently_ignored(self):
        document = {'nodes': [{'name': 'Parent', 'children': [1], 'scale': [2, 1, 1]}]}
        with self.assertRaisesRegex(ValueError, 'identity transforms'):
            diagnose(document, b'')


if __name__ == '__main__':
    unittest.main()
