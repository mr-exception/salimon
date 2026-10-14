import copy
import unittest
from profile_flight import WORKLOADS, compare, rows


def report():
    return {"schema": 1, "kind": "flight-cpu-profile", "fixture": "deck", "rows": [
        {"count": count, "workload": name, "cargo_mass_kg": 16 if count else 0,
         "moving_objects": [count]*20, "hardware_fps_validated": False,
         "cpu_update": {"p50_ms": .01, "p95_ms": .01, "p99_ms": .01, "max_ms": .01, "samples": 20, "raw_ms": [.01]*20}}
        for count in (0, 500) for name in sorted(WORKLOADS)]}


class FlightProfileTests(unittest.TestCase):
    def test_relative_regression_fails_and_reports_exact_workload(self):
        baseline = report()
        current = copy.deepcopy(baseline)
        r = current["rows"][0]
        r["cpu_update"] = {**r["cpu_update"], "p50_ms": 2000.0, "p95_ms": 2000.0, "p99_ms": 2000.0, "max_ms": 2000.0, "raw_ms": [2000.0]*20}
        self.assertTrue(compare(current, baseline))
        self.assertEqual(compare(baseline, baseline), [])
        mismatch = copy.deepcopy(baseline)
        mismatch["fixture"] = "stacked"
        with self.assertRaises(ValueError): compare(mismatch, baseline)

    def test_missing_duplicate_nonfinite_incomplete_and_offboard_fail(self):
        for mutation in [lambda d: d["rows"].pop(), lambda d: d["rows"].append(d["rows"][0]),
                         lambda d: d["rows"][0]["cpu_update"]["raw_ms"].pop(),
                         lambda d: d["rows"][0]["cpu_update"]["raw_ms"].__setitem__(0, float("nan")),
                         lambda d: d["rows"][0].__setitem__("cargo_mass_kg", 100),
                         lambda d: d["rows"][0]["cpu_update"].__setitem__("p95_ms", 99)]:
            data = report()
            mutation(data)
            with self.assertRaises(ValueError): rows(data)

    def test_failed_process_preserves_evidence_and_nonzero_status(self):
        import json
        import subprocess
        import sys
        import tempfile
        from pathlib import Path
        from unittest.mock import patch
        import profile_flight
        with tempfile.TemporaryDirectory() as temp:
            output = Path(temp) / "failed.json"
            with patch.object(sys, "argv", ["profile_flight", "--binary", "missing", "--output", str(output)]), patch.object(subprocess, "run", side_effect=subprocess.TimeoutExpired("profile", 300, output=b"partial", stderr=b"row evidence")):
                self.assertEqual(profile_flight.main(), 1)
            data = json.loads(output.read_text())
            self.assertEqual(data["status"], "failed")
            self.assertEqual(data["stderr"], "row evidence")
