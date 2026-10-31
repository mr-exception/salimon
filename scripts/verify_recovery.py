"""Production Fabricator/tray/Archive recovery route with explicit starting materials."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import subprocess
import time

# Adapted from the retained #515 route at d003bd58d506e04e3df97744f29f68449b6e2b3e.
from e2e_artifacts import Artifacts
from salimon_test import Game, native_capture


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


def run(binary, output):
    setup = {'scenario': 'scout-recovery', 'seed': 515, 'step_ms': 96}
    art = Artifacts(setup, output, ['native']); art.open_logs()
    deadline = time.monotonic() + 1200
    records = []
    try:
        with Game([str(binary.resolve())], setup, deadline, art, native_input=False) as g:
            def state(): return g.request({'op': 'inspect'})
            def request(**kw): return g.request(kw)
            def key(name, pressed): request(op='key', key=name, pressed=pressed)
            def press(name): key(name, True); key(name, False)
            def shot(name, view='clear'):
                request(op='scout_reference', view=view)
                before = state()
                assert native_capture(g, art, name, deadline)['status'] == 'captured'
                after = state()
                for field in ['ship', 'recovery', 'archive', 'fabricator', 'player']:
                    assert before[field] == after[field], field
                records.append({'name': name, 'state': after})
                return after
            initial = state(); held = initial['carrying']['object_id']
            assert held is not None and initial['recovery']['module'] is None
            aim(g, [-3, 1.65, 3.55]); request(op='interact')
            fed = state()
            assert fed['carrying']['object_id'] is None
            assert fed['ship']['total_mass_kg'] == initial['ship']['total_mass_kg']
            assert not any(p['id'] == held for p in fed['world']['fragments'])
            aim(g, [-3.8, 2, 3.69]); press('interact')
            assert state()['fabricator']['ui_open']
            for _ in range(7): request(op='crafting', action='next_recipe')
            request(op='crafting', action='add')
            jobs = state()['fabricator']['jobs']
            assert len(jobs) == 1 and jobs[0]['recipe'] == 'data-recovery-module'
            request(op='crafting', action='add')
            assert state()['fabricator']['jobs'] == jobs
            request(op='crafting', action='close')
            shot('module-crafting', 'fabricator-close')
            request(op='step', frames=600); request(op='step', frames=25)
            ready = shot('module-tray', 'recovery-tray')
            tray = ready['fabricator']['tray']; assert tray['category'] == 'Component'
            assert tray['mass_kg'] == .75
            aim(g, [-2.55, .991, 3.4]); press('interact')
            installed = shot('module-installed', 'recovery-tray')
            module = installed['recovery']['module']
            assert installed['fabricator']['tray'] is None
            assert (module['job'], module['unit']) == (tray['job'], tray['unit'])
            assert installed['ship']['total_mass_kg'] == ready['ship']['total_mass_kg']
            press('interact'); assert state()['recovery']['module'] == module
            # Walk across the cabin using production movement, then aim at the Archive.
            aim(g, [-3, 1.8, -4.13]); key('forward', True)
            request(op='step', frames=12); key('forward', False)
            aim(g, [-3, 1.7, -4.13])
            assert state()['interaction'] == 'archive', state()['player']
            press('interact'); assert state()['archive']['open']
            for _ in range(5): press('thruster_down')
            assert state()['archive']['source']['selected'] == 'received.damaged-01'
            press('recovery'); assert state()['recovery']['active'] == 0
            request(op='step', frames=100)
            partial = shot('partial-working')
            assert 0 < partial['recovery']['spent_joules'][0] < 3e9
            press('recovery'); stopped = state()
            request(op='step', frames=10)
            assert state()['recovery']['spent_joules'] == stopped['recovery']['spent_joules']
            shot('stopped-retains-paid-work')
            press('recovery'); press('pause')
            paused = state(); request(op='step', frames=100)
            assert state()['recovery'] == paused['recovery']
            press('resume'); press('interact')
            assert state()['archive']['open']
            # Paid progress comes exclusively from live production ticks.
            for record, cost in [(0, 3e9), (1, 5e9)]:
                if record == 1: press('thruster_down'); press('recovery')
                for _ in range(20):
                    if state()['recovery']['spent_joules'][record] == cost: break
                    request(op='step', frames=600)
                completed = state()
                assert completed['recovery']['spent_joules'][record] == cost
                assert completed['recovery']['active'] is None
                assert ('UNVERIFIED' if record == 0 else 'IRRECOVERABLE') in completed['archive']['text']
                assert 'NO NAVIGATION FIX' in completed['archive']['text']
                for w, h in [(1280, 800), (1920, 1080)]:
                    request(op='window', action='resize', width=w, height=h)
                    until = time.monotonic() + 30
                    while state()['visual_context']['viewport_pixels'] != [w, h]:
                        assert time.monotonic() < until
                    shot(f'{w}-record-{record+1}-complete')
        art.write_json('result.json', {'status': 'passed', 'captures': len(records), 'scope': 'explicit initial resources; production intake/craft/collect/walk/R/funded ticks'})
    except BaseException as exc:
        art.write_json('result.json', {'status': 'failed', 'error': str(exc)})
        raise
    finally:
        art.write_json('states.json', records)
        art.write_json('manifest.json', {'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(), 'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(), 'input': 'production protocol, desktop input isolated; 96 ms requests / 16 ms simulation ticks'})
        art.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', required=True, type=Path)
    p.add_argument('--output', required=True, type=Path)
    a = p.parse_args(); run(a.binary, a.output)
