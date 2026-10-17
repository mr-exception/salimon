"""Opt-in macOS matrix wrapper: focus new benchmark PIDs; retain local OS records."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--helper', type=Path, required=True)
parser.add_argument('--records', type=Path, required=True)
parser.add_argument('command', nargs=argparse.REMAINDER)
args = parser.parse_args()
command = args.command[1:] if args.command[:1] == ['--'] else args.command
if not command:
    parser.error('provide benchmark_fragments.py command after --')
args.records.parent.mkdir(parents=True, exist_ok=True)
process = subprocess.Popen(command)
focused = set()
records = []
try:
    while process.poll() is None:
        rows = subprocess.check_output(['ps', '-axo', 'pid=,ppid=,command='], text=True, timeout=5)
        for row in rows.splitlines():
            parts = row.strip().split(None, 2)
            if len(parts) != 3 or int(parts[1]) != process.pid or 'salimon-client --benchmark' not in parts[2]:
                continue
            pid = int(parts[0])
            if pid in focused:
                continue
            result = subprocess.run([str(args.helper.resolve()), str(pid), 'focus'],
                                    capture_output=True, text=True, timeout=5)
            records.append(dict(pid=pid, monotonic=time.monotonic(), returncode=result.returncode,
                                stdout=result.stdout, stderr=result.stderr))
            args.records.write_text(json.dumps(records, indent=2)+'\n')
            if result.returncode == 0:
                focused.add(pid)
        time.sleep(.5)
    sys.exit(process.returncode)
finally:
    if process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
