"""Reproduce #288's proposed budget evaluation without granting signoff.

The #255 verifier checks the retained ten-drop workload, admission, OS focus,
IDs/mass, active-only sample sufficiency and matched empty prefixes first.
Historical failed relative gates remain independent of the proposed budget.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
PRIOR = ROOT / 'reports/issue-255/2026-10-17-remediation'
sys.path.insert(0, str(ROOT / 'scripts'))
sys.path.insert(0, str(ROOT / 'reports/issue-255'))
spec = importlib.util.spec_from_file_location('remediation_verify', PRIOR / 'verify.py')
prior = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prior)


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


def settled_comparison(root, name):
    paths, runs = prior.windows(root, name)
    if len(runs) != 3 or len(paths) != 1:
        raise ValueError(f'{name}: expected three windows from one prepared population')
    for run in runs:
        summary = prior.summarize(run)
        if (run['sampling_stride'] != 1 or not summary['sleeping_observed']
                or summary['frame_samples_retained'] < 100
                or run['config']['count'] != 500):
            raise ValueError(f'{name}: requires undecimated fully sleeping 500-piece windows')
    stored = prior.load(root / name / 'summary.json')
    if stored['failures']:
        raise ValueError(f'{name}: collection failed')
    # Rebuild summaries from raw data instead of trusting derived metrics.
    stored['cases'] = [dict(prior.summarize(run),
                            config=dict(run['config'], repeat=i + 1))
                       for i, run in enumerate(runs)]
    return stored, runs


def verify(root):
    for directory in (PRIOR, root):
        for relative, digest in prior.load(directory / 'raw-sha256.json').items():
            if hashlib.sha256((directory / relative).read_bytes()).hexdigest() != digest:
                raise ValueError(f'raw evidence checksum mismatch: {directory / relative}')
    budget = prior.load(root / 'proposed-budget.json')
    evidence = prior.verify(PRIOR)
    result = evaluate(evidence, budget)
    before, before_runs = settled_comparison(root, 'settled-before')
    after, after_runs = settled_comparison(root, 'settled-after')
    # Binary lineage is independent of the checkout used to launch the runner.
    original = prior.load(PRIOR / 'before/summary.json')
    optimized = prior.load(PRIOR / 'final/summary.json')
    if before['binary_sha256'] != original['binary_sha256']:
        raise ValueError('unexpected pre-optimization executable')
    if after['binary_sha256'] != optimized['binary_sha256']:
        raise ValueError('unexpected optimized executable')
    if (before['environment'] != original['environment']
            or after['environment'] != optimized['environment']
            or any(run['renderer'] != optimized['cases'][0]['renderer']
                   for run in before_runs + after_runs)):
        raise ValueError('fresh comparison changed reference machine/renderer settings')
    if any(a['accounting'] != b['accounting'] for a, b in zip(before_runs, after_runs, strict=True)):
        raise ValueError('settled comparison changed identities/mass')
    result['fresh_settled_relative_failures'] = prior.compare(
        after, before, budget['whole_window_relative_tolerance'])
    result['fresh_settled_cadence_checks'] = {
        name: [c['metrics']['frame_ms']['p95'] <= budget['settled_frame_p95_ms']
               for c in summary['cases']]
        for name, summary in [('before', before), ('after', after)]}
    result['fresh_settled'] = {
        name: [dict(metrics=c['metrics'], frame_samples=c['frame_samples_retained'])
               for c in summary['cases']]
        for name, summary in [('before', before), ('after', after)]}
    result['active_evidence'] = evidence['stages']
    result['matched_empty_prefix'] = evidence['matched_empty_prefix']
    result['raw_checksums_verified'] = True
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parent)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.write_text(json.dumps(verify(args.root), indent=2) + '\n')
