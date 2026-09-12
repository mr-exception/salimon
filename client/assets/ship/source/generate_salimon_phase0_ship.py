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
HORIZONTAL_SCALE_FROM_TASK7 = 2.0
TARGET_HEIGHT_METERS = 4.0
VERTICAL_SCALE_FROM_TASK7 = TARGET_HEIGHT_METERS / TASK7_BASELINE_DIMENSIONS_METERS[1]
PLAYER_BODY_HEIGHT_METERS = 1.80
PLAYER_EYE_HEIGHT_METERS = 1.75
SCALED_FLOOR_HEIGHT_METERS = 0.23 * VERTICAL_SCALE_FROM_TASK7
COCKPIT_SEAT_MARKER_METERS = [
    1.38 * HORIZONTAL_SCALE_FROM_TASK7,
    1.05 * VERTICAL_SCALE_FROM_TASK7,
    0.0,
]
COCKPIT_VIEWPOINT_METERS = [
    COCKPIT_SEAT_MARKER_METERS[0],
    COCKPIT_SEAT_MARKER_METERS[1] + 0.67,
    0.0,
]
STANDING_VIEWPOINT_METERS = [
    0.65 * HORIZONTAL_SCALE_FROM_TASK7,
    SCALED_FLOOR_HEIGHT_METERS + PLAYER_EYE_HEIGHT_METERS,
    0.0,
]
PLAYER_START_METERS = [
    0.50,
    SCALED_FLOOR_HEIGHT_METERS + PLAYER_EYE_HEIGHT_METERS,
    -2.20,
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
    return [
        values[0] * HORIZONTAL_SCALE_FROM_TASK7,
        values[1] * VERTICAL_SCALE_FROM_TASK7,
        values[2] * HORIZONTAL_SCALE_FROM_TASK7,
    ]


def scaled_geometry(geometry: Geometry) -> Geometry:
    return Geometry(
        positions=[tuple(scaled_vector(position)) for position in geometry.positions],
        normals=[
            normalize(
                (
                    normal[0] / HORIZONTAL_SCALE_FROM_TASK7,
                    normal[1] / VERTICAL_SCALE_FROM_TASK7,
                    normal[2] / HORIZONTAL_SCALE_FROM_TASK7,
                )
            )
            for normal in geometry.normals
        ],
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


def extrude_y(footprint: Sequence[tuple[float, float]], y0: float, y1: float) -> Geometry:
    """Extrude a clockwise X/Z footprint between two Y planes."""
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
    ("Hull Graphite", [0.065, 0.115, 0.14, 1.0], 0.68, 0.42, None, None),
    ("Hull Ceramic", [0.72, 0.75, 0.69, 1.0], 0.52, 0.20, None, None),
    ("Copper Accent", [0.66, 0.30, 0.13, 1.0], 0.42, 0.62, None, None),
    ("Cockpit Glass", [0.10, 0.28, 0.30, 0.10], 0.18, 0.12, None, None),
    ("Interior Light", [0.82, 0.71, 0.53, 1.0], 0.70, 0.08, None, None),
    ("Interior Dark", [0.095, 0.13, 0.14, 1.0], 0.78, 0.12, None, None),
    ("Cyan Display", [0.015, 0.22, 0.27, 1.0], 0.28, 0.18, [0.0, 0.75, 0.92], None),
    ("Floor Grip", [0.42, 0.45, 0.45, 1.0], 0.86, 0.04, None, 0),
    ("Engine Glow", [0.12, 0.24, 0.30, 1.0], 0.30, 0.28, [0.08, 0.52, 0.78], None),
    ("Warm Lamp", [0.75, 0.42, 0.12, 1.0], 0.60, 0.05, [1.0, 0.56, 0.20], None),
    ("Terracotta Upholstery", [0.49, 0.19, 0.115, 1.0], 0.92, 0.0, None, None),
    ("Petrol Teal", [0.07, 0.29, 0.29, 1.0], 0.72, 0.15, None, None),
    ("Honey Wood", [0.42, 0.25, 0.12, 1.0], 0.88, 0.0, None, None),
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


def combined(*parts: Geometry) -> Geometry:
    result = Geometry()
    for part in parts:
        result.extend(part)
    return result


def ring_x(x0: float, x1: float, cy: float, cz: float,
           outer0: float, outer1: float, inner0: float, inner1: float,
           segments: int = 12) -> Geometry:
    """Hollow faceted nozzle: real inner wall and annular lips, no solid cap."""
    result = Geometry()
    def point(x: float, radius: float, i: int) -> tuple[float, float, float]:
        angle = 2 * math.pi * i / segments
        return (x, cy + math.cos(angle) * radius, cz + math.sin(angle) * radius)
    for i in range(segments):
        j = (i + 1) % segments
        a, b = point(x0, outer0, i), point(x0, outer0, j)
        c, d = point(x1, outer1, j), point(x1, outer1, i)
        e, f = point(x0, inner0, i), point(x0, inner0, j)
        g, h = point(x1, inner1, j), point(x1, inner1, i)
        for face in ([a,b,c,d], [h,g,f,e], [e,f,b,a], [d,c,g,h]):
            result.add_face(face)
    return result


def cylinder_y(cx: float, cz: float, y0: float, y1: float,
               radius0: float, radius1: float, segments: int = 12) -> Geometry:
    result = Geometry()
    lower = [(cx + math.cos(i*2*math.pi/segments)*radius0, y0,
              cz + math.sin(i*2*math.pi/segments)*radius0) for i in range(segments)]
    upper = [(cx + math.cos(i*2*math.pi/segments)*radius1, y1,
              cz + math.sin(i*2*math.pi/segments)*radius1) for i in range(segments)]
    result.add_face(lower)
    result.add_face(list(reversed(upper)))
    for i in range(segments):
        j = (i+1) % segments
        result.add_face([lower[i], upper[i], upper[j], lower[j]])
    return result


def core_radial(geometry: Geometry, angle: float) -> Geometry:
    """Place local radial/Y/tangent geometry around the Core's vertical axis."""
    cosine, sine = math.cos(angle), math.sin(angle)
    def rotate(point: tuple[float, float, float]) -> tuple[float, float, float]:
        x, y, z = point
        return (cosine*x-sine*z, y, sine*x+cosine*z)
    return Geometry(
        positions=[(x-.50,y,z) for x,y,z in map(rotate, geometry.positions)],
        normals=list(map(rotate, geometry.normals)),
        texcoords=geometry.texcoords.copy(), indices=geometry.indices.copy(),
    )


def core_ring(y0: float, y1: float, outer: float, inner: float,
              start: float = 0.0, sweep: float = 2*math.pi,
              segments: int = 8) -> Geometry:
    """Closed hollow ring/arc with outward winding and an open central bore."""
    result = Geometry()
    def point(radius: float, y: float, index: int) -> tuple[float,float,float]:
        angle = start+sweep*index/segments
        return (-.50+radius*math.cos(angle), y, radius*math.sin(angle))
    for i in range(segments):
        a,b = point(outer,y0,i),point(outer,y0,i+1)
        c,d = point(outer,y1,i+1),point(outer,y1,i)
        e,f = point(inner,y0,i),point(inner,y0,i+1)
        g,h = point(inner,y1,i+1),point(inner,y1,i)
        for face in ([a,d,c,b],[e,f,g,h],[e,a,b,f],[d,h,g,c]):
            result.add_face(face)
    if sweep < 2*math.pi-1e-6:
        result.add_face([point(inner,y0,0),point(inner,y1,0),point(outer,y1,0),point(outer,y0,0)])
        result.add_face([point(outer,y0,segments),point(outer,y1,segments),point(inner,y1,segments),point(inner,y0,segments)])
    return result


def ship_components() -> list[Component]:
    components: list[Component] = []

    def add(name: str, geometry: Geometry, material: int, group: str, **extras: object) -> None:
        components.append(Component(name, scaled_geometry(geometry), material, group, extras))

    # Wide lifting-body scout; horizontal dimensions authored at half scale.
    # Wall assemblies leave actual openings from waist height to the ceiling.
    add("Hull_Belly", tapered_box(-4.2, 3.3, -0.10, 0.18, 2.55, -0.10, 0.18, 2.15), 0, "Exterior")
    roof_outline = [(-3.90,-2.18),(-3.55,-2.62),(2.15,-2.62),(2.58,-2.30),
                    (2.58,2.30),(2.15,2.62),(-3.55,2.62),(-3.90,2.18)]
    add("Hull_Roof", extrude_y(list(reversed(roof_outline)),2.72,3.08), 1, "Exterior")
    # Beveled shoulder rails break up the broad roof without cutting cabin space.
    shoulders = Geometry()
    for sign in (-1,1):
        shoulders.extend(tapered_box(-3.45,2.14,2.94,3.14,.11,2.96,3.08,.07))
        start = len(shoulders.positions) - 24
        for i in range(start,len(shoulders.positions)):
            x,y,z=shoulders.positions[i]
            shoulders.positions[i]=(x,y,z+sign*2.46)
    add("Roof_Shoulder_Rails",shoulders,0,"Exterior")
    for label, sign in (("Port", 1), ("Starboard", -1)):
        z0, z1 = sorted((sign*2.30, sign*2.55))
        frames = combined(box(-3.8, 2.6, 0.18, 0.87, z0, z1),
                          box(-3.8, 2.6, 2.52, 2.73, z0, z1))
        for x in (-3.72, -1.70, 0.40, 2.44):
            frames.extend(box(x-.06, x+.06, .87, 2.52, z0, z1))
        add(f"Hull_{label}_Side", frames, 0, "Exterior")
        glass = Geometry()
        for x0,x1 in ((-3.66,-1.76),(-1.64,.34),(.46,2.38)):
            z=sign*2.42
            glass.add_face([(x0,.87,z),(x1,.87,z),(x1,2.52,z),(x0,2.52,z)])
        add(f"Cabin_{label}_Glazing", glass, 3, "Exterior", exterior_visibility=True)
        add(f"Hull_{label}_Stripe", box(-3.60,2.30,.68,.82,
            *sorted((sign*2.555,sign*2.58))), 2, "Exterior")
    add("Hull_Nose", tapered_box(2.5, 5.25, 0.10, .89, 2.55, .48, .66, .62), 1, "Exterior")
    # Rear observation windows flank a real door opening; no solid aft cap.
    aft = combined(box(-3.96,-3.78,.18,.88,-2.55,-.72),
                   box(-3.96,-3.78,.18,.88,.72,2.55),
                   box(-3.96,-3.78,2.52,2.73,-2.55,2.55))
    for z in (-2.48,-.76,.76,2.48):
        aft.extend(box(-3.96,-3.78,.88,2.52,z-.06,z+.06))
    add("Hull_Aft_Cap", aft, 0, "Exterior")
    aft_glass=Geometry()
    for z0,z1 in ((-2.42,-.82),(.82,2.42)):
        aft_glass.add_face([(-3.86,.88,z0),(-3.86,2.52,z0),(-3.86,2.52,z1),(-3.86,.88,z1)])
    add("Cabin_Aft_Glazing", aft_glass, 3, "Exterior", exterior_visibility=True)
    for label, sign in (("Port",1),("Starboard",-1)):
        def wing_poly(points):
            values=[(x,z*sign) for x,z in points]
            return values if sign==1 else list(reversed(values))
        add(f"Wing_{label}",extrude_y(wing_poly([(2.45,2.30),(-2.90,2.30),(-4.05,5.0),(.70,4.05)]),.18,.42),0,"Exterior")
        add(f"Wing_{label}_Armor",extrude_y(wing_poly([(1.70,2.66),(-2.76,2.66),(-3.55,4.70),(.48,3.82)]),.43,.52),1,"Exterior")
        add(f"Wing_{label}_Accent",extrude_y(wing_poly([(.55,3.92),(-3.64,4.74),(-3.42,4.44),(.63,3.69)]),.53,.58),2,"Exterior")
        cy,cz=1.00,sign*3.55
        add(f"Engine_{label}",ring_x(-4.47,-1.72,cy,cz,.62,.62,.36,.36),0,"Exterior")
        add(f"Engine_{label}_Intake_Cowl",ring_x(-2.0,-1.50,cy,cz,.66,.43,.36,.30),1,"Exterior")
        add(f"Engine_{label}_Intake",cylinder_x(-1.68,-1.65,cy,cz,.29),5,"Exterior")
        # Swept stabilizers provide an aft silhouette beyond the flat wing deck.
        fin = Geometry()
        profile=[(-3.82,.58),(-3.48,2.65),(-2.96,2.35),(-2.15,.58)]
        lower=[(x,y,cz-.055) for x,y in profile]
        upper=[(x,y,cz+.055) for x,y in profile]
        fin.add_face(lower)
        fin.add_face(list(reversed(upper)))
        for i in range(len(profile)):
            j=(i+1)%len(profile)
            fin.add_face([lower[i],upper[i],upper[j],lower[j]])
        add(f"Engine_{label}_Swept_Fin",fin,0,"Exterior")
        # Flared hollow nozzles, stepped heat shields, recessed ion emitters.
        add(f"Engine_{label}_Nozzle",ring_x(-4.90,-4.14,cy,cz,.70,.54,.55,.40),2,"Exterior")
        add(f"Engine_{label}_Liner",ring_x(-4.86,-4.18,cy,cz,.54,.39,.48,.31),5,"Exterior")
        add(f"Engine_{label}_Glow",cylinder_x(-4.61,-4.58,cy,cz,.43),8,"Exterior")
        rings=combined(ring_x(-4.74,-4.67,cy,cz,.72,.72,.68,.68),
                       ring_x(-4.10,-3.99,cy,cz,.66,.66,.60,.60),
                       ring_x(-2.13,-2.02,cy,cz,.65,.65,.60,.60))
        add(f"Engine_{label}_Bands",rings,1,"Exterior")
        vanes=Geometry()
        for i in range(8):
            angle=2*math.pi*i/8
            y,z=cy+math.cos(angle)*.61,cz+math.sin(angle)*.61
            vanes.extend(box(-3.85,-2.30,y-.055,y+.055,z-.045,z+.045))
        add(f"Engine_{label}_Cooling_Fins",vanes,11,"Exterior")
        add(f"Engine_{label}_Running_Light",box(-3.6,-2.6,1.63,1.67,cz-.045,cz+.045),8,"Exterior")
    add("Dorsal_Spine",tapered_box(-3.40,2.30,3.07,3.62,.36,3.08,3.18,.12),0,"Exterior")
    roof_panels=Geometry()
    for x in (-3.2,-1.8,-.4,1.0):
        for z in (-1.55,1.55):
            roof_panels.extend(box(x,x+1.1,3.081,3.115,z-.55,z+.55))
    add("Roof_Service_Panels",roof_panels,11,"Exterior")
    add("Cockpit_Glazing",cockpit_glazing(2.50,4.92,.89,2.72,2.42,.66,2.14,.62),3,"Exterior",exterior_visibility=True)
    canopy_frame=Geometry()
    # Sloping side edges, no center mullion across the pilot's forward view.
    for sign in (-1,1):
        canopy_frame.extend(box(2.44,2.56,.89,2.74,*sorted((sign*2.36,sign*2.48))))
        canopy_frame.extend(box(4.86,4.98,.66,2.18,*sorted((sign*.56,sign*.68))))
    canopy_frame.extend(box(4.85,4.99,2.12,2.22,-.68,.68))
    add("Cockpit_Window_Frame",canopy_frame,2,"Exterior")

    # Warm living cabin: structural lower walls, deep sills and generous glazing.
    add("Deck_Walkable",box(-3.78,3.28,.13,.23,-2.30,2.30),7,"Interior",walkable=True)
    add("Ceiling_Inner",box(-3.72,2.46,2.61,2.71,-2.30,2.30),4,"Interior")
    for label,sign in (("Port",1),("Starboard",-1)):
        add(f"Wall_{label}_Inner",combined(
            box(-3.72,2.45,.23,.86,*sorted((sign*2.23,sign*2.30))),
            box(-3.72,2.45,2.53,2.62,*sorted((sign*2.23,sign*2.30)))),4,"Interior")
        add(f"Window_{label}_Sill",box(-3.62,2.38,.84,.92,*sorted((sign*2.10,sign*2.30))),12,"Interior")
    add("Aft_Bulkhead_Port",box(-3.78,-3.70,.23,.87,.80,2.30),4,"Interior")
    add("Aft_Bulkhead_Starboard",box(-3.78,-3.70,.23,.87,-2.30,-.80),4,"Interior")
    add("Aft_Bulkhead_Header",box(-3.78,-3.62,2.34,2.62,-.80,.80),5,"Interior")
    # Combine door surface accents into its geometry so the complete door moves.
    add("Exit_Door",box(-3.91,-3.77,.23,2.34,-.70,.70),11,"Interior",interactive="exit-door",pivot=scaled_vector([-3.84,.23,.70]))
    door_frame=combined(box(-3.77,-3.60,.23,2.37,-.80,-.70),box(-3.77,-3.60,.23,2.37,.70,.80),box(-3.77,-3.60,2.34,2.42,-.80,.80))
    add("Door_Frame",door_frame,2,"Interior")
    add("Door_Threshold",box(-3.94,-3.55,.17,.23,-.78,.78),2,"Interior")
    add("Cockpit_Console_Center",tapered_box(2.16,3.12,.23,.87,.74,.32,1.0,.52),5,"Interior")
    add("Cockpit_Console_Port",box(1.78,3.02,.23,.74,.82,1.48),11,"Interior")
    add("Cockpit_Console_Starboard",box(1.78,3.02,.23,.74,-1.48,-.82),11,"Interior")
    add("Monitor_Center",box(2.155,2.18,.68,.98,-.48,.48),6,"Interior",interactive="cockpit-monitor")
    add("Monitor_Port",box(2.08,2.13,.74,.88,.91,1.38),6,"Interior")
    add("Monitor_Starboard",box(2.08,2.13,.74,.88,-1.38,-.91),6,"Interior")
    add("Pilot_Seat_Base",combined(box(.94,1.62,.23,.51,-.36,.36),box(.90,1.62,.51,.65,-.40,.40)),5,"Interior")
    add("Pilot_Seat_Back",box(.78,1.00,.51,1.40,-.40,.40),10,"Interior",interactive="cockpit-seat")
    add("Pilot_Seat_Accent",combined(box(1.0,1.06,.66,1.31,-.32,.32),box(1.05,1.55,.65,.71,-.36,.36)),12,"Interior")
    # Furnishings are kept against the wall, leaving two broad circulation lanes.
    add("Cabin_Bench_Port",box(-2.85,-1.12,.23,.47,1.83,2.22),5,"Interior")
    cushions=Geometry()
    for x in (-2.82,-2.26,-1.70):
        cushions.extend(box(x,x+.51,.47,.67,1.80,2.20))
        cushions.extend(box(x,x+.51,.67,1.03,2.10,2.22))
    add("Cabin_Bench_Cushions",cushions,10,"Interior")
    add("Cabin_Storage_Starboard",box(-2.88,-1.6,.23,.79,-2.22,-1.88),11,"Interior")
    add("Cabin_Worktop",box(-2.98,-1.50,.79,.86,-2.24,-1.79),12,"Interior")
    drawers=Geometry()
    for x in (-2.76,-2.18):
        for y in (.37,.61):
            drawers.extend(box(x,x+.42,y,y+.12,-1.887,-1.872))
    add("Cabin_Drawer_Fronts",drawers,4,"Interior")
    trim=Geometry()
    for sign in (-1,1):
        trim.extend(box(-3.55,1.60,.232,.242,*sorted((sign*1.57,sign*1.61))))
    add("Deck_Copper_Inlay",trim,2,"Interior")
    runner=Geometry()
    for sign in (-1,1):
        runner.extend(box(-2.80,.63,.232,.237,*sorted((sign*.78,sign*1.40))))
    add("Cabin_Woven_Runners",runner,10,"Interior")

    # Recoverable energy-storage heart: segmented cells in an armored service
    # cage. All solid details fit the existing collision envelope and reuse
    # the ship palette; luminous inserts need no new lights or render passes.
    add("Core_Pedestal",combined(cylinder_y(-.50,0,.23,.32,.50,.50,8),
                                cylinder_y(-.50,0,.32,.43,.46,.38,8)),0,"Interior",purpose="energy-core-housing")
    add("Core_Cradle",combined(cylinder_y(-.50,0,.43,.53,.38,.32,8),
                              cylinder_y(-.50,0,1.76,1.84,.32,.40,8),
                              core_ring(1.84,1.88,.40,.19)),2,"Interior")
    add("Energy_Core",combined(cylinder_y(-.50,0,.58,.90,.18,.23,8),
                               cylinder_y(-.50,0,.94,1.29,.23,.23,8),
                               cylinder_y(-.50,0,1.33,1.69,.23,.15,8)),6,"Interior",purpose="energy-storage-core",phase0="visual-only")
    add("Core_Cell_Separators",combined(cylinder_y(-.50,0,.89,.95,.245,.245,8),
                                       cylinder_y(-.50,0,1.28,1.34,.245,.245,8)),0,"Interior")
    # A stepped crown and segmented ceramic armor distinguish the removable
    # energy cartridge from its heavier installed plinth.
    armor, ribs, insets, contacts, vents = (Geometry() for _ in range(5))
    for i in range(4):
        angle=math.pi/4+i*math.pi/2
        spine=combined(box(.30,.35,.51,1.78,-.04,.04),
                       box(.30,.43,.47,.66,-.06,.06),
                       box(.30,.43,1.62,1.82,-.06,.06))
        ribs.extend(core_radial(spine,angle))
        insets.extend(core_radial(box(.351,.359,.72,1.57,-.018,.018),angle))
        contacts.extend(core_radial(box(.431,.437,.54,.60,-.025,.025),angle))
        contacts.extend(core_radial(box(.431,.437,1.68,1.73,-.025,.025),angle))
    for i in range(8):
        angle=i*math.pi/4
        armor.extend(core_ring(.34,.415,.445,.39,angle+.06,math.pi/4-.12,1))
        armor.extend(core_ring(1.80,1.86,.415,.355,angle+.07,math.pi/4-.14,1))
        # Small inset slots and locking tabs provide readable surface detail.
        for y in (.357,.381):
            vents.extend(core_radial(box(.426,.432,y,y+.009,-.055,.055),angle+math.pi/8))
    add("Core_Containment_Ribs",ribs,0,"Interior")
    add("Core_Ceramic_Armor",armor,1,"Interior")
    add("Core_Spine_Inlays",insets,8,"Interior")
    add("Core_Lock_Indicators",contacts,9,"Interior")
    add("Core_Service_Vents",vents,5,"Interior")
    add("Core_Containment_Rings",combined(core_ring(.65,.70,.325,.27),
                                         core_ring(1.10,1.15,.39,.30),
                                         core_ring(1.56,1.61,.325,.27)),2,"Interior")
    add("Core_Status_Lights",combined(core_ring(.315,.338,.47,.425),
                                     cylinder_y(-.50,0,1.862,1.895,.15,.11,8)),8,"Interior")
    # Recessed forward service panel with a segmented charge gauge.
    add("Core_Service_Panel",core_radial(box(.355,.40,.91,1.30,-.105,.105),0),5,"Interior")
    gauge=Geometry()
    for y in (.96,1.02,1.08,1.14):
        gauge.extend(core_radial(box(.401,.405,y,y+.027,-.07,.045),0))
    add("Core_Charge_Gauge",gauge,6,"Interior")
    add("Core_Service_Markings",core_radial(combined(box(.401,.405,1.23,1.25,-.07,.07),
                                                   box(.401,.405,.955,1.185,.065,.078)),0),1,"Interior")
    add("Core_Power_Conduits",combined(box(-3.30,-1.0,.235,.26,-.055,.055),
                                     box(0,.50,.235,.26,-.055,.055)),2,"Interior")
    ceiling_lamps=Geometry()
    lamp_housings=Geometry()
    for sign in (-1,1):
        z=sign*1.73
        for x0,x1 in ((-3.3,-1.05),(-.30,2.0)):
            lamp_housings.extend(box(x0-.06,x1+.06,2.51,2.61,z-.12,z+.12))
            ceiling_lamps.extend(box(x0,x1,2.505,2.52,z-.055,z+.055))
    ceiling_ribs=Geometry()
    for x in (-3.42,-1.70,.40,2.20):
        ceiling_ribs.extend(box(x-.035,x+.035,2.55,2.61,-2.18,2.18))
    add("Ceiling_Copper_Ribs",ceiling_ribs,2,"Interior")
    ceiling_insets=Geometry()
    for x0,x1 in ((-3.27,-1.85),(-1.55,.25),(.55,2.05)):
        ceiling_insets.extend(box(x0,x1,2.598,2.609,-.57,.57))
    add("Ceiling_Teal_Inset_Panels",ceiling_insets,11,"Interior")
    add("Ceiling_Lamp_Housings",lamp_housings,2,"Interior")
    add("Ceiling_Warm_Lights",ceiling_lamps,9,"Interior")
    guidance=Geometry()
    for sign in (-1,1):
        guidance.extend(box(-3.55,1.7,.30,.34,*sorted((sign*2.21,sign*2.23))))
    add("Cabin_Low_Guidance_Lights",guidance,9,"Interior")
    add("Door_Welcome_Light",box(-3.60,-3.56,2.37,2.42,-.50,.50),9,"Interior")
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
        ("COLLIDER_InteriorFloor", [-0.25, 0.18, 0.0], [7.06, 0.10, 4.60], "walkable-floor"),
        ("COLLIDER_InteriorPortWall", [-0.55, 1.45, 2.37], [6.34, 2.34, 0.14], "interior-wall"),
        ("COLLIDER_InteriorStarboardWall", [-0.55, 1.45, -2.37], [6.34, 2.34, 0.14], "interior-wall"),
        ("COLLIDER_InteriorCeiling", [-0.63, 2.66, 0.0], [6.18, 0.10, 4.60], "ceiling"),
        ("COLLIDER_CabinBench", [-1.985, .63, 2.01], [1.73, .80, .42], "interior-obstacle"),
        ("COLLIDER_CabinWorktop", [-2.24, .545, -2.015], [1.48, .63, .45], "interior-obstacle"),
        ("COLLIDER_EnergyCore", [-0.50, 1.07, 0.0], [1.0, 1.68, 1.0], "interior-obstacle"),
        ("COLLIDER_AftDoor", [-3.84, 1.285, 0.0], [0.14, 2.11, 1.40], "interactive-door"),
        ("COLLIDER_ExteriorHull", [0.175, 1.76, 0.0], [10.15, 3.72, 10.00], "broad-phase-hull"),
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
                    "assetVersion": 6,
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
                "scaleFromTask7Baseline": [
                    HORIZONTAL_SCALE_FROM_TASK7,
                    VERTICAL_SCALE_FROM_TASK7,
                    HORIZONTAL_SCALE_FROM_TASK7,
                ],
                "task7BaselineDimensionsMeters": TASK7_BASELINE_DIMENSIONS_METERS,
                "overallDimensionsMeters": [
                    TASK7_BASELINE_DIMENSIONS_METERS[0] * HORIZONTAL_SCALE_FROM_TASK7,
                    TARGET_HEIGHT_METERS,
                    20.0,
                ],
                "lowestLocalYMeters": -0.10 * VERTICAL_SCALE_FROM_TASK7,
                "walkableInteriorClearanceMeters": {
                    "width": 4.60 * HORIZONTAL_SCALE_FROM_TASK7,
                    "height": 2.38 * VERTICAL_SCALE_FROM_TASK7,
                },
                "designRevision": "wide-cozy-cabin-sci-fi-core",
                "energyCore": {"centerMeters": [-1.0, 1.2, 0.0], "role": "energy-storage", "phase0": "visual-only"},
                "humanBodyHeightMeters": PLAYER_BODY_HEIGHT_METERS,
                "humanEyeHeightMeters": PLAYER_EYE_HEIGHT_METERS,
                "cockpitWindows": {
                    "glazingNode": "Cockpit_Glazing",
                    "frameNode": "Cockpit_Window_Frame",
                    "cabinGlazingNodes": ["Cabin_Port_Glazing", "Cabin_Starboard_Glazing", "Cabin_Aft_Glazing"],
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
