#!/usr/bin/env python3
"""Scheduled CI planning, compact reports, summaries and a static history dashboard.

Standard library only. Native test/benchmark tools remain the source of truth;
this module never fabricates measurements or treats missing evidence as success.
"""
import argparse
import base64
import datetime
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import re
import shutil
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
HISTORY_PATH = 'data/history.json'
LANES = {'nightly', 'benchmarks', 'macos', 'windows'}
MAX_RUNS = 90


def read_json(path):
    return json.loads(Path(path).read_text(encoding='utf-8'))


def write_json(path, data):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, allow_nan=False) + '\n', encoding='utf-8')


def validate_history(data):
    if data.get('schema') != 1 or not isinstance(data.get('runs'), list):
        raise ValueError('unsupported CI history')
    for row in data['runs']:
        if row['lane'] not in LANES or row['status'] not in ('success', 'failure', 'pending'):
            raise ValueError('invalid CI history row')
    return data


def remote_history():
    """Only a missing branch/file (404) initializes history; auth/network errors fail."""
    repository = os.environ['GITHUB_REPOSITORY']
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repository):
        raise ValueError('invalid repository')
    api = os.environ.get('GITHUB_API_URL', 'https://api.github.com').rstrip('/')
    if not api.startswith('https://'):
        raise ValueError('GitHub API must use HTTPS')
    request = urllib.request.Request(
        f'{api}/repos/{repository}/contents/{HISTORY_PATH}?ref=ci-reports',
        headers={'Authorization': 'Bearer ' + os.environ['GH_TOKEN'],
                 'Accept': 'application/vnd.github+json',
                 'X-GitHub-Api-Version': '2022-11-28'})
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            envelope = json.load(response)
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return {'schema': 1, 'runs': []}
        raise
    return validate_history(json.loads(base64.b64decode(envelope['content'], validate=False)))


def should_run(history, kind, revision, force=False):
    required = ['nightly'] if kind == 'nightly' else ['benchmarks', 'macos', 'windows']
    if force:
        return True
    # A failed latest run is retried even if an earlier run of this revision passed.
    for lane in required:
        latest = next((r for r in history['runs'] if r['lane'] == lane), None)
        if latest is None or latest['revision'] != revision or latest['status'] != 'success':
            return True
    return False


def cpu_model():
    if platform.system() == 'Linux':
        for line in Path('/proc/cpuinfo').read_text().splitlines():
            if line.startswith('model name'):
                return line.split(':', 1)[1].strip()
    return platform.processor() or platform.machine()


def environment():
    return {'platform': platform.platform(), 'architecture': platform.machine(),
            'cpu': cpu_model(), 'runner_image': os.environ.get('ImageOS', 'local'),
            'runner_image_version': os.environ.get('ImageVersion', 'unknown')}


def number(value):
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value) or value < 0:
        raise ValueError('invalid benchmark measurement')
    return value


def metric(name, value, context, env, configuration=None):
    compatible = {'platform': env['platform'], 'cpu': env['cpu'],
                  'architecture': env['architecture'], 'runner_image': env['runner_image'],
                  'context': context, 'name': name, 'configuration': configuration, 'metric_schema': 1}
    return {'name': name, 'value': number(value), 'unit': 'ms', 'context': context,
            'configuration': configuration,
            'comparison_key': hashlib.sha256(json.dumps(compatible, sort_keys=True).encode()).hexdigest()}


def benchmark_metrics(root, env, revision=None):
    metrics = []
    flight = root / 'flight-profile.json'
    fragments = root / 'fragments' / 'summary.json'
    # Missing/malformed outputs are failures, not an empty successful chart.
    report = read_json(flight)
    from profile_flight import rows
    rows(report)
    if revision and (report['environment']['source_revision'] != revision or report['environment']['source_dirty']):
        raise ValueError('flight benchmark source provenance mismatch')
    for row in report['rows']:
        if row['cpu_update']['samples'] < 100:
            raise ValueError('flight profile requires 100 samples per row')
        metrics.append(metric('Flight update p95', row['cpu_update']['p95_ms'],
                              f"{report['fixture']} / {row['count']} fragments / {row['workload']}", env,
                              {'fixture': report['fixture'], 'count': row['count'], 'workload': row['workload'],
                               'samples': row['cpu_update']['samples']}))
    summary = read_json(fragments)
    if revision and (summary['revision'] != revision or summary['dirty'] or
                     summary['binary_sha256'] != report['environment']['binary_sha256']):
        raise ValueError('fragment benchmark source/binary provenance mismatch')
    if summary.get('schema') != 1 or not summary.get('cases') or summary.get('failures'):
        raise ValueError('incomplete fragment benchmark')
    expected = {(count, layout, state) for count in (500, 1000)
                for layout in ('scattered', 'dense') for state in ('settled', 'active')}
    observed = set()
    for row in summary['cases']:
        config = row['config']
        state = {'active': 'active', 'settled_requested': 'settled'}[config['initial_state']]
        key = (config['count'], config['layout'], state)
        if key in observed or key not in expected or row['mode'] != 'physics_wall_clock':
            raise ValueError('unexpected or duplicate fragment benchmark case')
        observed.add(key)
        if row['samples_retained'] < 100 or not row['tail_statistics_reliable']:
            raise ValueError('fragment benchmark requires 100 samples per case')
        measured = row['metrics']['cpu_update_ms']
        if measured is None:
            raise ValueError('missing CPU update measurement')
        state_label = 'settled requested' if state == 'settled' else 'initially active'
        metrics.append(metric('Fragment CPU update p95', measured['p95'],
                              f"physics / {key[0]} fragments / {key[1]} / {state_label} / seed {config['seed']}", env, config))
    if observed != expected:
        raise ValueError('missing fragment benchmark cases')
    return metrics


def collect(lane, revision, steps, required, root):
    if lane not in LANES or not re.fullmatch('[0-9a-f]{40}', revision):
        raise ValueError('invalid lane or source revision')
    checks = [{'name': key, 'status': steps.get(key, {}).get('outcome', 'missing')}
              for key in required]
    for check in checks:
        if check['name'] == 'appearance' and check['status'] == 'success':
            coverage = steps.get('appearance', {}).get('outputs', {}).get('coverage')
            check['status'] = coverage if coverage in ('success', 'pending') else 'missing'
    status = ('success' if checks and all(c['status'] == 'success' for c in checks)
              else 'pending' if checks and all(c['status'] in ('success', 'pending') for c in checks)
              else 'failure')
    env = environment()
    metrics, error = [], None
    if lane == 'benchmarks':
        try:
            metrics = benchmark_metrics(root, env, revision)
        except (OSError, ValueError, KeyError, TypeError) as failure:
            status, error = 'failure', str(failure)
    repository = os.environ['GITHUB_REPOSITORY']
    run_id, attempt = os.environ['GITHUB_RUN_ID'], os.environ['GITHUB_RUN_ATTEMPT']
    server = os.environ.get('GITHUB_SERVER_URL', 'https://github.com')
    return {'lane': lane, 'revision': revision, 'status': status,
            'run_id': run_id, 'attempt': attempt,
            'created_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'url': f'{server}/{repository}/actions/runs/{run_id}/attempts/{attempt}',
            'checks': checks, 'metrics': metrics, 'error': error, 'environment': env}


def select_reports(reports, expected, revision, run_id, attempt):
    """A failed-job rerun reuses successful jobs/artifacts from earlier attempts."""
    selected, seen = {}, set()
    for row in reports:
        observed_attempt = int(row['attempt'])
        if (row['lane'] not in expected or row['revision'] != revision or
                row['run_id'] != run_id or not 1 <= observed_attempt <= int(attempt)):
            raise ValueError('report provenance mismatch')
        identity = (row['lane'], observed_attempt)
        if identity in seen:
            raise ValueError('duplicate suite reports for the same attempt')
        seen.add(identity)
        previous = selected.get(row['lane'])
        if previous is None or observed_attempt > int(previous['attempt']):
            selected[row['lane']] = row
    return [selected[lane] for lane in expected if lane in selected]


def merge_history(history, reports):
    validate_history(history)
    identities = {(r['lane'], r['run_id'], r['attempt']) for r in reports}
    rows = reports + [r for r in history['runs'] if (r['lane'], r['run_id'], r['attempt']) not in identities]
    rows.sort(key=lambda r: (r['created_at'], int(r['run_id']), int(r['attempt'])), reverse=True)
    return validate_history({'schema': 1, 'runs': rows[:MAX_RUNS]})


def summary(reports):
    lines = ['## Salimon periodic verification', '', '| Suite | Result | Revision | Details |',
             '| --- | --- | --- | --- |']
    for row in reports:
        lines.append(f"| {row['lane']} | {row['status']} | `{row['revision'][:8]}` | [Run + artifacts]({row['url']}) |")
    for row in reports:
        lines += ['', '### ' + row['lane'], '', '| Check | Outcome |', '| --- | --- |']
        lines += [f"| {check['name']} | {check['status']} |" for check in row['checks']]
        if row.get('error'):
            # JSON errors can contain arbitrary source text; keep it out of Markdown markup.
            lines += ['', 'Report error: ' + row['error'].replace('\n', ' ').replace('<', '&lt;')]
    lines += ['', 'Download **ci-dashboard** for the standalone dashboard; raw logs/screenshots are in suite artifacts.',
              'Hosted CPU diagnostics do not certify native Apple M1 FPS.']
    return '\n'.join(lines) + '\n'


def render_site(history, destination):
    validate_history(history)
    destination.mkdir(parents=True, exist_ok=True)
    write_json(destination / HISTORY_PATH, history)
    # Inline data makes the downloaded dashboard work from file:// as well as Pages.
    embedded = json.dumps(history, allow_nan=False).replace('<', '\\u003c').replace('&', '\\u0026')
    template = (ROOT / 'scripts/ci_dashboard/index.html').read_text(encoding='utf-8')
    (destination / 'index.html').write_text(template.replace('__CI_HISTORY__', embedded), encoding='utf-8')
    for name in ('dashboard.js', 'dashboard.css'):
        shutil.copy2(ROOT / 'scripts/ci_dashboard' / name, destination / name)
    (destination / '.nojekyll').write_text('')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    plan = sub.add_parser('plan')
    plan.add_argument('--kind', choices=('nightly', 'weekly'), required=True)
    plan.add_argument('--force', action='store_true')
    record = sub.add_parser('record')
    record.add_argument('--lane', choices=sorted(LANES), required=True)
    record.add_argument('--required', nargs='+', required=True)
    record.add_argument('--root', type=Path, default=Path('artifacts/periodic'))
    publish = sub.add_parser('site')
    publish.add_argument('--kind', choices=('nightly', 'weekly'), required=True)
    publish.add_argument('--input', type=Path, required=True)
    publish.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    revision = os.environ['SOURCE_REVISION']
    if args.command == 'plan':
        run = should_run(remote_history(), args.kind, revision, args.force)
        with open(os.environ['GITHUB_OUTPUT'], 'a', encoding='utf-8') as output:
            output.write(f'run={str(run).lower()}\nsha={revision}\nkind={args.kind}\n')
        message = 'Run suites' if run else 'No source change since the latest successful suites; skipping'
        print(message)
        with open(os.environ['GITHUB_STEP_SUMMARY'], 'a', encoding='utf-8') as output:
            output.write(f'## Periodic verification plan\n\n{message}. Suite: **{args.kind}**; revision `{revision[:8]}`.\n')
    elif args.command == 'record':
        row = collect(args.lane, revision, json.loads(os.environ['CI_STEPS']), args.required, args.root)
        write_json(args.root / f'report-{args.lane}.json', row)
        with open(os.environ['GITHUB_STEP_SUMMARY'], 'a', encoding='utf-8') as output:
            output.write(summary([row]))
        # Preserve the report even on failure; workflow job remains failed as well.
        return int(row['status'] != 'success')
    else:
        expected = ['nightly'] if args.kind == 'nightly' else ['benchmarks', 'macos', 'windows']
        reports = [read_json(p) for p in args.input.rglob('report-*.json')]
        reports = select_reports(reports, expected, revision, os.environ['GITHUB_RUN_ID'],
                                 os.environ['GITHUB_RUN_ATTEMPT'])
        for lane in expected:
            if not any(r['lane'] == lane for r in reports):
                missing = collect(lane, revision, {}, ['suite-report'], Path('/missing'))
                missing['error'] = 'Suite report missing; inspect the workflow job logs.'
                reports.append(missing)
        history = merge_history(remote_history(), reports)
        render_site(history, args.output)
        with open(os.environ['GITHUB_STEP_SUMMARY'], 'a', encoding='utf-8') as output:
            output.write(summary(reports))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, TypeError, urllib.error.URLError) as error:
        raise SystemExit(f'CI reporting failed: {error}')
