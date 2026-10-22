#!/usr/bin/env python3
"""Run versioned portable experience checks; preserve native/backend pending work."""
import argparse
import hashlib
import json
import re
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]


def run(matrix_path, output, execute=subprocess.run):
    matrix = json.loads(matrix_path.read_text())
    if matrix.get('format') != 'salimon.experience-validation' or matrix.get('version') != 1:
        raise ValueError('unsupported experience matrix')
    cases = matrix['cases']
    ids = [case['id'] for case in cases]
    if not cases or len(ids) != len(set(ids)):
        raise ValueError('matrix must contain unique cases')
    for case in cases:
        if not isinstance(case['id'], str) or not re.fullmatch('[A-Z][A-Z0-9_-]{1,39}', case['id']):
            raise ValueError('invalid case ID')
        for field in ('invariant', 'native_protocol'):
            if not isinstance(case.get(field), str) or not case[field].strip():
                raise ValueError(f'case requires {field}')
        if case.get('portable_test'):
            if not re.fullmatch('[a-zA-Z0-9_:]+', case['portable_test']):
                raise ValueError('invalid exact portable test name')
        elif not case.get('blocked_by'):
            raise ValueError('unimplemented case must name explicit blockers')
    # Reserve once: reruns must not overwrite a previous failure or its logs.
    output.mkdir(parents=True, exist_ok=False)
    revision = execute(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True, capture_output=True, check=True).stdout.strip()
    results = []
    for case in cases:
        row = {'id': case['id'], 'invariant': case['invariant']}
        test = case.get('portable_test')
        if test:
            command = ['cargo', 'test', '-p', 'salimon-client', '--locked', '--bin', 'salimon-client', test, '--', '--exact']
            start = time.monotonic()
            try:
                process = execute(command, cwd=ROOT, text=True, capture_output=True, timeout=600)
                log = process.stdout + process.stderr
                (output / (case['id'] + '.log')).write_text(log)
                # Cargo exits zero for a typo/removed test. Such a run is missing
                # evidence, never a successful scenario.
                passed = process.returncode == 0 and 'test result: ok. 1 passed;' in process.stdout
                row.update(status='passed' if passed else 'failed', command=command,
                           log=case['id'] + '.log', seconds=time.monotonic() - start)
            except subprocess.TimeoutExpired:
                row.update(status='failed', command=command, reason='portable test exceeded 600 seconds')
        else:
            row.update(status='blocked', blocked_by=case['blocked_by'])
        row['native_status'] = 'not_run'
        row['native_protocol'] = case['native_protocol']
        results.append(row)
    failed = any(row['status'] == 'failed' for row in results)
    summary = {'schema': 1, 'matrix_version': matrix['version'], 'revision': revision,
               'matrix_sha256': hashlib.sha256(matrix_path.read_bytes()).hexdigest(),
               'runner_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
               'status': 'failed' if failed else 'pending', 'native_signoff': False,
               'backend_signoff': False, 'cases': results}
    (output / 'results.json').write_text(json.dumps(summary, indent=2) + '\n')
    return 1 if failed else 3


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--matrix', type=Path, default=ROOT / 'scenarios/portable/experience-v1.json')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    try:
        return run(args.matrix, args.output)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        parser.exit(1, f'experience suite failed: {error}\n')


if __name__ == '__main__':
    raise SystemExit(main())
