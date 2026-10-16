#!/usr/bin/env python3
"""Collect and validate portable sphere CPU evidence; never synthesize GPU FPS."""
import argparse
import csv
import json
import math
import os
from pathlib import Path
import platform
import statistics
import subprocess

COUNTS = (10, 100, 500)
STATES = ('resting', 'active', 'pile-resting')


def distribution(values):
    if len(values) != 100 or any(not math.isfinite(v) or v < 0 for v in values):
        raise ValueError('expected 100 finite nonnegative samples')
    values = sorted(values)
    return dict(mean_ms=statistics.mean(values), p95_ms=values[94], max_ms=values[-1])


def summarize(cpu_path, renderer_path):
    cpu = json.loads(Path(cpu_path).read_text())
    if cpu.get('schema') != 1 or cpu.get('mode') != 'cpu-runtime-adapter':
        raise ValueError('unsupported CPU report')
    expected = {(n, s) for n in COUNTS for s in STATES}
    actual = [(r['count'], r['state']) for r in cpu['rows']]
    if len(actual) != len(expected) or set(actual) != expected:
        raise ValueError('incomplete or duplicate CPU matrix')
    physics = []
    for row in cpu['rows']:
        samples = row['samples']
        total = distribution([s['adapter_total_ms'] for s in samples])
        for sample in samples:
            if row['state'] != 'active' and (sample['sleeping'] != row['count'] or sample['pair_visits'] != 0 or sample['substeps'] != 0):
                raise ValueError('resting workload did not retain complete sleep')
        physics.append(dict(count=row['count'], state=row['state'], **total,
                            contact=distribution([s['contact_ms'] for s in samples]),
                            preparation_steps=row['preparation_steps'],
                            sleeping_min=min(s['sleeping'] for s in samples),
                            moving_max=max(s['moving'] for s in samples),
                            pair_visits_max=max(s['pair_visits'] for s in samples)))
    with Path(renderer_path).open(newline='') as stream:
        rows = list(csv.DictReader(stream))
    rendering = []
    for variant in ('legacy-shards', 'sphere'):
        for count in COUNTS:
            for preview in ('false', 'true'):
                group = [r for r in rows if r['variant'] == variant and int(r['count']) == count and r['preview'] == preview]
                timing = distribution([float(r['prepare_ms']) for r in group])
                if {int(r['sample']) for r in group} != set(range(100)):
                    raise ValueError('incomplete renderer sample indices')
                size = count + (preview == 'true')
                for r in group:
                    if int(r['submitted_instances']) != size or int(r['encoded_instance_bytes']) != size * 48:
                        raise ValueError('instance count/encoding regression')
                    if variant == 'sphere' and (int(r['batches']) != 1 or int(r['submitted_triangles']) != size * 80):
                        raise ValueError('sphere batch/triangle regression')
                rendering.append(dict(variant=variant, count=count, preview=preview == 'true', **timing,
                                      batches=int(group[0]['batches']), encoded_instance_bytes=int(group[0]['encoded_instance_bytes']),
                                      submitted_triangles=int(group[0]['submitted_triangles']), scratch_capacity_bytes=int(group[0]['scratch_capacity_bytes'])))
    if len(rows) != 1200:
        raise ValueError('unexpected renderer rows')
    return dict(schema=1, physics=physics, renderer=rendering,
                unmeasured=['presented FPS', '1% low FPS', 'GPU frame/pass time', 'actual GPU upload bytes', 'resident process/GPU memory'],
                note='CPU samples and encoded submission structure, not a native frame benchmark')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=Path('artifacts/sphere-benchmarks'))
    parser.add_argument('--run', action='store_true')
    parser.add_argument('--cargo', default='cargo')
    parser.add_argument('--toolchain')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    cpu, renderer = args.output/'cpu.json', args.output/'renderer.csv'
    if args.run:
        env = dict(os.environ, SALIMON_SPHERE_CPU_REPORT=str(cpu.resolve()), SALIMON_SPHERE_RENDER_REPORT=str(renderer.resolve()))
        command = [args.cargo] + ([f'+{args.toolchain}'] if args.toolchain else [])
        command += ['test', '--release', '--locked', '-p', 'salimon-client', '-p', 'salimon-renderer', 'sphere_', '--', '--ignored']
        subprocess.run(command, env=env, check=True, timeout=1200)
        (args.output/'environment.json').write_text(json.dumps(dict(platform=platform.platform(), processor=platform.processor(), command=command), indent=2)+'\n')
    report = summarize(cpu, renderer)
    (args.output/'summary.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
