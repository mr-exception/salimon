"""Benchmark reporting contracts, including incompatible/failed baseline rejection."""
import copy
import sys
import unittest
import json
import tempfile
from unittest.mock import patch
from pathlib import Path
from salimon_test import parse_scenario, ScenarioError
from benchmark_fragments import compare, distribution, summarize, main, prepared_batch


class BenchmarkTests(unittest.TestCase):
    def result(self):
        return {"schema": 1, "mode": "physics_wall_clock", "config": {"count": 1},
            "accounting_preserved": True, "accounting": {"fragments": [{"id": 1, "mass_kg": 2}], "extracted_mass_kg": 2},
            "renderer": None, "samples": [{"frame_ms": None, "physics_total_ms": 2, "moving_objects": 0}]}

    def test_population_setup_retained_and_invalid_options_fail(self):
        root = Path(__file__).resolve().parents[1]
        setup = parse_scenario(root / "scenarios/fragment-load.json")["setup"]
        self.assertEqual(setup["fragment_count"], 500)
        self.assertEqual(setup["fragment_layout"], "scattered")
        for fragment in ({"fragment_count": True}, {"fragment_count": None}, {"fragment_count": 1001},
                         {"fragment_count": 12, "fragment_layout": []},
                         {"fragment_layout": "dense"}, {"fragment_count": 12, "fragment_layout": "bad"},
                         {"fragment_settle_steps": 10}, {"fragment_count": 12, "fragment_settle_steps": True},
                         {"fragment_count": 12, "fragment_settle_steps": 37501}):
            with tempfile.TemporaryDirectory() as temp:
                path = Path(temp) / "invalid.json"
                path.write_text(json.dumps({"setup": fragment, "steps": [{"action": {"op": "inspect"}}]}))
                with self.assertRaises(ScenarioError):
                    parse_scenario(path)

    def test_percentiles_keep_spikes(self):
        self.assertEqual(distribution(list(range(1, 101))), {"median": 50.5, "p95": 95, "p99": 99, "max": 100})
        self.assertIsNone(distribution([]))

    def test_cpu_cost_never_becomes_fps_or_gpu_time(self):
        summary = summarize(self.result())
        self.assertIsNone(summary["median_interval_fps"])
        self.assertIsNone(summary["metrics"]["gpu_latest_ms"])
        self.assertTrue(summary["settled_observed"])

    def test_sleep_counters_distinguish_velocity_rest_from_persistent_sleep(self):
        result = self.result()
        self.assertFalse(summarize(result)["sleeping_observed"])
        result["samples"][0].update(awake_objects=0, sleeping_objects=1, integrated_objects=0)
        summary = summarize(result)
        self.assertTrue(summary["sleeping_observed"])
        self.assertEqual(summary["metrics"]["sleeping_objects"]["median"], 1)

    def test_preparation_is_settled_only_and_rejects_waking_samples(self):
        for sleeping in (0, 1):
            commands = []
            def run(command, **kwargs):
                commands.append(command)
                result = self.result()
                result["samples"][0]["sleeping_objects"] = sleeping
                kwargs["stdout"].write(json.dumps(result))
                return type("Completed", (), {"returncode": 0})()
            with tempfile.TemporaryDirectory() as temp:
                binary = Path(temp) / "binary"
                binary.write_bytes(b"fixture")
                output = Path(temp) / "results"
                with patch("benchmark_fragments.command_output", return_value="revision"), \
                     patch("benchmark_fragments.platform.platform", return_value="test"), \
                     patch("benchmark_fragments.platform.processor", return_value="test"), \
                     patch("benchmark_fragments.subprocess.run", side_effect=run):
                    code = main(["--binary", str(binary), "--output", str(output),
                                 "--counts", "1", "--layouts", "dense", "--cameras", "facing",
                                 "--states", "settled", "active", "--settle-steps", "1200",
                                 "--machine-label", "test"])
                self.assertIn("--settle-steps", commands[0])
                self.assertNotIn("--settle-steps", commands[1])
                summary = json.loads((output / "summary.json").read_text())
                self.assertEqual(code, 0 if sleeping else 1)
                self.assertEqual(len(summary["failures"]), 0 if sleeping else 1)
                if not sleeping:
                    self.assertIn("woke", summary["failures"][0]["error"])

    def test_missing_samples_loss_mass_and_duplicate_identity_fail(self):
        for mutation in (lambda r: r.update(samples=[]), lambda r: r.update(accounting_preserved=False),
            lambda r: r["accounting"].update(extracted_mass_kg=3),
            lambda r: r["accounting"]["fragments"].append({"id": 1, "mass_kg": 2})):
            result = self.result()
            mutation(result)
            with self.assertRaises(ValueError):
                summarize(result)

    def test_repeat_windows_and_fresh_active_processes(self):
        for incomplete in (False, True):
            commands = []
            def run(command, **kwargs):
                commands.append(command)
                result = self.result()
                result["samples"][0]["sleeping_objects"] = 1
                output = result
                if "--repeats" in command:
                    output = {"schema": 2, "runs": [copy.deepcopy(result) for _ in range(2 if incomplete else 3)]}
                kwargs["stdout"].write(json.dumps(output))
                return type("Completed", (), {"returncode": 0})()
            with tempfile.TemporaryDirectory() as temp:
                binary = Path(temp) / "binary"
                binary.write_bytes(b"fixture")
                output = Path(temp) / "results"
                with patch("benchmark_fragments.command_output", return_value="revision"), \
                     patch("benchmark_fragments.platform.platform", return_value="test"), \
                     patch("benchmark_fragments.platform.processor", return_value="test"), \
                     patch("benchmark_fragments.subprocess.run", side_effect=run):
                    code = main(["--binary", str(binary), "--output", str(output),
                        "--counts", "1", "--layouts", "dense", "--cameras", "facing",
                        "--states", "settled", "active", "--repeats", "3", "--machine-label", "test"])
                summary = json.loads((output / "summary.json").read_text())
                self.assertEqual(len(commands), 4)
                self.assertTrue(all("--repeats" not in command for command in commands[1:]))
                self.assertEqual(code, int(incomplete))
                self.assertEqual(len(summary["cases"]), 3 if incomplete else 6)
                self.assertEqual([case["config"]["repeat"] for case in summary["cases"][-3:]], [1, 2, 3])
                if incomplete:
                    self.assertIn("incomplete repeat", summary["failures"][0]["error"])

    def test_relative_p95_gate_and_environment_guard(self):
        baseline = {"environment": {"machine_label": "A"}, "cases": [summarize(self.result())]}
        baseline["cases"][0]["samples_retained"] = 100
        current = copy.deepcopy(baseline)
        current["cases"][0]["metrics"]["physics_total_ms"]["p95"] = 3
        self.assertEqual(len(compare(current, baseline, .2)), 1)
        current["environment"]["machine_label"] = "software-vulkan"
        with self.assertRaises(ValueError):
            compare(current, baseline, .2)
        current = copy.deepcopy(baseline)
        current["cases"][0]["config"]["count"] = 10
        with self.assertRaises(ValueError):
            compare(current, baseline, .2)


class PreparationLifecycleTests(unittest.TestCase):
    def test_all_preparations_finish_before_any_measurement_starts(self):
        script = """import sys, time
from pathlib import Path
path, name, delay = sys.argv[1:4]
time.sleep(float(delay))
with Path(path).open('a') as log: log.write(name + ':ready\\n')
print('SALIMON_BENCHMARK_PREPARED', file=sys.stderr, flush=True)
assert sys.stdin.readline().strip() == 'SALIMON_BENCHMARK_START'
with Path(path).open('a') as log: log.write(name + ':start\\n')
"""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            log = root / 'events'
            jobs = [{'name': name, 'command': [sys.executable, '-c', script, str(log), name, delay]}
                    for name, delay in [('a', '0'), ('b', '.1')]]
            self.assertEqual(prepared_batch(jobs, root, 5), {'a': None, 'b': None})
            events = log.read_text().splitlines()
            self.assertEqual(set(events[:2]), {'a:ready', 'b:ready'})
            self.assertEqual(events[2:], ['a:start', 'b:start'])

    def test_early_exit_and_readiness_timeout_are_failures(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            jobs = [{'name': 'exit', 'command': [sys.executable, '-c', 'raise SystemExit(2)']},
                    {'name': 'stall', 'command': [sys.executable, '-c', 'import time; time.sleep(30)']}]
            outcomes = prepared_batch(jobs, root, .2)
            self.assertIn('exited 2 before native readiness', outcomes['exit'])
            self.assertIn('preparation exceeded', outcomes['stall'])
