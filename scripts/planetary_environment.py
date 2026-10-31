"""Run the fixed planetary environment evidence matrix; never certify hardware FPS."""
import argparse
import hashlib
import json
import math
import os
import platform
import statistics
from pathlib import Path
import subprocess
import sys
import time
import wave
import array

from scenario_suites import PLANETARY_ENVIRONMENT
from salimon_test import parse_scenario
from PIL import Image
from review_universe_captures import review as review_terrain

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    hasher = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def distribution(values):
    values = sorted(values)
    if not values:
        return None
    if any(type(v) not in (int, float) or not math.isfinite(v) or v < 0 for v in values):
        raise ValueError("invalid performance sample")
    return {"count": len(values), "median": statistics.median(values),
            "p95": values[math.ceil(len(values) * .95) - 1], "max": values[-1]}


def review_native(summary):
    """Require complete routes, real captures and raw metrics, retaining each context."""
    if summary.get("status") != "passed":
        raise ValueError("native suite failed")
    results = summary["results"]
    if len(results) != len(PLANETARY_ENVIRONMENT):
        raise ValueError("incomplete native matrix")
    rows = []
    for expected, result in zip(PLANETARY_ENVIRONMENT, results):
        path = Path(result["result_file"])
        source = json.loads(path.read_text())
        scenario = json.loads((path.parent / "scenario.json").read_text())
        contract = parse_scenario(ROOT / "scenarios/evidence" / expected)
        if scenario != contract or source["status"] != "passed":
            raise ValueError(f"wrong/failed scenario: {expected}")
        if [step["step"] for step in source["steps"]] != contract["steps"]:
            raise ValueError(f"incomplete/changed steps: {expected}")
        captures = []
        for step in source["steps"]:
            if step.get("status") != "passed":
                raise ValueError(f"failed step: {expected}")
            if "screenshot" not in step["step"]:
                continue
            capture, state = step["screenshot"], step["state"]
            if capture.get("status") != "captured" or capture.get("source") != "GPU color surface":
                raise ValueError("missing production GPU capture")
            original = Path(capture["path"])
            with Image.open(original) as image:
                image.load()
                if image.format != "PNG" or list(image.size) != state["visual_context"]["viewport_pixels"]:
                    raise ValueError("invalid capture dimensions/format")
            performance = state["performance"]
            raw = performance["raw_frames"]
            if not raw or len(raw) != performance["samples"] or len(raw) > 3600:
                raise ValueError("missing raw frame samples")
            if any(len(frame) != 6 for frame in raw):
                raise ValueError("invalid raw frame layout")
            metrics = {}
            for index, name in enumerate(("present_ms", "cpu_update_ms", "cpu_render_ms", "gpu_ms", "scene_draws", "total_draws")):
                values = [frame[index] for frame in raw]
                if index != 3 and any(v is None for v in values):
                    raise ValueError("missing CPU/draw measurement")
                metrics[name] = distribution([v for v in values if v is not None])
            captures.append({"label": step["step"]["screenshot"], "path": str(original),
                             "sha256": digest(original), "context": state["visual_context"],
                             "terrain": state.get("terrain"), "metrics": metrics,
                             "reference_camera": state.get("reference_camera")})
        if not captures:
            raise ValueError(f"no captures: {expected}")
        rows.append({"scenario": expected, "result_file": str(path), "sha256": digest(path),
                     "steps": len(source["steps"]), "captures": captures})
    return rows


def review_audio(directory):
    metrics = {}
    for name in ("vacuum-quiet", "suit-local-weather", "scout-ramp-open", "scout-ramp-closed"):
        with wave.open(str(directory / f"{name}.wav"), "rb") as clip:
            if (clip.getnchannels(), clip.getsampwidth(), clip.getframerate(), clip.getnframes()) != (2, 2, 48000, 96000):
                raise ValueError("wrong callback PCM format/duration")
            samples = array.array("h", clip.readframes(clip.getnframes()))
            if sys.byteorder != "little":
                samples.byteswap()
        metrics[name] = math.sqrt(sum(v*v for v in samples) / len(samples))
    if metrics["vacuum-quiet"] != 0 or not 0 < metrics["scout-ramp-closed"] < metrics["scout-ramp-open"]:
        raise ValueError("vacuum silence or Scout transmission failed")
    return {"rms_pcm16": metrics, "source": "production callback PCM, not device recording"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native-summary", type=Path, help="reuse a complete native summary; contracts and captures are revalidated")
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True, help="new isolated directory")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    report = {"status": "failed",
        "environment": {"platform": platform.platform(), "python": platform.python_version(),
                        "wgpu_backend": os.environ.get("WGPU_BACKEND"),
                        "vulkan_driver": os.environ.get("VK_DRIVER_FILES"),
                        "software_threads": os.environ.get("LP_NUM_THREADS"),
                        "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip()},
        "parent_revision": subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "binary_sha256": digest(args.binary.resolve()),
        "source_sha256": {str(p.relative_to(ROOT)): digest(p)
                          for folder in ("client", "scripts", "scenarios")
                          for p in sorted((ROOT / folder).rglob("*"))
                          if p.is_file() and p.suffix in {".rs", ".py", ".wgsl", ".json", ".toml"}},
        "commands": [],
        "qualification": {"physical_input": "unverified #557", "presentation": "unverified #561",
                          "hardware_M1_Metal": "unverified #563"}}

    def run(name, argv, env=None):
        start = time.monotonic()
        with (output / f"{name}.log").open("w") as log:
            completed = subprocess.run(argv, cwd=ROOT, env={**os.environ, **(env or {})},
                                       stdout=log, stderr=subprocess.STDOUT, timeout=3600)
        report["commands"].append({"name": name, "argv": argv, "env": env or {},
                                   "exit_code": completed.returncode,
                                   "elapsed_seconds": time.monotonic() - start})
        if completed.returncode:
            raise ValueError(f"{name} failed; see {output / (name + '.log')}")

    def cargo(name, package, test, env):
        run(name, ["cargo", "test", "-p", package, "--locked", test,
                   "--", "--ignored", "--nocapture"], env)
        # A misspelled test filter must not turn zero executed tests into success.
        if "1 passed; 0 failed" not in (output / f"{name}.log").read_text():
            raise ValueError(f"{name}: expected exactly one opt-in evidence test")

    try:
        if args.native_summary:
            summary = json.loads(args.native_summary.read_text())
            report["native_summary"] = str(args.native_summary.resolve())
        else:
            run("native", [sys.executable, "scripts/salimon-test", "suite", "--group",
                           "planetary-environment", "--binary", str(args.binary.resolve()),
                           "--artifacts", str(output / "native"), "--screenshot-command", '["native"]', "--summary"],
                {"SALIMON_SETTINGS_FILE": str(output / "settings.json")})
            summary = json.loads((output / "native.log").read_text())
        report["native"] = review_native(summary)
        report["terrain_review"] = review_terrain(Path(report["native"][0]["result_file"]))
        for name, test, variable in (
            ("weather", "weather_integrated_frame_export", "SALIMON_WEATHER_FRAME_FILE"),
            ("specimens", "organism_specimen_export", "SALIMON_ORGANISMS_FILE"),
            ("natural", "natural_site_production_export", "SALIMON_NATURAL_SITES_FILE"),
            ("landmark", "landmark_production_patch_export", "SALIMON_LANDMARK_PATCH_FILE"),
        ):
            original = output / f"{name}.json"
            repeated = output / f"{name}-repeat.json"
            cargo(name + "-export", "salimon-client", test, {variable: str(original)})
            cargo(name + "-repeat", "salimon-client", test, {variable: str(repeated)})
            if original.read_bytes() != repeated.read_bytes():
                raise ValueError(f"{name}: canonical source export changed across processes")
            report.setdefault("repeat_exports", {})[name] = digest(original)
            render_env = {variable: str(original)}
            if name == "weather":
                render_test = "weather_production_reference"
                render_env["SALIMON_WEATHER_DIR"] = str(output / "weather")
            elif name == "landmark":
                render_test = "landmark_production_reference"
                render_env["SALIMON_TERRAIN_STYLE_DIR"] = str(output / "landmark")
            else:
                render_test = "organism_production_reference"
                render_env = {"SALIMON_ORGANISMS_FILE": str(original),
                              "SALIMON_ORGANISM_DIR": str(output / name)}
            cargo(name + "-render", "salimon-renderer", render_test, render_env)
        cargo("audio", "salimon-client", "export_environment_audio_627",
              {"SALIMON_AUDIO_EVIDENCE": str(output / "audio")})
        report["audio_review"] = review_audio(output / "audio")
        report["status"] = "passed"
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as exc:
        report["error"] = str(exc)
    finally:
        report["files"] = {str(p.relative_to(output)): digest(p) for p in sorted(output.rglob("*"))
                           if p.is_file() and p.name != "matrix.json"}
        (output / "matrix.json").write_text(json.dumps(report, indent=2, allow_nan=False) + "\n")
    print(json.dumps({"status": report["status"], "report": str(output / "matrix.json"),
                      "error": report.get("error")}))
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
