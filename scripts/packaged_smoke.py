#!/usr/bin/env python3
"""Launch a packaged game normally and verify rendering and OS keyboard input."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

from e2e_artifacts import Artifacts

ROOT = Path(__file__).resolve().parent.parent


def command(argv, timeout=5):
    result = subprocess.run(argv, capture_output=True, text=True, timeout=timeout)
    if result.returncode:
        raise RuntimeError(f"Helper exited {result.returncode}: {result.stderr[-2000:]}")
    return result.stdout.strip()


def input_key(pid, key):
    """Focus only the launched process; deliver native OS input, never E2E commands."""
    if sys.platform.startswith("linux"):
        if not os.environ.get("DISPLAY") or not shutil.which("xdotool"):
            raise RuntimeError("X11 DISPLAY and xdotool required (Wayland unsupported)")
        windows = command(["xdotool", "search", "--onlyvisible", "--pid", str(pid)]).splitlines()
        if not windows:
            raise RuntimeError("No visible game window for launched PID")
        window = windows[-1]
        command(["xdotool", "windowfocus", "--sync", window])
        command(["xdotool", "key", "--clearmodifiers", key])
    elif sys.platform == "darwin":
        code = 120
        command(["osascript", "-e", f'''tell application "System Events"
set gameProcess to first application process whose unix id is {pid}
set frontmost of gameProcess to true
key code {code}
end tell'''])
    elif sys.platform == "win32":
        command(["powershell", "-NoProfile", "-NonInteractive", "-File",
                 str(Path(__file__).with_name("input_windows.ps1")),
                 "-GamePid", str(pid), "-Key", key])
    else:
        raise RuntimeError(f"Unsupported OS input platform: {sys.platform}")


def wait_log(process, log, marker, deadline, offset=0):
    while time.monotonic() < deadline:
        if marker in log.read_text(encoding="utf-8", errors="replace")[offset:]:
            return
        if process.poll() is not None:
            raise RuntimeError(f"Game exited {process.returncode} before {marker!r}")
        time.sleep(0.05)
    raise TimeoutError(f"Timed out waiting for {marker!r}")


def run(binary, root, timeout=60):
    binary = Path(binary).resolve()
    evidence = Artifacts({"binary": str(binary), "mode": "packaged-black-box"}, root)
    report = {"schema_version": 1, "binary": str(binary), "platform": sys.platform,
              "artifacts": str(evidence.directory), "checks": [], "success": False}
    started = time.monotonic()
    deadline = started + timeout
    process = None
    stderr = evidence.directory / "stderr.log"
    try:
        if not binary.is_file():
            raise RuntimeError(f"Executable does not exist: {binary}")
        # An empty, unrelated cwd proves the package needs no source-relative assets.
        with tempfile.TemporaryDirectory(prefix="salimon-smoke-cwd-") as cwd, \
                (evidence.directory / "stdout.log").open("w") as out, stderr.open("w") as err:
            env = os.environ.copy()
            env["RUST_LOG"] = "info,salimon_client=debug"
            process = subprocess.Popen([str(binary)], cwd=cwd, env=env,
                                       stdin=subprocess.DEVNULL, stdout=out, stderr=err)
            report["pid"] = process.pid
            for marker in ("native window created:", "renderer ready:",
                           "render attempt 1 completed with Presented"):
                wait_log(process, stderr, marker, deadline)
                report["checks"].append(marker)
            report["startup_screenshot"] = evidence.screenshot("startup")
            if report["startup_screenshot"]["status"] != "captured":
                raise RuntimeError(f"Startup capture failed: {report['startup_screenshot']}")
            for mode in ("PrecisionTour", "Gameplay"):
                offset = len(stderr.read_text(encoding="utf-8", errors="replace"))
                input_key(process.pid, "F2")
                wait_log(process, stderr, f"view mode changed to {mode}", deadline, offset)
                report["checks"].append(f"OS F2 switched to {mode}")
            report["gameplay_screenshot"] = evidence.screenshot("gameplay")
            if report["gameplay_screenshot"]["status"] != "captured":
                raise RuntimeError(f"Gameplay capture failed: {report['gameplay_screenshot']}")
            if process.poll() is not None:
                raise RuntimeError(f"Game exited unexpectedly with {process.returncode}")
            report["checks"].append("game remains playable after OS input")
            report["success"] = True
    except (OSError, RuntimeError, subprocess.SubprocessError, TimeoutError) as error:
        report["error"] = str(error)
        report["failure_screenshot"] = evidence.screenshot("failure")
    finally:
        if process is not None:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=2)
            report["process_exit_code"] = process.returncode
        report["duration_ms"] = round((time.monotonic() - started) * 1000)
        evidence.write_json("result.json", report)
    return report


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--artifacts", type=Path, default=ROOT / "artifacts" / "packaged-smoke")
    parser.add_argument("--timeout", type=float, default=60)
    args = parser.parse_args(argv)
    if not 0 < args.timeout <= 600:
        parser.error("--timeout must be greater than zero and at most 600 seconds")
    report = run(args.binary, args.artifacts, args.timeout)
    print(json.dumps(report))
    return 0 if report["success"] else 1


if __name__ == "__main__":
    sys.exit(main())
