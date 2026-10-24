"""Review original scanner GPU captures and exact frozen-frame repeats; no baselines."""
import argparse
import json
from pathlib import Path

from PIL import Image


def review(result_path):
    result = json.loads(Path(result_path).read_text())
    if result.get("status") != "passed":
        raise ValueError("scanner scenario did not pass")
    images = {}
    rows = []
    repeats = 0
    fresh = 0
    for step in result["steps"]:
        label = step["step"].get("screenshot")
        if label is None:
            continue
        capture = step["screenshot"]
        if step.get("status") != "passed" or capture.get("status") != "captured" or capture.get("source") != "GPU color surface":
            raise ValueError("scanner review requires original native GPU captures")
        context = step["state"]["visual_context"]
        with Image.open(capture["path"]) as source:
            image = source.convert("RGB")
        if list(image.size) != context["viewport_pixels"]:
            raise ValueError("capture dimensions disagree with recorded viewport")
        images[label] = image
        if label == "cockpit-coexistence":
            # This fixture deliberately switches to the wider cockpit camera;
            # the engineering projector ROI below does not describe that view.
            rows.append({"label": label, "context": context,
                         "checks": ["native source", "viewport dimensions"]})
            continue
        # Camera-relative projector ROI, scaled by vertical FOV, not desktop size.
        w, h = image.size
        scale = h / 800
        roi = tuple(round(v) for v in (
            w / 2 - 120 * scale, h / 2 - 160 * scale,
            w / 2 + 120 * scale, h / 2 + 75 * scale))
        raw = iter(image.crop(roi).tobytes())
        pixels = zip(raw, raw, raw)
        amber = sum(r > 220 and g > 130 and b < 140 for r, g, b in pixels)
        scanner = step["state"]["scanner"]
        active = scanner["powered"] and scanner["active_count"] > 0
        if active:
            fresh += 1
            if amber < 5 or "FRESH" not in (scanner["hint"] or ""):
                raise ValueError(f"{label}: fresh selected contact/readout missing")
        elif amber != 0:
            raise ValueError(f"{label}: unpowered/unknown projector reveals selection")
        original = {"fresh-repeat": "fresh", "identified-repeat": "identified-local-contact"}.get(label)
        if original:
            if original not in images or image.size != images[original].size or image.tobytes() != images[original].tobytes():
                raise ValueError(f"{label}: frozen production frames differ")
            repeats += 1
        rows.append({"label": label, "context": context, "roi": roi,
                     "selected_amber_pixels": amber, "fresh": active})
    if fresh == 0 or repeats == 0:
        raise ValueError("scanner review requires fresh and repeated GPU captures")
    return {"result": str(result_path), "captures": rows, "exact_repeats": repeats}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("results", type=Path, nargs="+")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        report = {"status": "passed", "runs": [review(p) for p in args.results]}
    except (OSError, ValueError, KeyError, TypeError) as exc:
        report = {"status": "failed", "error": str(exc)}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"status": report["status"], "output": str(args.output)}))
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
