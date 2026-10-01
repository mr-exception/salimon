"""Named gameplay prerequisites share the baseline's authoritative routes."""

SHIP_EVA = ("cargo-room.json", "space-airlock.json", "moving-eva.json", "nearby-eva.json")


def select_scenarios(directory, group=None, evidence=False):
    if evidence:
        directory = directory / "evidence"
    if group == "ship-eva":
        # Explicit paths ensure a missing prerequisite fails instead of being skipped.
        return [directory / name for name in SHIP_EVA]
    if group is not None:
        raise ValueError(f"unknown scenario group: {group}")
    return sorted(directory.glob("*.json"))
