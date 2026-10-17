"""Recheck raw admission, OS focus, accounting and the matched empty prefix."""
import argparse
import gzip
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from analyze import analyze, distribution
from benchmark_fragments import compare, summarize


def load(path):
    with (gzip.open(path, 'rt') if path.suffix == '.gz' else path.open()) as stream:
        return json.load(stream)


def windows(root, name):
    paths = sorted((root / name).glob('*.stdout*'))
    if not paths:
        raise ValueError(f'missing raw windows: {name}')
    focus = load(root / f'{name}-focus.json')
    admitted = {r['pid'] for r in focus if r['returncode'] == 0
                and f"frontmost={r['pid']} target={r['pid']}" in r['stdout']}
    result = []
    for path in paths:
        raw = load(path)
        for run in raw.get('runs', [raw]):
            summarize(run)
            if run['presentation']['pid'] not in admitted:
                raise ValueError(f'missing verified target-PID focus: {path}')
            result.append(run)
    return paths, result


def verify(root):
    stages = {}
    raw_stages = {}
    accounting = None
    signature = None
    for name in ('before', 'after', 'final'):
        paths, runs = windows(root, name)
        if len(runs) != 10:
            raise ValueError(f'{name}: expected ten independent drops')
        result = analyze(paths)
        if not result['active_tail_sample_requirement_met']:
            raise ValueError(f'{name}: insufficient moving intervals')
        for run in runs:
            accounting = accounting or run['accounting']
            signature = signature or (run['config'], run['renderer'])
            if run['accounting'] != accounting or (run['config'], run['renderer']) != signature:
                raise ValueError('changed workload, renderer or IDs/mass across stages')
        stages[name] = {k: v for k, v in result.items() if k != 'windows'}
        raw_stages[name] = runs

    _, empty = windows(root, 'empty-launch')
    if len(empty) != 10:
        raise ValueError('expected ten matched empty launches')
    matched_empty = []
    for loaded, reference in zip(raw_stages['final'], empty, strict=True):
        expected_config = dict(loaded['config'], count=0)
        if reference['config'] != expected_config or reference['renderer'] != loaded['renderer']:
            raise ValueError('empty-launch configuration mismatch')
        completed = [s for s in loaded['samples'] if s['frame_ms'] is not None]
        indices = [i for i, s in enumerate(completed) if s['moving_objects'] > 0]
        if indices != list(range(len(indices))):
            raise ValueError('active intervals are not a contiguous launch prefix')
        reference_completed = [s for s in reference['samples'] if s['frame_ms'] is not None]
        if len(reference_completed) < len(indices):
            raise ValueError('short empty launch')
        matched_empty.extend(reference_completed[:len(indices)])

    regressions = compare(load(root / 'final/summary.json'), load(root / 'before/summary.json'), .2)
    references = {}
    for name, count, repeats in [('empty-stable', 0, 3), ('settled', 500, 3), ('stress', 500, 1)]:
        _, runs = windows(root, name)
        if len(runs) != repeats or any(run['config']['count'] != count for run in runs):
            raise ValueError(f'{name}: reference population/window mismatch')
        references[name] = [summarize(run) for run in runs]
        if not all(run['sleeping_observed'] for run in references[name]):
            raise ValueError(f'{name}: not naturally asleep throughout the reference')
    return dict(stages=stages, focus_and_accounting_verified=True,
                matched_empty_prefix=dict(intervals=len(matched_empty),
                    frame_ms=distribution([s['frame_ms'] for s in matched_empty]),
                    frames_over_50ms=sum(s['frame_ms'] > 50 for s in matched_empty)),
                whole_window_relative_gate_failures=regressions,
                references=references)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parent)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.write_text(json.dumps(verify(args.root), indent=2) + '\n')
