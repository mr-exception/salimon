"""Named gameplay suites share the baseline's authoritative routes."""

SHIP_EVA = ("cockpit-nose.json", "space-airlock.json", "moving-eva.json", "nearby-eva.json")

RESOURCE_COLLECTION = (
    "resource-deposits.json", "mining.json", "mining-emission.json", "carrying.json",
    "fragment-transfer.json", "resource-streaming.json", "resource-loop.json",
)
TERRAIN = tuple(
    f"terrain-{body}{suffix}.json"
    for body in ("earth", "moon", "mars")
    for suffix in ("", "-crater", "-touchdown", "-mining")
) + ("terrain-steep-site.json",)
REALISTIC_FLIGHT = ("inertial-flight.json", "core-restart.json", "crash-recovery.json", "cargo-flight.json", "moving-eva.json", "cargo-blackout.json", "landing-speed-cutoff.json")
SCOUT_CONSTRUCTION = (
    "starter-layout.json", "cockpit-nose.json", "inertial-flight.json",
    "core-restart.json", "cargo-door.json", "boarding-ramp.json",
    "fragment-transfer.json", "habitation.json", "habitation-use.json",
    "cabin-bunk.json", "cabin-table.json", "cabin-locker.json",
    "room-finishes.json", "room-lighting-blackout.json",
    "core-upgrade-blackout.json", "core-upgrade-powered.json",
    "blueprint-placement.json", "construction-materials.json",
    "construction-build.json", "construction-cancel-0.json",
    "construction-cancel-50.json", "construction-cancel-99.json",
    "construction-room.json", "construction-interlocks.json",
    "construction-interlocks-cockpit.json", "construction-module.json",
    "construction-module-upgrade.json", "construction-module-restore.json",
    "construction-damage.json", "construction-detachment.json",
    "construction-offline.json", "construction-offline-upgrade.json",
)
GROUPS = {"scout-construction": SCOUT_CONSTRUCTION, "realistic-flight": REALISTIC_FLIGHT, "terrain": TERRAIN, "ship-eva": SHIP_EVA, "resource-collection": RESOURCE_COLLECTION}


def select_scenarios(directory, group=None, evidence=False):
    # This integrated acceptance group always uses production capture routes.
    if evidence or group == "scout-construction":
        directory = directory / "evidence"
    if group in GROUPS:
        # Explicit paths ensure a missing prerequisite fails instead of being skipped.
        return [directory / name for name in GROUPS[group]]
    if group is not None:
        raise ValueError(f"unknown scenario group: {group}")
    return sorted(directory.glob("*.json"))
