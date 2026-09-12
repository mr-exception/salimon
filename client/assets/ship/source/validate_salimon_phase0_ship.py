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
TASK10_REVISED_DIMENSIONS = [20.30, 4.00, 20.00]
TASK10_REVISED_SCALE = [2.0, 4.0 / 3.72, 2.0]


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


def validate_sightlines(document: dict, binary: bytes) -> None:
    """Ray-test real exported triangles, catching opaque walls behind new panes."""
    def values(index):
        accessor=document["accessors"][index]
        view=document["bufferViews"][accessor["bufferView"]]
        size=3 if accessor["type"]=="VEC3" else 1
        kind="f" if accessor["componentType"]==5126 else "H"
        start=view.get("byteOffset",0)+accessor.get("byteOffset",0)
        return list(struct.iter_unpack("<"+kind*size,binary[start:start+accessor["count"]*struct.calcsize("<"+kind*size)]))
    def sub(a,b): return tuple(x-y for x,y in zip(a,b))
    def dot(a,b): return sum(x*y for x,y in zip(a,b))
    def cross(a,b): return (a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
    def hit(origin,direction,tri):
        a,b,c=tri; e1,e2=sub(b,a),sub(c,a); h=cross(direction,e2); det=dot(e1,h)
        if abs(det)<1e-8:return None
        inv=1/det; delta=sub(origin,a); u=inv*dot(delta,h)
        if u<0 or u>1:return None
        q=cross(delta,e1); v=inv*dot(direction,q)
        if v<0 or u+v>1:return None
        distance=inv*dot(e2,q)
        return distance if distance>.001 else None
    meshes=[]
    for mesh in document["meshes"]:
        p=mesh["primitives"][0]; positions=values(p["attributes"]["POSITION"]); indices=values(p["indices"])
        triangles=[[positions[indices[i+j][0]] for j in range(3)] for i in range(0,len(indices),3)]
        glass=document["materials"][p["material"]]["name"]=="Cockpit Glass"
        meshes.append((mesh["name"],glass,triangles))
    eye=.23*4/3.72+1.75
    checks=[("behind chair forward",(1.3,eye,0),(1,0,0)),
            ("seated forward",(2.76,1.799032258064516,0),(1,0,0)),
            ("port window",(-4.5,eye,2.2),(0,0,1)),
            ("starboard window",(-4.5,eye,-2.2),(0,0,-1)),
            ("aft port window",(-5.8,eye,2.6),(-1,0,0)),
            ("aft starboard window",(-5.8,eye,-2.6),(-1,0,0))]
    for label,origin,direction in checks:
        glass_hit=False
        for name,glass,triangles in meshes:
            distances=[d for tri in triangles if (d:=hit(origin,direction,tri)) is not None]
            if distances:
                if glass:glass_hit=True
                else:assert False, f"{label} obstructed by {name} at {min(distances):.2f}m"
        assert glass_hit, f"{label} misses modeled glazing"
    # End caps used to conceal the thruster emitters. Trace down each nozzle.
    for label, z in (("Port",7.10),("Starboard",-7.10)):
        origin=(-10.5,4/3.72,z)
        hits=[]
        for name,glass,triangles in meshes:
            if not glass:
                hits.extend((distance,name) for tri in triangles
                            if (distance:=hit(origin,(1,0,0),tri)) is not None)
        assert min(hits)[1] == f"Engine_{label}_Glow", f"{label} nozzle emitter is occluded"


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
        "Cabin_Port_Glazing",
        "Cabin_Starboard_Glazing",
        "Cabin_Aft_Glazing",
        "Energy_Core",
        "Core_Pedestal",
        "COLLIDER_EnergyCore",
        "Ceiling_Warm_Lights",
        "Engine_Port_Nozzle",
        "Engine_Starboard_Nozzle",
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
    assert glazing_min[1] <= 1.08 and glazing_max[1] >= 2.83
    assert glazing_min[2] <= -3.0 and glazing_max[2] >= 3.0
    assert nose_max[1] <= 1.10, "solid nose must stay below the forward window"
    assert console_max[1] <= 1.30, "console must stay below the seated eye line"
    assert nose_max[0] >= console_max[0], "nose must still extend beyond the forward console"

    windows = document["extras"]["salimon"]["cockpitWindows"]
    assert windows["glazingNode"] == "Cockpit_Glazing"
    assert windows["material"] == "Cockpit Glass"
    assert_vectors_close(windows["seatedViewpointMeters"], [2.76, 1.799032258064516, 0.0])
    assert_vectors_close(windows["standingViewpointMeters"], [1.30, 1.9973118279569892, 0.0])
    assert windows["dynamicShadows"] is False
    assert windows["postEffects"] is False

    metrics = document["extras"]["salimon"]
    assert_vectors_close(metrics["scaleFromTask7Baseline"], TASK10_REVISED_SCALE)
    assert metrics["humanBodyHeightMeters"] == 1.80
    assert metrics["humanEyeHeightMeters"] == 1.75
    assert_vectors_close(metrics["task7BaselineDimensionsMeters"], TASK7_BASELINE_DIMENSIONS)
    assert_vectors_close(metrics["overallDimensionsMeters"], TASK10_REVISED_DIMENSIONS)
    overall_min, overall_max = overall_position_bounds(document)
    measured_dimensions = [maximum - minimum for minimum, maximum in zip(overall_min, overall_max)]
    assert_vectors_close(measured_dimensions, TASK10_REVISED_DIMENSIONS)
    assert measured_dimensions[0] >= TASK7_BASELINE_DIMENSIONS[0] * 2.0
    assert measured_dimensions[2] >= TASK7_BASELINE_DIMENSIONS[2] * 2.0
    assert abs(overall_min[1] - metrics["lowestLocalYMeters"]) <= 1.0e-6
    assert_vectors_close(node(document, "MARKER_CockpitSeat")["translation"], [2.76, 1.129032258064516, 0.0])
    assert_vectors_close(node(document, "MARKER_ExitDoor")["translation"], [-7.10, 1.3440860215053763, 0.0])
    assert_vectors_close(node(document, "MARKER_PlayerStart")["translation"], [0.50, 1.9973118279569892, -2.20])
    exterior_collider = node(document, "COLLIDER_ExteriorHull")
    assert_vectors_close(exterior_collider["translation"], [0.35, 1.76*4/3.72, 0.0])
    assert_vectors_close(
        exterior_collider["extras"]["salimon"]["sizeMeters"],
        [20.30, 4.00, 20.0],
    )
    assert metrics["externalAssetDependencies"] == 0
    assert metrics["triangleCount"] <= 4500, "triangle budget exceeded"
    assert metrics["drawPrimitiveCount"] <= 100, "primitive budget exceeded"
    assert metrics["materialCount"] <= 13, "material budget exceeded"
    assert len(glb_path.read_bytes()) <= 512 * 1024, "GLB size budget exceeded"

    # Compare measured payload counts with the declared budget contract.
    manifest = json.loads((ROOT / "asset-manifest.json").read_text())
    assert manifest["assetVersion"] == document["asset"]["extras"]["salimon"]["assetVersion"]
    assert manifest["budgets"]["maximumTriangles"] == 4500
    measured_triangles = sum(document["accessors"][p["indices"]]["count"] // 3
                             for mesh in document["meshes"] for p in mesh["primitives"])
    assert measured_triangles == metrics["triangleCount"]
    assert len(document["meshes"]) == metrics["drawPrimitiveCount"]
    assert len(document["materials"]) == metrics["materialCount"]
    assert_vectors_close(manifest["task10Scale"]["dimensionsMeters"], measured_dimensions)
    assert metrics["walkableInteriorClearanceMeters"]["width"] == 9.2
    floor = node(document, "COLLIDER_InteriorFloor")
    floor_top = floor["translation"][1] + floor["extras"]["salimon"]["sizeMeters"][1] / 2
    deck_min, deck_max = node_position_bounds(document, "Deck_Walkable")
    assert abs(floor_top - deck_max[1]) < 1e-6, "visible floor and walking plane differ"
    _, seat_max = node_position_bounds(document, "Pilot_Seat_Back")
    assert seat_max[1] < windows["standingViewpointMeters"][1] - .35
    assert_vectors_close(node(document,"COLLIDER_EnergyCore")["translation"],[-1.0,1.07*4/3.72,0.0])
    # Visual redesigns must remain inside the gameplay collider so the cage
    # never snags a walking route or protrudes through the player's capsule.
    core_collider = node(document, "COLLIDER_EnergyCore")
    core_center = core_collider["translation"]
    core_size = core_collider["extras"]["salimon"]["sizeMeters"]
    for name in mesh_names:
        if name.startswith(("Core_", "Energy_Core")) and name != "Core_Power_Conduits":
            core_min, core_max = node_position_bounds(document, name)
            for axis in range(3):
                assert core_min[axis] >= core_center[axis]-core_size[axis]/2-1e-6, name
                assert core_max[axis] <= core_center[axis]+core_size[axis]/2+1e-6, name
    for collider_name, mesh_name in (("COLLIDER_AftDoor", "Exit_Door"),
                                     ("COLLIDER_InteriorCeiling", "Ceiling_Inner")):
        collider = node(document, collider_name)
        center = collider["translation"]
        size = collider["extras"]["salimon"]["sizeMeters"]
        mesh_min, mesh_max = node_position_bounds(document, mesh_name)
        for axis in range(3):
            assert abs(center[axis]-size[axis]/2-mesh_min[axis]) < 1e-6
            assert abs(center[axis]+size[axis]/2-mesh_max[axis]) < 1e-6
    assert metrics["energyCore"]["role"] == "energy-storage"
    assert metrics["energyCore"]["phase0"] == "visual-only"
    roof = node(document, "Hull_Roof")
    roof_primitive = document["meshes"][roof["mesh"]]["primitives"][0]
    position_accessor = document["accessors"][roof_primitive["attributes"]["POSITION"]]
    normal_accessor = document["accessors"][roof_primitive["attributes"]["NORMAL"]]
    position_view = document["bufferViews"][position_accessor["bufferView"]]
    normal_view = document["bufferViews"][normal_accessor["bufferView"]]
    roof_top_y = position_accessor["max"][1]
    upward_top_vertices = 0
    for index in range(position_accessor["count"]):
        position = struct.unpack_from("<fff", geometry, position_view["byteOffset"] + index*12)
        normal = struct.unpack_from("<fff", geometry, normal_view["byteOffset"] + index*12)
        if abs(position[1] - roof_top_y) < 1e-6 and abs(normal[1]) > .99:
            assert normal[1] > 0, "roof top winds inward in standard glTF viewers"
            upward_top_vertices += 1
    assert upward_top_vertices >= 4
    validate_sightlines(document, geometry)

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
