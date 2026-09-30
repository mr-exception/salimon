#!/usr/bin/env python3
"""Build and stage a host-native Salimon executable using only Python's stdlib."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parent.parent


class BuildError(Exception):
    pass


def command_output(argv):
    try:
        return subprocess.check_output(argv, cwd=ROOT, text=True, stderr=subprocess.STDOUT).strip()
    except (OSError, subprocess.CalledProcessError) as error:
        detail = getattr(error, "output", str(error))
        raise BuildError(f"{' '.join(argv)} failed: {detail}") from error


def prerequisites():
    system = platform.system()
    if system not in ("Darwin", "Windows", "Linux"):
        raise BuildError(f"Unsupported host: {system}. Use macOS, Windows, or Debian Linux.")
    if system == "Linux" and not Path("/etc/debian_version").exists():
        raise BuildError("Linux builds currently support Debian-based distributions only.")
    for tool in ("cargo", "rustc"):
        if not shutil.which(tool):
            raise BuildError(f"Missing {tool}; install Rust via rustup and reopen your terminal.")
    rust = command_output(["rustc", "--version", "--verbose"])
    release = re.search(r"^release: (\d+)\.(\d+)\.(\d+)", rust, re.MULTILINE)
    host = re.search(r"^host: (\S+)", rust, re.MULTILINE)
    if not release or not host:
        raise BuildError("Cannot determine the rustc version/host from rustc --version --verbose.")
    if tuple(map(int, release.groups())) < (1, 89, 0):
        raise BuildError("Rust 1.89 or newer is required; run rustup update stable.")
    if system == "Darwin":
        command_output(["xcode-select", "-p"])
    elif system == "Linux":
        if not shutil.which("cc"):
            raise BuildError("Missing C linker; install Debian package build-essential.")
    elif not host.group(1).endswith("-pc-windows-msvc"):
        raise BuildError("Windows builds require the Rust MSVC toolchain and Visual Studio C++ Build Tools.")
    # Passing --target explicitly prevents a local Cargo build.target setting
    # from silently turning this native build into a cross-compiled artifact.
    return system, host.group(1), rust.splitlines()[0]


def compile_game(host, profile):
    argv = ["cargo", "build", "--locked", "-p", "salimon-client", "--target", host,
            "--message-format", "json-render-diagnostics"]
    if profile == "release":
        argv.append("--release")
    print(f"Building: {' '.join(argv)}", file=sys.stderr)
    executable = None
    with subprocess.Popen(argv, cwd=ROOT, stdout=subprocess.PIPE, text=True) as process:
        for line in process.stdout:
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                print(line.rstrip(), file=sys.stderr)
                continue
            if message.get("reason") == "compiler-message":
                rendered = message.get("message", {}).get("rendered")
                if rendered:
                    print(rendered, end="", file=sys.stderr)
            if (message.get("reason") == "compiler-artifact"
                    and message.get("target", {}).get("name") == "salimon-client"
                    and "bin" in message.get("target", {}).get("kind", [])
                    and message.get("executable")):
                executable = Path(message["executable"])
        code = process.wait()
    if code:
        raise BuildError(f"Cargo build failed (exit {code}); see compiler/linker diagnostics above.")
    if executable is None or not executable.is_file():
        raise BuildError("Cargo did not report an existing salimon-client executable.")
    return executable


def stage(executable, destination, host, profile, rust):
    destination.mkdir(parents=True, exist_ok=True)
    name = "salimon-client.exe" if host.endswith("-pc-windows-msvc") else "salimon-client"
    # Stage each file beside its destination before replacement so a partial
    # executable copy never masquerades as a successful build.
    with tempfile.TemporaryDirectory(prefix=".stage-", dir=destination) as temporary:
        staged = Path(temporary) / name
        shutil.copy2(executable, staged)
        info = {"schema_version": 1, "host": host, "profile": profile, "rust": rust,
                "executable": name, "sha256": hashlib.sha256(staged.read_bytes()).hexdigest()}
        (Path(temporary) / "build-info.json").write_text(json.dumps(info, indent=2) + "\n", encoding="utf-8")
        (Path(temporary) / "README.txt").write_text(
            f"Salimon native {profile} build ({host})\n\n"
            f"Run {name} from any directory. Game assets are embedded in the executable.\n"
            "A graphical desktop and compatible GPU/driver are required.\n"
            "Linux: install libxkbcommon0, libxkbcommon-x11-0, libwayland-client0, libx11-6, libxi6,\n"
            "libxcursor1, libxrandr2, libvulkan1 and a compatible Vulkan driver.\n"
            "Windows: install the Visual C++ 2015-2022 runtime if missing.\n"
            "macOS: this executable is unsigned; allow it through local security settings.\n"
            "Host-native only; signing, installers and auto-updates are out of scope.\n",
            encoding="utf-8")
        for filename in (name, "build-info.json", "README.txt"):
            os.replace(Path(temporary) / filename, destination / filename)
    return destination / name


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("debug", "release"), default="debug")
    parser.add_argument("--output", type=Path, help="Exact staging directory (default: artifacts/build/HOST/PROFILE)")
    args = parser.parse_args(argv)
    try:
        _, host, rust = prerequisites()
        executable = compile_game(host, args.profile)
        destination = (args.output or ROOT / "artifacts" / "build" / host / args.profile).resolve()
        result = stage(executable, destination, host, args.profile, rust)
    except (BuildError, OSError) as error:
        print(f"Salimon build error: {error}", file=sys.stderr)
        return 1
    print(f"Built {result}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
