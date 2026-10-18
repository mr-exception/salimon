#!/usr/bin/env python3
"""Compare fragment CPU costs on identical current-sphere fixtures, never native FPS.

Builds isolated worktrees; only the test-only attribution module is copied into
older revisions. Production code and damping remain those of each supplied ref.
Use a baseline containing #304 to isolate indexing/scratch from damping changes.
"""
from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def run(command, cwd, env=None, stdout=None):
    subprocess.run(command, cwd=cwd, env=env, stdout=stdout, check=True, timeout=1200)


def measure(ref, label, fixture, output, target, allocations):
    directory = Path(tempfile.mkdtemp(prefix=f"salimon-fragments-{label}-"))
    tree = directory / "source"
    run(["git", "worktree", "add", "--detach", str(tree), ref], ROOT)
    destination = output / label
    destination.mkdir()
    try:
        (tree / "client/runtime/src/fragment_costs.rs").write_bytes(fixture)
        main = tree / "client/runtime/src/main.rs"
        content = main.read_text()
        if "mod fragment_costs;" not in content:
            main.write_text(content.replace("mod fragment_physics;", "#[cfg(test)]\nmod fragment_costs;\nmod fragment_physics;", 1))
        env = dict(os.environ, CARGO_TARGET_DIR=str(target),
                   SALIMON_FRAGMENT_COST_REPORT=str(destination / "timings.json"))
        if label == "before":
            run(["cargo", "test", "-p", "salimon-physics", "--locked"], tree, env)
        run(["cargo", "test", "--release", "-p", "salimon-client", "--locked",
             "fragment_update_costs", "--", "--ignored", "--nocapture"], tree, env)
        with (destination / "timings.json").open("rb") as source:
            with gzip.open(destination / "timings.json.gz", "wb") as compressed:
                shutil.copyfileobj(source, compressed)
        (destination / "timings.json").unlink()
        if allocations:
            with (destination / "executables.jsonl").open("w") as stream:
                run(["cargo", "test", "--release", "-p", "salimon-client", "--locked",
                     "--no-run", "--message-format=json"], tree, env, stream)
            messages = [json.loads(line) for line in (destination / "executables.jsonl").read_text().splitlines()]
            binary = next(item["executable"] for item in messages
                          if item.get("reason") == "compiler-artifact"
                          and item.get("target", {}).get("name") == "salimon-client"
                          and item.get("profile", {}).get("test") and item.get("executable"))
            env.update(SALIMON_FRAGMENT_COST_QUICK="1",
                       SALIMON_FRAGMENT_COST_REPORT=str(destination / "allocation-fixture.json"))
            run(["heaptrack", "-o", str(destination / "allocations.gz"), binary,
                 "fragment_update_costs", "--ignored", "--nocapture"], tree, env)
            with (destination / "allocations.txt").open("w") as stream:
                run(["heaptrack_print", "-f", str(destination / "allocations.gz")], tree, env, stream)
    finally:
        run(["git", "worktree", "remove", "--force", str(tree)], ROOT)
        shutil.rmtree(directory)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--before", required=True)
    parser.add_argument("--after", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--heaptrack", action="store_true")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    if args.heaptrack and (not shutil.which("heaptrack") or not shutil.which("heaptrack_print")):
        parser.error("--heaptrack requires heaptrack and heaptrack_print")
    fixture = (ROOT / "client/runtime/src/fragment_costs.rs").read_bytes()
    metadata = {"fixture_sha256": hashlib.sha256(fixture).hexdigest(),
                "machine": platform.platform(), "processor": platform.processor(),
                "rustc": subprocess.check_output(["rustc", "-Vv"], text=True),
                "refs": {key: subprocess.check_output(["git", "rev-parse", value], cwd=ROOT, text=True).strip()
                         for key, value in (("before", args.before), ("after", args.after))},
                "heaptrack": args.heaptrack,
                "allocation_scope": "whole quick test process, including identical fixture/setup/warmup/replay; not per-tick counts",
                "gpu_fps": None}
    (output / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    target = ROOT / "artifacts/fragment-attribution-target"
    for ref, label in ((args.before, "before"), (args.after, "after")):
        measure(ref, label, fixture, output, target, args.heaptrack)


if __name__ == "__main__":
    main()
