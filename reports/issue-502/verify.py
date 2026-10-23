"""Capture real wgpu Scout screens at two viewports and simulated X11 scales."""
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
sys.path.insert(0, str(ROOT / 'reports' / 'issue-489'))
from e2e_artifacts import Artifacts
from salimon_test import Game, native_capture
from walkthrough import aim


def run(binary, output, scale, only):
    os.environ['WINIT_X11_SCALE_FACTOR'] = str(scale)
    records = []
    for scenario in (['core-restart', 'scout-fabricator'] if only == 'all' else [only]):
        setup = {'scenario': scenario, 'seed': 502, 'step_ms': 96}
        art = Artifacts(setup, output / scenario, ['native'])
        art.open_logs()
        deadline = time.monotonic() + 900
        try:
            with Game([str(binary.resolve())], setup, deadline, art, native_input=False) as g:
                def state(): return g.request({'op': 'inspect'})
                def key(name, pressed): g.request({'op': 'key', 'key': name, 'pressed': pressed})
                def press(name): key(name, True); key(name, False)
                def viewport(width, height):
                    g.request({'op': 'window', 'action': 'resize', 'width': width, 'height': height})
                    until = time.monotonic() + 30
                    while time.monotonic() < until:
                        if state()['visual_context']['viewport_pixels'] == [width, height]: break
                        time.sleep(.05)
                    assert state()['visual_context']['viewport_pixels'] == [width, height]
                    assert state()['visual_context']['scale_factor'] == scale
                def shot(name, view='clear'):
                    g.request({'op': 'scout_reference', 'view': view})
                    before = state()
                    assert native_capture(g, art, name, deadline)['status'] == 'captured'
                    after = state()
                    for field in ['ship', 'player', 'fabricator', 'survival', 'machinery']:
                        assert before[field] == after[field], field
                    records.append({'name': name, 'path': str(art.directory / (name+'.png')), 'state': after})
                    return after
                if scenario == 'core-restart':
                    assert not state()['ship']['power_available']
                    for w, h in [(1280, 800), (1920, 1080)]:
                        viewport(w, h)
                        for view in ['cockpit', 'core', 'fabricator']:
                            shot(f'{w}-{view}-blackout', view)
                    # Camera override never changes the physical Core intake target.
                    g.request({'op': 'interact'})
                    g.request({'op': 'step', 'frames': 1})
                    assert state()['ship']['power_available']
                    assert state()['carrying']['object_id'] is None
                    for w, h in [(1280, 800), (1920, 1080)]:
                        viewport(w, h)
                        for view in ['cockpit', 'core', 'fabricator']:
                            shot(f'{w}-{view}-restored', view)
                else:
                    aim(g, [-3.8, 2, 3.69])
                    assert state()['interaction'] == 'fabricator_screen'
                    for w, h in [(1280, 800), (1920, 1080)]:
                        viewport(w, h)
                        shot(f'{w}-physical-prompt')
                        g.request({'op': 'interact'})
                        assert state()['fabricator']['ui_open']
                        before = state()
                        key('consumable_wheel', True); press('status')
                        after = state()
                        assert not after['consumable_wheel']['open']
                        assert not after['survival_details_open']
                        assert before['ship'] == after['ship']
                        shot(f'{w}-crafting-exclusive')
                        key('consumable_wheel', False)
                        g.request({'op': 'crafting', 'action': 'close'})
                        press('pause')
                        paused = state(); g.request({'op': 'step', 'frames': 10})
                        assert state()['ship'] == paused['ship']
                        shot(f'{w}-pause')
                        press('resume')
            art.write_json('result.json', {'status': 'passed', 'scenario': scenario, 'scale': scale})
        except BaseException as exc:
            art.write_json('result.json', {'status': 'failed', 'error': str(exc)})
            raise
        finally:
            art.write_json('captures.json', records)
            art.close()
    (output / 'manifest.json').write_text(json.dumps({'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(), 'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(), 'simulated_scale': scale, 'input': 'production protocol; desktop input isolated', 'captures': len(records)}, indent=2)+'\n')


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', required=True, type=Path)
    p.add_argument('--output', required=True, type=Path)
    p.add_argument('--scenario', choices=['all', 'core-restart', 'scout-fabricator'], default='all')
    p.add_argument('--scale', required=True, type=float, choices=[1, 1.5, 2])
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=False)
    run(a.binary, a.output, a.scale, a.scenario)
