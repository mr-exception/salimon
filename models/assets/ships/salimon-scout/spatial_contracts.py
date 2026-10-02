#!/usr/bin/env python3
"""Extract runtime spatial contracts from the Blender-authored scout export."""
from __future__ import annotations

import json

SNAP_EPSILON = 1e-5
THRUSTER_COLLIDERS = (
    "COLLIDER_Engine_Port_Body",
    "COLLIDER_Engine_Port_SweptFin",
    "COLLIDER_Engine_Starboard_Body",
    "COLLIDER_Engine_Starboard_SweptFin",
)
MARKERS = ("MARKER_CockpitSeat", "MARKER_ExitDoor", "MARKER_PlayerStart")


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
    size = metadata.get("sizeMeters")
    if not isinstance(size, list) or len(size) != 3 or not all(float(value) > 0 for value in size):
        raise ValueError(f"{name}: sizeMeters must contain three positive values")
    if node.get("rotation", [0, 0, 0, 1]) != [0, 0, 0, 1] or node.get("scale", [1, 1, 1]) != [1, 1, 1] or "matrix" in node:
        raise ValueError(f"{name}: authored box proxies must stay axis-aligned with applied scale")
    expected = baseline.get(name, {})
    center = _snap(node.get("translation", [0, 0, 0]), expected.get("translation"))
    expected_size = expected.get("extras", {}).get("salimon", {}).get("sizeMeters")
    size = _snap(size, expected_size)
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
    if node.get("rotation", [0, 0, 0, 1]) != [0, 0, 0, 1] or node.get("scale", [1, 1, 1]) != [1, 1, 1] or "matrix" in node:
        raise ValueError(f"{name}: marker transform must use an applied identity frame")
    expected = baseline.get(name, {})
    metadata = node.get("extras", {}).get("salimon", {})
    return {
        "positionMeters": _snap(node.get("translation", [0, 0, 0]), expected.get("translation")),
        "purpose": metadata.get("purpose"),
        "facing": metadata.get("facing"),
    }


def _cargo_room(colliders):
    deck = colliders["COLLIDER_Cargo_Deck"]
    ceiling = colliders["COLLIDER_Cargo_Ceiling"]
    aft = colliders["COLLIDER_Cargo_Aft_Wall"]
    forward = colliders["COLLIDER_Cargo_Forward_Wall"]
    port = colliders["COLLIDER_Cargo_Port_Wall"]
    partition = colliders["COLLIDER_Cargo_Inboard_Partition"]
    header = colliders["COLLIDER_Cargo_Passage_Header"]
    clear_min = [aft["upperMeters"][0], deck["upperMeters"][1], partition["upperMeters"][2]]
    clear_max = [forward["lowerMeters"][0], ceiling["lowerMeters"][1], port["lowerMeters"][2]]
    return {
        "clearMinMeters": clear_min,
        "clearMaxMeters": clear_max,
        "passageMinXMeters": partition["upperMeters"][0],
        "passageMaxXMeters": forward["lowerMeters"][0],
        "passageClearHeightMeters": header["lowerMeters"][1] - deck["upperMeters"][1],
        "floorAreaSquareMeters": (clear_max[0] - clear_min[0]) * (clear_max[2] - clear_min[2]),
        "role": "physical-room-for-loose-fragments-and-future-containers",
    }


def build_spatial_contracts(document, preservation=None):
    nodes = _nodes(document)
    baseline = _baseline_nodes(preservation)
    collider_names = sorted(name for name in nodes if name.startswith("COLLIDER_"))
    if len(collider_names) != 22:
        raise ValueError(f"expected 22 authored scout colliders, found {len(collider_names)}")
    colliders = {name: _box(nodes, baseline, name) for name in collider_names}
    markers = {name: _marker(nodes, baseline, name) for name in MARKERS}
    cargo = _cargo_room(colliders)
    return {
        "schemaVersion": 1,
        "assetId": "ship.salimon-scout",
        "source": "Blender-authored GLTF node transforms and salimon extras",
        "colliders": colliders,
        "markers": markers,
        "anchors": {
            "cockpitSeat": markers["MARKER_CockpitSeat"]["positionMeters"],
            "exitDoor": markers["MARKER_ExitDoor"]["positionMeters"],
            "playerStart": markers["MARKER_PlayerStart"]["positionMeters"],
            "enginePort": colliders["COLLIDER_Engine_Port_Body"]["centerMeters"],
            "engineStarboard": colliders["COLLIDER_Engine_Starboard_Body"]["centerMeters"],
            "cargoCenter": [(cargo["clearMinMeters"][axis] + cargo["clearMaxMeters"][axis]) / 2
                            for axis in range(3)],
        },
        "cargoRoom": cargo,
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


def cargo_rust_source(contracts):
    cargo = contracts["cargoRoom"]
    colliders = contracts["colliders"]
    z0 = colliders["COLLIDER_Cargo_Deck"]["lowerMeters"][2]
    z1 = cargo["clearMinMeters"][2]
    walls = [
        [colliders["COLLIDER_InteriorPortWall"]["lowerMeters"][0],
         colliders["COLLIDER_InteriorPortWall"]["upperMeters"][0], z0, z1],
        [colliders["COLLIDER_PortWallForward"]["lowerMeters"][0],
         colliders["COLLIDER_PortWallForward"]["upperMeters"][0], z0, z1],
    ]
    for name in ("COLLIDER_Cargo_Aft_Wall", "COLLIDER_Cargo_Forward_Wall", "COLLIDER_Cargo_Port_Wall"):
        box = colliders[name]
        walls.append([box["lowerMeters"][0], box["upperMeters"][0],
                      box["lowerMeters"][2], box["upperMeters"][2]])
    lines = [
        "// Generated from Blender-authored scout spatial contracts; do not edit by hand.",
        "pub const CARGO_ROOM_MIN_METERS: [f64; 3] = " + _rust_array(cargo["clearMinMeters"]) + ";",
        "pub const CARGO_ROOM_MAX_METERS: [f64; 3] = " + _rust_array(cargo["clearMaxMeters"]) + ";",
        "// [forward min/max, port min/max]; includes the cabin partition at the passage.",
        "pub(super) const CARGO_WALLS: [[f64; 4]; 5] = [",
    ]
    for wall in walls:
        lines.extend(_rust_array_entry(wall))
    lines.append("];")
    return "\n".join(lines) + "\n"


def thruster_rust_source(contracts):
    lines = [
        "// Generated from Blender-authored scout spatial contracts; do not edit by hand.",
        "// [forward min/max, up min/max, port min/max] in ship-local meters.",
        "pub(super) const THRUSTER_COLLIDERS: [[f64; 6]; 4] = [",
    ]
    for name in THRUSTER_COLLIDERS:
        box = contracts["colliders"][name]
        bounds = [box["lowerMeters"][0], box["upperMeters"][0],
                  box["lowerMeters"][1], box["upperMeters"][1],
                  box["lowerMeters"][2], box["upperMeters"][2]]
        lines.extend((f"    // {name}", "    ["))
        lines.extend("        " + _rust_number(value) + "," for value in bounds)
        lines.append("    ],")
    lines.append("];")
    return "\n".join(lines) + "\n"


def anchors_rust_source(contracts):
    anchors = contracts["anchors"]
    names = (
        ("COCKPIT_SEAT_MARKER_METERS", "cockpitSeat"),
        ("EXIT_DOOR_MARKER_METERS", "exitDoor"),
        ("PLAYER_START_MARKER_METERS", "playerStart"),
        ("ENGINE_PORT_ANCHOR_METERS", "enginePort"),
        ("ENGINE_STARBOARD_ANCHOR_METERS", "engineStarboard"),
        ("CARGO_ANCHOR_METERS", "cargoCenter"),
    )
    lines = ["// Generated from Blender-authored scout spatial contracts; do not edit by hand."]
    lines.extend(
        f"pub const {rust_name}: [f64; 3] = {_rust_array(anchors[key])};"
        for rust_name, key in names
    )
    return "\n".join(lines) + "\n"


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
