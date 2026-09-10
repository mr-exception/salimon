#!/usr/bin/env python3
"""Render quick orthographic preview sheets from the procedural ship source."""

from __future__ import annotations

import argparse
import math
from pathlib import Path

from PIL import Image, ImageDraw

from generate_salimon_phase0_ship import MATERIALS, ship_components


def rotate(point: tuple[float, float, float], yaw: float, pitch: float) -> tuple[float, float, float]:
    x, y, z = point
    cy, sy = math.cos(yaw), math.sin(yaw)
    x, z = x * cy + z * sy, -x * sy + z * cy
    cp, sp = math.cos(pitch), math.sin(pitch)
    return (x, y * cp - z * sp, y * sp + z * cp)


def render(
    yaw: float,
    pitch: float,
    size: tuple[int, int],
    groups: set[str] | None = None,
    hidden: set[str] | None = None,
) -> Image.Image:
    width, height = size
    background = (18, 23, 28, 255)
    image = Image.new("RGBA", size, background)
    draw = ImageDraw.Draw(image, "RGBA")
    triangles = []
    for component in ship_components():
        if groups is not None and component.group not in groups:
            continue
        if hidden is not None and component.name in hidden:
            continue
        color = tuple(round(channel * 255) for channel in MATERIALS[component.material][1])
        positions = component.geometry.positions
        for offset in range(0, len(component.geometry.indices), 3):
            points = [rotate(positions[component.geometry.indices[offset + index]], yaw, pitch) for index in range(3)]
            depth = sum(point[2] for point in points) / 3.0
            triangles.append((depth, points, color))
    scale = min(width / 13.0, height / 10.0) * 0.76
    for _, points, color in sorted(triangles, key=lambda item: item[0]):
        polygon = [(width / 2 + point[0] * scale, height * 0.58 - point[1] * scale) for point in points]
        draw.polygon(polygon, fill=color, outline=(8, 12, 15, 90))
    return image


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    panel_size = (720, 560)
    sheet = Image.new("RGBA", (panel_size[0] * 3, panel_size[1]), (18, 23, 28, 255))
    sheet.alpha_composite(render(math.radians(-38), math.radians(18), panel_size), (0, 0))
    sheet.alpha_composite(render(math.radians(142), math.radians(13), panel_size), (panel_size[0], 0))
    sheet.alpha_composite(
        render(
            math.radians(150),
            math.radians(10),
            panel_size,
            {"Interior"},
            {
                "Ceiling_Inner",
                "Wall_Port_Inner",
                "Wall_Starboard_Inner",
                "Aft_Bulkhead_Port",
                "Aft_Bulkhead_Starboard",
                "Aft_Bulkhead_Header",
                "Interior_Window",
            },
        ),
        (panel_size[0] * 2, 0),
    )
    draw = ImageDraw.Draw(sheet)
    draw.text((24, 22), "SALIMON PHASE 0 SCOUT — FORWARD", fill=(225, 235, 237, 255))
    draw.text((panel_size[0] + 24, 22), "AFT SILHOUETTE", fill=(225, 235, 237, 255))
    draw.text((panel_size[0] * 2 + 24, 22), "INTERIOR CUTAWAY", fill=(225, 235, 237, 255))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    sheet.convert("RGB").save(args.output, quality=92)
    print(f"Rendered {args.output}")


if __name__ == "__main__":
    main()
