"""Geometric counterexamples for the issue-224 read-only probe."""
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
from diagnose_scout_visibility import (coplanar_candidates, overlap_polygon, ray_hit,
                            signed_area, witness_views)


def face(name, points):
    return dict(node=name, primitive=0, face=0, material='opaque',
                vertices=points, normal=(0, 0, 1), plane=points[0][2])


class ScoutDiagnosticsTests(unittest.TestCase):
    def test_shared_edge_and_separate_triangles_are_not_overlap(self):
        a = [(0, 0), (1, 0), (0, 1)]
        b = [(1, 0), (1, 1), (0, 1)]
        self.assertEqual(signed_area(overlap_polygon(a, b)), 0)
        self.assertEqual(overlap_polygon(a, [(3, 3), (4, 3), (3, 4)]), [])

    def test_reversed_winding_and_containment_have_positive_area(self):
        a = [(0, 0), (2, 0), (0, 2)]
        b = [(0.2, 0.2), (0.2, 0.4), (0.4, 0.2)]
        self.assertAlmostEqual(abs(signed_area(overlap_polygon(a, b))), 0.02)
        self.assertAlmostEqual(abs(signed_area(overlap_polygon(b, a))), 0.02)

    def test_depth_separation_does_not_become_coplanar(self):
        a = face('A', [(0, 0, 0), (1, 0, 0), (0, 1, 0)])
        b = face('B', [(0, 0, 0.0001), (1, 0, 0.0001), (0, 1, 0.0001)])
        self.assertEqual(coplanar_candidates([a, b]), [])
        b['vertices'] = a['vertices']
        b['plane'] = 0
        overlap = coplanar_candidates([a, b])
        self.assertEqual(len(overlap), 1)
        self.assertAlmostEqual(overlap[0]['projectedOverlapAreaM2'], 0.5)
        views = witness_views(overlap[0], [a, b])
        self.assertTrue(all(v['witnessExposed'] for v in views))
        self.assertTrue(all(len(v['nearestOpaqueHits']) == 2 for v in views))

    def test_occluder_hides_witness_from_one_side(self):
        a = face('A', [(0, 0, 0), (1, 0, 0), (0, 1, 0)])
        b = dict(a, node='B')
        wall = face('wall', [(0, 0, 0.5), (1, 0, 0.5), (0, 1, 0.5)])
        candidate = coplanar_candidates([a, b])[0]
        views = witness_views(candidate, [a, b, wall])
        self.assertTrue(views[0]['witnessExposed'])
        self.assertFalse(views[1]['witnessExposed'])
        self.assertEqual(views[1]['nearestOpaqueHits'][0]['node'], 'wall')
        self.assertIsNone(ray_hit((0.2, 0.2, 1), (0, 0, 1), a))


if __name__ == '__main__':
    unittest.main()
