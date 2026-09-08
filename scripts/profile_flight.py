#!/usr/bin/env python3
"""Record CPU flight/cargo distributions and enforce explicit regression limits."""
import argparse
import hashlib
import json
import platform
import subprocess
from pathlib import Path

WORKLOADS = {"mass-idle", "mass-crossing", "moving-powered", "moving-zero-g", "power-transition", "dense-contacts"}
# Portable CPU regression guardrails, not an FPS or hardware acceptance budget.
LIMITS = {
    "mass-idle": (4.0, 0.25, 2.0),
    "mass-crossing": (4.0, 0.5, 5.0),
    "moving-powered": (4.0, 5.0, 100.0),
    "moving-zero-g": (4.0, 5.0, 100.0),
    "power-transition": (4.0, 10.0, 200.0),
    "dense-contacts": (4.0, 20.0, 1000.0),
}


def rows(report):
    import math
    if report.get("schema") != 1 or report.get("kind") != "flight-cpu-profile" or report.get("fixture") not in ("deck", "stacked"):
        raise ValueError("unsupported flight profile schema")
    result = {}
    for row in report.get("rows", []):
        key = (row["count"], row["workload"])
        if key in result or key[0] not in (0, 500) or key[1] not in WORKLOADS:
            raise ValueError("duplicate or unknown profile row")
        metrics = row["cpu_update"]
        if metrics["samples"] < 20 or len(metrics["raw_ms"]) != metrics["samples"]:
            raise ValueError("profile requires at least 20 retained samples per row")
        if len(row["moving_objects"]) != metrics["samples"]:
            raise ValueError("profile motion observations missing")
        if any(not isinstance(v, (int, float)) or not math.isfinite(v) or v < 0 for v in metrics["raw_ms"]):
            raise ValueError("nonfinite/negative timing")
        ordered = sorted(metrics["raw_ms"])
        for field, fraction in (("p50_ms", .50), ("p95_ms", .95), ("p99_ms", .99), ("max_ms", 1)):
            if metrics[field] != ordered[math.ceil(len(ordered)*fraction)-1]:
                raise ValueError("summary does not match raw timing")
        expected = 0 if key[0] == 0 else 16
        if abs(row["cargo_mass_kg"] - expected) > 1e-6:
            raise ValueError("profile cargo is not all physically onboard")
        if row.get("hardware_fps_validated") is not False:
            raise ValueError("CPU profile must not claim hardware FPS")
        result[key] = row
    if set(result) != {(count, name) for count in (0, 500) for name in WORKLOADS}:
        raise ValueError("missing required workload/count row")
    return result


def compare(current, baseline=None):
    observed = rows(current)
    previous = rows(baseline) if baseline else None
    if baseline and current["fixture"] != baseline["fixture"]:
        raise ValueError("cannot compare different collision fixtures")
    violations = []
    for key, row in sorted(observed.items()):
        ratio, slack, absolute = LIMITS[key[1]]
        limit = absolute
        if previous:
            limit = min(limit, previous[key]["cpu_update"]["p95_ms"] * ratio + slack)
        value = row["cpu_update"]["p95_ms"]
        if value > limit:
            violations.append({"count": key[0], "workload": key[1], "p95_ms": value, "limit_ms": limit})
    return violations


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--samples", type=int, default=20, choices=range(20, 1001), metavar="20..1000")
    parser.add_argument("--stacked", action="store_true", help="expensive non-overlapping multilayer stress (not default CI)")
    args = parser.parse_args()
    try:
        completed = subprocess.run([str(args.binary.resolve()), "--flight-profile", "--samples", str(args.samples)] + (["--stacked"] if args.stacked else []), text=True, capture_output=True, timeout=300, check=True)
        report = json.loads(completed.stdout)
    except (subprocess.TimeoutExpired, subprocess.CalledProcessError, json.JSONDecodeError, OSError) as error:
        def captured(name):
            value = getattr(error, name, "") or ""
            return value.decode(errors="replace") if isinstance(value, bytes) else value
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps({"status": "failed", "error": str(error), "stdout": captured("stdout"), "stderr": captured("stderr")}, indent=2) + "\n")
        print(json.dumps({"status": "failed", "report": str(args.output), "error": str(error)}))
        return 1
    baseline = json.loads(args.baseline.read_text()) if args.baseline else None
    failures = compare(report, baseline)
    report["environment"] = {"platform": platform.platform(), "cpu": platform.processor() or platform.machine(), "binary_sha256": hashlib.sha256(args.binary.read_bytes()).hexdigest(), "source_revision": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(), "source_dirty": bool(subprocess.check_output(["git", "status", "--porcelain"], text=True).strip())}
    report["limits"] = {name: {"baseline_multiplier": ratio, "slack_ms": slack, "absolute_p95_ms": absolute} for name, (ratio, slack, absolute) in LIMITS.items()}
    report["baseline"] = str(args.baseline) if args.baseline else None
    report["violations"] = failures
    report["status"] = "failed" if failures else "passed"
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"status": report["status"], "report": str(args.output), "violations": failures}))
    return int(bool(failures))


if __name__ == "__main__":
    raise SystemExit(main())
