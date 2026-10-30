"""Check native canonical terrain captures, frozen repeats and bounded residency."""
import argparse
import hashlib
import json
import math
from pathlib import Path

from PIL import Image


LIMITS = {"active_patches": 512, "cache_patches": 1024,
          "gpu_cache_bytes": 32 * 1024 * 1024, "generated_peak": 8,
          "peak_upload_bytes": 2 * 1024 * 1024}
REQUIRED = {"universe-orbit", "universe-surface", "universe-edge",
            "universe-orbit-revisit"}


def orbit_diameter(image):
    # universe-orbit: camera at 2.5 radii, centered sphere, 70 degree vertical FOV.
    # This explicit terrain probe has a black background and no HUD on mid-row.
    width, height = image.size
    xs = [x for x in range(width) if min(image.getpixel((x, height // 2))) > 50]
    if not xs:
        raise ValueError("orbit disk missing")
    measured = xs[-1] - xs[0] + 1
    expected = height * math.tan(math.asin(1 / 2.5)) / math.tan(math.radians(35))
    # Four physical pixels cover raster edge rounding and <=1,800 m published
    # relief on the multi-megameter body; this is not a general visual tolerance.
    if abs(measured - expected) > 4 or abs((xs[0] + xs[-1]) / 2 - width / 2) > 2:
        raise ValueError("orbit projected diameter/center disagrees with physical scale")
    return {"measured_pixels": measured, "expected_pixels": expected}


def review(result_path):
    result = json.loads(Path(result_path).read_text())
    if result.get("status") != "passed":
        raise ValueError("canonical terrain scenario did not pass")
    images, contexts, rows = {}, {}, []
    for step in result["steps"]:
        label = step["step"].get("screenshot")
        if label is None:
            continue
        if label in images:
            raise ValueError("duplicate capture label")
        capture, state = step["screenshot"], step["state"]
        if (step.get("status") != "passed" or capture.get("status") != "captured"
                or capture.get("source") != "GPU color surface"):
            raise ValueError("original native GPU captures required")
        context, terrain = state["visual_context"], state["terrain"]
        reference = state["reference_camera"]
        expected_view = label.removesuffix("-repeat").removesuffix("-revisit")
        clearance = reference["canonical_terrain_clearance_meters"]
        if reference["name"] != expected_view or isinstance(clearance, bool) or not isinstance(clearance, (int, float)) or not math.isfinite(clearance) or clearance <= 0:
            raise ValueError("reference camera intersects terrain or names another view")
        if terrain["mode"] != "Canonical" or not terrain["presentation_ready"] or terrain["pending_patches"] != 0:
            raise ValueError("canonical terrain was not settled")
        for key, limit in LIMITS.items():
            value = terrain[key]
            if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value) or not 0 <= value <= limit:
                raise ValueError(f"{key} exceeds streaming bound")
        if terrain["active_patches"] == 0 or terrain["triangles"] == 0:
            raise ValueError("empty terrain")
        with Image.open(capture["path"]) as source:
            image = source.convert("RGB")
        if list(image.size) != context["viewport_pixels"]:
            raise ValueError("capture dimensions disagree with viewport")
        if len(image.getcolors(image.width * image.height)) < 16:
            raise ValueError("blank or degenerate capture")
        performance = state["performance"]
        raw = performance["raw_frames"]
        if not raw or len(raw) != performance["samples"] or len(raw) > 3600:
            raise ValueError("missing or inconsistent raw frame samples")
        for frame in raw:
            if len(frame) != 6 or any(v is None or not math.isfinite(v) or v < 0 for i, v in enumerate(frame) if i != 3):
                raise ValueError("invalid raw frame telemetry")
            if frame[3] is not None and (not math.isfinite(frame[3]) or frame[3] < 0):
                raise ValueError("invalid GPU time")
        images[label] = image
        contexts[label] = context
        rows.append({"label": label, "sha256": hashlib.sha256(Path(capture["path"]).read_bytes()).hexdigest(),
                     "context": context, "reference_camera": reference, "terrain": terrain, "performance": performance})
    if set(images) != REQUIRED | {name + "-repeat" for name in REQUIRED}:
        raise ValueError("missing required orbit/surface/edge/revisit captures or repeats")
    for name in REQUIRED:
        repeat = name + "-repeat"
        if contexts[name] != contexts[repeat] or images[name].size != images[repeat].size or images[name].tobytes() != images[repeat].tobytes():
            raise ValueError(f"{name}: frozen production frames differ")
    # Returning after surface/face-edge preparation must reconstruct the same view.
    if contexts["universe-orbit"] != contexts["universe-orbit-revisit"] or images["universe-orbit"].tobytes() != images["universe-orbit-revisit"].tobytes():
        raise ValueError("orbit revisit differs")
    return {"status": "passed", "result": str(result_path), "captures": rows,
            "exact_repeats": 4, "exact_revisits": 1,
            "orbit_diameter": orbit_diameter(images["universe-orbit"])}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("result", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        report = review(args.result)
    except (OSError, ValueError, KeyError, TypeError) as exc:
        report = {"status": "failed", "error": str(exc)}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"status": report["status"], "output": str(args.output)}))
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
