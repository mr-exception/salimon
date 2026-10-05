"""Named gameplay suites share the baseline's authoritative routes."""

SHIP_EVA = ("cockpit-nose.json", "space-airlock.json", "moving-eva.json", "nearby-eva.json")

RESOURCE_COLLECTION = (
    "resource-deposits.json", "mining.json", "carrying.json",
    "fragment-transfer.json", "resource-streaming.json", "resource-loop.json",
)
GROUPS = {"ship-eva": SHIP_EVA, "resource-collection": RESOURCE_COLLECTION}


def select_scenarios(directory, group=None, evidence=False):
    if evidence:
        directory = directory / "evidence"
    if group in GROUPS:
        # Explicit paths ensure a missing prerequisite fails instead of being skipped.
        return [directory / name for name in GROUPS[group]]
    if group is not None:
        raise ValueError(f"unknown scenario group: {group}")
    return sorted(directory.glob("*.json"))
