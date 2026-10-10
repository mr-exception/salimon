"""Release fragment matrix runner; fixed-delta CPU costs are never labeled FPS."""
import argparse
import datetime
import hashlib
import json
import math
import os
import platform
import statistics
import subprocess
import sys
from pathlib import Path

COUNTS = (0, 10, 25, 50, 100, 250, 500, 1000)
METRICS = ("frame_ms", "cpu_update_ms", "cpu_simulation_ms", "cpu_scene_prepare_ms", "physics_total_ms", "physics_adapter_ms",
           "physics_integration_ms", "physics_contact_ms", "cpu_render_ms",
           "resource_prepare_ms", "resource_upload_ms", "gpu_latest_ms", "pair_visits",
           "radius_candidates", "narrow_phase_tests", "contacts", "substeps",
           "matrix_allocations", "projection_storage_bytes", "resource_upload_bytes",
           "resource_triangles", "draw_calls", "moving_objects", "objects_simulated")


def distribution(values):
    values = sorted(values)
    if not values:
        return None
    def percentile(p):
        return values[max(0, math.ceil(len(values) * p) - 1)]
    return {"median": statistics.median(values), "p95": percentile(.95),
            "p99": percentile(.99), "max": values[-1]}


def summarize(result):
    if result.get("schema") != 1 or not result.get("samples") or result.get("accounting_preserved") is not True:
        raise ValueError("invalid/incomplete benchmark or accounting failure")
    fragments = result["accounting"]["fragments"]
    if len(fragments) != result["config"]["count"] or len({p["id"] for p in fragments}) != len(fragments):
        raise ValueError("fragment count/identity mismatch")
    if not math.isclose(sum(p["mass_kg"] for p in fragments), result["accounting"]["extracted_mass_kg"], abs_tol=1e-9):
        raise ValueError("fragment mass is not conserved")
    metrics = {key: distribution([s[key] for s in result["samples"] if s.get(key) is not None]) for key in METRICS}
    frame = metrics["frame_ms"]
    metrics["frames_over_16_67ms"] = sum(s.get("frame_ms") is not None and s["frame_ms"] > 1000 / 60 for s in result["samples"])
    metrics["frames_over_50ms"] = sum(s.get("frame_ms") is not None and s["frame_ms"] > 50 for s in result["samples"])
    return {"config": result["config"], "mode": result["mode"], "renderer": result["renderer"],
            "samples_retained": len(result["samples"]), "tail_statistics_reliable": len(result["samples"]) >= 100, "updates_observed": result.get("updates_observed"),
            "sampling_stride": result.get("sampling_stride", 1), "metrics": metrics,
            "settled_observed": all(s["moving_objects"] == 0 for s in result["samples"]),
            "active_observed": any(s["moving_objects"] > 0 for s in result["samples"]),
            "median_interval_fps": 1000 / frame["median"] if frame and frame["median"] > 0 else None}


def command_output(args, cwd):
    return subprocess.check_output(args, cwd=cwd, text=True, timeout=10).strip()


def compare(current, baseline, tolerance):
    # Exact hardware/settings/revision lineage is reviewed separately. Match case
    # and execution metadata; never compare CPU-only/software data to native GPUs.
    if current["environment"] != baseline["environment"]:
        raise ValueError("baseline environment differs; compare only the same machine/configuration")
    old = {(c["mode"], json.dumps(c["config"], sort_keys=True)): c for c in baseline["cases"]}
    failures = []
    for case in current["cases"]:
        key = (case["mode"], json.dumps(case["config"], sort_keys=True))
        if key not in old or old[key]["renderer"] != case["renderer"]:
            raise ValueError("baseline case or renderer configuration missing/different")
        if min(case["samples_retained"], old[key]["samples_retained"]) < 100:
            raise ValueError("relative performance gates require at least 100 retained samples per case")
        for metric in ("frame_ms", "physics_total_ms", "cpu_render_ms"):
            a, b = case["metrics"][metric], old[key]["metrics"][metric]
            if a is not None and b is not None and b["p95"] >= .1 and a["p95"] > b["p95"] * (1 + tolerance):
                failures.append({"case": case["config"], "metric": metric, "baseline_p95": b["p95"], "current_p95": a["p95"]})
    return failures


def binary_hash(path):
    digest = hashlib.sha256()
    with path.open("rb") as binary:
        for chunk in iter(lambda: binary.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, default=Path("artifacts/fragment-benchmarks"))
    parser.add_argument("--counts", type=int, nargs="+", default=COUNTS)
    parser.add_argument("--layouts", nargs="+", choices=("scattered", "dense", "surface"), default=("scattered", "dense", "surface"))
    parser.add_argument("--states", nargs="+", choices=("settled", "active"), default=("settled", "active"))
    parser.add_argument("--cameras", nargs="+", choices=("facing", "away"), default=("facing", "away"))
    parser.add_argument("--mode", choices=("native", "physics"), default="native")
    parser.add_argument("--seed", type=int, default=155)
    parser.add_argument("--warmup-seconds", type=int, default=10)
    parser.add_argument("--sample-seconds", type=int, default=30)
    parser.add_argument("--timeout", type=int, default=300, help="per case; failures preserve stdout/stderr")
    parser.add_argument("--machine-label", required=True, help="explicit hardware/configuration label; use software-vulkan for lavapipe")
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--tolerance", type=float, default=.2)
    args = parser.parse_args(argv)
    if any(n < 0 or n > 1000 for n in args.counts) or not 0 <= args.seed < 2**64 or not 0 <= args.warmup_seconds <= 600 or not 1 <= args.sample_seconds <= 600 or args.timeout <= 0 or not 0 <= args.tolerance <= 1:
        parser.error("invalid population, duration, seed, timeout or tolerance")
    root = Path(__file__).resolve().parents[1]
    binary = args.binary.resolve()
    args.output.mkdir(parents=True, exist_ok=True)
    summary = {"schema": 1, "created_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "revision": command_output(["git", "rev-parse", "HEAD"], root),
        "binary_sha256": binary_hash(binary),
        "dirty": bool(command_output(["git", "status", "--porcelain"], root)),
        "environment": {"machine_label": args.machine_label, "platform": platform.platform(), "machine": platform.machine(),
            "cpu": platform.processor(), "logical_cpus": os.cpu_count(),
            "wgpu_backend": os.environ.get("WGPU_BACKEND"), "vk_driver_files": os.environ.get("VK_DRIVER_FILES")},
        "cases": [], "failures": []}
    cameras = ("facing",) if args.mode == "physics" else args.cameras
    for count in args.counts:
        for layout in args.layouts:
            for state in args.states:
                for camera in cameras:
                    name = f"{args.mode}-{count}-{layout}-{state}-{camera}"
                    command = [str(binary), "--benchmark", "--count", str(count), "--layout", layout,
                        "--state", state, "--camera", camera, "--mode", args.mode, "--seed", str(args.seed),
                        "--warmup-seconds", str(args.warmup_seconds), "--sample-seconds", str(args.sample_seconds)]
                    print(f"Running {name}", file=sys.stderr, flush=True)
                    try:
                        # File-backed capture avoids buffering unbounded process logs.
                        with (args.output / f"{name}.stdout").open("w") as stdout, (args.output / f"{name}.stderr").open("w") as stderr:
                            completed = subprocess.run(command, cwd=root, stdout=stdout, stderr=stderr, timeout=args.timeout, check=False)
                        if completed.returncode:
                            raise ValueError(f"benchmark exited {completed.returncode}")
                        result = json.loads((args.output / f"{name}.stdout").read_text())
                        case = summarize(result)
                        case["raw_file"] = f"{name}.stdout"
                        summary["cases"].append(case)
                    except (OSError, subprocess.TimeoutExpired, ValueError, KeyError) as exc:
                        summary["failures"].append({"case": name, "error": str(exc)})
                    (args.output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    if args.baseline:
        summary["regressions"] = compare(summary, json.loads(args.baseline.read_text()), args.tolerance)
    (args.output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    return 1 if summary["failures"] or summary.get("regressions") else 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, subprocess.SubprocessError) as exc:
        print(f"Fragment benchmark failed: {exc}", file=sys.stderr)
        raise SystemExit(1)
