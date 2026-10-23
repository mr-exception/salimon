"""Capture #489 production routes. Video time is stepped simulation, not display FPS."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import platform
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT/'scripts'))
from salimon_test import Game, native_capture
from e2e_artifacts import Artifacts


def aim(game, target):
    s = game.request({'op': 'inspect'})
    # These two initial fixtures have identity ship orientation; reject drift.
    assert s['ship']['orientation'] == [0, 0, 0, 1]
    eye = s['player']['ship_local_eye_position_meters']
    old = [b-a for a, b in zip(s['player']['eye_position_meters'], s['player']['look_target_meters'])]
    new = [b-a for a, b in zip(eye, target)]
    yaw = (math.atan2(new[2], new[0])-math.atan2(old[2], old[0])+math.pi)%(2*math.pi)-math.pi
    pitch = math.atan2(new[1], math.hypot(new[0], new[2]))-math.atan2(old[1], math.hypot(old[0], old[2]))
    game.request({'op': 'look', 'dx': yaw/.0022, 'dy': -pitch/.0022})


def run(binary, output, scenario, step_ms, lane):
    art = Artifacts({'scenario': scenario, 'seed': 0, 'step_ms': step_ms, 'lane': lane,
                     'scope': 'production protocol with explicit initial fixtures; fixed-step GPU surfaces; no OS input or display FPS'}, output, ['native'])
    art.open_logs()
    states = []
    deadline = time.monotonic()+600
    try:
        with Game([str(binary.resolve())], {'scenario': scenario, 'seed': 0, 'step_ms': step_ms}, deadline, art, native_input=False) as g:
            def inspect(): return g.request({'op': 'inspect'})
            def frame(name, view, advance=0):
                if advance: g.request({'op': 'step', 'frames': advance})
                g.request({'op': 'scout_reference', 'view': view})
                before = inspect()
                shot = native_capture(g, art, name, deadline)
                assert shot['status'] == 'captured', shot
                after = inspect()
                for key in ['ship', 'player', 'machinery', 'fabricator', 'carrying', 'world', 'survival']:
                    assert before[key] == after[key], f'capture mutated {key}'
                states.append({'name': name, 'state': after, 'step_frames': advance, 'step_ms': step_ms, 'image': name+'.png'})
                return after
            def key(name, pressed): g.request({'op': 'key', 'key': name, 'pressed': pressed})
            if lane == 'fabricator':
                before = inspect(); held = before['carrying']['object_id']; assert held is not None
                aim(g, [-3, 1.65, 3.55]); g.request({'op': 'interact'})
                fed = inspect(); assert fed['carrying']['object_id'] is None
                assert fed['ship']['total_mass_kg'] == before['ship']['total_mass_kg']
                assert not any(p['id'] == held for p in fed['world']['fragments'])
                aim(g, [-3.8, 2, 3.69]); g.request({'op': 'interact'})
                assert inspect()['fabricator']['ui_open']
                g.request({'op': 'crafting', 'action': 'add'}); g.request({'op': 'crafting', 'action': 'close'})
                for i in range(12):
                    s = frame(f'work-{i:03}', 'fabricator-close', 5)
                    assert s['machinery']['fabricator'] == 'Working'
                key('pause', True); key('pause', False)
                paused = inspect(); g.request({'op': 'step', 'frames': 30})
                assert paused['machinery'] == inspect()['machinery']
                frame('paused', 'fabricator-close')
                key('pause', True); key('pause', False)
                s = frame('blocked', 'fabricator-close', 100)
                assert s['machinery']['fabricator'] == 'Blocked'
                item = s['fabricator']['tray']; assert item is not None
                aim(g, [-2.55, .991, 3.4]); g.request({'op': 'interact'})
                s = frame('collected', 'fabricator-close')
                assert s['fabricator']['tray'] is None
                assert any(p and (p['job'], p['unit']) == (item['job'], item['unit']) for p in s['consumables']['food'])
                g.request({'op': 'interact'}); assert inspect()['consumables'] == s['consumables']
            elif lane == 'blackout':
                s = frame('no-power-core', 'core'); assert not s['ship']['power_available']
                s = frame('no-power-work', 'fabricator-close', 3)
                assert s['machinery']['fabricator'] == 'Unpowered' and s['machinery']['fabrication_progress'] == 0
                g.request({'op': 'interact'})
                for i in range(8):
                    s = frame(f'conversion-{i:03}', 'core', 2)
                    assert s['machinery']['converting'] and s['ship']['power_available']
                    assert s['machinery']['fabricator'] == 'Working'
                assert s['carrying']['object_id'] is None and s['world']['fragment_count'] == 0
                frame('restored-work', 'fabricator-close')
            elif lane == 'cockpit':
                for _ in range(5): g.request({'op': 'thruster', 'direction': 1})
                for i in range(12):
                    if i == 0: key('left', True)
                    if i == 4: key('left', False); key('right', True)
                    if i == 8: key('right', False)
                    s = frame(f'pedals-{i:03}', 'pedals-close', 1)
                    assert s['machinery']['cockpit_active'] and abs(s['machinery']['rudder']) <= 1
                    assert s['machinery']['thrust'] > 0
                for _ in range(5): g.request({'op': 'thruster', 'direction': -1})
                s = frame('coasting', 'cockpit', 1); assert s['machinery']['thrust'] == 0
                for _ in range(5): g.request({'op': 'thruster', 'direction': -1})
                s = frame('reverse', 'cockpit', 1); assert s['machinery']['thrust'] < 0
            elif lane == 'ramp':
                route = json.loads((ROOT/'scenarios/boarding-ramp.json').read_text())
                for step in route['steps']:
                    action = step.get('action')
                    if action and action['op'] == 'interact': break
                    if action: g.request(action)
                assert inspect()['player']['location'] == 'InsideShip'
                g.request({'op': 'interact'})
                fractions = []
                for i in range(16):
                    if i == 5: g.request({'op': 'interact'})
                    s = frame(f'ramp-{i:03}', 'aft-door', 5)
                    fractions.append(s['ship']['door_open_fraction'])
                assert max(fractions) > 0 and fractions[-1] < max(fractions), fractions
            else: raise ValueError(lane)
        art.write_json('states.json', states)
        art.write_json('result.json', {'status': 'passed', 'lane': lane, 'frames': len(states)})
        return art.directory
    except BaseException as exc:
        art.write_json('result.json', {'status': 'failed', 'lane': lane, 'frames': len(states), 'error': str(exc)})
        art.write_json('states.json', states)
        raise
    finally: art.close()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--lane', choices=['fabricator', 'blackout', 'cockpit', 'ramp', 'all'], default='all')
    a = p.parse_args(); a.output.mkdir(parents=True, exist_ok=False)
    lanes = {'fabricator': ('scout-fabricator', 100), 'blackout': ('scout-work-blackout', 100), 'cockpit': ('flight-regression', 100), 'ramp': ('landed-earth', 16)}
    outputs = {}
    for lane, (scenario, step) in lanes.items():
        if a.lane in ('all', lane): outputs[lane] = str(run(a.binary, a.output/lane, scenario, step, lane))
    (a.output/'manifest.json').write_text(json.dumps({'binary_sha256': hashlib.sha256(a.binary.read_bytes()).hexdigest(), 'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(), 'platform': platform.platform(), 'input': 'production E2E protocol, native input isolated', 'time': 'fixed simulation steps; captures are not real-time FPS evidence', 'lanes': outputs}, indent=2)+'\n')

if __name__ == '__main__': main()
