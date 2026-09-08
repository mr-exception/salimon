#!/usr/bin/env python3
"""Capture the native desktop after a short presentation settling interval.

E2E replies describe simulation state before asynchronous GPU presentation.
This helper reduces stale-frame captures on software Vulkan; it is not a GPU
fence or a graphics correctness assertion. The runner still enforces its capture
and scenario deadlines. Use only on a dedicated, unobscured test desktop.
"""

import subprocess
import sys
import time

from e2e_artifacts import capture_command


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: capture_settled.py OUTPUT.png")
    command = capture_command()
    if command is None:
        raise SystemExit("no native desktop screenshot helper available")
    time.sleep(0.5)
    subprocess.run([arg.replace("{path}", sys.argv[1]) for arg in command], check=True)


if __name__ == "__main__":
    main()
