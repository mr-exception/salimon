"""Per-run evidence and optional OS screenshot capture (standard library only)."""

import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


def capture_command():
    """Return an argv template; no shell or implicit screenshot dependencies."""
    if sys.platform == "darwin" and shutil.which("screencapture"):
        return ["screencapture", "-x", "{path}"]
    if sys.platform.startswith("linux") and os.environ.get("DISPLAY") and shutil.which("import"):
        return ["import", "-window", "root", "{path}"]
    if sys.platform == "win32" and shutil.which("powershell"):
        return ["powershell", "-NoProfile", "-NonInteractive", "-File",
                str(Path(__file__).with_name("capture_windows.ps1")), "-OutputPath", "{path}"]
    return None


class Artifacts:
    def __init__(self, scenario, root, screenshot_command=None):
        root = Path(root).resolve()
        root.mkdir(parents=True, exist_ok=True)
        # mkdtemp avoids collisions between concurrent/repeated runs.
        self.directory = Path(tempfile.mkdtemp(prefix="run-", dir=root))
        self.command = capture_command() if screenshot_command is None else screenshot_command
        self.logs = {}
        self.write_json("scenario.json", scenario)

    def write_json(self, name, value):
        path = self.directory / name
        temporary = path.with_suffix(path.suffix + ".tmp")
        temporary.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n", encoding="utf-8")
        temporary.replace(path)
        return str(path)

    def open_logs(self):
        for name in ("stdout.log", "stderr.log", "protocol.jsonl"):
            self.logs[name] = (self.directory / name).open("w", encoding="utf-8")

    def log(self, name, value):
        stream = self.logs.get(name)
        if stream:
            stream.write(value + "\n")
            stream.flush()

    def close(self):
        for stream in self.logs.values():
            stream.close()
        self.logs.clear()

    def screenshot(self, name, timeout=5):
        path = self.directory / f"{name}.png"
        if not self.command:
            return {"status": "unavailable", "reason": "no screenshot command/display available"}
        argv = [part.replace("{path}", str(path)) for part in self.command]
        try:
            completed = subprocess.run(argv, capture_output=True, text=True, timeout=timeout)
            # Preserve helper diagnostics, including on successful captures.
            (self.directory / f"{name}.capture.log").write_text(
                completed.stdout + completed.stderr, encoding="utf-8")
            if completed.returncode:
                raise OSError(f"capture command exited {completed.returncode}: {completed.stderr[-1000:]}")
            with path.open("rb") as image:
                signature = image.read(8)
            if signature != b"\x89PNG\r\n\x1a\n":
                raise OSError("capture command did not produce a PNG")
            return {"status": "captured", "path": str(path)}
        except (OSError, subprocess.SubprocessError) as exc:
            return {"status": "failed", "reason": str(exc)}
