"""Probe killed native resume workers and real-clock restart without clock replay."""
from contextlib import contextmanager
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import time

from salimon_test import Game
from e2e_artifacts import Artifacts

ROOT = Path(__file__).resolve().parents[1]


def slots(directory):
    return {p.name: hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(directory.glob('checkpoint-*.sav'))}


def newest(directory):
    candidates = []
    for path in directory.glob('checkpoint-*.sav'):
        raw = path.read_bytes()
        candidates.append((int.from_bytes(raw[8:16], 'little'), json.loads(raw[28:])))
    return max(candidates, key=lambda pair: pair[0])


def motion_ns(source):
    epoch = source['state']['motion']['epoch']
    assert epoch['scale'] == 'tt'
    return epoch['day'] * 86_400_000_000_000 + epoch['nanos']


def wait(game, predicate):
    while True:
        state = game.request({'op': 'inspect'})
        if predicate(state):
            return state
        game.request({'op': 'step', 'frames': 1})



@contextmanager
def recorded_game(binary, setup, deadline, output, label):
    artifacts = Artifacts({'phase': label, 'setup': setup}, output / label, ['native'])
    artifacts.open_logs()
    try:
        with Game([str(binary)], setup, deadline, artifacts, native_input=False) as game:
            yield game
    finally:
        artifacts.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    binary = args.binary.resolve()
    report = {'status': 'failed', 'platform': platform.platform(),
              'source_revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
              'clock': 'actual local clock on restart; only aborted attempt requests controlled +60 s',
              'checks': [], 'restarts': []}
    original_env = os.environ.pop('SALIMON_E2E_LAUNCH_DIR', None)
    setup = {'scenario': 'canonical-journey', 'seed': 0, 'step_ms': 16}
    try:
        with recorded_game(binary, {**setup, 'scenario': 'canonical-flight'}, time.monotonic() + 180, output, 'interruption') as game:
            wait(game, lambda s: s['terrain']['presentation_ready'])
            game.request({'op': 'landing'})
            game.request({'op': 'step', 'frames': 375})
            assert game.request({'op': 'inspect'})['ship']['flight_state'] == 'Flying'
            game.request({'op': 'offline_launch', 'seconds': 60, 'hold': True})
            state = wait(game, lambda s: s['launch']['progress']['stage'] == 'Propagating')
            assert not state['launch']['ready'] and state['paused']
            assert 0 < state['launch']['percentage'] < 100
            directory = Path(state['launch']['fixture_directory'])
            before = slots(directory)
            generation, source = newest(directory)
            report['interrupted_progress'] = state['launch']
            report['source_before'] = {'generation': generation, 'saved_at': source['saved_at_unix_seconds'],
                                       'motion': source['state']['motion'], 'slots': before}
            game.process.kill()  # SIGKILL: no Drop/cooperative cancellation or final autosave.
            game.process.wait(timeout=10)
            assert slots(directory) == before
            assert not list(directory.glob('*.tmp'))
            report['checks'].append('SIGKILL before publication preserves source slot bytes; no public temporary slot')
        os.environ['SALIMON_E2E_LAUNCH_DIR'] = str(directory)
        for attempt in range(2):
            started = time.monotonic()
            with recorded_game(binary, setup, time.monotonic() + 180, output, f'restart-{attempt+1}') as game:
                state = wait(game, lambda s: s.get('launch') and s['launch']['ready'])
                assert state['paused'] and state['launch']['source_ready']
                next_generation, recovered = newest(directory)
                elapsed = recovered['saved_at_unix_seconds'] - source['saved_at_unix_seconds']
                physical = motion_ns(recovered) - motion_ns(source)
                assert elapsed >= 0 and physical == elapsed * 1_000_000_000, (elapsed, physical)
                assert next_generation == generation + 1, (next_generation, generation)
                assert len(slots(directory)) <= 2 and not list(directory.glob('*.tmp'))
                rss = [line for line in Path(f'/proc/{game.process.pid}/status').read_text().splitlines()
                       if line.startswith(('VmRSS:', 'VmHWM:'))]
                report['restarts'].append({'attempt': attempt + 1, 'seconds_to_region_ready': time.monotonic() - started,
                                           'actual_clock_elapsed_seconds': elapsed, 'motion_elapsed_nanos': physical,
                                           'generation': next_generation, 'slots': slots(directory), 'process_memory': rss,
                                           'launch': state['launch'], 'visual_context': state['visual_context'],
                                           'performance': state['performance']})
                game.process.kill()
                game.process.wait(timeout=10)
                source, generation = recovered, next_generation
            # A measured real absence, never an assertion that sleeping advanced a source.
            time.sleep(1.1)
        report['checks'].append('both actual-clock restarts credit exactly their checkpoint delta once; aborted +60 s is never published')
        report['status'] = 'passed'
    except Exception as error:
        report['error'] = f'{type(error).__name__}: {error}'
    finally:
        if original_env is None:
            os.environ.pop('SALIMON_E2E_LAUNCH_DIR', None)
        else:
            os.environ['SALIMON_E2E_LAUNCH_DIR'] = original_env
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'status': report['status'], 'report': str(output / 'result.json'), 'error': report.get('error')}))
    return report['status'] != 'passed'


if __name__ == '__main__':
    raise SystemExit(main())
