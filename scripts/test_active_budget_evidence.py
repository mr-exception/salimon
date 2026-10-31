"""Guard the distinction between #288's numerical proposal and acceptance."""
import copy
from pathlib import Path
import json
import active_budget as verify
import unittest

BUDGET = Path(__file__).resolve().parent / 'fixtures/active-budget-proposal.json'


class ActiveBudgetEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.budget = json.loads(BUDGET.read_text())
        metrics = {}
        for path, limit in self.budget['maximums'].items():
            metric, statistic = path.split('.')
            metrics.setdefault(metric, {})[statistic] = limit
        self.evidence = {
            'stages': {'final': {
                'active_intervals': self.budget['minimum_active_intervals'],
                'active_metrics': metrics,
                'active_frames_over_50ms': 0}},
            'matched_empty_prefix': {'frame_ms': {'p95': metrics['frame_ms']['p95']}},
            'whole_window_relative_gate_failures': []}

    def test_passing_limits_never_grants_owner_acceptance(self):
        result = verify.evaluate(self.evidence, self.budget)
        self.assertTrue(result['proposal_checks_passed'])
        self.assertTrue(result['historical_relative_checks_passed'])
        self.assertFalse(result['acceptance_granted'])

    def test_historical_failures_cannot_be_overridden_by_passing_proposal(self):
        failure = {'metric': 'cpu_render_ms', 'baseline_p95': .5, 'current_p95': 1.0}
        self.evidence['whole_window_relative_gate_failures'] = [failure]
        result = verify.evaluate(self.evidence, self.budget)
        self.assertTrue(result['proposal_checks_passed'])
        self.assertFalse(result['historical_relative_checks_passed'])
        self.assertEqual(result['historical_whole_window_relative_failures'], [failure])
        self.assertFalse(result['acceptance_granted'])

    def test_sparse_samples_spikes_and_each_over_budget_metric_fail(self):
        candidate = self.evidence['stages']['final']
        cases = []
        sparse = copy.deepcopy(self.evidence)
        sparse['stages']['final']['active_intervals'] -= 1
        cases.append(sparse)
        spike = copy.deepcopy(self.evidence)
        spike['stages']['final']['active_frames_over_50ms'] = 1
        cases.append(spike)
        for path in self.budget['maximums']:
            metric, statistic = path.split('.')
            over = copy.deepcopy(self.evidence)
            over['stages']['final']['active_metrics'][metric][statistic] *= 1.01
            cases.append(over)
        for case in cases:
            with self.subTest(case=case):
                self.assertFalse(verify.evaluate(case, self.budget)['proposal_checks_passed'])
        self.assertEqual(candidate['active_intervals'], self.budget['minimum_active_intervals'])

    def test_matching_empty_reference_is_an_independent_gate(self):
        self.evidence['matched_empty_prefix']['frame_ms']['p95'] /= 2
        result = verify.evaluate(self.evidence, self.budget)
        self.assertFalse(result['proposal_checks']['frame_p95_ratio_to_matched_empty']['passed'])
        self.assertFalse(result['proposal_checks_passed'])
        self.evidence['matched_empty_prefix']['frame_ms']['p95'] = 0
        with self.assertRaisesRegex(ValueError, 'positive'):
            verify.evaluate(self.evidence, self.budget)


if __name__ == '__main__':
    unittest.main()
