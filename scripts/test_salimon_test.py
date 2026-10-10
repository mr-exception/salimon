"""Contract tests for the scenario runner without requiring a GPU."""

import json
import sys
import tempfile
import time
import unittest
from pathlib import Path

from salimon_test import Game, ScenarioError, matches, parse_scenario, run


FAKE = '''import json, os, sys, time
mode = os.environ.get("FAKE_MODE", "normal")
if mode == "no_ready":
    time.sleep(10)
    sys.exit(0)
setup = dict(zip(sys.argv[2::2], sys.argv[3::2]))
print("SALIMON_E2E_READY scenario=%s seed=%s step_ms=%s" % (setup["--scenario"], setup["--seed"], setup["--step-ms"]), flush=True)
print(json.dumps({"protocol": 1, "event": "ready", "scenario": setup["--scenario"], "seed": int(setup["--seed"]), "step_ms": int(setup["--step-ms"]), "timeout_ms": 5000}), flush=True)
for line in sys.stdin:
    request = json.loads(line)
    if mode == "no_response":
        time.sleep(10)
    if mode == "wrong_id":
        request["id"] += 1
    if mode == "rejected":
        reply = {"protocol": 1, "id": request["id"], "ok": False, "error": {"code": "bad", "message": "rejected"}}
    else:
        reply = {"protocol": 1, "id": request["id"], "ok": True, "result": {"ship": {"speed": 4}} if request["op"] == "inspect" else {}}
    print(json.dumps(reply), flush=True)
'''


class RunnerTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.directory = Path(self.tmp.name)
        self.fake = self.directory / "fake.py"
        self.fake.write_text(FAKE)
        self.scenario = self.directory / "case.json"
        self.data = {"setup": {"scenario": "orbit-earth", "seed": 3}, "timeout_ms": 1000,
                     "steps": [{"wait": {"path": "ship.speed", "gte": 4}},
                               {"assert": {"path": "ship.speed", "approx": 4, "tolerance": 0.1}}]}
        self.save()

    def save(self):
        self.scenario.write_text(json.dumps(self.data))

    def execute(self):
        return run(self.scenario, [sys.executable, str(self.fake)], self.directory / "artifacts", [])

    def test_parser_rejects_unknown_and_ambiguous_steps(self):
        self.data["steps"][0]["assert"] = {"path": "ship.speed", "equals": 4}
        self.save()
        with self.assertRaises(ScenarioError):
            parse_scenario(self.scenario)
        self.data["steps"] = [{"action": {"op": "warp"}}]
        self.save()
        with self.assertRaises(ScenarioError):
            parse_scenario(self.scenario)

    def test_left_mouse_action_is_accepted_and_forwarded(self):
        self.data["steps"] = [{"action": {"op": "mouse", "button": "left", "pressed": True}},
                              {"action": {"op": "mouse", "button": "left", "pressed": False}}]
        self.save()
        self.assertEqual(self.execute()["status"], "passed")

    def test_success_and_process_cleanup(self):
        self.assertEqual(self.execute()["status"], "passed")
        with Game([sys.executable, str(self.fake)], parse_scenario(self.scenario)["setup"], time.monotonic() + 2) as game:
            self.assertIsNone(game.process.poll())
            game.request({"op": "inspect"})
        self.assertIsNotNone(game.process.poll())

    def test_fragment_load_startup_and_argument_forwarding(self):
        self.data["setup"].update(fragment_count=500, fragment_layout="scattered")
        self.save()
        self.fake.write_text(FAKE.replace(
            'print("SALIMON_E2E_READY',
            'assert setup["--fragment-count"] == "500"\n'
            'assert setup["--fragment-layout"] == "scattered"\n'
            'print("SALIMON_E2E_READY', 1))
        self.assertEqual(self.execute()["status"], "passed")

    def test_failed_assertion_and_missing_path(self):
        self.data["steps"] = [{"assert": {"path": "ship.speed", "equals": 8}}]
        self.save()
        self.assertIn("assertion failed", self.execute()["error"])
        self.data["steps"][0]["assert"]["path"] = "ship.missing"
        self.save()
        self.assertIn("missing inspection path", self.execute()["error"])

    def test_startup_timeout_and_cleanup(self):
        self.fake.write_text("import time\ntime.sleep(10)\n")
        result = self.execute()
        self.assertEqual(result["status"], "failed")
        self.assertIn("timeout waiting for ready signal", result["error"])
        self.assertLess(result["duration_ms"], 4000)

    def test_command_timeout(self):
        self.fake.write_text(FAKE.replace('mode = os.environ.get("FAKE_MODE", "normal")', 'mode = "no_response"'))
        self.data["steps"][0]["timeout_ms"] = 100
        self.save()
        self.assertIn("timeout waiting for response", self.execute()["error"])

    def test_protocol_mismatch_and_rejection(self):
        for mode, expected in (("wrong_id", "invalid protocol response"), ("rejected", "game rejected")):
            self.fake.write_text(FAKE.replace('mode = os.environ.get("FAKE_MODE", "normal")', f'mode = "{mode}"'))
            self.assertIn(expected, self.execute()["error"])

    def test_result_contains_evidence_and_full_logs(self):
        self.fake.write_text(FAKE.replace('for line in sys.stdin:',
                                        'for i in range(100): print("diagnostic %s" % i, file=sys.stderr, flush=True)\nfor line in sys.stdin:'))
        result = self.execute()
        directory = Path(result["artifact_directory"])
        persisted = json.loads((directory / "result.json").read_text())
        self.assertEqual(persisted, result)
        self.assertTrue(result["steps"][1]["assertion"]["passed"])
        self.assertEqual(result["steps"][1]["assertion"]["actual"], 4)
        self.assertEqual(result["steps"][1]["state"]["ship"]["speed"], 4)
        self.assertIn("diagnostic 0\n", (directory / "stderr.log").read_text())
        self.assertIn("diagnostic 99\n", (directory / "stderr.log").read_text())
        self.assertIn("SALIMON_E2E_READY", (directory / "stdout.log").read_text())
        self.assertIn('"direction": "request"', (directory / "protocol.jsonl").read_text())
        self.assertTrue((directory / "step-002.json").exists())
        self.assertNotEqual(result["artifact_directory"], self.execute()["artifact_directory"])

    def test_failure_records_actual_and_last_state(self):
        self.data["steps"] = [{"assert": {"path": "ship.speed", "equals": 8}}]
        self.save()
        result = self.execute()
        self.assertEqual(result["steps"][0]["assertion"]["actual"], 4)
        self.assertFalse(result["steps"][0]["assertion"]["passed"])
        self.assertEqual(result["failure_screenshot"]["status"], "unavailable")
        self.assertTrue((Path(result["artifact_directory"]) / "failure-state.json").exists())

    def test_parse_and_launch_failures_preserve_results(self):
        self.scenario.write_text("{")
        result = self.execute()
        self.assertEqual(result["status"], "failed")
        self.assertTrue(Path(result["result_file"]).exists())
        self.save()
        result = run(self.scenario, [str(self.directory / "missing")], self.directory / "artifacts", [])
        self.assertEqual(result["status"], "failed")
        self.assertTrue(Path(result["result_file"]).exists())

    def test_existence_and_collection_contracts(self):
        state = {"entities": ["ship", "player"], "interaction": None}
        self.assertTrue(matches(state, {"path": "entities", "contains": "ship"}))
        self.assertFalse(matches(state, {"path": "entities", "contains": "cargo"}))
        self.assertTrue(matches(state, {"path": "interaction", "exists": True}))
        self.assertTrue(matches(state, {"path": "absent", "exists": False}))
        self.assertTrue(matches(state, {"path": "entities.0", "equals": "ship"}))
        with self.assertRaises(ScenarioError):
            matches(state, {"path": "interaction", "contains": "ship"})

    def test_capture_checkpoint_and_failure_before_termination(self):
        # The helper is local trusted argv, never taken from scenario JSON.
        capture = self.directory / "capture.py"
        pid_file = self.directory / "pid"
        self.fake.write_text("import pathlib, os\npathlib.Path(%r).write_text(str(os.getpid()))\n" % str(pid_file) + FAKE)
        capture.write_text('''import pathlib, os, sys
pid = int(pathlib.Path(%r).read_text())
if os.name == "nt":
    # Windows signal 0 sends CTRL_C_EVENT; query the process without signalling.
    import ctypes
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.OpenProcess.argtypes = [ctypes.c_uint32, ctypes.c_int, ctypes.c_uint32]
    kernel.OpenProcess.restype = ctypes.c_void_p
    kernel.WaitForSingleObject.argtypes = [ctypes.c_void_p, ctypes.c_uint32]
    kernel.WaitForSingleObject.restype = ctypes.c_uint32
    kernel.CloseHandle.argtypes = [ctypes.c_void_p]
    handle = kernel.OpenProcess(0x00100000, False, pid)
    assert handle, "game process unavailable"
    try:
        assert kernel.WaitForSingleObject(handle, 0) == 258, "game already terminated"
    finally:
        kernel.CloseHandle(handle)
else:
    os.kill(pid, 0)
pathlib.Path(sys.argv[1]).write_bytes(b"\\x89PNG\\r\\n\\x1a\\n" + b"fake")
''' % str(pid_file))
        self.data["steps"] = [{"screenshot": "cockpit"}, {"assert": {"path": "ship.speed", "equals": 8}}]
        self.save()
        result = run(self.scenario, [sys.executable, str(self.fake)], self.directory / "artifacts",
                     [sys.executable, str(capture), "{path}"])
        self.assertEqual(result["steps"][0]["screenshot"]["status"], "captured")
        self.assertEqual(result["failure_screenshot"]["status"], "captured")
        self.assertTrue(Path(result["failure_screenshot"]["path"]).exists())
        self.assertIsNotNone(result["game_exit_code"])

    def test_unavailable_and_failed_checkpoints_fail_explicitly(self):
        self.data["steps"] = [{"screenshot": "view"}]
        self.save()
        self.assertEqual(self.execute()["status"], "failed")
        result = run(self.scenario, [sys.executable, str(self.fake)], self.directory / "artifacts",
                     [sys.executable, "-c", "raise SystemExit(2)", "{path}"])
        self.assertEqual(result["steps"][0]["screenshot"]["status"], "failed")
        self.assertIn("exited 2", result["steps"][0]["screenshot"]["reason"])

    def test_capture_timeout_and_invalid_png(self):
        self.data["steps"] = [{"screenshot": "view", "timeout_ms": 100}]
        self.save()
        result = run(self.scenario, [sys.executable, str(self.fake)], self.directory / "artifacts",
                     [sys.executable, "-c", "import time; time.sleep(0.2)", "{path}"])
        self.assertEqual(result["status"], "failed")
        self.assertIn("timed out", result["steps"][0]["screenshot"]["reason"])
        result = run(self.scenario, [sys.executable, str(self.fake)], self.directory / "artifacts",
                     [sys.executable, "-c", "import pathlib,sys; pathlib.Path(sys.argv[1]).write_text('bad')", "{path}"])
        self.assertIn("did not produce a PNG", result["steps"][0]["screenshot"]["reason"])

    def test_checkpoint_names_and_exists_are_validated(self):
        for step in ({"screenshot": "../escape"}, {"screenshot": ""},
                     {"assert": {"path": "ship", "exists": 1}}):
            self.data["steps"] = [step]
            self.save()
            with self.assertRaises(ScenarioError):
                parse_scenario(self.scenario)

    @unittest.skipIf(sys.platform == "win32", "CLI fixture requires a POSIX executable shebang")
    def test_cli_writes_report_and_propagates_failure_exit(self):
        executable = self.directory / "fake-client"
        executable.write_text(f"#!{sys.executable}\n" + FAKE)
        executable.chmod(0o755)
        import subprocess
        cli = Path(__file__).with_name("salimon-test")
        command = [sys.executable, str(cli), "run", str(self.scenario), "--binary", str(executable),
                   "--artifacts", str(self.directory / "artifacts"), "--screenshot-command", "[]"]
        success = subprocess.run(command, capture_output=True, text=True)
        self.assertEqual(success.returncode, 0, success.stderr)
        self.assertEqual(json.loads(success.stdout)["status"], "passed")
        self.data["steps"] = [{"assert": {"path": "ship.speed", "equals": 8}}]
        self.save()
        failure = subprocess.run(command, capture_output=True, text=True)
        self.assertEqual(failure.returncode, 1)
        report = json.loads(failure.stdout)
        self.assertEqual(report["status"], "failed")
        self.assertTrue(Path(report["results"][0]["result_file"]).exists())


if __name__ == "__main__":
    unittest.main()
