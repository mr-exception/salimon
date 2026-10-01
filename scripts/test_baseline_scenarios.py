"""Keep baseline gameplay and optional visual evidence on the same action path."""

import unittest
from pathlib import Path

from salimon_test import parse_scenario


SCENARIOS = Path(__file__).resolve().parent.parent / "scenarios"
BASELINES = ("landed-earth.json", "orbit-earth.json", "resource-deposits.json", "mining.json", "resource-streaming.json", "carrying.json", "fragment-transfer.json", "cargo-room.json", "space-airlock.json", "moving-eva.json", "nearby-eva.json")


class BaselineScenarioTests(unittest.TestCase):
    def test_visual_variants_only_add_checkpoints(self):
        for name in BASELINES:
            path = SCENARIOS / name
            with self.subTest(scenario=path.name):
                baseline = parse_scenario(path)
                visual = parse_scenario(SCENARIOS / "evidence" / path.name)
                checkpoints = [step for step in visual["steps"] if "screenshot" in step]
                self.assertTrue(checkpoints)
                self.assertEqual(len({step["screenshot"] for step in checkpoints}), len(checkpoints))
                visual["steps"] = [step for step in visual["steps"] if "screenshot" not in step]
                self.assertEqual(baseline, visual)

    def test_movement_is_released_before_authority_changes(self):
        for name in BASELINES:
            path = SCENARIOS / name
            with self.subTest(scenario=path.name):
                held = set()
                for step in parse_scenario(path)["steps"]:
                    action = step.get("action", {})
                    if action.get("op") == "key":
                        if action["pressed"]:
                            held.add(action["key"])
                        else:
                            held.discard(action["key"])
                    elif action.get("op") == "interact":
                        self.assertFalse(held, f"held keys at interaction: {held}")
                self.assertFalse(held, "scenario must release all movement keys")


if __name__ == "__main__":
    unittest.main()
