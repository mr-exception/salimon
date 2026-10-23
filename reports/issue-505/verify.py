"""Read every Archive record through real controls and capture actual wgpu output."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))
from e2e_artifacts import Artifacts
from salimon_test import Game, native_capture


def power(binary, output):
    setup = {'scenario': 'core-restart', 'seed': 505, 'step_ms': 96}
    art = Artifacts(setup, output, ['native']); art.open_logs()
    deadline = time.monotonic() + 300
    states = []
    try:
        with Game([str(binary.resolve())], setup, deadline, art, native_input=False) as g:
            g.request({'op': 'scout_reference', 'view': 'archive'})
            for phase in ['blackout', 'restored']:
                if phase == 'restored':
                    g.request({'op': 'interact'}); g.request({'op': 'step', 'frames': 1})
                for w, h in [(1280, 800), (1920, 1080)]:
                    g.request({'op': 'window', 'action': 'resize', 'width': w, 'height': h})
                    until = time.monotonic() + 30
                    while g.request({'op': 'inspect'})['visual_context']['viewport_pixels'] != [w, h]:
                        assert time.monotonic() < until
                    before = g.request({'op': 'inspect'})
                    assert before['ship']['power_available'] == (phase == 'restored')
                    assert native_capture(g, art, f'{w}-{phase}', deadline)['status'] == 'captured'
                    after = g.request({'op': 'inspect'})
                    assert before['archive'] == after['archive']
                    assert before['ship'] == after['ship']
                    states.append({'phase': phase, 'state': after})
        art.write_json('result.json', {'status': 'passed', 'captures': len(states)})
    except BaseException as exc:
        art.write_json('result.json', {'status': 'failed', 'error': str(exc)})
        raise
    finally:
        art.write_json('states.json', states); art.close()


def run(binary, output, scale):
    os.environ['WINIT_X11_SCALE_FACTOR'] = str(scale)
    setup = {'scenario': 'scout-archive', 'seed': 505, 'step_ms': 96}
    art = Artifacts(setup, output, ['native']); art.open_logs()
    deadline = time.monotonic() + 900
    records = []
    try:
        with Game([str(binary.resolve())], setup, deadline, art, native_input=False) as g:
            def state(): return g.request({'op': 'inspect'})
            def key(name, pressed): g.request({'op': 'key', 'key': name, 'pressed': pressed})
            def press(name): key(name, True); key(name, False)
            def shot(name):
                before = state()
                assert native_capture(g, art, name, deadline)['status'] == 'captured'
                after = state()
                for field in ['ship', 'player', 'archive', 'survival']:
                    assert before[field] == after[field], field
                records.append({'name': name, 'state': after})
                return after
            assert state()['interaction'] == 'archive'
            assert state()['archive']['source']['read'] == []
            for w, h in [(1280, 800), (1920, 1080)]:
                g.request({'op': 'window', 'action': 'resize', 'width': w, 'height': h})
                until = time.monotonic() + 30
                while state()['visual_context']['viewport_pixels'] != [w, h]:
                    assert time.monotonic() < until
                    time.sleep(.05)
                assert state()['visual_context']['scale_factor'] == scale
                shot(f'{w}-physical-terminal')
                press('interact')
                assert state()['archive']['open']
                for _ in range(6): press('thruster_up')
                for i in range(7):
                    s = shot(f'{w}-record-{i+1:02}')
                    text = s['archive']['text']
                    assert f'SCOUT ARCHIVE  {i+1}/7' in text
                    assert 'NO NAVIGATION FIX' in text
                    assert not any(word in text for word in ['Hall', 'wormhole', 'coordinates:', 'Absenat'])
                    if i < 6: press(['thruster_down', 'roll_right', 'wheel_next'][i % 3])
                assert len(state()['archive']['source']['read']) == 7
                saved = state()
                key('consumable_wheel', True); press('status'); key('forward', True)
                assert not state()['consumable_wheel']['open']
                assert not state()['survival_details_open']
                g.request({'op': 'step', 'frames': 1})
                assert state()['survival']['hunger'] < saved['survival']['hunger']
                assert state()['archive']['source'] == saved['archive']['source']
                key('forward', False); key('consumable_wheel', False)
                press('interact'); assert not state()['archive']['open']
            press('interact'); assert state()['archive']['open']
            press('pause'); assert state()['paused'] and not state()['archive']['open']
            shot('escape-paused-reader-closed')
        art.write_json('result.json', {'status': 'passed', 'captures': len(records), 'simulated_scale': scale})
    except BaseException as exc:
        art.write_json('result.json', {'status': 'failed', 'error': str(exc)})
        raise
    finally:
        art.write_json('states.json', records)
        art.write_json('manifest.json', {'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(), 'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(), 'input': 'production key protocol; desktop input isolated', 'simulated_scale': scale})
        art.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', required=True, type=Path)
    p.add_argument('--output', required=True, type=Path)
    p.add_argument('--scale', default=1, type=float, choices=[1, 1.5, 2])
    p.add_argument('--power', action='store_true')
    a = p.parse_args()
    if a.power: power(a.binary, a.output)
    else: run(a.binary, a.output, a.scale)
