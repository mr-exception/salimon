#!/usr/bin/env python3
"""Extract runtime spatial contracts from the Blender-authored scout export."""
from __future__ import annotations

import json
import math
import subprocess

SNAP_EPSILON = 1e-5
THRUSTER_COLLIDERS = (
    "COLLIDER_Engine_Port_Body",
    "COLLIDER_Engine_Port_SweptFin",
    "COLLIDER_Engine_Starboard_Body",
    "COLLIDER_Engine_Starboard_SweptFin",
)
WING_COLLIDERS = ("COLLIDER_Wing_Port", "COLLIDER_Wing_Starboard")
MARKERS = ("MARKER_CockpitSeat", "MARKER_ExitDoor", "MARKER_PlayerStart",
           "MARKER_DoorwayTransition")
LAYOUT_BOXES = {
    "INTERIOR_FLOOR_BOUNDS": "COLLIDER_InteriorFloor",
    "NOSE_FLOOR_BOUNDS": "COLLIDER_CockpitNoseFloor",
    "CABIN_TRAVERSAL_BOUNDS": "COLLIDER_CabinTraversalEnvelope",
    "CABIN_EXTERIOR_BOUNDS": "COLLIDER_CabinExteriorEnvelope",
    "AFT_DOOR_BOUNDS": "COLLIDER_AftDoor",
    "CORE_BOUNDS": "COLLIDER_EnergyCore",
    "PILOT_CHAIR_BOUNDS": "COLLIDER_PilotChair",
    "CONSOLE_BOUNDS": "COLLIDER_CockpitCenterConsole",
    "COCKPIT_PORT_HULL_BOUNDS": "COLLIDER_CockpitPortHull",
    "COCKPIT_STARBOARD_HULL_BOUNDS": "COLLIDER_CockpitStarboardHull",
}
REQUIRED_COLLIDERS = set(LAYOUT_BOXES.values()) | set(THRUSTER_COLLIDERS) | set(WING_COLLIDERS) | {
    "COLLIDER_InteriorPortWall", "COLLIDER_InteriorStarboardWall",
    "COLLIDER_InteriorCeiling", "COLLIDER_ExteriorHull",
}


def _vector(values, name, positive=False):
    if (not isinstance(values, list) or len(values) != 3
            or not all(isinstance(value, (int, float)) and not isinstance(value, bool)
                       and math.isfinite(value) and (not positive or value > 0) for value in values)):
        qualifier = "positive finite" if positive else "finite"
        raise ValueError(f"{name}: must contain three {qualifier} values")
    return values


def _identity(node, name):
    if (node.get("rotation", [0, 0, 0, 1]) != [0, 0, 0, 1]
            or node.get("scale", [1, 1, 1]) != [1, 1, 1] or "matrix" in node):
        raise ValueError(f"{name}: spatial contracts must stay axis-aligned with applied scale")


def _validate_parents(document, nodes, names):
    # Runtime boxes are asset-local, never silently parent-relative. The scout
    # uses identity groups; reject transformed/reparented contracts explicitly.
    parents = {document["nodes"][child]["name"]: node["name"]
               for node in document["nodes"] for child in node.get("children", [])}
    for name in names:
        group = "Collision_Proxies" if name.startswith("COLLIDER_") else "Interaction_Markers"
        if parents.get(name) != group or parents.get(group) != "Salimon_Phase0_Scout":
            raise ValueError(f"{name}: expected {group} under the scout root")
        for ancestor in (group, "Salimon_Phase0_Scout"):
            _identity(nodes[ancestor], ancestor)
            if nodes[ancestor].get("translation", [0, 0, 0]) != [0, 0, 0]:
                raise ValueError(f"{ancestor}: spatial parent must have an identity translation")


def _nodes(document):
    nodes = {node["name"]: node for node in document["nodes"]}
    if len(nodes) != len(document["nodes"]):
        raise ValueError("scout spatial contracts require unique node names")
    return nodes


def _baseline_nodes(preservation):
    return {} if preservation is None else {node["name"]: node for node in preservation.get("nodes", [])}


def _snap(values, baseline):
    if baseline is None:
        return [float(value) for value in values]
    if len(values) != len(baseline):
        raise ValueError("spatial vector length changed")
    return [float(expected if abs(float(actual) - float(expected)) <= SNAP_EPSILON else actual)
            for actual, expected in zip(values, baseline)]


def _box(nodes, baseline, name):
    node = nodes[name]
    metadata = node.get("extras", {}).get("salimon", {})
    if metadata.get("collisionShape") != "box":
        raise ValueError(f"{name}: collisionShape must be box")
    if "mesh" in node:
        raise ValueError(f"{name}: collision contract must stay nonvisual")
    size = _vector(metadata.get("sizeMeters"), f"{name}: sizeMeters", positive=True)
    _identity(node, name)
    expected = baseline.get(name, {})
    center = _snap(_vector(node.get("translation", [0, 0, 0]), f"{name}: translation"), expected.get("translation"))
    expected_size = expected.get("extras", {}).get("salimon", {}).get("sizeMeters")
    size = _snap(size, expected_size)
    if not all(math.isfinite(center[axis] + sign * size[axis] / 2)
               for axis in range(3) for sign in (-1, 1)):
        raise ValueError(f"{name}: box bounds must be finite")
    return {
        "centerMeters": center,
        "sizeMeters": size,
        "lowerMeters": [center[axis] - size[axis] / 2 for axis in range(3)],
        "upperMeters": [center[axis] + size[axis] / 2 for axis in range(3)],
        "shape": "box",
        "purpose": metadata.get("purpose"),
    }


def _marker(nodes, baseline, name):
    node = nodes[name]
    if "mesh" in node:
        raise ValueError(f"{name}: marker must stay nonvisual")
    _identity(node, name)
    expected = baseline.get(name, {})
    metadata = node.get("extras", {}).get("salimon", {})
    return {
        "positionMeters": _snap(_vector(node.get("translation", [0, 0, 0]), f"{name}: translation"), expected.get("translation")),
        "purpose": metadata.get("purpose"),
        "facing": metadata.get("facing"),
    }


def build_spatial_contracts(document, preservation=None):
    nodes = _nodes(document)
    baseline = _baseline_nodes(preservation)
    collider_names = sorted(name for name in nodes if name.startswith("COLLIDER_"))
    missing = (REQUIRED_COLLIDERS | set(MARKERS)) - nodes.keys()
    unexpected = set(collider_names) - REQUIRED_COLLIDERS
    if missing or unexpected:
        raise ValueError(f"scout spatial nodes: missing {sorted(missing)}, unexpected {sorted(unexpected)}")
    _validate_parents(document, nodes, [*collider_names, *MARKERS])
    colliders = {name: _box(nodes, baseline, name) for name in collider_names}
    markers = {name: _marker(nodes, baseline, name) for name in MARKERS}
    # The planar controller uses centered widths for these volumes. Reject an
    # unsupported asymmetric edit rather than silently mirroring the port side.
    for name in ("COLLIDER_CabinTraversalEnvelope", "COLLIDER_CabinExteriorEnvelope",
                 "COLLIDER_CockpitNoseFloor", "COLLIDER_AftDoor", "COLLIDER_EnergyCore"):
        if colliders[name]["centerMeters"][2] != 0:
            raise ValueError(f"{name}: traversal width must remain centered on port axis")
    floor = colliders["COLLIDER_InteriorFloor"]["upperMeters"][1]
    nose = colliders["COLLIDER_CockpitNoseFloor"]["upperMeters"][1]
    if abs(floor - nose) > SNAP_EPSILON:
        raise ValueError("COLLIDER_CockpitNoseFloor: nose and cabin floors must be level")
    for name in ("COLLIDER_CabinTraversalEnvelope", "COLLIDER_CabinExteriorEnvelope",
                 "COLLIDER_PilotChair", "COLLIDER_CockpitPortHull", "COLLIDER_CockpitStarboardHull"):
        expected_purpose = baseline.get(name, {}).get("extras", {}).get("salimon", {}).get("purpose")
        if expected_purpose is not None and colliders[name]["purpose"] != expected_purpose:
            raise ValueError(f"{name}: spatial purpose changed")
    return {
        "schemaVersion": 1,
        "assetId": "ship.salimon-scout",
        "source": "Blender-authored GLTF node transforms and salimon extras",
        "colliders": colliders,
        "markers": markers,
        "anchors": {
            "cockpitSeat": markers["MARKER_CockpitSeat"]["positionMeters"],
            "exitDoor": markers["MARKER_ExitDoor"]["positionMeters"],
            "doorwayTransition": markers["MARKER_DoorwayTransition"]["positionMeters"],
            "playerStart": markers["MARKER_PlayerStart"]["positionMeters"],
            "enginePort": colliders["COLLIDER_Engine_Port_Body"]["centerMeters"],
            "engineStarboard": colliders["COLLIDER_Engine_Starboard_Body"]["centerMeters"],
        },
    }


def _rust_number(value):
    value = float(value)
    return "0.0" if value == 0 else repr(value)


def _rust_array(values):
    return "[" + ", ".join(_rust_number(value) for value in values) + "]"


def _rust_array_entry(values, indent="    ", max_width=60):
    inline = indent + _rust_array(values) + ","
    if len(inline) <= max_width:
        return [inline]
    child = indent + "    "
    return [indent + "[", *(child + _rust_number(value) + "," for value in values), indent + "],"]


def _bounds(box):
    return [value for axis in range(3)
            for value in (box["lowerMeters"][axis], box["upperMeters"][axis])]


def spatial_rust_source(contracts):
    lines = [
        "// Generated from Blender-authored scout spatial contracts; do not edit by hand.",
        "// Bounds: [forward min/max, up min/max, port min/max] in ship-local meters.",
    ]
    for rust_name, key in (
        ("COCKPIT_SEAT_MARKER_METERS", "cockpitSeat"),
        ("EXIT_DOOR_MARKER_METERS", "exitDoor"),
        ("PLAYER_START_MARKER_METERS", "playerStart"),
        ("ENGINE_PORT_ANCHOR_METERS", "enginePort"),
        ("ENGINE_STARBOARD_ANCHOR_METERS", "engineStarboard"),
        ("DOORWAY_TRANSITION_MARKER_METERS", "doorwayTransition"),
    ):
        visibility = "pub" if key != "doorwayTransition" else "pub(crate)"
        lines.append(f"{visibility} const {rust_name}: [f64; 3] = {_rust_array(contracts['anchors'][key])};")
    for rust_name, name in LAYOUT_BOXES.items():
        lines.append(f"// {name}")
        lines.append(f"pub(crate) const {rust_name}: [f64; 6] = {_rust_array(_bounds(contracts['colliders'][name]))};")
    for rust_name, names in (("THRUSTER_COLLIDERS", THRUSTER_COLLIDERS),
                             ("WING_COLLIDERS", WING_COLLIDERS)):
        lines.append(f"pub(crate) const {rust_name}: [[f64; 6]; {len(names)}] = [")
        for name in names:
            lines.append(f"    // {name}")
            lines.extend(_rust_array_entry(_bounds(contracts["colliders"][name])))
        lines.append("];")
    # Use the repository formatter so generated output stays byte-for-byte
    # reproducible across short/long numeric values and cargo fmt checks.
    return subprocess.run(["rustfmt", "--edition", "2024", "--emit", "stdout"],
                          input="\n".join(lines) + "\n", text=True,
                          capture_output=True, check=True).stdout


def _compact_json_numbers(value):
    if isinstance(value, dict):
        return {key: _compact_json_numbers(item) for key, item in value.items()}
    if isinstance(value, list):
        return [_compact_json_numbers(item) for item in value]
    if isinstance(value, float) and value.is_integer():
        return int(value)
    return value


def sidecar_json(contracts):
    return json.dumps(_compact_json_numbers(contracts), indent=2, sort_keys=True) + "\n"
