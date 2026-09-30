"""Lifecycle regression contracts; fixtures do not claim native GPU validation."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import packaged_smoke as smoke


@unittest.skipIf(sys.platform == 'win32', 'POSIX executable fixture')
class SmokeContracts(unittest.TestCase):
    def fixture(self, root, mode='ready'):
        binary = root / 'game with spaces'
        binary.write_text(f'''#!/usr/bin/env python3
import sys, time
from pathlib import Path
Path({str(root / 'cwd')!r}).write_text(str(Path.cwd()))
print("native window created:", file=sys.stderr, flush=True)
{'sys.exit(7)' if mode == 'crash' else ''}
print("renderer ready:", file=sys.stderr, flush=True)
{'print("render attempt 1 completed with Presented", file=sys.stderr, flush=True)' if mode == 'ready' else ''}
while True:
    time.sleep(.01)
    signal = Path({str(root / 'key')!r})
    if signal.exists():
        key = signal.read_text()
        signal.unlink()
        if key == 'Escape':
            print("application exiting cleanly", file=sys.stderr, flush=True)
            sys.exit(0)
        print("view mode changed to " + key, file=sys.stderr, flush=True)
''')
        binary.chmod(0o755)
        return binary

    def test_success_real_process_cleanup_and_independent_cwd(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            binary = self.fixture(root)
            modes = iter(['PrecisionTour', 'Gameplay'])
            def key(pid, value):
                (root / 'key').write_text('Escape' if value == 'Escape' else next(modes))
            with patch.object(smoke, 'input_key', side_effect=key), \
                    patch.object(smoke.Artifacts, 'screenshot', return_value={'status': 'captured'}):
                report = smoke.run(binary, root / 'evidence', 3)
            self.assertTrue(report['success'], report)
            self.assertIsNotNone(report['process_exit_code'])
            self.assertNotEqual((root / 'cwd').read_text(), str(binary.parent))
            self.assertEqual(len(report['checks']), 6)
            self.assertEqual(json.loads((Path(report['artifacts']) / 'result.json').read_text()), report)

    def test_crash_and_missing_present_fail_with_logs(self):
        for mode in ('crash', 'no-present'):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                with patch.object(smoke.Artifacts, 'screenshot', return_value={'status': 'unavailable'}):
                    report = smoke.run(self.fixture(root, mode), root / 'evidence', .3)
                self.assertFalse(report['success'])
                self.assertIn('error', report)
                self.assertIsNotNone(report['process_exit_code'])
                self.assertTrue((Path(report['artifacts']) / 'stderr.log').exists())

    def test_input_and_capture_failures_do_not_pass(self):
        for capture in ('captured', 'failed'):
            with self.subTest(capture=capture), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                with patch.object(smoke.Artifacts, 'screenshot', return_value={'status': capture}), \
                        patch.object(smoke, 'input_key', side_effect=RuntimeError('input unavailable')):
                    report = smoke.run(self.fixture(root), root / 'evidence', 3)
                self.assertFalse(report['success'])
                self.assertIsNotNone(report['process_exit_code'])
                self.assertIn('failure_screenshot', report)


class PortableContracts(unittest.TestCase):
    def test_missing_binary_emits_nonzero_and_result(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = subprocess.run([sys.executable, str(Path(smoke.__file__)), '--binary',
                                     str(Path(tmp) / 'missing'), '--artifacts', tmp],
                                    capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 1)
            self.assertFalse(json.loads(result.stdout)['success'])
            self.assertEqual(len(list(Path(tmp).glob('run-*/result.json'))), 1)

    def test_linux_missing_input_tool_is_explicit(self):
        with patch.object(smoke.sys, 'platform', 'linux'), \
                patch.dict(smoke.os.environ, {'DISPLAY': ':123'}), \
                patch.object(smoke.shutil, 'which', return_value=None):
            with self.assertRaisesRegex(RuntimeError, 'xdotool'):
                smoke.input_key(123, 'F2')
