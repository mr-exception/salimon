import copy
from pathlib import Path
import struct
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
from geometry_quality import diagnose


def fixture(triangles, normals=None):
    points = [p for tri in triangles for p in tri]
    binary = struct.pack('<' + 'f'*len(points)*3, *(v for p in points for v in p))
    document = {'nodes': [{'name': 'Root', 'children': [1]}, {'name': 'Panel', 'mesh': 0}],
                'meshes': [{'primitives': [{'attributes': {'POSITION': 0}}]}],
                'accessors': [{'bufferView': 0, 'componentType': 5126, 'type': 'VEC3', 'count': len(points)}],
                'bufferViews': [{'byteOffset': 0}]}
    if normals:
        document['meshes'][0]['primitives'][0]['attributes']['NORMAL'] = 1
        document['accessors'].append({'bufferView': 1, 'componentType': 5126, 'type': 'VEC3', 'count': len(points)})
        document['bufferViews'].append({'byteOffset': len(binary)})
        binary += struct.pack('<'+'f'*len(points)*3, *(v for _ in points for v in normals))
    return document, binary


TRIANGLE = [(0, 0, 0), (1, 0, 0), (0, 1, 0)]


class QualityTests(unittest.TestCase):
    def test_normal_degenerate_duplicate_and_shared_edges(self):
        for triangles, normals, code in [([TRIANGLE], (0, 0, -1), 'normal-winding'),
                ([[(0, 0, 0)]*3], None, 'degenerate'),
                ([TRIANGLE, TRIANGLE[::-1]], None, 'duplicate-face')]:
            result = diagnose(*fixture(triangles, normals), {})
            self.assertIn(code, [f['code'] for f in result['findings']])
            self.assertEqual(result['status'], 'failed')
            self.assertIn('locationMeters', result['findings'][0])
        adjacent = [(1, 0, 0), (1, 1, 0), (0, 1, 0)]
        self.assertEqual(diagnose(*fixture([TRIANGLE, adjacent]), {})['status'], 'passed')

    def test_overlap_tolerance_across_plane_boundaries_and_exemption(self):
        shifted = [(x+0.2, y+0.2, 0.000009) for x, y, z in TRIANGLE]
        doc, binary = fixture([TRIANGLE, shifted])
        self.assertEqual(diagnose(doc, binary, {})['findings'][0]['code'], 'surface-overlap')
        policy = {'overlapExemptions': [{'nodes': ['Panel', 'Panel'], 'reason': 'Intentional layered fixture', 'maxPairs': 1}]}
        self.assertEqual(diagnose(doc, binary, policy)['status'], 'passed')
        self.assertEqual(diagnose(doc, binary, policy), diagnose(doc, binary, policy))

    def test_world_transforms_bounds_anchors_clearance_and_topology(self):
        doc, binary = fixture([TRIANGLE])
        doc['nodes'][0]['translation'] = [5, 0, 0]
        policy = {'boundsMeters': {'Panel': [[5, 0, 0], [6, 1, 0]]},
                  'anchorsMeters': {'Panel': [5, 0, 0]}}
        self.assertEqual(diagnose(doc, binary, policy)['status'], 'passed')
        doc['nodes'][1]['translation'] = [0.01, 0, 0]
        self.assertEqual(diagnose(doc, binary, policy)['status'], 'failed')
        policy = {'closedNodes': ['Panel'], 'clearanceBoxes': [{'name': 'aisle',
            'boundsMeters': [[5, 0, -1], [6, 1, 1]], 'obstacles': ['Panel']}]}
        self.assertEqual({f['code'] for f in diagnose(doc, binary, policy)['findings']}, {'non-manifold', 'clearance'})

    def test_style_replacement_preserves_spatial_interface(self):
        # Option B adds a center vertex and three facets with the same envelope.
        a = fixture([TRIANGLE])
        center = (1/3, 1/3, 0)
        b = fixture([[TRIANGLE[i], TRIANGLE[(i+1)%3], center] for i in range(3)])
        for document, _ in (a, b):
            document['nodes'][0]['children'].extend([2, 3])
            document['nodes'].extend([{'name': 'COLLIDER_Body', 'extras': {'sizeMeters': [1, 1, 1]}},
                                      {'name': 'MARKER_Use', 'translation': [0.5, 0.5, 0]}])
        policy = {'boundsMeters': {'Panel': [[0, 0, 0], [1, 1, 0]]},
                  'anchorsMeters': {'Root': [0, 0, 0], 'COLLIDER_Body': [0, 0, 0], 'MARKER_Use': [0.5, 0.5, 0]}}
        for data in (a, b):
            self.assertEqual(diagnose(*data, policy)['status'], 'passed')
        self.assertEqual(diagnose(*a, policy)['originsMeters'], diagnose(*b, policy)['originsMeters'])

    def test_spatial_sweep_large_separated_set(self):
        triangles = [[(x+i*2, y, z) for x, y, z in TRIANGLE] for i in range(2000)]
        result = diagnose(*fixture(triangles), {})
        self.assertEqual(result['candidatePairs'], 0)
        self.assertEqual(result['status'], 'passed')

    def test_generic_export_gate_rejects_policy_violation(self):
        from test_validate_asset import ValidationTests
        from export_asset import ExportError
        fixture = ValidationTests()
        fixture.setUp()
        try:
            fixture.manifest['quality'] = {'boundsMeters': {'Iron_Fragment': [[0, 0, 0], [2, 1, 0]]}}
            with self.assertRaisesRegex(ExportError, 'quality.*bounds'):
                fixture.run_validation()
        finally:
            fixture.doCleanups()

    def test_exemption_budget_cannot_hide_additional_overlap(self):
        result = diagnose(*fixture([TRIANGLE]*3), {'overlapExemptions': [
            {'nodes': ['Panel', 'Panel'], 'reason': 'One intentional layered pair', 'maxPairs': 1}]})
        self.assertEqual(result['status'], 'failed')
