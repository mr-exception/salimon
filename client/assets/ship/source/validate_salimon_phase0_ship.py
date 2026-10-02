#!/usr/bin/env python3
"""Validate the checked-in Salimon Phase 0 glTF and GLB exports."""

from __future__ import annotations

import json
import math
import struct
from pathlib import Path

from generate_salimon_phase0_ship import (
    cockpit_components, ship_components, thruster_collision_rust_source,
    thruster_collision_specs, CARGO_SOLIDS, CARGO_ROOM_METADATA, cargo_layout_rust_source,
)


ROOT = Path(__file__).resolve().parents[1]
EXPORT = ROOT / "export"
TEXTURE = ROOT / "textures" / "salimon_floor_grip.png"
NAME = "salimon_phase0_ship"
TASK7_BASELINE_DIMENSIONS = [10.15, 3.72, 8.30]
TASK10_REVISED_DIMENSIONS = [20.90, 4.00, 21.00]
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


def accessor_values(document: dict, binary: bytes, index: int) -> list[tuple]:
    accessor = document["accessors"][index]
    view = document["bufferViews"][accessor["bufferView"]]
    size = {"VEC3": 3, "VEC2": 2, "SCALAR": 1}[accessor["type"]]
    kind = "f" if accessor["componentType"] == 5126 else "H"
    start = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
    layout = "<" + kind * size
    return list(struct.iter_unpack(layout, binary[start:start + accessor["count"] * struct.calcsize(layout)]))


def validate_center_monitor_scale(document: dict, binary: bytes, manifest: dict) -> None:
    """Check every assembly vertex, including details in shared-material meshes."""
    # Center is authored first in each merged mesh. Explicit vertex counts keep
    # the scale assertion independent of the implementation's resizing helper.
    center_vertex_counts = {
        "Monitor_Center": 4,
        "Cockpit_Monitor_Housings": 104,
        "Cockpit_Monitor_Bezels": 16,
        "Cockpit_Instrument_Fasteners": 24,
        "Cockpit_Tactile_Keys": 72,
        "Cockpit_Ready_Indicators": 24,
    }
    pivot, scale, seatward = [4.426, .80, 0.0], .49, .40
    before_positions, after_positions = [], []
    current = {component.name: component for component in cockpit_components()}
    for baseline in cockpit_components(center_monitor_scale=1.0, monitor_seatward_offset=0.0):
        exported = node(document, baseline.name)
        primitive = document["meshes"][exported["mesh"]]["primitives"][0]
        positions = accessor_values(document, binary, primitive["attributes"]["POSITION"])
        center_count = center_vertex_counts.get(baseline.name, 0)
        expected_geometry = current[baseline.name].geometry
        # DCC export may merge/reorder vertices. Compare oriented triangles with
        # all shader attributes rather than incidental accessor/index ordering.
        normals = accessor_values(document, binary, primitive["attributes"]["NORMAL"])
        uvs = accessor_values(document, binary, primitive["attributes"]["TEXCOORD_0"])
        expected = list(zip(expected_geometry.positions, expected_geometry.normals,
                            expected_geometry.texcoords))
        actual = list(zip(positions, normals, uvs))
        for vertex in actual:
            assert any(all(abs(a-b) <= (2e-4 if attribute == 1 else 1e-6)
                           for attribute, (av, ev) in enumerate(zip(vertex, candidate))
                           for a, b in zip(av, ev)) for candidate in expected), baseline.name
        from collections import Counter
        def triangles(vertices, indices):
            def key(i):
                vertex = vertices[i]
                candidate = min(expected, key=lambda e: sum(abs(x-y) for k in (0, 2)
                                                           for x, y in zip(vertex[k], e[k])))
                return tuple(x for k in (0, 2) for x in candidate[k])
            result = []
            for offset in range(0, len(indices), 3):
                triangle = tuple(key(i) for i in indices[offset:offset+3])
                result.append(min(triangle[i:] + triangle[:i] for i in range(3)))
            return Counter(result)
        indices = [v[0] for v in accessor_values(document, binary, primitive["indices"])]
        assert triangles(actual, indices) == triangles(expected, expected_geometry.indices), baseline.name
        for original, actual_position in zip(baseline.geometry.positions[:center_count],
                                             expected_geometry.positions[:center_count]):
            expected_position = [anchor + scale * (value-anchor) - (seatward if axis == 0 else 0)
                                 for axis, (value, anchor) in enumerate(zip(original, pivot))]
            assert_vectors_close(actual_position, expected_position)
            before_positions.append(original)
            after_positions.append(actual_position)

    def bounds(positions):
        return {"min": [min(point[axis] for point in positions) for axis in range(3)],
                "max": [max(point[axis] for point in positions) for axis in range(3)]}

    before, after = bounds(before_positions), bounds(after_positions)
    assert_vectors_close(before["min"], [4.264, .8, -1.02])
    assert_vectors_close(before["max"], [4.511, 1.695, 1.02])
    assert_vectors_close(after["min"], [3.94662, .8, -.4998])
    assert_vectors_close(after["max"], [4.06765, 1.23855, .4998])
    for axis in range(3):
        assert abs((after["max"][axis]-after["min"][axis]) /
                   (before["max"][axis]-before["min"][axis]) - scale) < 1e-5
    metadata = document["extras"]["salimon"]["cockpitInstruments"]["centerAssembly"]
    assert metadata == manifest["cockpitInstruments"]["centerAssembly"]
    assert metadata["uniformScaleFromAssetVersion7"] == scale
    assert metadata["uniformScaleFromAssetVersion8"] == .7
    assert metadata["seatwardOffsetMeters"] == seatward
    assert_vectors_close(metadata["pivotMeters"], pivot)
    for bound in ("min", "max"):
        assert_vectors_close(metadata["baselineBoundsMeters"][bound], before[bound])
        assert_vectors_close(metadata["boundsMeters"][bound], after[bound])
    collider = node(document, metadata["collisionNode"])
    center = collider["translation"]
    size = collider["extras"]["salimon"]["sizeMeters"]
    assert_vectors_close([value-width/2 for value, width in zip(center, size)], [3.94662, .25, -1.06])
    assert_vectors_close([value+width/2 for value, width in zip(center, size)], [6.12, 1.23855, 1.06])


def validate_sightlines(document: dict, binary: bytes) -> None:
    """Ray-test real exported triangles, catching opaque walls behind new panes."""
    def values(index):
        return accessor_values(document, binary, index)
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
    # Sample both sides of the console from the real seated eye. Rays must
    # pass through the LOWER pane, then reach the world without any opaque
    # hull, deck, monitor, furniture or collision proxy covering the view.
    for pitch,absolute_yaw in ((12,28),(18,28),(26,32),(30,34),(30,36)):
        for yaw in (-absolute_yaw, absolute_yaw):
            p,y=math.radians(pitch),math.radians(yaw)
            direction=(math.cos(p)*math.cos(y),-math.sin(p),math.cos(p)*math.sin(y))
            hits=[(d,name,glass) for name,glass,triangles in meshes for tri in triangles
                  if (d:=hit((2.76,1.799032258064516,0),direction,tri)) is not None]
            assert hits, f"lower window ray misses glazing: {pitch}/{yaw}"
            assert any(name == "Cockpit_Lower_Glazing" for _,name,_ in hits)
            assert all(glass for _,_,glass in hits), f"lower view {pitch}/{yaw} blocked: {hits}"
    # A standing player sees through the actual cargo doorway to the far wall;
    # neither the old sill/glazing, wing nor an opaque hull cap covers the opening.
    for x in (.7, 1.3, 1.8):
        origin=(x,eye,3.5)
        hits=[(d,name) for name,glass,triangles in meshes
              for tri in triangles if (d:=hit(origin,(0,0,1),tri)) is not None]
        assert min(hits)[1] == "Cargo_Port_Wall", f"cargo passage blocked at X={x}: {min(hits)}"
        assert abs(min(hits)[0]-7.1) < 1e-5
    origin=(-.5,eye,8.0)
    hits=[(d,name) for name,glass,triangles in meshes for tri in triangles
          if (d:=hit(origin,(0,-1,0),tri)) is not None]
    assert min(hits)[1] == "Cargo_Deck", "cargo floor must be level and not covered by wing geometry"
    assert abs(min(hits)[0]-1.75) < 1e-5

    # Live cockpit surfaces must present an unobstructed rectangle from the
    # actual seated eye, with top-left UVs and normals toward the pilot.
    seated_eye=(2.76,1.799032258064516,0.)
    for label, role, x, y0, y1, z0, z1 in (
        ("Center","speed",3.95446,.8735,1.2067,-.441,.441),
        ("Port","thruster-power",3.70,.905,1.22,2.035,2.665),
        ("Starboard","thruster-power",3.70,.905,1.22,-2.665,-2.035),
    ):
        name=f"Monitor_{label}"
        monitor=node(document,name)
        metadata=monitor["extras"]["salimon"]
        assert metadata["displayRole"] == role
        assert metadata["uvOrigin"] == "top-left"
        if label != "Center":
            assert metadata["powerSource"] == "shared-thruster-command"
        primitive=document["meshes"][monitor["mesh"]]["primitives"][0]
        positions=values(primitive["attributes"]["POSITION"])
        normals=values(primitive["attributes"]["NORMAL"])
        uvs=values(primitive["attributes"]["TEXCOORD_0"])
        assert len(positions)==4 and len(values(primitive["indices"]))==6
        center=(x,(y0+y1)/2,(z0+z1)/2)
        toward_eye=sub(seated_eye,center)
        horizontal_length=math.hypot(toward_eye[0],toward_eye[2])
        expected_normal=(toward_eye[0]/horizontal_length,0.,toward_eye[2]/horizontal_length)
        tangent=(expected_normal[2],0.,-expected_normal[0])
        for position, normal, uv in zip(positions,normals,uvs):
            horizontal_offset=(uv[0]-.5)*(z1-z0)
            assert_vectors_close(position,[
                center[0]+tangent[0]*horizontal_offset,
                y1-(y1-y0)*uv[1],
                center[2]+tangent[2]*horizontal_offset,
            ])
            assert_vectors_close(normal,expected_normal)
        assert set(uvs)=={(0.,0.),(0.,1.),(1.,0.),(1.,1.)}
        assert_vectors_close(metadata["pilotFacingTargetMeters"],list(seated_eye))
        expected_yaw=math.degrees(math.atan2(expected_normal[2],-expected_normal[0]))
        assert abs(metadata["pilotFacingYawDegrees"]-expected_yaw) < 1e-5
        if label != "Center":
            assert abs(expected_yaw) > 45., f"{name} still faces mostly toward ship rear"
        origin=seated_eye
        for u,v in ((.08,.08),(.92,.08),(.5,.5),(.08,.92),(.92,.92)):
            horizontal_offset=(u-.5)*(z1-z0)
            target=(center[0]+tangent[0]*horizontal_offset,
                    y1-(y1-y0)*v,
                    center[2]+tangent[2]*horizontal_offset)
            direction=sub(target,origin)
            hits=[(distance,mesh_name) for mesh_name,glass,triangles in meshes
                  if not glass for tri in triangles
                  if (distance:=hit(origin,direction,tri)) is not None]
            assert hits, f"{name} inspection ray missed all surfaces"
            nearest=min(hits)
            assert nearest[1] == name, f"{name} display occluded by {nearest[1]} at UV {(u,v)}"
            assert abs(nearest[0]-1.) < 1e-5

    # End caps used to conceal the thruster emitters. Trace down each nozzle.
    for label, z in (("Port",7.10),("Starboard",-7.10)):
        origin=(-10.5,4/3.72,z)
        hits=[]
        for name,glass,triangles in meshes:
            if not glass:
                hits.extend((distance,name) for tri in triangles
                            if (distance:=hit(origin,(1,0,0),tri)) is not None)
        assert min(hits)[1] == f"Engine_{label}_Glow", f"{label} nozzle emitter is occluded"


def main(export: Path = EXPORT) -> None:
    gltf_path = export / f"{NAME}.gltf"
    bin_path = export / f"{NAME}.bin"
    glb_path = export / f"{NAME}.glb"
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
    components = ship_components()
    assert (ROOT.parents[1] / "character" / "src" / "thruster_collision.rs").read_text() == (
        thruster_collision_rust_source(components)
    ), "portable thruster collision bounds differ from the editable asset"
    assert (ROOT.parents[1] / "character" / "src" / "cargo_layout.rs").read_text() == cargo_layout_rust_source(components), "portable cargo walls differ from the asset source"
    collision_group = node(document, "Collision_Proxies")
    for name, lower, upper in thruster_collision_specs(components):
        collider = node(document, name)
        assert document["nodes"].index(collider) in collision_group["children"]
        assert "mesh" not in collider, "collision proxies must not add draw calls"
        metadata = collider["extras"]["salimon"]
        assert metadata["collisionShape"] == "box"
        assert metadata["purpose"] == "exterior-thruster"
        assert_vectors_close(collider["translation"], [(a + b) / 2 for a, b in zip(lower, upper)])
        assert_vectors_close(metadata["sizeMeters"], [b - a for a, b in zip(lower, upper)])
    # The two engines must not become one invisible wall across the aft gate.
    port = node(document, "COLLIDER_Engine_Port_Body")
    starboard = node(document, "COLLIDER_Engine_Starboard_Body")
    assert port["translation"][2] - port["extras"]["salimon"]["sizeMeters"][2] / 2 > 5.1
    assert starboard["translation"][2] + starboard["extras"]["salimon"]["sizeMeters"][2] / 2 < -5.1
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
        "Pilot_Seat_Shell",
        "Pilot_Seat_Cushions",
        "Pilot_Seat_Bolsters",
        "Pilot_Seat_Headrest",
        "Pilot_Seat_Armrests",
        "Pilot_Seat_Controls",
        "Cockpit_Monitor_Housings",
        "Cockpit_Monitor_Bezels",
        "Monitor_Center",
        "Monitor_Port",
        "Monitor_Starboard",
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
    assert windows["lowerGlazingNode"] == "Cockpit_Lower_Glazing"
    lower_node=node(document, windows["lowerGlazingNode"])
    lower_primitive=document["meshes"][lower_node["mesh"]]["primitives"][0]
    assert document["materials"][lower_primitive["material"]]["name"] == "Cockpit Glass"
    assert lower_node["extras"]["salimon"]["exterior_visibility"] is True
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
    assert_vectors_close(exterior_collider["translation"], [0.05, 1.76*4/3.72, .5])
    assert_vectors_close(
        exterior_collider["extras"]["salimon"]["sizeMeters"],
        [20.90, 4.00, 21.0],
    )
    assert metrics["externalAssetDependencies"] == 0
    assert metrics["triangleCount"] <= 6000, "triangle budget exceeded"
    assert metrics["drawPrimitiveCount"] <= 120, "primitive budget exceeded"
    assert metrics["materialCount"] <= 13, "material budget exceeded"
    assert len(glb_path.read_bytes()) <= 512 * 1024, "GLB size budget exceeded"

    # Compare measured payload counts with the declared budget contract.
    manifest = json.loads((ROOT / "asset-manifest.json").read_text())
    assert manifest["assetVersion"] == document["asset"]["extras"]["salimon"]["assetVersion"]
    assert manifest["cockpitWindows"]["lowerGlazingNode"] == windows["lowerGlazingNode"]
    assert manifest["cockpitWindows"]["lowerViewSamplesDegrees"] == windows["lowerViewSamplesDegrees"]
    lower_normals=accessor_values(document,geometry,lower_primitive["attributes"]["NORMAL"])
    assert all(abs(sum(n*n for n in normal)-1.0)<1e-5 for normal in lower_normals)
    assert any(normal[1] > .99 for normal in lower_normals), "floor panes face upward"
    assert any(normal[1] < -.9 for normal in lower_normals), "nose pane faces downward"

    assert manifest["budgets"]["maximumTriangles"] == 6000
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
    assert metrics["cargoRoom"] == manifest["interior"]["cargoRoom"]
    assert metrics["cargoRoom"] == CARGO_ROOM_METADATA
    for name in CARGO_SOLIDS:
        lower, upper = node_position_bounds(document, name)
        proxy = node(document, "COLLIDER_" + name)
        assert_vectors_close(proxy["translation"], [(a+b)/2 for a,b in zip(lower,upper)])
        assert_vectors_close(proxy["extras"]["salimon"]["sizeMeters"], [b-a for a,b in zip(lower,upper)])
    assert abs(node_position_bounds(document, "Cargo_Deck")[1][1] - .23*4/3.72) < 1e-6
    # All corners of the expanded exterior remain within the flight sphere.
    assert max(sum(p[a]**2 for a in range(3))**.5
               for component in components for p in component.geometry.positions) < 16.0
    # Profile geometry must remain shaped (not a relabeled cube) and compact.
    for name in ("Pilot_Seat_Shell","Pilot_Seat_Back","Pilot_Seat_Cushions",
                 "Pilot_Seat_Bolsters","Pilot_Seat_Headrest"):
        shaped=node(document,name)
        primitive=document["meshes"][shaped["mesh"]]["primitives"][0]
        assert document["accessors"][primitive["attributes"]["POSITION"]]["count"] >= 40
        lower,upper=node_position_bounds(document,name)
        assert lower[0] >= 1.55 and upper[0] <= 3.35
        assert lower[2] >= -.80 and upper[2] <= .80
        assert upper[1] < windows["standingViewpointMeters"][1]-.25
    assert metrics["cockpitInstruments"]["powerSource"] == "shared-thruster-command"
    assert metrics["cockpitInstruments"]["additionalDraws"] == 0
    instruments = manifest["cockpitInstruments"]
    assert_vectors_close(instruments["pilotFacingTargetMeters"], windows["seatedViewpointMeters"])
    for label in ("Center", "Port", "Starboard"):
        name = f"Monitor_{label}"
        if label != "Center":
            assert instruments["sideYawDegrees"][name] == node(document, name)["extras"]["salimon"]["pilotFacingYawDegrees"]
        lower, upper = node_position_bounds(document, name)
        assert_vectors_close(instruments["surfaceBoundsMeters"][name]["min"], lower)
        assert_vectors_close(instruments["surfaceBoundsMeters"][name]["max"], upper)
    validate_center_monitor_scale(document, geometry, manifest)
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
        assert primitive.get("mode", 4) == 4
        assert 0 <= primitive["material"] < len(document["materials"])

    print(
        "Validated glTF + GLB: "
        f"{metrics['triangleCount']} triangles, {metrics['drawPrimitiveCount']} primitives, "
        f"{metrics['materialCount']} materials, {glb_path.stat().st_size} byte GLB"
    )


if __name__ == "__main__":
    main()
