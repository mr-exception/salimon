"""Evaluate the historical active-budget proposal; numerical passes never grant approval.

Raw historical evidence replay remains in the archived #288 verifier.
"""

def evaluate(evidence, budget):
    """Report individual numerical checks; proposal approval is never inferred."""
    candidate = evidence['stages']['final']
    checks = {}
    count = candidate['active_intervals']
    checks['active_interval_count'] = dict(
        actual=count, minimum=budget['minimum_active_intervals'],
        passed=count >= budget['minimum_active_intervals'])
    for path, limit in budget['maximums'].items():
        metric, statistic = path.split('.')
        actual = candidate['active_metrics'][metric][statistic]
        checks[path] = dict(actual=actual, maximum=limit, passed=actual <= limit)
    empty = evidence['matched_empty_prefix']['frame_ms']['p95']
    if empty <= 0:
        raise ValueError('matched empty p95 must be positive')
    ratio = candidate['active_metrics']['frame_ms']['p95'] / empty
    limit = budget['maximum_frame_p95_ratio_to_matched_empty']
    checks['frame_p95_ratio_to_matched_empty'] = dict(
        actual=ratio, maximum=limit, passed=ratio <= limit)
    spikes = candidate['active_frames_over_50ms']
    limit = budget['maximum_active_frames_over_50ms']
    checks['active_frames_over_50ms'] = dict(actual=spikes, maximum=limit,
                                            passed=spikes <= limit)
    failures = evidence['whole_window_relative_gate_failures']
    return dict(budget_status=budget['status'], proposal_checks=checks,
                proposal_checks_passed=all(c['passed'] for c in checks.values()),
                historical_whole_window_relative_failures=failures,
                historical_relative_checks_passed=not failures,
                acceptance_granted=False,
                reason='Owner review is required; numerical proposal checks do not waive relative failures.')

