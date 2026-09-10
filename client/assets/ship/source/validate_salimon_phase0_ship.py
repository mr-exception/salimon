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

    required_nodes = {
        "Exterior",
        "Interior",
        "Collision_Proxies",
        "Interaction_Markers",
        "Hull_Nose",
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

    metrics = document["extras"]["salimon"]
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
