#!/usr/bin/env python3
"""Validate the checked-in Salimon Phase 0 glTF and GLB exports."""

from __future__ import annotations

import json
import struct
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
EXPORT = ROOT / "export"
TEXTURE = ROOT / "textures" / "salimon_floor_grip.png"
NAME = "salimon_phase0_ship"
TASK7_BASELINE_DIMENSIONS = [10.15, 3.72, 8.30]
TASK10_DIMENSIONS = [20.30, 7.44, 16.60]


def parse_glb(path: Path) -> tuple[dict[str, object], bytes]:
    data = path.read_bytes()
    magic, version, total_length = struct.unpack_from("<4sII", data)
    assert magic == b"glTF", "GLB magic mismatch"
    assert version == 2, "GLB must use glTF 2.0"
    assert total_length == len(data), "GLB header length mismatch"
    json_length, json_kind = struct.unpack_from("<I4s", data, 12)
    assert json_kind == b"JSON", "GLB first chunk must be JSON"
    json_start = 20
    document = json.loads(data[json_start : json_start + json_length])
    binary_header = json_start + json_length
    binary_length, binary_kind = struct.unpack_from("<I4s", data, binary_header)
    assert binary_kind == b"BIN\0", "GLB second chunk must be BIN"
    binary = data[binary_header + 8 : binary_header + 8 + binary_length]
    assert binary_header + 8 + binary_length == len(data), "unexpected trailing GLB data"
    return document, binary


def validate_buffer_ranges(document: dict[str, object], binary_length: int) -> None:
    views = document["bufferViews"]
    accessors = document["accessors"]
    for index, view in enumerate(views):
        start = int(view.get("byteOffset", 0))
        end = start + int(view["byteLength"])
        assert 0 <= start <= end <= binary_length, f"bufferView {index} exceeds its buffer"
    for index, accessor in enumerate(accessors):
        assert 0 <= int(accessor["bufferView"]) < len(views), f"accessor {index} has invalid view"
        assert int(accessor["count"]) > 0, f"accessor {index} is empty"


def node_position_bounds(document: dict[str, object], node_name: str) -> tuple[list[float], list[float]]:
    node = next(node for node in document["nodes"] if node["name"] == node_name)
    primitive = document["meshes"][node["mesh"]]["primitives"][0]
    accessor = document["accessors"][primitive["attributes"]["POSITION"]]
    return accessor["min"], accessor["max"]


def overall_position_bounds(document: dict[str, object]) -> tuple[list[float], list[float]]:
    position_accessors = [
        accessor
        for accessor in document["accessors"]
        if accessor["type"] == "VEC3" and "min" in accessor and "max" in accessor
    ]
    return (
        [min(accessor["min"][axis] for accessor in position_accessors) for axis in range(3)],
        [max(accessor["max"][axis] for accessor in position_accessors) for axis in range(3)],
    )


def node(document: dict[str, object], name: str) -> dict[str, object]:
    return next(candidate for candidate in document["nodes"] if candidate["name"] == name)


def assert_vectors_close(actual: list[float], expected: list[float]) -> None:
    assert len(actual) == len(expected)
    assert all(abs(left - right) <= 1.0e-6 for left, right in zip(actual, expected)), (
        actual,
        expected,
    )


def main() -> None:
    gltf_path = EXPORT / f"{NAME}.gltf"
    bin_path = EXPORT / f"{NAME}.bin"
    glb_path = EXPORT / f"{NAME}.glb"
    document = json.loads(gltf_path.read_text(encoding="utf-8"))
    glb_document, glb_binary = parse_glb(glb_path)
    geometry = bin_path.read_bytes()

    assert document["asset"]["version"] == "2.0"
    assert document["buffers"][0]["byteLength"] == len(geometry)
    assert glb_document["buffers"][0]["byteLength"] == len(glb_binary)
    assert TEXTURE.read_bytes().startswith(b"\x89PNG\r\n\x1a\n")
    validate_buffer_ranges(document, len(geometry))
    validate_buffer_ranges(glb_document, len(glb_binary))

    mesh_names = [mesh["name"] for mesh in document["meshes"]]
    node_names = [node["name"] for node in document["nodes"]]
    material_names = [material["name"] for material in document["materials"]]
    assert len(node_names) == len(set(node_names)), "node names must be unique"
    assert mesh_names == [mesh["name"] for mesh in glb_document["meshes"]]
    assert node_names == [node["name"] for node in glb_document["nodes"]]
    assert material_names == [material["name"] for material in glb_document["materials"]]
    assert document["accessors"] == glb_document["accessors"]
    assert document["nodes"] == glb_document["nodes"]
    assert geometry == glb_binary[: len(geometry)], "glTF and GLB geometry payloads differ"

    required_nodes = {
        "Exterior",
        "Interior",
        "Collision_Proxies",
        "Interaction_Markers",
        "Hull_Nose",
        "Cockpit_Glazing",
        "Cockpit_Window_Frame",
        "Wing_Port",
        "Wing_Starboard",
        "Cockpit_Console_Center",
        "Pilot_Seat_Back",
        "Exit_Door",
        "Deck_Walkable",
        "COLLIDER_InteriorFloor",
        "COLLIDER_AftDoor",
        "MARKER_CockpitSeat",
        "MARKER_ExitDoor",
        "MARKER_PlayerStart",
    }
    assert required_nodes <= set(node_names), f"missing nodes: {sorted(required_nodes - set(node_names))}"

    glass = document["materials"][material_names.index("Cockpit Glass")]
    assert glass["alphaMode"] == "BLEND", "cockpit glass must use inexpensive alpha blending"
    assert glass["doubleSided"] is True, "cockpit glass must render from inside and outside"
    assert 0.0 < glass["pbrMetallicRoughness"]["baseColorFactor"][3] < 0.5

    glazing_min, glazing_max = node_position_bounds(document, "Cockpit_Glazing")
    _, nose_max = node_position_bounds(document, "Hull_Nose")
    _, console_max = node_position_bounds(document, "Cockpit_Console_Center")
    assert glazing_min[0] <= 5.2 and glazing_max[0] >= 9.8
    assert glazing_min[1] <= 2.0 and glazing_max[1] >= 5.2
    assert glazing_min[2] <= -3.0 and glazing_max[2] >= 3.0
    assert nose_max[1] <= 2.04, "solid nose must stay below the forward window"
    assert console_max[1] <= 2.40, "console must stay below the seated eye line"
    assert nose_max[0] >= console_max[0], "nose must still extend beyond the forward console"

    windows = document["extras"]["salimon"]["cockpitWindows"]
    assert windows["glazingNode"] == "Cockpit_Glazing"
    assert windows["material"] == "Cockpit Glass"
    assert_vectors_close(windows["seatedViewpointMeters"], [2.76, 2.77, 0.0])
    assert_vectors_close(windows["standingViewpointMeters"], [1.30, 2.08, 0.0])
    assert windows["dynamicShadows"] is False
    assert windows["postEffects"] is False

    metrics = document["extras"]["salimon"]
    assert metrics["linearScaleFromTask7Baseline"] >= 2.0
    assert_vectors_close(metrics["task7BaselineDimensionsMeters"], TASK7_BASELINE_DIMENSIONS)
    assert_vectors_close(metrics["overallDimensionsMeters"], TASK10_DIMENSIONS)
    overall_min, overall_max = overall_position_bounds(document)
    measured_dimensions = [maximum - minimum for minimum, maximum in zip(overall_min, overall_max)]
    assert_vectors_close(measured_dimensions, TASK10_DIMENSIONS)
    assert all(
        after >= before * 2.0
        for before, after in zip(TASK7_BASELINE_DIMENSIONS, measured_dimensions)
    )
    assert abs(overall_min[1] - metrics["lowestLocalYMeters"]) <= 1.0e-6
    assert_vectors_close(node(document, "MARKER_CockpitSeat")["translation"], [2.76, 2.10, 0.0])
    assert_vectors_close(node(document, "MARKER_ExitDoor")["translation"], [-7.10, 2.50, 0.0])
    assert_vectors_close(node(document, "MARKER_PlayerStart")["translation"], [-0.70, 2.08, 0.0])
    exterior_collider = node(document, "COLLIDER_ExteriorHull")
    assert_vectors_close(exterior_collider["translation"], [0.50, 2.96, 0.0])
    assert_vectors_close(
        exterior_collider["extras"]["salimon"]["sizeMeters"],
        [21.0, 6.32, 8.20],
    )
    assert metrics["externalAssetDependencies"] == 0
    assert metrics["triangleCount"] <= 700, "triangle budget exceeded"
    assert metrics["drawPrimitiveCount"] <= 45, "primitive budget exceeded"
    assert metrics["materialCount"] <= 9, "material budget exceeded"
    assert len(glb_path.read_bytes()) <= 256 * 1024, "GLB size budget exceeded"

    for mesh in document["meshes"]:
        assert len(mesh["primitives"]) == 1
        primitive = mesh["primitives"][0]
        assert set(primitive["attributes"]) == {"POSITION", "NORMAL", "TEXCOORD_0"}
        assert primitive["mode"] == 4
        assert 0 <= primitive["material"] < len(document["materials"])

    print(
        "Validated glTF + GLB: "
        f"{metrics['triangleCount']} triangles, {metrics['drawPrimitiveCount']} primitives, "
        f"{metrics['materialCount']} materials, {glb_path.stat().st_size} byte GLB"
    )


if __name__ == "__main__":
    main()
