"""Benchmark reporting contracts, including incompatible/failed baseline rejection."""
import copy
import unittest
import json
import tempfile
from pathlib import Path
from salimon_test import parse_scenario, ScenarioError
from benchmark_fragments import compare, distribution, summarize


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
                         {"fragment_layout": "dense"}, {"fragment_count": 12, "fragment_layout": "bad"}):
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

    def test_missing_samples_loss_mass_and_duplicate_identity_fail(self):
        for mutation in (lambda r: r.update(samples=[]), lambda r: r.update(accounting_preserved=False),
            lambda r: r["accounting"].update(extracted_mass_kg=3),
            lambda r: r["accounting"]["fragments"].append({"id": 1, "mass_kg": 2})):
            result = self.result()
            mutation(result)
            with self.assertRaises(ValueError):
                summarize(result)

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
