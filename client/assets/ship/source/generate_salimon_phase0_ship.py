#!/usr/bin/env python3
"""Generate Salimon's custom Phase 0 ship as glTF 2.0 and GLB.

The script is the editable, deterministic DCC source for Tasks 7, 9, and 10. It uses only
the Python standard library so later agents can reshape the ship by changing
named dimensions/components and regenerate the runtime exports without Blender.
The emitted .gltf can also be imported directly into Blender for hand editing.
"""

from __future__ import annotations

import argparse
import copy
import json
import math
import struct
import zlib
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable, Sequence


ROOT = Path(__file__).resolve().parents[1]
EXPORT_DIR = ROOT / "export"
TEXTURE_DIR = ROOT / "textures"
MODEL_NAME = "salimon_phase0_ship"
TASK7_BASELINE_DIMENSIONS_METERS = [10.15, 3.72, 8.30]
LINEAR_SCALE_FROM_TASK7 = 2.0
PLAYER_EYE_HEIGHT_METERS = 1.62
SCALED_FLOOR_HEIGHT_METERS = 0.23 * LINEAR_SCALE_FROM_TASK7
COCKPIT_SEAT_MARKER_METERS = [1.38 * LINEAR_SCALE_FROM_TASK7, 1.05 * LINEAR_SCALE_FROM_TASK7, 0.0]
COCKPIT_VIEWPOINT_METERS = [
    COCKPIT_SEAT_MARKER_METERS[0],
    COCKPIT_SEAT_MARKER_METERS[1] + 0.67,
    0.0,
]
STANDING_VIEWPOINT_METERS = [
    0.65 * LINEAR_SCALE_FROM_TASK7,
    SCALED_FLOOR_HEIGHT_METERS + PLAYER_EYE_HEIGHT_METERS,
    0.0,
]
PLAYER_START_METERS = [
    -0.35 * LINEAR_SCALE_FROM_TASK7,
    SCALED_FLOOR_HEIGHT_METERS + PLAYER_EYE_HEIGHT_METERS,
    0.0,
]


def vector_sub(a: Sequence[float], b: Sequence[float]) -> tuple[float, float, float]:
    return (a[0] - b[0], a[1] - b[1], a[2] - b[2])


def cross(a: Sequence[float], b: Sequence[float]) -> tuple[float, float, float]:
    return (
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    )


def normalize(value: Sequence[float]) -> tuple[float, float, float]:
    length = math.sqrt(sum(component * component for component in value))
    if length == 0.0:
        raise ValueError("zero-length face normal")
    return tuple(component / length for component in value)  # type: ignore[return-value]


@dataclass
class Geometry:
    positions: list[tuple[float, float, float]] = field(default_factory=list)
    normals: list[tuple[float, float, float]] = field(default_factory=list)
    texcoords: list[tuple[float, float]] = field(default_factory=list)
    indices: list[int] = field(default_factory=list)

    def extend(self, other: Geometry) -> None:
        base = len(self.positions)
        self.positions.extend(other.positions)
        self.normals.extend(other.normals)
        self.texcoords.extend(other.texcoords)
        self.indices.extend(base + index for index in other.indices)

    def add_face(self, points: Sequence[tuple[float, float, float]]) -> None:
        if len(points) < 3:
            raise ValueError("a face requires at least three points")
        normal = normalize(cross(vector_sub(points[1], points[0]), vector_sub(points[2], points[0])))
        base = len(self.positions)
        self.positions.extend(points)
        self.normals.extend([normal] * len(points))
        if len(points) == 4:
            self.texcoords.extend([(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)])
        else:
            center_x = sum(point[0] for point in points) / len(points)
            center_z = sum(point[2] for point in points) / len(points)
            radius = max(
                max(abs(point[0] - center_x), abs(point[2] - center_z)) for point in points
            ) or 1.0
            self.texcoords.extend(
                [
                    (0.5 + (point[0] - center_x) / (2.0 * radius), 0.5 + (point[2] - center_z) / (2.0 * radius))
                    for point in points
                ]
            )
        for index in range(1, len(points) - 1):
            self.indices.extend((base, base + index, base + index + 1))


def scaled_vector(values: Sequence[float]) -> list[float]:
    return [value * LINEAR_SCALE_FROM_TASK7 for value in values]


def scaled_geometry(geometry: Geometry) -> Geometry:
    return Geometry(
        positions=[tuple(scaled_vector(position)) for position in geometry.positions],
        normals=geometry.normals.copy(),
        texcoords=geometry.texcoords.copy(),
        indices=geometry.indices.copy(),
    )


def box(x0: float, x1: float, y0: float, y1: float, z0: float, z1: float) -> Geometry:
    geometry = Geometry()
    geometry.add_face([(x1, y0, z0), (x1, y1, z0), (x1, y1, z1), (x1, y0, z1)])
    geometry.add_face([(x0, y0, z1), (x0, y1, z1), (x0, y1, z0), (x0, y0, z0)])
    geometry.add_face([(x0, y1, z0), (x0, y1, z1), (x1, y1, z1), (x1, y1, z0)])
    geometry.add_face([(x0, y0, z1), (x0, y0, z0), (x1, y0, z0), (x1, y0, z1)])
    geometry.add_face([(x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)])
    geometry.add_face([(x1, y0, z0), (x0, y0, z0), (x0, y1, z0), (x1, y1, z0)])
    return geometry


def tapered_box(
    x0: float,
    x1: float,
    y0_start: float,
    y1_start: float,
    z_start: float,
    y0_end: float,
    y1_end: float,
    z_end: float,
) -> Geometry:
    """Build a closed rectangular frustum along +X."""
    geometry = Geometry()
    a = (x0, y0_start, -z_start)
    b = (x0, y1_start, -z_start)
    c = (x0, y1_start, z_start)
    d = (x0, y0_start, z_start)
    e = (x1, y0_end, -z_end)
    f = (x1, y1_end, -z_end)
    g = (x1, y1_end, z_end)
    h = (x1, y0_end, z_end)
    geometry.add_face([e, f, g, h])
    geometry.add_face([d, c, b, a])
    geometry.add_face([b, c, g, f])
    geometry.add_face([d, a, e, h])
    geometry.add_face([d, h, g, c])
    geometry.add_face([a, b, f, e])
    return geometry


def cockpit_glazing(
    x0: float,
    x1: float,
    y0_start: float,
    y1_start: float,
    z_start: float,
    y0_end: float,
    y1_end: float,
    z_end: float,
) -> Geometry:
    """Build the front, roof, and side panes of an open-backed canopy."""
    geometry = Geometry()
    a = (x0, y0_start, -z_start)
    b = (x0, y1_start, -z_start)
    c = (x0, y1_start, z_start)
    d = (x0, y0_start, z_start)
    e = (x1, y0_end, -z_end)
    f = (x1, y1_end, -z_end)
    g = (x1, y1_end, z_end)
    h = (x1, y0_end, z_end)
    geometry.add_face([e, f, g, h])
    geometry.add_face([b, c, g, f])
    geometry.add_face([d, h, g, c])
    geometry.add_face([a, b, f, e])
    return geometry


def cockpit_window_frame() -> Geometry:
    """Combine lightweight canopy rails into one draw primitive."""
    geometry = Geometry()
    for rail in (
        box(2.50, 2.62, 1.00, 2.72, 1.48, 1.62),
        box(2.50, 2.62, 1.00, 2.72, -1.62, -1.48),
        box(2.50, 4.92, 2.55, 2.69, -0.09, 0.09),
        box(4.80, 4.96, 0.72, 2.04, 0.32, 0.46),
        box(4.80, 4.96, 0.72, 2.04, -0.46, -0.32),
    ):
        geometry.extend(rail)
    return geometry


def extrude_y(footprint: Sequence[tuple[float, float]], y0: float, y1: float) -> Geometry:
    """Extrude a counter-clockwise X/Z footprint between two Y planes."""
    geometry = Geometry()
    bottom = [(x, y0, z) for x, z in reversed(footprint)]
    top = [(x, y1, z) for x, z in footprint]
    geometry.add_face(bottom)
    geometry.add_face(top)
    for index, (x0, z0) in enumerate(footprint):
        x1, z1 = footprint[(index + 1) % len(footprint)]
        geometry.add_face([(x0, y0, z0), (x1, y0, z1), (x1, y1, z1), (x0, y1, z0)])
    return geometry


def cylinder_x(x0: float, x1: float, cy: float, cz: float, radius: float, segments: int = 12) -> Geometry:
    geometry = Geometry()
    start = []
    end = []
    for index in range(segments):
        angle = (2.0 * math.pi * index) / segments
        point = (cy + math.cos(angle) * radius, cz + math.sin(angle) * radius)
        start.append((x0, point[0], point[1]))
        end.append((x1, point[0], point[1]))
    geometry.add_face(list(reversed(start)))
    geometry.add_face(end)
    for index in range(segments):
        next_index = (index + 1) % segments
        geometry.add_face([start[index], start[next_index], end[next_index], end[index]])
    return geometry


def png_rgba(width: int, height: int, pixels: Iterable[tuple[int, int, int, int]]) -> bytes:
    raw = bytearray()
    iterator = iter(pixels)
    for _ in range(height):
        raw.append(0)
        for _ in range(width):
            raw.extend(next(iterator))

    def chunk(kind: bytes, payload: bytes) -> bytes:
        return struct.pack(">I", len(payload)) + kind + payload + struct.pack(">I", zlib.crc32(kind + payload))

    signature = b"\x89PNG\r\n\x1a\n"
    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return signature + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(bytes(raw), 9)) + chunk(b"IEND", b"")


def make_grip_texture() -> bytes:
    pixels = []
    for y in range(16):
        for x in range(16):
            groove = ((x + y) % 8 == 0) or ((x - y) % 8 == 0)
            pixels.append((48, 57, 62, 255) if groove else (76, 86, 91, 255))
    return png_rgba(16, 16, pixels)


class BufferBuilder:
    def __init__(self) -> None:
        self.data = bytearray()
        self.buffer_views: list[dict[str, object]] = []
        self.accessors: list[dict[str, object]] = []

    def align(self, amount: int = 4) -> None:
        self.data.extend(b"\0" * ((-len(self.data)) % amount))

    def append(self, payload: bytes, target: int | None = None) -> int:
        self.align()
        offset = len(self.data)
        self.data.extend(payload)
        view: dict[str, object] = {"buffer": 0, "byteOffset": offset, "byteLength": len(payload)}
        if target is not None:
            view["target"] = target
        self.buffer_views.append(view)
        return len(self.buffer_views) - 1

    def accessor_vec(self, values: Sequence[Sequence[float]], dimensions: int, attribute: str) -> int:
        payload = b"".join(struct.pack("<" + "f" * dimensions, *value) for value in values)
        view = self.append(payload, 34962)
        accessor: dict[str, object] = {
            "bufferView": view,
            "componentType": 5126,
            "count": len(values),
            "type": f"VEC{dimensions}",
        }
        if attribute == "POSITION":
            accessor["min"] = [min(value[index] for value in values) for index in range(dimensions)]
            accessor["max"] = [max(value[index] for value in values) for index in range(dimensions)]
        self.accessors.append(accessor)
        return len(self.accessors) - 1

    def accessor_indices(self, values: Sequence[int]) -> int:
        if max(values, default=0) > 65535:
            raise ValueError("mesh exceeds unsigned-short index range")
        view = self.append(b"".join(struct.pack("<H", value) for value in values), 34963)
        self.accessors.append(
            {
                "bufferView": view,
                "componentType": 5123,
                "count": len(values),
                "type": "SCALAR",
                "min": [min(values)],
                "max": [max(values)],
            }
        )
        return len(self.accessors) - 1


MATERIALS = [
    ("Hull Graphite", [0.12, 0.15, 0.17, 1.0], 0.68, 0.42, None, None),
    ("Hull Ceramic", [0.56, 0.60, 0.60, 1.0], 0.52, 0.20, None, None),
    ("Copper Accent", [0.48, 0.20, 0.08, 1.0], 0.42, 0.62, None, None),
    ("Cockpit Glass", [0.04, 0.22, 0.28, 0.24], 0.18, 0.12, [0.0, 0.05, 0.06], None),
    ("Interior Light", [0.55, 0.58, 0.56, 1.0], 0.70, 0.08, None, None),
    ("Interior Dark", [0.10, 0.12, 0.13, 1.0], 0.78, 0.12, None, None),
    ("Cyan Display", [0.015, 0.22, 0.27, 1.0], 0.28, 0.18, [0.0, 0.75, 0.92], None),
    ("Floor Grip", [0.42, 0.45, 0.45, 1.0], 0.86, 0.04, None, 0),
    ("Engine Glow", [0.12, 0.24, 0.30, 1.0], 0.30, 0.28, [0.08, 0.52, 0.78], None),
]


def make_materials() -> list[dict[str, object]]:
    result = []
    for name, color, roughness, metallic, emissive, texture in MATERIALS:
        pbr: dict[str, object] = {
            "baseColorFactor": color,
            "roughnessFactor": roughness,
            "metallicFactor": metallic,
        }
        if texture is not None:
            pbr["baseColorTexture"] = {"index": texture, "texCoord": 0}
        is_glass = name == "Cockpit Glass"
        material: dict[str, object] = {
            "name": name,
            "pbrMetallicRoughness": pbr,
            "doubleSided": is_glass,
        }
        if is_glass:
            material["alphaMode"] = "BLEND"
        if emissive is not None:
            material["emissiveFactor"] = emissive
        result.append(material)
    return result


@dataclass(frozen=True)
class Component:
    name: str
    geometry: Geometry
    material: int
    group: str
    extras: dict[str, object] = field(default_factory=dict)


def ship_components() -> list[Component]:
    components: list[Component] = []

    def add(name: str, geometry: Geometry, material: int, group: str, **extras: object) -> None:
        components.append(Component(name, scaled_geometry(geometry), material, group, extras))

    # Exterior: the Task 7 baseline is uniformly enlarged for the Task 10 pass.
    add("Hull_Belly", box(-4.2, 3.3, -0.10, 0.18, -2.05, 2.05), 0, "Exterior")
    add("Hull_Roof", box(-3.7, 2.5, 2.72, 3.08, -1.72, 1.72), 1, "Exterior")
    add("Hull_Port_Side", box(-3.8, 2.6, 0.18, 2.72, 1.72, 2.05), 0, "Exterior")
    add("Hull_Starboard_Side", box(-3.8, 2.6, 0.18, 2.72, -2.05, -1.72), 0, "Exterior")
    # Keep the lower nose solid while leaving the cockpit volume above it open.
    # The canopy panes below are the only geometry across the forward sightline.
    add("Hull_Nose", tapered_box(2.5, 5.25, 0.10, 1.02, 1.95, 0.62, 0.76, 0.36), 1, "Exterior")
    add("Hull_Aft_Cap", box(-4.25, -3.8, 0.18, 2.72, -2.05, 2.05), 0, "Exterior")
    port_wing = [(1.65, 1.88), (-2.60, 1.88), (-4.05, 4.15), (0.50, 3.42)]
    starboard_wing = [(0.50, -3.42), (-4.05, -4.15), (-2.60, -1.88), (1.65, -1.88)]
    add("Wing_Port", extrude_y(port_wing, 0.18, 0.38), 0, "Exterior")
    add("Wing_Starboard", extrude_y(starboard_wing, 0.18, 0.38), 0, "Exterior")
    add("Wing_Port_Accent", extrude_y([(0.55, 3.28), (-3.55, 3.93), (-3.25, 3.55), (0.65, 3.05)], 0.39, 0.44), 2, "Exterior")
    add("Wing_Starboard_Accent", extrude_y([(0.65, -3.05), (-3.25, -3.55), (-3.55, -3.93), (0.55, -3.28)], 0.39, 0.44), 2, "Exterior")
    add("Engine_Port", cylinder_x(-4.85, -1.90, 0.83, 2.78, 0.48), 0, "Exterior")
    add("Engine_Starboard", cylinder_x(-4.85, -1.90, 0.83, -2.78, 0.48), 0, "Exterior")
    add("Engine_Port_Glow", cylinder_x(-4.90, -4.84, 0.83, 2.78, 0.33), 8, "Exterior")
    add("Engine_Starboard_Glow", cylinder_x(-4.90, -4.84, 0.83, -2.78, 0.33), 8, "Exterior")
    add("Dorsal_Spine", tapered_box(-3.40, 2.30, 3.07, 3.62, 0.48, 3.08, 3.18, 0.12), 2, "Exterior")
    add("Cockpit_Glazing", cockpit_glazing(2.58, 4.92, 1.00, 2.64, 1.50, 0.72, 2.02, 0.42), 3, "Exterior", exterior_visibility=True)
    add("Cockpit_Window_Frame", cockpit_window_frame(), 2, "Exterior")
    add("Hull_Port_Detail", box(-2.70, 1.50, 1.02, 1.16, 2.05, 2.12), 2, "Exterior")
    add("Hull_Starboard_Detail", box(-2.70, 1.50, 1.02, 1.16, -2.12, -2.05), 2, "Exterior")

    # Interior: a clear 3.1 m wide path from the cockpit to the aft exit.
    add("Deck_Walkable", box(-3.78, 3.28, 0.18, 0.28, -1.55, 1.55), 7, "Interior", walkable=True)
    add("Ceiling_Inner", box(-3.62, 2.46, 2.61, 2.71, -1.54, 1.54), 4, "Interior")
    add("Wall_Port_Inner", box(-3.62, 2.45, 0.28, 2.62, 1.54, 1.68), 4, "Interior")
    add("Wall_Starboard_Inner", box(-3.62, 2.45, 0.28, 2.62, -1.68, -1.54), 4, "Interior")
    add("Aft_Bulkhead_Port", box(-3.78, -3.62, 0.28, 2.62, 0.72, 1.55), 5, "Interior")
    add("Aft_Bulkhead_Starboard", box(-3.78, -3.62, 0.28, 2.62, -1.55, -0.72), 5, "Interior")
    add("Aft_Bulkhead_Header", box(-3.78, -3.62, 2.34, 2.62, -0.72, 0.72), 5, "Interior")
    add(
        "Exit_Door",
        box(-3.91, -3.77, 0.28, 2.34, -0.70, 0.70),
        2,
        "Interior",
        interactive="exit-door",
        pivot=scaled_vector([-3.84, 0.28, 0.70]),
    )
    add("Door_Threshold", box(-3.94, -3.55, 0.20, 0.32, -0.78, 0.78), 2, "Interior")
    add("Cockpit_Console_Center", tapered_box(2.16, 3.12, 0.28, 1.02, 0.74, 0.42, 1.20, 0.52), 5, "Interior")
    add("Cockpit_Console_Port", box(1.78, 3.02, 0.34, 0.82, 0.82, 1.48), 5, "Interior")
    add("Cockpit_Console_Starboard", box(1.78, 3.02, 0.34, 0.82, -1.48, -0.82), 5, "Interior")
    add("Monitor_Center", box(2.34, 2.39, 0.75, 1.14, -0.52, 0.52), 6, "Interior", interactive="cockpit-monitor")
    add("Monitor_Port", box(2.08, 2.13, 0.74, 1.02, 0.91, 1.38), 6, "Interior")
    add("Monitor_Starboard", box(2.08, 2.13, 0.74, 1.02, -1.38, -0.91), 6, "Interior")
    add("Pilot_Seat_Base", box(0.90, 1.62, 0.28, 0.58, -0.48, 0.48), 5, "Interior")
    add("Pilot_Seat_Back", box(0.78, 1.05, 0.54, 1.78, -0.52, 0.52), 5, "Interior", interactive="cockpit-seat")
    add("Pilot_Seat_Accent", box(1.04, 1.12, 0.68, 1.54, -0.42, 0.42), 2, "Interior")
    add("Cabin_Bench_Port", box(-2.75, -0.75, 0.30, 0.72, 1.02, 1.48), 5, "Interior")
    add("Cabin_Storage_Starboard", box(-2.92, -1.58, 0.30, 1.22, -1.48, -1.02), 0, "Interior")
    add("Ceiling_Light_Forward", box(0.35, 1.90, 2.52, 2.61, -0.14, 0.14), 6, "Interior")
    add("Ceiling_Light_Aft", box(-2.65, -0.55, 2.52, 2.61, -0.14, 0.14), 6, "Interior")
    return components


def build_document(texture_bytes: bytes) -> tuple[dict[str, object], bytes]:
    builder = BufferBuilder()
    meshes: list[dict[str, object]] = []
    nodes: list[dict[str, object]] = []
    components = ship_components()

    root_index = 0
    groups = {"Exterior": 1, "Interior": 2, "Collision": 3, "Markers": 4}
    nodes.extend(
        [
            {"name": "Salimon_Phase0_Scout", "children": [1, 2, 3, 4]},
            {"name": "Exterior", "children": []},
            {"name": "Interior", "children": []},
            {"name": "Collision_Proxies", "children": []},
            {"name": "Interaction_Markers", "children": []},
        ]
    )

    for component in components:
        position = builder.accessor_vec(component.geometry.positions, 3, "POSITION")
        normal = builder.accessor_vec(component.geometry.normals, 3, "NORMAL")
        texcoord = builder.accessor_vec(component.geometry.texcoords, 2, "TEXCOORD_0")
        indices = builder.accessor_indices(component.geometry.indices)
        mesh_index = len(meshes)
        meshes.append(
            {
                "name": component.name,
                "primitives": [
                    {
                        "attributes": {"POSITION": position, "NORMAL": normal, "TEXCOORD_0": texcoord},
                        "indices": indices,
                        "material": component.material,
                        "mode": 4,
                    }
                ],
            }
        )
        node: dict[str, object] = {"name": component.name, "mesh": mesh_index}
        if component.extras:
            node["extras"] = {"salimon": component.extras}
        node_index = len(nodes)
        nodes.append(node)
        nodes[groups[component.group]]["children"].append(node_index)  # type: ignore[index,union-attr]

    baseline_collision_specs = [
        ("COLLIDER_InteriorFloor", [0.0, 0.18, 0.0], [7.06, 0.10, 3.10], "walkable-floor"),
        ("COLLIDER_InteriorPortWall", [-0.55, 1.45, 1.61], [6.07, 2.34, 0.14], "interior-wall"),
        ("COLLIDER_InteriorStarboardWall", [-0.55, 1.45, -1.61], [6.07, 2.34, 0.14], "interior-wall"),
        ("COLLIDER_InteriorCeiling", [-0.58, 2.61, 0.0], [6.08, 0.10, 3.08], "ceiling"),
        ("COLLIDER_AftDoor", [-3.84, 1.31, 0.0], [0.14, 2.06, 1.40], "interactive-door"),
        ("COLLIDER_ExteriorHull", [0.25, 1.48, 0.0], [10.50, 3.16, 4.10], "broad-phase-hull"),
    ]
    for name, center, size, purpose in baseline_collision_specs:
        node_index = len(nodes)
        nodes.append(
            {
                "name": name,
                "translation": scaled_vector(center),
                "extras": {
                    "salimon": {
                        "collisionShape": "box",
                        "sizeMeters": scaled_vector(size),
                        "purpose": purpose,
                    }
                },
            }
        )
        nodes[groups["Collision"]]["children"].append(node_index)  # type: ignore[index,union-attr]

    markers = [
        ("MARKER_CockpitSeat", COCKPIT_SEAT_MARKER_METERS, "+X", "cockpit-seat"),
        ("MARKER_ExitDoor", scaled_vector([-3.55, 1.25, 0.0]), "-X", "exit-door"),
        ("MARKER_PlayerStart", PLAYER_START_METERS, "+X", "player-start"),
    ]
    for name, position, facing, purpose in markers:
        node_index = len(nodes)
        nodes.append(
            {
                "name": name,
                "translation": position,
                "extras": {"salimon": {"purpose": purpose, "facing": facing}},
            }
        )
        nodes[groups["Markers"]]["children"].append(node_index)  # type: ignore[index,union-attr]

    document: dict[str, object] = {
        "asset": {
            "version": "2.0",
            "generator": "Salimon deterministic Phase 0 ship pipeline",
            "copyright": "Copyright 2026 Salimon contributors; custom original asset",
            "extras": {
                "salimon": {
                    "assetVersion": 3,
                    "units": "meters",
                    "upAxis": "+Y",
                    "forwardAxis": "+X",
                    "starboardAxis": "-Z",
                    "sourceWorkflow": "Python procedural DCC source; Blender-importable glTF",
                }
            },
        },
        "scene": 0,
        "scenes": [{"name": "Salimon Phase 0 Ship", "nodes": [root_index]}],
        "nodes": nodes,
        "meshes": meshes,
        "materials": make_materials(),
        "samplers": [{"name": "Grip Repeat", "magFilter": 9729, "minFilter": 9729, "wrapS": 10497, "wrapT": 10497}],
        "textures": [{"name": "Custom Floor Grip", "sampler": 0, "source": 0}],
        "images": [{"name": "Custom Floor Grip", "uri": "../textures/salimon_floor_grip.png"}],
        "buffers": [{"byteLength": len(builder.data), "uri": f"{MODEL_NAME}.bin"}],
        "bufferViews": builder.buffer_views,
        "accessors": builder.accessors,
        "extras": {
            "salimon": {
                "drawPrimitiveCount": len(meshes),
                "triangleCount": sum(len(component.geometry.indices) // 3 for component in components),
                "materialCount": len(MATERIALS),
                "externalAssetDependencies": 0,
                "linearScaleFromTask7Baseline": LINEAR_SCALE_FROM_TASK7,
                "task7BaselineDimensionsMeters": TASK7_BASELINE_DIMENSIONS_METERS,
                "overallDimensionsMeters": [
                    dimension * LINEAR_SCALE_FROM_TASK7
                    for dimension in TASK7_BASELINE_DIMENSIONS_METERS
                ],
                "lowestLocalYMeters": -0.10 * LINEAR_SCALE_FROM_TASK7,
                "walkableInteriorClearanceMeters": {
                    "width": 3.10 * LINEAR_SCALE_FROM_TASK7,
                    "height": 2.33 * LINEAR_SCALE_FROM_TASK7,
                },
                "humanEyeHeightMeters": PLAYER_EYE_HEIGHT_METERS,
                "cockpitWindows": {
                    "glazingNode": "Cockpit_Glazing",
                    "frameNode": "Cockpit_Window_Frame",
                    "material": "Cockpit Glass",
                    "seatedViewpointMeters": COCKPIT_VIEWPOINT_METERS,
                    "standingViewpointMeters": STANDING_VIEWPOINT_METERS,
                    "dynamicShadows": False,
                    "postEffects": False,
                },
            }
        },
    }
    return document, bytes(builder.data)


def glb_bytes(document: dict[str, object], geometry: bytes, texture: bytes) -> bytes:
    glb_document = copy.deepcopy(document)
    binary = bytearray(geometry)
    binary.extend(b"\0" * ((-len(binary)) % 4))
    image_offset = len(binary)
    binary.extend(texture)
    image_view = {"buffer": 0, "byteOffset": image_offset, "byteLength": len(texture)}
    glb_document["bufferViews"].append(image_view)  # type: ignore[union-attr]
    glb_document["buffers"] = [{"byteLength": len(binary)}]
    glb_document["images"] = [{"name": "Custom Floor Grip", "bufferView": len(glb_document["bufferViews"]) - 1, "mimeType": "image/png"}]  # type: ignore[arg-type]

    encoded_json = json.dumps(glb_document, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    encoded_json += b" " * ((-len(encoded_json)) % 4)
    binary.extend(b"\0" * ((-len(binary)) % 4))
    total_length = 12 + 8 + len(encoded_json) + 8 + len(binary)
    return b"".join(
        [
            struct.pack("<4sII", b"glTF", 2, total_length),
            struct.pack("<I4s", len(encoded_json), b"JSON"),
            encoded_json,
            struct.pack("<I4s", len(binary), b"BIN\0"),
            bytes(binary),
        ]
    )


def write_outputs() -> None:
    EXPORT_DIR.mkdir(parents=True, exist_ok=True)
    TEXTURE_DIR.mkdir(parents=True, exist_ok=True)
    texture = make_grip_texture()
    document, geometry = build_document(texture)
    (EXPORT_DIR / f"{MODEL_NAME}.gltf").write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
    (EXPORT_DIR / f"{MODEL_NAME}.bin").write_bytes(geometry)
    (TEXTURE_DIR / "salimon_floor_grip.png").write_bytes(texture)
    (EXPORT_DIR / f"{MODEL_NAME}.glb").write_bytes(glb_bytes(document, geometry, texture))
    print(
        f"Generated {MODEL_NAME}: {len(document['meshes'])} primitives, "
        f"{document['extras']['salimon']['triangleCount']} triangles, "  # type: ignore[index]
        f"{len(geometry)} geometry bytes"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.parse_args()
    write_outputs()


if __name__ == "__main__":
    main()
