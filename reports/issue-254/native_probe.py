"""Opt-in local macOS foreground/loss probe; uses the reviewed #198 PID input helper."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', type=Path, required=True)
parser.add_argument('--helper', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--lose-focus', action='store_true')
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, SALIMON_REFERENCE_SIZE='1', RUST_LOG='info')
command = [str(args.binary.resolve()), '--benchmark', '--count', '0', '--layout', 'onboard',
           '--state', 'settled', '--camera', 'facing', '--warmup-seconds', '2',
           '--sample-seconds', '8', '--minimum-samples', '100']
focus = []
with (args.output/'raw.json').open('w') as out, (args.output/'stderr.log').open('w') as err:
    process = subprocess.Popen(command, env=env, stdout=out, stderr=err)
    start = time.monotonic()
    focused_at = None
    lost = False
    try:
        while process.poll() is None:
            if time.monotonic()-start > 45:
                raise TimeoutError('native admission/collection deadline exceeded')
            if focused_at is None:
                result = subprocess.run([str(args.helper.resolve()), str(process.pid), 'focus'],
                                        text=True, capture_output=True, timeout=5)
                focus.append(dict(returncode=result.returncode, stdout=result.stdout, stderr=result.stderr))
                if result.returncode == 0:
                    focused_at = time.monotonic()
            if args.lose_focus and focused_at and not lost and time.monotonic()-focused_at > 5:
                result = subprocess.run(['osascript', '-e', 'tell application "Finder" to activate'],
                                        text=True, capture_output=True, timeout=5)
                focus.append(dict(action='activate Finder', returncode=result.returncode,
                                  stdout=result.stdout, stderr=result.stderr))
                result.check_returncode()
                lost = True
            time.sleep(.1)
        result = dict(command=command, pid=process.pid, returncode=process.returncode,
                      deliberately_lost_focus=lost, focus=focus)
        (args.output/'probe.json').write_text(json.dumps(result, indent=2)+'\n')
        if args.lose_focus:
            assert lost and process.returncode != 0
            assert 'benchmark native readiness lost' in (args.output/'stderr.log').read_text()
        else:
            assert process.returncode == 0
            raw = json.loads((args.output/'raw.json').read_text())
            assert raw['presentation']['foreground_visible'] is True
            assert len(raw['samples']) >= 100
        print(json.dumps(result))
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()
