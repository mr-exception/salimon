"""Measure normal canonical play using native OS Continue and per-frame game diagnostics."""
import argparse
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import time

from packaged_smoke import command, input_key, wait_log
from planetary_environment import digest, distribution

FRAME = re.compile(r'frame=(\d+) interval_ms=([\d.]+) present_wall_ms=([\d.]+) '
                   r'cpu_update_ms=([\d.]+) cpu_render_ms=([\d.]+) gpu_time=(.*?) '
                   r'ticks=(\d+) backlog_ms=([\d.]+) focused=(true|false) occluded=(true|false)')


def memory_sample(pid, system=None):
    """Process resident memory only; neither API establishes GPU allocation."""
    system = system or sys.platform
    if system == 'darwin':
        rss = int(command(['ps', '-o', 'rss=', '-p', str(pid)]))
        if rss <= 0:
            raise ValueError('invalid macOS resident memory sample')
        return {'rss_kib': rss, 'process_high_water_kib': None}
    if system.startswith('linux'):
        fields = dict(line.split(':', 1) for line in Path(f'/proc/{pid}/status').read_text().splitlines()
                      if ':' in line)
        if not {'VmRSS', 'VmHWM'} <= fields.keys():
            raise ValueError('Linux process RSS/high water unavailable')
        return {'rss_kib': int(fields['VmRSS'].split()[0]),
                'process_high_water_kib': int(fields['VmHWM'].split()[0])}
    raise RuntimeError(f'unsupported benchmark platform: {system}')


def window_context(pid, system=None, capture_path=None):
    system = system or sys.platform
    if system == 'darwin':
        argv = ['swift', str(Path(__file__).with_name('macos_window.swift')), str(pid)]
        if capture_path is not None:
            argv.append(str(capture_path))
        return json.loads(command(argv, timeout=30))
    if system.startswith('linux'):
        window = command(['xdotool', 'search', '--onlyvisible', '--pid', str(pid)]).splitlines()[-1]
        return {'window_geometry': command(['xdotool', 'getwindowgeometry', '--shell', window]),
                'display_geometry': command(['xdotool', 'getdisplaygeometry'])}
    raise RuntimeError(f'unsupported benchmark platform: {system}')


def frame_samples(text):
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
    return raw


def validate_frames(raw):
    if len(raw) < 100 or any(not r['focused'] or r['occluded'] for r in raw):
        raise ValueError(f'need 100 visible foreground frames, got {len(raw)}')
    if any(b['frame'] != a['frame'] + 1 for a, b in zip(raw, raw[1:])):
        raise ValueError('missing per-frame diagnostic samples')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--sample-seconds', type=int, choices=range(10, 61), default=30)
    parser.add_argument('--profile', choices=('debug', 'release'), required=True,
                        help='record the actual binary build profile; not inferred from its path')
    args = parser.parse_args()
    output, binary = args.output.resolve(), args.binary.resolve()
    output.mkdir(parents=True, exist_ok=False)
    log = output / 'stderr.log'
    report = {'status': 'failed', 'platform': platform.platform(),
              'binary_sha256': digest(binary), 'command': [str(binary)],
              'profile': args.profile, 'workload': 'normal canonical new journey; no E2E; no optional pack',
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
            report['window_context'] = window_context(process.pid, capture_path=output / 'gameplay.png')
            # A native capture helper may briefly take activation. Restore the
            # launched process before warmup; sampled focus loss still fails.
            input_key(process.pid, 'Return')
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
                memory = memory_sample(process.pid)
                report['memory_samples'].append({'seconds': time.monotonic() - sampling, 'kib': memory})
                time.sleep(.25)
            report['sample_seconds'] = time.monotonic() - sampling
            end = log.stat().st_size
            text = log.read_bytes()[offset:end].decode(errors='replace')
            raw = frame_samples(text)
            report['raw_frames'] = raw
            validate_frames(raw)
            report['metrics'] = {key: distribution([row[key] for row in raw])
                                 for key in ('interval_ms', 'wall_ms', 'cpu_update_ms', 'cpu_render_ms', 'backlog_ms')}
            report['metrics']['gpu_ms'] = distribution([row['gpu_ms'] for row in raw if row['gpu_ms'] is not None])
            report['submitted_frames_per_second'] = len(raw) / report['sample_seconds']
            report['submitted_interval_60fps_comparison'] = {
                'p95_limit_ms': 1000 / 60,
                'passed': report['metrics']['interval_ms']['p95'] <= 1000 / 60,
                'owner_budget_approval': False, 'physical_scanout_verified': False}
            report['renderer'] = next(line for line in log.read_text().splitlines() if 'renderer ready:' in line)
            report['native_window_events'] = [line for line in log.read_text().splitlines()
                                              if 'native window created:' in line or 'native drawable resized:' in line]
            report['status'] = 'passed'
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError, TimeoutError, StopIteration) as error:
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
                      'submitted_interval_60fps_comparison': report.get('submitted_interval_60fps_comparison'),
                      'error': report.get('error')}))
    return report['status'] != 'passed'


if __name__ == '__main__':
    raise SystemExit(main())
