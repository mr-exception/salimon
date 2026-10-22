import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from experience_suite import run


class ExperienceSuite(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.path = self.root / 'matrix.json'
        self.case = dict(id='SCOUT-01', invariant='physical gate', portable_test='module::tests::gate',
                         native_protocol='reference recording required')
        self.matrix = dict(format='salimon.experience-validation', version=1, cases=[self.case])
        self.path.write_text(json.dumps(self.matrix))

    def execute(self, output, code=0):
        def fake(command, **kwargs):
            if command[0] == 'git':
                return subprocess.CompletedProcess(command, 0, 'a' * 40, '')
            self.assertIn('--exact', command)
            self.assertIn('module::tests::gate', command)
            return subprocess.CompletedProcess(command, code, output, 'retained stderr')
        return fake

    def test_zero_selected_tests_and_real_failure_never_pass(self):
        for n, output, code in [(0, 'test result: ok. 0 passed;', 0),
                                 (1, 'test result: FAILED. 1 failed;', 101)]:
            dest = self.root / str(n)
            self.assertEqual(run(self.path, dest, self.execute(output, code)), 1)
            result = json.loads((dest / 'results.json').read_text())
            self.assertEqual(result['status'], 'failed')
            self.assertIn('retained stderr', (dest / 'SCOUT-01.log').read_text())

    def test_portable_pass_still_leaves_native_and_backend_pending(self):
        self.matrix['cases'].append(dict(id='HALL-01', invariant='permanent account unlock',
                                        blocked_by=[473, 474], native_protocol='real backend required'))
        self.path.write_text(json.dumps(self.matrix))
        dest = self.root / 'run'
        self.assertEqual(run(self.path, dest, self.execute('test result: ok. 1 passed;')), 3)
        result = json.loads((dest / 'results.json').read_text())
        self.assertEqual(result['status'], 'pending')
        self.assertEqual(result['cases'][1]['status'], 'blocked')
        self.assertFalse(result['native_signoff'])
        self.assertFalse(result['backend_signoff'])
        with self.assertRaises(FileExistsError):
            run(self.path, dest, self.execute('test result: ok. 1 passed;'))

    def test_unsafe_ids_and_missing_blockers_reject_before_execution(self):
        for bad in [{**self.case, 'id': '../escape'},
                    {**self.case, 'portable_test': '--help'},
                    {**self.case, 'portable_test': None}]:
            self.matrix['cases'] = [bad]
            self.path.write_text(json.dumps(self.matrix))
            with self.assertRaises(ValueError):
                run(self.path, self.root / 'out', self.execute('unused'))
            self.assertFalse((self.root / 'out').exists())
