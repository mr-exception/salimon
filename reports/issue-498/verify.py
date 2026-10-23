"""Engine production-protocol route; fixed ticks and GPU captures, not physical input."""
import argparse
import json
from pathlib import Path
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))
from e2e_artifacts import Artifacts
from salimon_test import Game, native_capture


def run(binary, output):
    setup = {'scenario': 'flight-regression', 'seed': 498, 'step_ms': 96}
    art = Artifacts(setup, output, ['native'])
    art.open_logs()
    deadline = time.monotonic() + 300
    states = []
    try:
        with Game([str(binary.resolve())], setup, deadline, art, native_input=False) as g:
            def state(): return g.request({'op': 'inspect'})
            def key(name, pressed):
                g.request({'op': 'key', 'key': name, 'pressed': pressed})
            def press(name):
                key(name, True)
                key(name, False)
            def shot(name):
                before = state()
                assert native_capture(g, art, name, deadline)['status'] == 'captured'
                after = state()
                for field in ['ship', 'player', 'machinery', 'survival']:
                    assert before[field] == after[field], field
                states.append({'name': name, 'state': after})
                return after
            g.request({'op': 'scout_reference', 'view': 'cockpit'})
            initial = shot('on-idle')
            assert initial['ship']['engines_enabled']
            for _ in range(100): g.request({'op': 'thruster', 'direction': 1})
            g.request({'op': 'step', 'frames': 1})
            forward = shot('on-forward')
            dv = forward['ship']['velocity_meters_per_second'][0] - initial['ship']['velocity_meters_per_second'][0]
            assert abs(dv - 100_000_000 * .096 / initial['ship']['total_mass_kg']) < 1e-9, dv
            press('engines')
            off = shot('off-selected-forward')
            assert not off['ship']['engines_enabled'] and off['ship']['thruster_percentage'] == 100
            g.request({'op': 'step', 'frames': 10})
            coast = shot('off-coast')
            assert coast['ship']['velocity_meters_per_second'] == off['ship']['velocity_meters_per_second']
            assert coast['machinery']['thrust'] == 0
            assert coast['ship']['energy_core']['stored_joules'] < off['ship']['energy_core']['stored_joules']
            # The engine switch remains off while normal service loads are paid.
            press('pause'); press('engines')
            assert not state()['ship']['engines_enabled']
            press('pause'); press('engines')
            assert state()['ship']['engines_enabled']
            for _ in range(200): g.request({'op': 'thruster', 'direction': -1})
            g.request({'op': 'step', 'frames': 1})
            reverse = shot('on-reverse')
            assert reverse['ship']['thruster_percentage'] == -100
            dv = reverse['ship']['velocity_meters_per_second'][0] - coast['ship']['velocity_meters_per_second'][0]
            assert abs(dv + 50_000_000 * .096 / reverse['ship']['total_mass_kg']) < 1e-9, dv
            key('left', True); g.request({'op': 'step', 'frames': 1}); key('left', False)
            assert state()['ship']['orientation'] != reverse['ship']['orientation']
            g.request({'op': 'interact'})
            exited = shot('cockpit-exit')
            assert not exited['ship']['cockpit_control_active']
            press('engines')
            assert state()['ship']['engines_enabled']
        art.write_json('states.json', states)
        art.write_json('result.json', {'status': 'passed', 'captures': len(states), 'input': 'production protocol; fixed 96 ms requests / 16 ms ticks'})
    except BaseException as exc:
        art.write_json('states.json', states)
        art.write_json('result.json', {'status': 'failed', 'error': str(exc)})
        raise
    finally:
        art.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    run(a.binary, a.output)
