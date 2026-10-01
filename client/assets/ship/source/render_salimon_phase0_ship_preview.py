#!/usr/bin/env python3
"""Render fitted exterior/cutaway and eye-height ship previews using Pillow.

This optional software geometry QA helper uses the procedural source directly.
It approximates the native material shader, but is not a native-game capture.
No browser, GPU renderer, NumPy, or additional image assets are required.
"""

from __future__ import annotations

import argparse
import math
from dataclasses import dataclass
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

from generate_salimon_phase0_ship import (
    MATERIALS,
    STANDING_VIEWPOINT_METERS,
    Component,
    cross,
    normalize,
    ship_components,
    vector_sub,
)

Point = tuple[float, float, float]
Color = tuple[int, int, int, int]
BACKGROUND = (13, 20, 29)


def dot(a: Point, b: Point) -> float:
    return sum(x * y for x, y in zip(a, b))


def srgb(value: float) -> int:
    value = min(1.0, max(0.0, value))
    return round(255 * (12.92 * value if value <= 0.0031308 else 1.055 * value ** (1 / 2.4) - 0.055))


def face_color(component: Component, normal: Point) -> Color:
    """Mirror the native unshadowed face light and independent emission."""
    _, base, _, _, emission, _ = MATERIALS[component.material]
    if component.group == "Interior":
        light = tuple(channel + 0.16 * abs(normal[1]) for channel in (0.76, 0.69, 0.59))
    else:
        amount = 0.32 + 0.68 * abs(dot(normal, normalize((0.35, 0.82, 0.44))))
        light = (amount,) * 3
    emission = emission or (0.0, 0.0, 0.0)
    return tuple(srgb(base[i] * light[i] + emission[i]) for i in range(3)) + (round(base[3] * 255),)


@dataclass(frozen=True)
class Camera:
    eye: Point
    target: Point
    perspective: bool = False
    vertical_fov_degrees: float = 66.0

    def basis(self) -> tuple[Point, Point, Point]:
        forward = normalize(vector_sub(self.target, self.eye))
        right = normalize(cross(forward, (0.0, 1.0, 0.0)))
        return right, cross(right, forward), forward


def clip_near(points: list[Point], near: float = 0.06) -> list[Point]:
    """Clip before perspective division, including floors crossing the eye."""
    result = []
    for previous, current in zip(points[-1:] + points[:-1], points):
        previous_inside, current_inside = previous[2] >= near, current[2] >= near
        if previous_inside != current_inside:
            fraction = (near - previous[2]) / (current[2] - previous[2])
            result.append(tuple(previous[i] + fraction * (current[i] - previous[i]) for i in range(3)))
        if current_inside:
            result.append(current)
    return result


def rasterize(
    image: Image.Image,
    depth_buffer: list[float],
    points: list[Point],
    color: Color,
    perspective: bool,
) -> None:
    """Scan a triangle with a depth buffer; transparent glass never writes depth."""
    width, height = image.size
    (ax, ay, az), (bx, by, bz), (cx, cy, cz) = points
    denominator = (by - cy) * (ax - cx) + (cx - bx) * (ay - cy)
    if abs(denominator) < 1e-8:
        return
    minimum_y = max(0, math.ceil(min(ay, by, cy) - 0.5))
    maximum_y = min(height - 1, math.floor(max(ay, by, cy) - 0.5))
    if minimum_y > maximum_y:
        return
    weight_a_dx = (by - cy) / denominator
    weight_b_dx = (cy - ay) / denominator
    depths = [1 / z if perspective else -z for z in (az, bz, cz)]
    depth_dx = weight_a_dx * (depths[0] - depths[2]) + weight_b_dx * (depths[1] - depths[2])
    edges = ((points[0], points[1]), (points[1], points[2]), (points[2], points[0]))
    pixels = image.load()
    alpha = color[3] / 255
    opaque = color[3] == 255
    for y in range(minimum_y, maximum_y + 1):
        scan_y = y + 0.5
        crossings = []
        for first, second in edges:
            if min(first[1], second[1]) <= scan_y < max(first[1], second[1]):
                crossings.append(first[0] + (scan_y - first[1]) * (second[0] - first[0]) / (second[1] - first[1]))
        if len(crossings) < 2:
            continue
        minimum_x = max(0, math.ceil(min(crossings) - 0.5))
        maximum_x = min(width - 1, math.floor(max(crossings) - 0.5))
        if minimum_x > maximum_x:
            continue
        sample_x = minimum_x + 0.5
        weight_a = ((by - cy) * (sample_x - cx) + (cx - bx) * (scan_y - cy)) / denominator
        weight_b = ((cy - ay) * (sample_x - cx) + (ax - cx) * (scan_y - cy)) / denominator
        depth = weight_a * depths[0] + weight_b * depths[1] + (1 - weight_a - weight_b) * depths[2]
        index = y * width + minimum_x
        for x in range(minimum_x, maximum_x + 1):
            if depth > depth_buffer[index]:
                if opaque:
                    pixels[x, y] = color[:3]
                    depth_buffer[index] = depth
                else:
                    previous = pixels[x, y]
                    pixels[x, y] = tuple(round(color[i] * alpha + previous[i] * (1 - alpha)) for i in range(3))
            depth += depth_dx
            index += 1


def render(
    camera: Camera,
    size: tuple[int, int],
    groups: set[str] | None = None,
    hidden_prefixes: tuple[str, ...] = (),
    included_prefixes: tuple[str, ...] = (),
) -> Image.Image:
    width, height = size
    image = Image.new("RGB", size, BACKGROUND)
    depth_buffer = [-math.inf] * (width * height)
    right, up, forward = camera.basis()
    triangles = []
    projected_bounds = []
    for component in ship_components():
        if groups is not None and component.group not in groups:
            continue
        if component.name.startswith(hidden_prefixes):
            continue
        if included_prefixes and not component.name.startswith(included_prefixes):
            continue
        geometry = component.geometry
        positions = []
        for position in geometry.positions:
            relative = vector_sub(position, camera.eye)
            positions.append((dot(relative, right), dot(relative, up), dot(relative, forward)))
        projected_bounds.extend(positions)
        for offset in range(0, len(geometry.indices), 3):
            indices = geometry.indices[offset:offset + 3]
            normal = geometry.normals[indices[0]]
            color = face_color(component, normal)
            # The native ship pipeline uses its default unculled primitive
            # state, so both windings participate in this preview's depth test.
            polygon = [positions[index] for index in indices]
            if camera.perspective:
                polygon = clip_near(polygon)
            for index in range(1, len(polygon) - 1):
                points = [polygon[0], polygon[index], polygon[index + 1]]
                triangles.append((color, points))

    if camera.perspective:
        scale = height / (2 * math.tan(math.radians(camera.vertical_fov_degrees) / 2))
        def project(point: Point) -> Point:
            return (width / 2 + point[0] * scale / point[2], height / 2 - point[1] * scale / point[2], point[2])
    else:
        minimum_x = min(point[0] for point in projected_bounds)
        maximum_x = max(point[0] for point in projected_bounds)
        minimum_y = min(point[1] for point in projected_bounds)
        maximum_y = max(point[1] for point in projected_bounds)
        scale = 0.88 * min(width / (maximum_x - minimum_x), height / (maximum_y - minimum_y))
        center_x, center_y = (minimum_x + maximum_x) / 2, (minimum_y + maximum_y) / 2
        def project(point: Point) -> Point:
            return (width / 2 + (point[0] - center_x) * scale, height / 2 - (point[1] - center_y) * scale, point[2])

    # As in the native renderer, draw opaque geometry first, then depth-tested
    # glass without depth writes. Sort this preview's glass back to front.
    triangles.sort(key=lambda item: (item[0][3] != 255, -sum(point[2] for point in item[1])))
    for color, points in triangles:
        rasterize(image, depth_buffer, [project(point) for point in points], color, camera.perspective)
    return image


def render_core_previews(output: Path) -> None:
    """Create detailed Core plates using exactly the exported source meshes.

    Isolated views omit the deck conduits; the cabin view retains every ship
    component. No generated concept artwork or substitute geometry is used.
    """
    output.mkdir(parents=True, exist_ok=True)
    core_names = ("Core_", "Energy_Core")
    title_font = ImageFont.load_default(size=44)
    eyebrow_font = ImageFont.load_default(size=17)
    caption_font = ImageFont.load_default(size=19)
    detail_font = ImageFont.load_default(size=15)
    views = [
        (
            "core-hero.png", "01 / THREE-QUARTER VIEW", "ENERGY CORE",
            "Containment assembly / central energy-storage artifact",
            Camera((3.8, 2.75, -5.0), (-1.0, 1.15, 0.0)),
            ("Core_Power_Conduits",), core_names, (1600, 1400),
        ),
        (
            "core-detail.png", "02 / REVERSE VIEW", "CONTAINMENT DETAIL",
            "Opposite-side inspection / crown, shell and service cradle",
            Camera((-4.9, 3.65, 4.8), (-1.0, 1.15, 0.0)),
            ("Core_Power_Conduits",), core_names, (1600, 1400),
        ),
        (
            "core-cabin.png", "03 / CABIN CONTEXT", "THE HEART OF THE SHIP",
            "Standing cabin perspective / warm interior and cyan Core",
            Camera((1.8, STANDING_VIEWPOINT_METERS[1], -3.0), (-1.05, 1.26, 0.1), True, 61.0),
            (), (), (1800, 1200),
        ),
    ]
    for filename, eyebrow, title, caption, camera, hidden, included, size in views:
        width, height = size
        header, footer, margin = 160, 102, 38
        panel_size = (width - margin * 2, height - header - footer)
        # Supersample real geometry before adding crisp presentation labels.
        panel = render(
            camera,
            (panel_size[0] * 2, panel_size[1] * 2),
            hidden_prefixes=hidden,
            included_prefixes=included,
        ).resize(panel_size, Image.Resampling.LANCZOS)
        plate = Image.new("RGB", size, BACKGROUND)
        plate.paste(panel, (margin, header))
        draw = ImageDraw.Draw(plate)
        draw.text((margin, 24), "SALIMON / SCOUT SYSTEMS", fill=(102, 220, 224), font=eyebrow_font)
        draw.text((margin, 61), title, fill=(232, 235, 228), font=title_font)
        draw.text((margin, 121), caption, fill=(159, 176, 184), font=caption_font)
        draw.line((margin, height - footer + 21, width - margin, height - footer + 21), fill=(54, 80, 87))
        draw.text((margin, height - footer + 35), eyebrow, fill=(200, 162, 113), font=eyebrow_font)
        draw.text(
            (margin, height - 34),
            "SOURCE-GEOMETRY PREVIEW / Approximate materials and lighting. Not a game screenshot.",
            fill=(137, 157, 167), font=detail_font,
        )
        plate.save(output / filename)
        print(f"Rendered {output / filename}", flush=True)



def render_cockpit_previews(output: Path) -> None:
    """Inspect the exported pilot station and chair with actual source geometry."""
    output.mkdir(parents=True, exist_ok=True)
    views=[
        ("cockpit-station.png","PILOT STATION","Faceted housings / inset bezels / tactile controls / deck pedals",
         Camera((.4,3.4,-5.4),(3.7,1.0,0.0)),("Cockpit_Console","Cockpit_Monitor","Cockpit_Instrument",
         "Cockpit_Tactile","Cockpit_Ready","Cockpit_Service","Cockpit_Deck","Cockpit_Rudder","Monitor_","Pilot_")),
        ("pilot-chair.png","PILOT BUCKET","Contoured shell / split cushions / side bolsters / compact headrest",
         Camera((5.9,2.9,-4.3),(2.5,1.0,0.0)),("Pilot_",)),
        ("cockpit-seated.png","AT THE CONTROLS","Authored seated eye / slight downward inspection / clear forward glazing",
         Camera((2.76,1.799032258064516,0.0),(5.5,1.38,0.0),True,74.0),()),
    ]
    for filename,title,caption,camera,included in views:
        panel=render(camera,(1800,1040),included_prefixes=included).resize((1440,832),Image.Resampling.LANCZOS)
        plate=Image.new("RGB",(1520,1050),BACKGROUND)
        plate.paste(panel,(40,136))
        draw=ImageDraw.Draw(plate)
        draw.text((40,24),"SALIMON / SCOUT SYSTEMS",fill=(102,220,224),font=ImageFont.load_default(size=17))
        draw.text((40,59),title,fill=(232,235,228),font=ImageFont.load_default(size=40))
        draw.text((40,110),caption,fill=(159,176,184),font=ImageFont.load_default(size=18))
        draw.line((40,984,1480,984),fill=(54,80,87))
        draw.text((40,1000),"SOURCE GEOMETRY / Approximate lighting; cyan instrument faces receive live telemetry artwork in the runtime.",
                  fill=(137,157,167),font=ImageFont.load_default(size=15))
        plate.save(output/filename)
        print(f"Rendered {output/filename}",flush=True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--core", action="store_true", help="Render three Core detail PNGs into the output directory")
    parser.add_argument("--cockpit", action="store_true", help="Render pilot station, chair and seated-view PNGs into the output directory")
    args = parser.parse_args()
    if args.cockpit:
        render_cockpit_previews(args.output)
        return
    if args.core:
        render_core_previews(args.output)
        return
    panel_width, panel_height = 840, 560
    title_height, footer_height = 60, 48
    row_height = panel_height + title_height
    sheet = Image.new("RGB", (panel_width * 3, row_height * 2 + footer_height), BACKGROUND)
    eye = tuple(STANDING_VIEWPOINT_METERS)
    views = [
        ("01 / FORWARD EXTERIOR", "Wide hull, panoramic canopy and copper trim", Camera((23, 17, 28), (0, 1.5, 0)), None, ()),
        ("02 / AFT EXTERIOR", "Recessed twin nozzles, heat shields and observation windows", Camera((-25, 14, -28), (0, 1.5, 0)), None, ()),
        ("03 / CABIN CUTAWAY", "Roof and wall groups hidden to inspect circulation", Camera((13, 22, 18), (-0.5, 0, 0)), {"Interior"}, ("Ceiling_", "Wall_", "Window_", "Aft_Bulkhead", "Exit_Door", "Door_Frame", "Door_Welcome", "Cargo_Ceiling", "Cargo_Aft_Wall", "Cargo_Forward_Wall", "Cargo_Port_Wall", "Cargo_Inboard_Partition", "Cargo_Passage_Header", "Cargo_Lights")),
        ("04 / STANDING BEHIND THE CHAIR", "Authored standing eye height; forward view remains open", Camera(eye, (11, eye[1], 0), True), None, ()),
        ("05 / CORE AND SIDE WINDOWS", "Warm lounge, central energy Core and broad side glazing", Camera((0.3, eye[1], -2.6), (-2.3, 1.5, 2.1), True), None, ()),
        ("06 / REAR CABIN VIEW", "Aft observation windows beside the exit door", Camera((0.6, eye[1], 2.45), (-7.7, 1.9, -0.2), True), None, ()),
    ]
    draw = ImageDraw.Draw(sheet)
    title_font = ImageFont.load_default(size=20)
    caption_font = ImageFont.load_default(size=14)
    for index, (title, caption, camera, groups, hidden) in enumerate(views):
        x = (index % 3) * panel_width
        y = (index // 3) * row_height
        panel = render(camera, (panel_width * 2, panel_height * 2), groups, hidden)
        panel = panel.resize((panel_width, panel_height), Image.Resampling.LANCZOS)
        sheet.paste(panel, (x, y + title_height))
        draw.text((x + 20, y + 10), title, fill=(231, 225, 208), font=title_font)
        draw.text((x + 20, y + 36), caption, fill=(151, 169, 182), font=caption_font)
        draw.line((x, y + row_height - 1, x + panel_width, y + row_height - 1), fill=(47, 58, 65))
    draw.text((20, row_height * 2 + 15), "SALIMON / SOFTWARE ASSET QA PREVIEW - source geometry and approximate native materials; game sky and dynamic scene are omitted.", fill=(151, 169, 182), font=caption_font)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(args.output, quality=94)
    print(f"Rendered {args.output}")


if __name__ == "__main__":
    main()
