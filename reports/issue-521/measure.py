"""Serial Linux release CPU/RSS diagnostics; does not establish hardware FPS or model budgets."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import statistics
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()


def run(binary_dir, output, repeats, supervisor):
    output.mkdir(parents=True, exist_ok=False)
    fixtures = ['client/universe/fixtures/references-v1.json',
                'client/universe/fixtures/storage-observed-v1.json',
                'client/universe/fixtures/partition-v1.bin',
                'catalogs/sources/fixtures/jpl-earth-j2000.json',
                'client/universe/examples/support/reference_inputs.rs', 'Cargo.lock']
    cpu = next(line.split(':', 1)[1].strip() for line in Path('/proc/cpuinfo').read_text().splitlines() if line.startswith('model name'))
    context = {'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
               'rustc': subprocess.check_output(['rustc', '-Vv'], text=True),
               'platform': platform.platform(), 'cpu': cpu, 'python': platform.python_version(),
               'profile': 'release; locked Cargo build; default checked-in profile',
               'fixtures_sha256': {p: digest(ROOT / p) for p in fixtures},
               'binaries_sha256': {p: digest(binary_dir / p) for p in ['reference_bench', 'partition_bench', 'reference_compare']},
               'supervisor_sha256': digest(supervisor),
               'cgroup': {p: (Path('/sys/fs/cgroup') / p).read_text().strip() for p in ['cpu.max', 'memory.max', 'cpuset.cpus.effective']},
               'input': 'frozen local fixtures; no graphics, live sources, wall-clock model or backend',
               'measurement': 'serial fresh child processes from small C parent; Linux wait4 ru_maxrss in KiB; executable wall time includes launch; inner raw JSON retains workload timings',
               'loadavg_before': Path('/proc/loadavg').read_text().strip(),
               'cpu_stat_before': Path('/sys/fs/cgroup/cpu.stat').read_text()}
    (output / 'context.json').write_text(json.dumps(context, indent=2)+'\n')
    runs = []
    for name in ['reference_bench', 'partition_bench']:
        for repeat in range(repeats):
            stem = output / f'{name}-{repeat+1:02}'
            args = [str((binary_dir / name).resolve())]
            with stem.with_suffix('.json').open('w') as out, stem.with_suffix('.stderr').open('w') as err:
                completed = subprocess.run([str(supervisor.resolve()), str(stem.with_suffix('.measurement.json').resolve()), *args], stdout=out, stderr=err, timeout=60)
            row = json.loads(stem.with_suffix('.measurement.json').read_text())
            row.update({'command': args, 'repeat': repeat+1})
            stem.with_suffix('.measurement.json').write_text(json.dumps(row, indent=2)+'\n')
            runs.append(row)
            assert completed.returncode == row['exit_code'] == 0, row
            raw = json.loads(stem.with_suffix('.json').read_text())
            assert raw['os'] == 'linux' and raw['arch'] == 'x86_64'
            if name == 'reference_bench':
                assert raw['profile'] == 'release' and raw['reference_preflight']['passed']
                assert all(r['iterations_per_sample'] == 1000 and len(r['elapsed_ms']) == 7 for r in raw['workloads'])
            else:
                assert [r['cells'] for r in raw['rows']] == [1, 32, 512]
                assert [r['records'] for r in raw['rows']] == [4, 128, 2048]
    def distribution(values):
        return {'raw': values, 'min': min(values), 'median': statistics.median(values), 'max': max(values)}
    summary = {'processes': {name: {metric: distribution([r[metric] for r in runs if Path(r['command'][0]).name == name])
                                  for metric in ['wall_ms', 'max_rss_kib', 'user_seconds', 'system_seconds']}
                            for name in ['reference_bench', 'partition_bench']}, 'workloads': {}}
    baseline_ref = json.loads((ROOT/'reports/issue-404/reference-bench.json').read_text())
    baseline_part = json.loads((ROOT/'reports/issue-405/partition-bench.json').read_text())
    for base in baseline_ref['workloads']:
        values = []
        for path in sorted(output.glob('reference_bench-??.json')):
            row = next(r for r in json.loads(path.read_text())['workloads'] if r['workload'] == base['workload'])
            assert row['iterations_per_sample'] == base['iterations_per_sample']
            values.extend(row['elapsed_ms'])
        summary['workloads'][base['workload']] = {**distribution(values), 'iterations_per_sample': 1000,
            'historical_linux_median_ms': base['median_ms'], 'median_ratio': statistics.median(values)/base['median_ms']}
    for base in baseline_part['rows']:
        for metric in ['encode_ms', 'index_open_ms', 'read_all_reverse_ms']:
            values = []
            for path in sorted(output.glob('partition_bench-??.json')):
                row = next(r for r in json.loads(path.read_text())['rows'] if r['cells'] == base['cells'])
                for field in ['records', 'raw_bytes', 'pack_bytes']:
                    assert row[field] == base[field], f'unmatched workload: {field}'
                values.append(row[metric])
            summary['workloads'][f'partition-{base["cells"]}-{metric}'] = {**distribution(values),
                'cells': base['cells'], 'records': base['records'], 'historical_linux_ms': base[metric],
                'median_ratio': statistics.median(values)/base[metric]}
    summary['limitations'] = 'Historical EPYC 9V45 versus this shared cloud CPU; identical sizes/bytes, different host/revision. No dedicated-hardware regression, frame-rate, model-drift or M1 acceptance. Seven processes are not tail certification.'
    summary['cpu_stat_after'] = Path('/sys/fs/cgroup/cpu.stat').read_text()
    summary['loadavg_after'] = Path('/proc/loadavg').read_text().strip()
    (output/'summary.json').write_text(json.dumps(summary, indent=2)+'\n')


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary-dir', type=Path, default=Path('target/release/examples'))
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--supervisor', type=Path, default=Path('artifacts/521-measure-rss'))
    p.add_argument('--repeats', type=int, default=7, choices=range(3, 21))
    a = p.parse_args(); run(a.binary_dir, a.output, a.repeats, a.supervisor)
