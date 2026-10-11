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
import time
from pathlib import Path

COUNTS = (0, 10, 25, 50, 100, 250, 500, 1000)
METRICS = ("frame_ms", "cpu_update_ms", "cpu_simulation_ms", "cpu_scene_prepare_ms", "physics_total_ms", "physics_adapter_ms",
           "physics_integration_ms", "physics_contact_ms", "physics_activation_ms", "cpu_render_ms",
           "resource_prepare_ms", "resource_upload_ms", "gpu_latest_ms", "pair_visits",
           "radius_candidates", "narrow_phase_tests", "contacts", "substeps",
           "matrix_allocations", "projection_storage_bytes", "resource_upload_bytes",
           "resource_triangles", "draw_calls", "moving_objects", "objects_simulated", "awake_objects", "sleeping_objects", "quiet_objects", "integrated_objects")


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
            "sleeping_observed": all(s.get("sleeping_objects") == result["config"]["count"] for s in result["samples"]),
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


def prepared_batch(jobs, output, timeout):
    """Prepare concurrently, then run each native window with all other jobs idle."""
    processes = []
    outcomes = {}
    try:
        for job in jobs:
            stdout = (output / (job["name"] + ".stdout")).open("w")
            stderr_path = output / (job["name"] + ".stderr")
            stderr = stderr_path.open("w")
            try:
                process = subprocess.Popen(job["command"] + ["--defer-start", "true"],
                    stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, text=True)
            except OSError as exc:
                stdout.close()
                stderr.close()
                outcomes[job["name"]] = str(exc)
                continue
            processes.append({"job": job, "process": process, "stdout": stdout,
                "stderr": stderr, "stderr_path": stderr_path,
                "deadline": time.monotonic() + timeout, "ready": False})
        while any(not item["ready"] and item["job"]["name"] not in outcomes for item in processes):
            for item in processes:
                name, process = item["job"]["name"], item["process"]
                if name in outcomes or item["ready"]:
                    continue
                code = process.poll()
                if code is not None:
                    outcomes[name] = f"benchmark exited {code} before native readiness"
                elif time.monotonic() >= item["deadline"]:
                    process.kill()
                    process.wait()
                    outcomes[name] = f"preparation exceeded {timeout} second deadline"
                elif "SALIMON_BENCHMARK_PREPARED" in item["stderr_path"].read_text():
                    item["ready"] = True
                    print(f"Prepared {name}", file=sys.stderr, flush=True)
            if any(not item["ready"] and item["job"]["name"] not in outcomes for item in processes):
                time.sleep(.05)
        for item in processes:
            name, process = item["job"]["name"], item["process"]
            if name in outcomes:
                continue
            try:
                remaining = item["deadline"] - time.monotonic()
                if remaining <= 0:
                    raise subprocess.TimeoutExpired(item["job"]["command"], timeout)
                print(f"Measuring {name} (other fixtures idle)", file=sys.stderr, flush=True)
                process.stdin.write("SALIMON_BENCHMARK_START\n")
                process.stdin.flush()
                process.stdin.close()
                code = process.wait(timeout=remaining)
                outcomes[name] = None if code == 0 else f"benchmark exited {code}"
            except (OSError, subprocess.TimeoutExpired) as exc:
                outcomes[name] = str(exc)
    finally:
        for item in processes:
            process = item["process"]
            if process.poll() is None:
                process.kill()
                process.wait()
            if process.stdin and not process.stdin.closed:
                process.stdin.close()
            item["stdout"].close()
            item["stderr"].close()
    return outcomes


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
    parser.add_argument("--settle-steps", type=int, default=0,
                        help="prepare settled cases using up to this many production 16ms updates; fails unless all objects sleep")
    parser.add_argument("--repeats", type=int, default=1,
                        help="settled sampling windows in one process; active rows use fresh-process runs")
    parser.add_argument("--prepare-workers", type=int, default=1,
                        help="1..4 concurrent initializations; native sample windows remain sequential")
    parser.add_argument("--capture-binary", type=Path,
                        help="identical binary in an app bundle for the settled dense 500 capture surface")
    parser.add_argument("--solver-baseline-samples", type=int, default=0,
                        help="0..1000 identical-snapshot solver replays for settled dense 500/1000 cases")
    parser.add_argument("--timeout", type=int, default=300, help="per case; failures preserve stdout/stderr")
    parser.add_argument("--machine-label", required=True, help="explicit hardware/configuration label; use software-vulkan for lavapipe")
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--tolerance", type=float, default=.2)
    args = parser.parse_args(argv)
    if any(n < 0 or n > 1000 for n in args.counts) or not 0 <= args.seed < 2**64 or not 0 <= args.warmup_seconds <= 600 or not 1 <= args.sample_seconds <= 600 or not 0 <= args.settle_steps <= 37500 or not 1 <= args.repeats <= 10 or not 1 <= args.prepare_workers <= 4 or not 0 <= args.solver_baseline_samples <= 1000 or args.timeout <= 0 or not 0 <= args.tolerance <= 1:
        parser.error("invalid population, duration, seed, timeout or tolerance")
    if args.prepare_workers > 1 and args.mode != "native":
        parser.error("concurrent preparation is for native windows only")
    if args.solver_baseline_samples and not args.settle_steps:
        parser.error("solver baseline requires natural preparation")
    if any(len(set(values)) != len(values) for values in (args.counts, args.layouts, args.states, args.cameras)):
        parser.error("matrix selections must be unique")
    root = Path(__file__).resolve().parents[1]
    binary = args.binary.resolve()
    args.output.mkdir(parents=True, exist_ok=True)
    capture_binary = args.capture_binary.resolve() if args.capture_binary else binary
    if binary_hash(capture_binary) != binary_hash(binary):
        parser.error("capture binary must have the same SHA-256 as the benchmark binary")
    summary = {"schema": 1, "created_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "revision": command_output(["git", "rev-parse", "HEAD"], root),
        "binary_sha256": binary_hash(binary),
        "dirty": bool(command_output(["git", "status", "--porcelain"], root)),
        "environment": {"machine_label": args.machine_label, "platform": platform.platform(), "machine": platform.machine(),
            "cpu": platform.processor(), "logical_cpus": os.cpu_count(),
            "wgpu_backend": os.environ.get("WGPU_BACKEND"), "vk_driver_files": os.environ.get("VK_DRIVER_FILES")},
        "cases": [], "failures": []}
    cameras = ("facing",) if args.mode == "physics" else args.cameras
    jobs = []
    for count in args.counts:
        for layout in args.layouts:
            for state in args.states:
                views = ("both",) if state == "settled" and set(cameras) == {"facing", "away"} else cameras
                for camera in views:
                    for trial in range(args.repeats if state == "active" else 1):
                        name = f"{args.mode}-{count}-{layout}-{state}-{camera}"
                        if state == "active" and args.repeats > 1:
                            name += f"-repeat-{trial + 1}"
                        executable = capture_binary if (args.mode, count, layout, state) == ("native", 500, "dense", "settled") else binary
                        command = [str(executable), "--benchmark", "--count", str(count), "--layout", layout,
                            "--state", state, "--camera", camera, "--mode", args.mode, "--seed", str(args.seed),
                            "--warmup-seconds", str(args.warmup_seconds), "--sample-seconds", str(args.sample_seconds)]
                        if state == "settled" and args.settle_steps:
                            command += ["--settle-steps", str(args.settle_steps)]
                        if state == "settled" and args.repeats != 1:
                            command += ["--repeats", str(args.repeats)]
                        if state == "settled" and layout == "dense" and count in (500, 1000) and args.solver_baseline_samples:
                            command += ["--baseline-samples", str(args.solver_baseline_samples)]
                        jobs.append({"name": name, "command": command, "state": state, "trial": trial,
                            "expected_runs": args.repeats * (2 if camera == "both" else 1) if state == "settled" else 1})
    for offset in range(0, len(jobs), args.prepare_workers):
        batch = jobs[offset:offset + args.prepare_workers]
        if args.prepare_workers > 1:
            outcomes = prepared_batch(batch, args.output, args.timeout)
        else:
            job = batch[0]
            print(f"Running {job['name']}", file=sys.stderr, flush=True)
            outcomes = {}
            try:
                with (args.output / (job["name"] + ".stdout")).open("w") as stdout, (args.output / (job["name"] + ".stderr")).open("w") as stderr:
                    completed = subprocess.run(job["command"], cwd=root, stdout=stdout, stderr=stderr, timeout=args.timeout, check=False)
                outcomes[job["name"]] = None if completed.returncode == 0 else f"benchmark exited {completed.returncode}"
            except (OSError, subprocess.TimeoutExpired) as exc:
                outcomes[job["name"]] = str(exc)
        for job in batch:
            name = job["name"]
            try:
                if outcomes[name]:
                    raise ValueError(outcomes[name])
                output = json.loads((args.output / (name + ".stdout")).read_text())
                runs = output["runs"] if output.get("schema") == 2 else [output]
                if len(runs) != job["expected_runs"]:
                    raise ValueError("incomplete repeat windows")
                cases = []
                for index, result in enumerate(runs):
                    case = summarize(result)
                    if job["state"] == "settled" and args.settle_steps and not case["sleeping_observed"]:
                        raise ValueError("prepared settled population woke during the measured window")
                    case["config"]["repeat"] = index % args.repeats + 1 if job["state"] == "settled" else job["trial"] + 1
                    case["raw_file"] = name + ".stdout"
                    cases.append(case)
                summary["cases"].extend(cases)
            except (OSError, ValueError, KeyError) as exc:
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
