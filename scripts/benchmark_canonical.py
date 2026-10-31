"""Measure normal canonical play using native OS Continue and per-frame game diagnostics."""
import argparse
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import time

from packaged_smoke import command, input_key, wait_log
from planetary_environment import digest, distribution

FRAME = re.compile(r'frame=(\d+) interval_ms=([\d.]+) present_wall_ms=([\d.]+) '
                   r'cpu_update_ms=([\d.]+) cpu_render_ms=([\d.]+) gpu_time=(.*?) '
                   r'ticks=(\d+) backlog_ms=([\d.]+) focused=(true|false) occluded=(true|false)')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--sample-seconds', type=int, choices=range(10, 61), default=30)
    args = parser.parse_args()
    output, binary = args.output.resolve(), args.binary.resolve()
    output.mkdir(parents=True, exist_ok=False)
    log = output / 'stderr.log'
    report = {'status': 'failed', 'platform': platform.platform(),
              'binary_sha256': digest(binary), 'command': [str(binary)],
              'profile': 'locked debug; normal canonical new journey; no E2E; no optional pack',
              'instrumentation': 'game per-frame debug diagnostics enabled; intervals capped at 250 ms by FrameClock',
              'hardware_fps_validated': False, 'memory_samples': []}
    process = None
    try:
        env = {**os.environ, 'RUST_LOG': 'info,salimon_client=debug,salimon_client::frame=trace',
               'SALIMON_SAVE_DIR': str(output / 'saves'), 'SALIMON_SETTINGS_FILE': str(output / 'settings.json')}
        env.pop('SALIMON_UNIVERSE_PACK', None)
        with log.open('w') as err, (output / 'stdout.log').open('w') as out:
            started = time.monotonic()
            deadline = started + 180
            process = subprocess.Popen([str(binary)], cwd=output, env=env, stdin=subprocess.DEVNULL,
                                       stdout=out, stderr=err)
            wait_log(process, log, 'render attempt 1 completed with Presented', deadline)
            while True:
                input_key(process.pid, 'Return')
                try:
                    wait_log(process, log, 'verified journey resumed', min(deadline, time.monotonic() + 1))
                    break
                except TimeoutError:
                    if time.monotonic() >= deadline:
                        raise
            report['seconds_to_verified_continue'] = time.monotonic() - started
            window = command(['xdotool', 'search', '--onlyvisible', '--pid', str(process.pid)]).splitlines()[-1]
            report['window_geometry'] = command(['xdotool', 'getwindowgeometry', '--shell', window])
            report['display_geometry'] = command(['xdotool', 'getdisplaygeometry'])
            # Warm the actual scene; no command pacing or frame-stepping during samples.
            warmup = time.monotonic() + 5
            while time.monotonic() < warmup:
                if process.poll() is not None:
                    raise RuntimeError('native process exited during warmup')
                time.sleep(.1)
            offset = log.stat().st_size
            sampling = time.monotonic()
            while time.monotonic() - sampling < args.sample_seconds:
                if process.poll() is not None:
                    raise RuntimeError('native process exited during sampling')
                fields = Path(f'/proc/{process.pid}/status').read_text().splitlines()
                memory = {key: int(next(line for line in fields if line.startswith(key + ':')).split()[1])
                          for key in ('VmRSS', 'VmHWM')}
                report['memory_samples'].append({'seconds': time.monotonic() - sampling, 'kib': memory})
                time.sleep(.25)
            report['sample_seconds'] = time.monotonic() - sampling
            end = log.stat().st_size
            text = log.read_bytes()[offset:end].decode(errors='replace')
            raw = []
            for match in FRAME.finditer(text):
                number, interval, wall, update, render, gpu, ticks, backlog, focus, occluded = match.groups()
                measured = re.fullmatch(r'Measured\(([\d.]+)(ns|µs|ms|s)\)', gpu)
                gpu_ms = (float(measured[1]) * {'ns': 1e-6, 'µs': 1e-3, 'ms': 1, 's': 1000}[measured[2]]
                          if measured else None)
                raw.append({'gpu_ms': gpu_ms, 'frame': int(number), 'interval_ms': float(interval), 'wall_ms': float(wall),
                            'cpu_update_ms': float(update), 'cpu_render_ms': float(render),
                            'gpu_diagnostic': gpu, 'ticks': int(ticks), 'backlog_ms': float(backlog),
                            'focused': focus == 'true', 'occluded': occluded == 'true'})
            if len(raw) < 100 or any(not r['focused'] or r['occluded'] for r in raw):
                raise ValueError(f'need 100 visible foreground frames, got {len(raw)}')
            if any(b['frame'] != a['frame'] + 1 for a, b in zip(raw, raw[1:])):
                raise ValueError('missing per-frame diagnostic samples')
            report['raw_frames'] = raw
            report['metrics'] = {key: distribution([row[key] for row in raw])
                                 for key in ('interval_ms', 'wall_ms', 'cpu_update_ms', 'cpu_render_ms', 'backlog_ms')}
            report['metrics']['gpu_ms'] = distribution([row['gpu_ms'] for row in raw if row['gpu_ms'] is not None])
            report['submitted_frames_per_second'] = len(raw) / report['sample_seconds']
            report['software_60fps_budget'] = {'p95_limit_ms': 1000 / 60,
                                              'passed': report['metrics']['interval_ms']['p95'] <= 1000 / 60}
            report['renderer'] = next(line for line in log.read_text().splitlines() if 'renderer ready:' in line)
            report['status'] = 'passed'
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        report['error'] = str(error)
    finally:
        if process is not None and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'status': report['status'], 'report': str(output / 'result.json'),
                      'software_60fps_budget': report.get('software_60fps_budget'), 'error': report.get('error')}))
    return report['status'] != 'passed'


if __name__ == '__main__':
    raise SystemExit(main())
