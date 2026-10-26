#!/usr/bin/env python3
"""Serial paired CPU diagnostics using prebuilt, identical-snapshot workloads."""
import argparse
import csv
import hashlib
import io
import json
from pathlib import Path
import statistics
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent', type=Path)
    parser.add_argument('current', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    rows = {name: [] for name in ('parent', 'current')}
    binaries = {}
    for name in rows:
        binary = getattr(args, name).resolve()
        binaries[name] = {'path': str(binary), 'sha256': hashlib.file_digest(binary.open('rb'), 'sha256').hexdigest()}
    for repeat in range(3):
        # Alternate pair order; never run compilation, rendering or another lane here.
        for name in (('parent', 'current') if repeat % 2 == 0 else ('current', 'parent')):
            result = subprocess.run([binaries[name]['path']], check=True,
                                    capture_output=True, text=True, timeout=60)
            (args.output / f'{name}-{repeat + 1}.csv').write_text(result.stdout)
            (args.output / f'{name}-{repeat + 1}.stderr').write_text(result.stderr)
            rows[name].append(list(csv.DictReader(io.StringIO(result.stdout))))
            print(f'{name} repeat {repeat + 1} completed', flush=True)
    comparisons = []
    for index in range(len(rows['parent'][0])):
        a = [repeat[index] for repeat in rows['parent']]
        b = [repeat[index] for repeat in rows['current']]
        fields = ('count', 'state', 'delta_ms', 'samples', 'substeps', 'integrated', 'sleeping')
        expected = {k: a[0][k] for k in fields}
        assert all({k: r[k] for k in fields} == expected for r in a + b)
        parent = statistics.median(float(r['median_ms']) for r in a)
        current = statistics.median(float(r['median_ms']) for r in b)
        comparisons.append(dict(expected, parent_median_ms=parent, current_median_ms=current,
                                median_ratio=current / parent,
                                parent_p95_ms=[float(r['p95_ms']) for r in a],
                                current_p95_ms=[float(r['p95_ms']) for r in b]))
    (args.output / 'summary.json').write_text(json.dumps({'binaries': binaries,
        'comparisons': comparisons, 'note': 'CPU diagnostics only; no new threshold or hardware FPS approval'}, indent=2) + '\n')


if __name__ == '__main__':
    main()
