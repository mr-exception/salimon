"""Reject incomplete or misleading evidence rather than synthesizing performance."""
import unittest
from benchmark_spheres import distribution


class EvidenceTests(unittest.TestCase):
    def test_raw_sample_count_and_percentile(self):
        result = distribution(list(range(100)))
        self.assertEqual(result['p95_ms'], 94)
        self.assertEqual(result['mean_ms'], 49.5)

    def test_incomplete_nonfinite_and_negative_timings_fail(self):
        for values in ([1] * 99, [1] * 101, [float('nan')] * 100,
                       [float('inf')] * 100, [-1] * 100):
            with self.subTest(values=values[:1]), self.assertRaises(ValueError):
                distribution(values)
