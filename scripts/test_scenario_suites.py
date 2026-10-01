"""Protect prerequisite selection against silently dropping missing coverage."""

import tempfile
import unittest
from pathlib import Path

from scenario_suites import SHIP_EVA, select_scenarios
from salimon_test import ScenarioError, parse_scenario


class ScenarioSuiteTests(unittest.TestCase):
    def test_missing_prerequisite_is_selected_and_fails(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for evidence in (False, True):
                paths = select_scenarios(root, "ship-eva", evidence)
                self.assertEqual([path.name for path in paths], list(SHIP_EVA))
                self.assertEqual(paths[0].parent, root / "evidence" if evidence else root)
                with self.assertRaises(ScenarioError):
                    parse_scenario(paths[0])

    def test_default_discovery_preserves_baseline(self):
        root = Path(__file__).resolve().parent.parent / "scenarios"
        self.assertEqual(select_scenarios(root), sorted(root.glob("*.json")))
        for evidence in (False, True):
            for path in select_scenarios(root, "ship-eva", evidence):
                parse_scenario(path)

    def test_unknown_group_is_rejected(self):
        with self.assertRaises(ValueError):
            select_scenarios(Path("scenarios"), "typo")
