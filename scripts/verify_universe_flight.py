"""Reproduce #437 flight, collision, navigation and Linux OS-input evidence."""
import argparse
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import time

from planetary_environment import ROOT, digest, review_native
from scenario_suites import UNIVERSE_FLIGHT


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True, help='new isolated evidence directory')
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    binary = args.binary.resolve()
    report = {
        'status': 'failed', 'commands': [],
        'source_revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        'source_dirty': bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True).strip()),
        'source_sha256': {str(p.relative_to(ROOT)): digest(p)
                          for directory in ('client', 'scripts', 'scenarios')
                          for p in sorted((ROOT / directory).rglob('*'))
                          if p.is_file() and p.suffix in {'.rs', '.py', '.json', '.toml', '.wgsl'}},
        'environment': {'platform': platform.platform(), 'python': platform.python_version(),
                        'rustc': subprocess.check_output(['rustc', '--version'], text=True).strip(),
                        'display': os.environ.get('DISPLAY'), 'wgpu_backend': os.environ.get('WGPU_BACKEND'),
                        'vulkan_driver': os.environ.get('VK_DRIVER_FILES'),
                        'software_threads': os.environ.get('LP_NUM_THREADS')},
        'boundaries': {
            'protocol_routes': 'production session controls; deterministic scenario initial conditions',
            'os_input': 'normal launch and X11 F2 press/release, not flight protocol input',
            'orbital_tests': 'real-scale production APIs with declared ephemerides; not a playable orbit claim',
            'physical_devices': 'unverified #557', 'presentation': 'unverified #561',
            'hardware_M1_Metal_FPS': 'unverified #563', 'offline_trajectory': 'not certified; #432-#434'}}

    def run(name, argv, timeout):
        started = time.monotonic()
        path = output / (name + '.log')
        entry = {'name': name, 'argv': argv, 'log': str(path)}
        report['commands'].append(entry)
        try:
            with path.open('w') as log:
                result = subprocess.run(argv, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, timeout=timeout,
                                        env={**os.environ, 'SALIMON_SAVE_DIR': str(output / 'saves'),
                                             'SALIMON_SETTINGS_FILE': str(output / 'settings.json')})
            entry['exit_code'] = result.returncode
            if result.returncode:
                raise ValueError(f'{name} failed: {path}')
        finally:
            entry['elapsed_seconds'] = time.monotonic() - started

    try:
        report['binary_sha256'] = digest(binary)
        run('ship-contracts', ['cargo', 'test', '-p', 'salimon-ship', '--locked', '--', '--nocapture'], 900)
        run('session-contracts', ['cargo', 'test', '-p', 'salimon-client', '--locked'], 900)
        run('native-routes', [sys.executable, str(ROOT / 'scripts/salimon-test'), 'suite',
                             '--group', 'universe-flight', '--binary', str(binary),
                             '--artifacts', str(output / 'native'), '--screenshot-command', '["native"]', '--summary'], 3600)
        summary = json.loads((output / 'native-routes.log').read_text())
        report['routes'] = review_native(summary, UNIVERSE_FLIGHT)
        run('os-input', [sys.executable, str(ROOT / 'scripts/packaged_smoke.py'), '--binary', str(binary),
                         '--artifacts', str(output / 'os-input'), '--timeout', '180'], 210)
        smoke = json.loads((output / 'os-input.log').read_text())
        if not smoke['success'] or not all('OS F2 switched to ' + mode in smoke['checks']
                                           for mode in ('PrecisionTour', 'Gameplay')):
            raise ValueError('normal-launch OS input was not verified')
        report['os_input'] = smoke
        report['status'] = 'passed'
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        report['error'] = str(error)
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'status': report['status'], 'report': str(output / 'result.json'),
                      'error': report.get('error')}))
    return report['status'] != 'passed'


if __name__ == '__main__':
    sys.exit(main())
