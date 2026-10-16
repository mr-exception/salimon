"""Focus each bundled benchmark window before its ten-second warmup ends."""
import subprocess, time, sys, json
from pathlib import Path
command = ['python3', 'scripts/benchmark_fragments.py', '--binary', 'artifacts/SalimonNative.app/Contents/MacOS/salimon-client', '--machine-label', 'apple-m1-imac-16gb-macos26.2', '--counts', '500', '--layouts', 'onboard', '--states', 'settled', '--cameras', 'facing', '--repeats', '3', '--minimum-samples', '100', '--settle-steps', '2400', '--timeout', '300', '--baseline', 'artifacts/native-199-previous/summary.json', '--output', 'artifacts/native-199-visible-final']
with Path('artifacts/native-queue/visible-final-runner.stdout').open('w') as out:
    proc = subprocess.Popen(command, stdout=out, stderr=subprocess.STDOUT)
    seen = set()
    records = []
    while proc.poll() is None:
        rows = subprocess.check_output(['ps', '-axo', 'pid=,ppid=,command='], text=True)
        for line in rows.splitlines():
            parts = line.strip().split(None, 2)
            if len(parts) == 3 and int(parts[1]) == proc.pid and ('salimon-client --benchmark' in parts[2]):
                pid = int(parts[0])
                if pid not in seen:
                    r = subprocess.run(['artifacts/native-input', str(pid), 'focus'], capture_output=True, text=True, timeout=5)
                    if r.returncode == 0:
                        seen.add(pid)
                    records.append({'pid': pid, 'command': parts[2], 'stdout': r.stdout, 'stderr': r.stderr, 'returncode': r.returncode, 'monotonic': time.monotonic()})
                    Path('artifacts/native-queue/benchmark-final-focus.json').write_text(json.dumps(records, indent=2) + '\n')
        time.sleep(0.5)
    pids = {r['pid'] for r in records}
    ready = bool(pids) and all((any((r['pid'] == pid and r['returncode'] == 0 for r in records)) for pid in pids))
    print('benchmark exit', proc.returncode, 'foreground confirmed', ready, flush=True)
    sys.exit(proc.returncode if ready else 1)
