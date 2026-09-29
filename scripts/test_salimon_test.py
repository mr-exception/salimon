"""Contract tests for the scenario runner without requiring a GPU."""

import json
import sys
import tempfile
import time
import unittest
from pathlib import Path

from salimon_test import Game, ScenarioError, parse_scenario, run


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
        return run(self.scenario, [sys.executable, str(self.fake)])

    def test_parser_rejects_unknown_and_ambiguous_steps(self):
        self.data["steps"][0]["assert"] = {"path": "ship.speed", "equals": 4}
        self.save()
        with self.assertRaises(ScenarioError):
            parse_scenario(self.scenario)
        self.data["steps"] = [{"action": {"op": "warp"}}]
        self.save()
        with self.assertRaises(ScenarioError):
            parse_scenario(self.scenario)

    def test_success_and_process_cleanup(self):
        self.assertEqual(self.execute()["status"], "passed")
        with Game([sys.executable, str(self.fake)], parse_scenario(self.scenario)["setup"], time.monotonic() + 2) as game:
            self.assertIsNone(game.process.poll())
            game.request({"op": "inspect"})
        self.assertIsNotNone(game.process.poll())

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


if __name__ == "__main__":
    unittest.main()
