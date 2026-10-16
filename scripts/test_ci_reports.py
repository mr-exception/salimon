"""CI report failure semantics, history, scheduling and dashboard isolation."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
import urllib.error
from unittest.mock import patch

import ci_reports as ci
from test_profile_flight import report as flight_report

REVISION = 'a' * 40
ENV = {'GITHUB_REPOSITORY': 'example/game', 'GITHUB_RUN_ID': '42',
       'GITHUB_RUN_ATTEMPT': '1', 'SOURCE_REVISION': REVISION}


def row(lane='nightly', status='success', revision=REVISION, run='42', attempt='1'):
    return {'lane': lane, 'status': status, 'revision': revision,
            'run_id': run, 'attempt': attempt, 'created_at': '2026-10-16T00:00:00Z',
            'url': 'https://github.com/example/game/actions/runs/42',
            'checks': [{'name': 'suite', 'status': status}], 'metrics': [],
            'environment': {'cpu': 'test CPU'}}


def fragment_report():
    return {'schema': 1, 'cases': [
        {'mode': 'physics_wall_clock', 'samples_retained': 100,
         'tail_statistics_reliable': True,
         'config': {'count': count, 'layout': layout, 'initial_state': state, 'seed': 155},
         'metrics': {'cpu_update_ms': {'p95': 1.25}}}
        for count in (500, 1000) for layout in ('scattered', 'dense')
        for state in ('settled_requested', 'active')], 'failures': []}


class CiReportTests(unittest.TestCase):
    def test_skip_only_latest_successful_unchanged_complete_lane_set(self):
        history = {'schema': 1, 'runs': [row()]}
        self.assertFalse(ci.should_run(history, 'nightly', REVISION))
        self.assertTrue(ci.should_run(history, 'nightly', REVISION, True))
        self.assertTrue(ci.should_run(history, 'nightly', 'b' * 40))
        history['runs'].insert(0, row(status='failure'))
        self.assertTrue(ci.should_run(history, 'nightly', REVISION))
        history['runs'] = [row(lane=lane) for lane in ('benchmarks', 'macos', 'windows')]
        self.assertFalse(ci.should_run(history, 'weekly', REVISION))
        history['runs'].pop()
        self.assertTrue(ci.should_run(history, 'weekly', REVISION))

    def test_network_errors_are_not_treated_as_first_run(self):
        with patch.dict(os.environ, {**ENV, 'GH_TOKEN': 'fixture'}):
            for code in (404, 403, 500):
                error = urllib.error.HTTPError('https://api.github.com', code, 'fixture', {}, None)
                with patch.object(ci.urllib.request, 'urlopen', side_effect=error):
                    if code == 404:
                        self.assertEqual(ci.remote_history()['runs'], [])
                    else:
                        with self.assertRaises(urllib.error.HTTPError):ci.remote_history()

    def test_missing_failed_and_skipped_steps_never_pass(self):
        with patch.dict(os.environ, ENV):
            for outcome in ('failure', 'skipped', 'cancelled', None):
                steps = {'build': {'outcome': outcome}} if outcome else {}
                result = ci.collect('windows', REVISION, steps, ['build'], Path('.'))
                self.assertEqual(result['status'], 'failure')
            result = ci.collect('windows', REVISION, {'build': {'outcome': 'success'}}, ['build'], Path('.'))
            self.assertEqual(result['status'], 'success')

    def test_metrics_require_real_complete_samples_and_keep_hardware_separate(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            flight = flight_report()
            for r in flight['rows']:
                r['cpu_update']['samples'] = 100
                r['cpu_update']['raw_ms'] *= 5
                r['moving_objects'] *= 5
            flight['environment'] = {'source_revision': REVISION, 'source_dirty': False, 'binary_sha256': 'same'}
            ci.write_json(root/'flight-profile.json', flight)
            fragments = fragment_report()
            fragments.update(revision=REVISION, dirty=False, binary_sha256='same')
            ci.write_json(root/'fragments/summary.json', fragments)
            env = {'platform': 'Linux', 'cpu': 'CPU A', 'architecture': 'x86_64', 'runner_image': 'ubuntu24'}
            metrics = ci.benchmark_metrics(root, env, REVISION)
            with self.assertRaisesRegex(ValueError, 'provenance'):
                ci.benchmark_metrics(root, env, 'b'*40)
            self.assertEqual(len(metrics), 20)
            self.assertTrue(all(m['unit'] == 'ms' and 'FPS' not in m['name'] for m in metrics))
            other = ci.metric(metrics[0]['name'], metrics[0]['value'], metrics[0]['context'], {**env, 'cpu': 'CPU B'})
            self.assertNotEqual(metrics[0]['comparison_key'], other['comparison_key'])
            for mutate in (lambda d:d['cases'].pop(),
                           lambda d:d['cases'][0].__setitem__('samples_retained', 99),
                           lambda d:d['cases'][0]['metrics']['cpu_update_ms'].__setitem__('p95', float('inf')),
                           lambda d:d['failures'].append({'error': 'fixture'})):
                data = fragment_report();mutate(data)
                # Deliberately permit nonfinite JSON to exercise the ingestion boundary.
                (root/'fragments/summary.json').write_text(json.dumps(data))
                with self.assertRaises(ValueError):ci.benchmark_metrics(root, env)

    def test_missing_benchmarks_preserve_failure_report(self):
        with patch.dict(os.environ, ENV):
            result = ci.collect('benchmarks', REVISION, {'build': {'outcome': 'success'}}, ['build'], Path('/does-not-exist'))
            self.assertEqual(result['status'], 'failure')
            self.assertIsNotNone(result['error'])
            self.assertEqual(result['metrics'], [])

    def test_retries_deduplicate_bound_history_and_do_not_erase_failure(self):
        history = {'schema': 1, 'runs': [row(run=str(i)) for i in range(100)]}
        failure = row(status='failure', run='42')
        combined = ci.merge_history(history, [failure])
        self.assertEqual(len(combined['runs']), ci.MAX_RUNS)
        matching = [r for r in combined['runs'] if r['run_id'] == '42']
        self.assertEqual(matching, [failure])
        retry = row(attempt='2')
        self.assertEqual(len([r for r in ci.merge_history(combined, [retry])['runs'] if r['run_id']=='42']), 2)

    def test_site_is_standalone_and_inline_json_cannot_escape_script(self):
        report = row();report['error'] = '</script><script>alert(1)</script>'
        with tempfile.TemporaryDirectory() as tmp:
            ci.render_site({'schema': 1, 'runs': [report]}, Path(tmp))
            html = (Path(tmp)/'index.html').read_text()
            self.assertNotIn('__CI_HISTORY__', html)
            self.assertNotIn(report['error'], html)
            self.assertIn('\\u003c/script', html)
            self.assertTrue((Path(tmp)/'dashboard.js').exists())
            self.assertEqual(ci.read_json(Path(tmp)/ci.HISTORY_PATH)['runs'][0]['error'], report['error'])

    def test_report_branch_commit_contains_only_site_and_keeps_ancestry(self):
        # Exercise the publishing commands on an isolated local repository twice.
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            def git(*args):return subprocess.check_output(['git', *args], cwd=root, text=True).strip()
            git('init','-q');git('config','user.name','fixture');git('config','user.email','fixture@example.test')
            (root/'source.rs').write_text('source');git('add','.');git('commit','-qm','source')
            site=root/'artifacts/site';site.mkdir(parents=True);(site/'index.html').write_text('first')
            git('checkout','--orphan','ci-reports')
            git('--work-tree=artifacts/site','add','-A','--','.')
            git('--work-tree=artifacts/site','commit','-qm','report')
            first=git('rev-parse','HEAD')
            self.assertEqual(git('ls-tree','--name-only','HEAD'),'index.html')
            (site/'index.html').write_text('second')
            git('--work-tree=artifacts/site','add','-A','--','.')
            git('--work-tree=artifacts/site','commit','-qm','report again')
            self.assertEqual(git('rev-parse','HEAD^'),first)
            self.assertEqual(git('show','HEAD:index.html'),'second')


    def test_site_missing_report_is_failure_and_wrong_provenance_rejected(self):
        import sys
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            incoming = root/'incoming';incoming.mkdir()
            site = root/'site'
            environment = {**ENV, 'GITHUB_STEP_SUMMARY': str(root/'summary.md')}
            argv = ['ci_reports', 'site', '--kind', 'nightly', '--input', str(incoming), '--output', str(site)]
            with patch.dict(os.environ, environment), patch.object(sys, 'argv', argv), patch.object(ci, 'remote_history', return_value={'schema': 1, 'runs': []}):
                self.assertEqual(ci.main(), 0)
                self.assertEqual(ci.read_json(site/ci.HISTORY_PATH)['runs'][0]['status'], 'failure')
                invalid = row(revision='b'*40)
                ci.write_json(incoming/'report-nightly.json', invalid)
                with self.assertRaisesRegex(ValueError, 'provenance'):ci.main()

    def test_plan_outputs_and_manual_force(self):
        import sys
        with tempfile.TemporaryDirectory() as tmp:
            for force, expected in ((False, 'false'), (True, 'true')):
                root = Path(tmp)
                output = root/('output-'+expected)
                env = {**ENV, 'GITHUB_OUTPUT': str(output), 'GITHUB_STEP_SUMMARY': str(root/'summary')}
                argv = ['ci_reports', 'plan', '--kind', 'nightly'] + (['--force'] if force else [])
                with patch.dict(os.environ, env), patch.object(sys, 'argv', argv), patch.object(ci, 'remote_history', return_value={'schema': 1, 'runs': [row()]}):
                    self.assertEqual(ci.main(), 0)
                outputs = dict(line.split('=', 1) for line in output.read_text().splitlines())
                self.assertEqual(outputs, {'run': expected, 'sha': REVISION, 'kind': 'nightly'})

    def test_multi_suite_summary_keeps_separate_result_and_check_tables(self):
        text = ci.summary([row(lane='benchmarks'), row(lane='macos')])
        self.assertLess(text.index('| macos |'), text.index('### benchmarks'))
        self.assertIn('### macos\n\n| Check | Outcome |', text)

    def test_failed_job_reruns_reuse_prior_passing_lanes_and_latest_attempt_wins(self):
        older = row(lane='benchmarks', status='failure')
        newer = row(lane='benchmarks', attempt='2')
        mac = row(lane='macos')
        selected = ci.select_reports([older, mac, newer], ['benchmarks', 'macos'], REVISION, '42', '2')
        self.assertEqual(selected, [newer, mac])
        with self.assertRaisesRegex(ValueError, 'provenance'):
            ci.select_reports([newer], ['benchmarks'], REVISION, '42', '1')
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            ci.select_reports([newer, newer], ['benchmarks'], REVISION, '42', '2')
