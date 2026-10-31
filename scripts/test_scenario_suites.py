"""Protect prerequisite selection against silently dropping missing coverage."""

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from scenario_suites import GROUPS, RESOURCE_COLLECTION, SCOUT_CONSTRUCTION, select_scenarios
from salimon_test import ScenarioError, parse_scenario


class ScenarioSuiteTests(unittest.TestCase):
    def test_scanner_suite_requires_integration_hologram_and_cockpit_collision_routes(self):
        root = Path(__file__).resolve().parents[1] / "scenarios"
        for evidence in (False, True):
            selected = select_scenarios(root, "scanner", evidence)
            self.assertEqual([p.name for p in selected], ["scanner-integration.json", "scanner-hologram.json", "cockpit-nose.json"])
            self.assertTrue(all(p.parent == root / "evidence" for p in selected))
            for path in selected:
                parse_scenario(path)

    def test_integrated_construction_always_selects_capture_routes(self):
        root = Path(__file__).resolve().parents[1] / "scenarios"
        for evidence in (False, True):
            selected = select_scenarios(root, "scout-construction", evidence)
            self.assertEqual(len(selected), len(set(SCOUT_CONSTRUCTION)))
            for path in selected:
                self.assertEqual(path.parent, root / "evidence")
                self.assertTrue(any("screenshot" in step for step in parse_scenario(path)["steps"]))

    def test_missing_prerequisite_is_selected_and_fails(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for group, names in GROUPS.items():
                for evidence in (False, True):
                    paths = select_scenarios(root, group, evidence)
                    self.assertEqual([path.name for path in paths], list(names))
                    self.assertEqual(paths[0].parent, root / "evidence" if evidence or group in {"scout-construction", "scanner", "universe-render"} else root)
                    with self.assertRaises(ScenarioError):
                        parse_scenario(paths[0])

    def test_default_discovery_preserves_baseline(self):
        root = Path(__file__).resolve().parent.parent / "scenarios"
        self.assertEqual(select_scenarios(root), sorted(root.glob("*.json")))
        for group in GROUPS:
            for evidence in (False, True):
                for path in select_scenarios(root, group, evidence):
                    parse_scenario(path)

    def test_landing_evidence_preserves_baseline_gameplay(self):
        root = Path(__file__).resolve().parents[1] / "scenarios"
        baseline = parse_scenario(root / "landing-speed-cutoff.json")
        evidence = parse_scenario(root / "evidence/landing-speed-cutoff.json")
        self.assertEqual(baseline["setup"], evidence["setup"])
        self.assertEqual(baseline["steps"],
                         [step for step in evidence["steps"] if "screenshot" not in step])

    def test_unknown_group_is_rejected(self):
        with self.assertRaises(ValueError):
            select_scenarios(Path("scenarios"), "typo")

    def test_summary_cli_retains_failure_exit_and_artifact_paths(self):
        root = Path(__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory() as temp:
            completed = subprocess.run(
                [sys.executable, str(root / "scripts/salimon-test"), "suite",
                 "--group", "ship-eva", "--summary", "--binary", str(Path(temp) / "missing"),
                 "--artifacts", str(Path(temp) / "evidence"), "--screenshot-command", "[]"],
                capture_output=True, text=True, timeout=30)
            self.assertEqual(completed.returncode, 1)
            report = json.loads(completed.stdout)
            self.assertEqual(report["status"], "failed")
            self.assertEqual(len(report["results"]), len(GROUPS["ship-eva"]))
            for result in report["results"]:
                self.assertTrue(result["error"])
                self.assertNotIn("steps", result)
                self.assertTrue(Path(result["result_file"]).is_file())


    def test_resource_cli_preserves_each_failed_launch(self):
        root = Path(__file__).resolve().parent.parent
        with tempfile.TemporaryDirectory() as temp:
            directory = Path(temp)
            for evidence in (False, True):
                artifacts = directory / str(evidence)
                command = [sys.executable, str(root / "scripts" / "salimon-test"),
                           "suite", "--group", "resource-collection",
                           "--binary", str(directory / "missing-client"),
                           "--artifacts", str(artifacts), "--screenshot-command", "[]"]
                if evidence:
                    command.append("--evidence")
                completed = subprocess.run(command, capture_output=True, text=True,
                                           cwd=directory, timeout=30)
                self.assertEqual(completed.returncode, 1, completed.stderr)
                report = json.loads(completed.stdout)
                self.assertEqual(report["status"], "failed")
                self.assertEqual(len(report["results"]), len(RESOURCE_COLLECTION))
                results = sorted(artifacts.glob("run-*/result.json"))
                self.assertEqual(len(results), len(RESOURCE_COLLECTION))
                for result in results:
                    saved = json.loads(result.read_text())
                    self.assertEqual(saved["status"], "failed")
                    self.assertTrue(saved["error"])
                    self.assertTrue((result.parent / "scenario.json").is_file())
