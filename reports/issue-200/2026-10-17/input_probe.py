"""Opt-in native OS input probe; fixed E2E clock supplies observations, not FPS."""
import argparse, json, subprocess, sys, time
from pathlib import Path
sys.path.insert(0, str(Path('scripts').resolve()))
from salimon_test import Game
from e2e_artifacts import Artifacts
p = argparse.ArgumentParser()
p.add_argument('--binary', type=Path, required=True)
p.add_argument('--helper', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--scenario', default='flight-regression')
p.add_argument('--mouse-only', action='store_true')
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
art = Artifacts({'os_input_probe': a.scenario}, a.output, ['native'])
art.open_logs()
records = []
with Game([str(a.binary.resolve())], {'scenario': a.scenario, 'seed': 198, 'step_ms': 16}, time.monotonic() + 300, art) as g:

    def tick(seconds):
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            start = time.monotonic()
            g.request({'op': 'step', 'frames': 1})
            time.sleep(max(0, 0.016 - (time.monotonic() - start)))

    def observe(name):
        st = g.request({'op': 'inspect'})
        records.append({'name': name, 'monotonic': time.monotonic(), 'state': st})
        (art.directory / 'observations.json').write_text(json.dumps(records, indent=2) + '\n')
        print(name, json.dumps({k: st['ship'][k] for k in ['thruster_percentage', 'orientation', 'speed_meters_per_second', 'flight_state', 'power_available']}), flush=True)
        return st

    def os_input(*args):
        proc = subprocess.Popen([str(a.helper.resolve()), str(g.process.pid), *map(str, args)], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        while proc.poll() is None:
            tick(0.032)
        out, err = proc.communicate()
        records.append({'input': args, 'stdout': out, 'stderr': err, 'returncode': proc.returncode, 'monotonic': time.monotonic()})
        if proc.returncode:
            raise RuntimeError(err)
        if args[0] == "focus" and f"frontmost={g.process.pid} target={g.process.pid}" not in out:
            raise RuntimeError("target PID is not foreground: " + out)
        tick(0.15)

    def capture(name):
        path = (art.directory / (name + '.png')).resolve()
        g.request({'op': 'capture', 'path': str(path)})
        tick(0.2)
        if not path.exists():
            raise RuntimeError('GPU capture missing: ' + str(path))
    os_input('focus')
    tick(0.2)
    observe('initial')
    capture('initial')
    if a.mouse_only:
        for _ in range(6):
            os_input('mouse', 20, 5)
        observe('relative-mouse-positive')
        capture('mouse-look')
        for _ in range(6):
            os_input('mouse', -20, -5)
        observe('relative-mouse-return')
    elif a.scenario == 'flight-regression':
        os_input('key', 126, 650)
        up = observe('held-up-released')
        tick(0.4)
        observe('throttle-remains-after-release')
        for i in range(30):
            st = g.request({'op': 'inspect'})
            n = st['ship']['thruster_percentage']
            if n == 0:
                break
            os_input('key', 125 if n > 0 else 126, 40)
        else:
            raise RuntimeError('native throttle could not return to zero')
        observe('coasting-before-steer')
        os_input('key', 0, 5000)
        observe('held-yaw-left-released')
        tick(0.5)
        observe('yaw-release-idle')
        os_input('key', 2, 5000)
        observe('held-yaw-right-released')
        os_input('key', 13, 600)
        observe('held-pitch-released')
        os_input('key', 124, 600)
        observe('held-roll-released')
        capture('steered')
        for _ in range(6):
            os_input('mouse', 20, 5)
        observe('relative-mouse-positive')
        capture('mouse-look')
        for _ in range(6):
            os_input('mouse', -20, -5)
        observe('relative-mouse-return')
        os_input('key', 125, 5000)
        os_input('key', 125, 5000)
        observe('native-reverse-selected')
        tick(6)
        observe('native-reverse-motion')
        capture('reverse-motion')
    elif a.scenario in ('core-restart', 'cargo-blackout'):
        os_input('key', 49, 50)
        observe('zero-g-space-released')
        tick(0.5)
        observe('zero-g-coast')
        os_input('key', 14, 40)
        observe('native-core-intake')
        tick(1)
        observe('conversion-restored-power')
        capture('core-converting')
        tick(10)
        observe('converted')
        capture('core-recharged')
    elif a.scenario == 'crash-recovery':
        tick(3)
        observe('crash-grounded')
        os_input('key', 37, 40)
        observe('native-recovery-started')
        tick(13)
        observe('recovery-landed')
        capture('recovered')
        os_input('key', 37, 40)
        observe('native-takeoff-started')
        tick(7)
        observe('takeoff-complete')
        capture('takeoff')
    observe('final')
    states = {r['name']: r['state'] for r in records if 'state' in r}
    checks = {}
    if a.scenario == 'flight-regression':
        if not a.mouse_only:
            checks['held_throttle_repeat'] = states['held-up-released']['ship']['thruster_percentage'] > 1
            checks['release_preserves_selection'] = states['held-up-released']['ship']['thruster_percentage'] == states['throttle-remains-after-release']['ship']['thruster_percentage']
            checks['held_yaw_changes_heading'] = states['coasting-before-steer']['ship']['orientation'] != states['held-yaw-left-released']['ship']['orientation']
            checks['released_yaw_stops'] = states['held-yaw-left-released']['ship']['orientation'] == states['yaw-release-idle']['ship']['orientation']
            checks['opposite_yaw_changes_heading'] = states['held-yaw-left-released']['ship']['orientation'] != states['held-yaw-right-released']['ship']['orientation']
            checks['pitch_changes_heading'] = states['held-yaw-right-released']['ship']['orientation'] != states['held-pitch-released']['ship']['orientation']
            checks['roll_changes_heading'] = states['held-pitch-released']['ship']['orientation'] != states['held-roll-released']['ship']['orientation']
            velocity = states['coasting-before-steer']['ship']['velocity_meters_per_second']
            checks['native_reverse_motion'] = states['native-reverse-motion']['ship']['velocity_meters_per_second'][0] < 0 and states['native-reverse-motion']['ship']['thruster_percentage'] < 0
            checks['coasting_velocity_preserved'] = all((states[n]['ship']['velocity_meters_per_second'] == velocity for n in ['held-yaw-left-released', 'yaw-release-idle', 'held-yaw-right-released', 'held-pitch-released', 'held-roll-released']))

        def direction(st):
            p = st['player']
            return [p['look_target_meters'][i] - p['eye_position_meters'][i] for i in range(3)]
        before = direction(states['initial'] if a.mouse_only else states['held-roll-released'])
        moved = direction(states['relative-mouse-positive'])
        returned = direction(states['relative-mouse-return'])
        checks['native_relative_mouse_changes_view'] = max((abs(x - y) for x, y in zip(before, moved))) > 0.05
        checks['inverse_mouse_restores_view'] = max((abs(x - y) for x, y in zip(before, returned))) < 1e-06
    elif a.scenario == 'crash-recovery':
        checks['grounded'] = states['crash-grounded']['ship']['flight_state'] == 'GroundedAfterCrash { body: Earth }'
        checks['native_l_recovery'] = states['native-recovery-started']['ship']['flight_state'] == 'AssistedLanding { body: Earth }'
        checks['recovered_landed'] = states['recovery-landed']['ship']['flight_state'] == 'Landed { body: Earth }'
        checks['native_l_takeoff'] = states['native-takeoff-started']['ship']['flight_state'] == 'AssistedTakeoff { body: Earth }'
        checks['inertial_handoff'] = states['takeoff-complete']['ship']['flight_state'] == 'Flying' and states['takeoff-complete']['ship']['speed_meters_per_second'] > 0 and (states['takeoff-complete']['ship']['thruster_percentage'] == 0)
    else:
        checks['unpowered_native_movement'] = states['zero-g-space-released']['player']['local_ship_position_meters'][1] > states['initial']['player']['local_ship_position_meters'][1]
        checks['zero_g_coasting'] = states['zero-g-coast']['player']['local_ship_position_meters'][1] > states['zero-g-space-released']['player']['local_ship_position_meters'][1]
        checks['native_e_consumes_once'] = states['native-core-intake']['carrying']['object_id'] is None and states['native-core-intake']['world']['fragment_count'] == 499
        checks['gradual_conversion_power'] = states['conversion-restored-power']['ship']['power_available'] and states['conversion-restored-power']['ship']['conversion_remaining_joules'] > 0
        checks['conversion_completed'] = states['converted']['ship']['conversion_remaining_joules'] == 0
        checks['identities_retained'] = {f['id'] for f in states['initial']['world']['fragments']} - {states['initial']['carrying']['object_id']} == {f['id'] for f in states['converted']['world']['fragments']}
    (art.directory / 'checks.json').write_text(json.dumps(checks, indent=2) + '\n')
    if not all(checks.values()):
        raise RuntimeError('native probe checks failed: ' + str(checks))
(art.directory / 'observations.json').write_text(json.dumps(records, indent=2) + '\n')
art.close()
print('ARTIFACTS', art.directory, flush=True)
