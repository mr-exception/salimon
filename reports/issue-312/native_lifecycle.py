"""Local PID input probe adapted from #198 for current #295/#299 contracts.

The helper must be explicitly authorized locally. Fixed E2E stepping is not FPS.
"""
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
        tick(0.15)

    def capture(name):
        path = (art.directory / (name + '.png')).resolve()
        g.request({'op': 'capture', 'path': str(path)})
        tick(0.2)
        if not path.exists():
            raise RuntimeError('GPU capture missing: ' + str(path))
    checks = {}
    def check(name, condition):
        checks[name] = bool(condition)
        (art.directory/'checks.json').write_text(json.dumps(checks,indent=2)+'\n')
        if not condition: raise RuntimeError('failed: '+name)
    os_input('focus')
    if g.request({'op':'inspect'})['paused']: os_input('key',36,40)
    initial=observe('initial')
    os_input('key',53,40)
    paused=observe('escape-paused')
    check('escape_pauses',paused['paused'])
    for code in [14,37,3,18,19]:os_input('key',code,40)
    blocked=observe('paused-actions-blocked')
    check('paused_actions_do_not_change_authority',blocked['equipment']==paused['equipment'] and blocked['ship']['flight_state']==paused['ship']['flight_state'] and blocked['player']['location']==paused['player']['location'])
    check('paused_time_frozen',blocked['journey']['active_duration']==paused['journey']['active_duration'])
    capture('paused')
    os_input('key',36,40)
    check('enter_resumes',not observe('resumed')['paused'])
    os_input('key',18,40)
    check('slot_one',observe('slot-one')['equipment']['selected_slot']==1)
    os_input('key',19,40)
    check('slot_two',observe('slot-two')['equipment']['selected_slot']==2)
    os_input('key',3,40)
    check('f_does_not_mine',not observe('f-release')['mining']['held'])
    os_input('key',120,40)
    tour=observe('tour')
    check('f2_enters_tour',tour['view_mode']!=initial['view_mode'])
    for code in [14,37,3,20]:os_input('key',code,40)
    tour_after=observe('tour-actions')
    check('tour_blocks_gameplay',tour_after['equipment']==tour['equipment'] and tour_after['ship']['flight_state']==tour['ship']['flight_state'] and tour_after['player']['location']==tour['player']['location'])
    os_input('key',120,40)
    check('f2_returns_gameplay',observe('gameplay-return')['view_mode']==initial['view_mode'])
    # Real held W crosses the native F2 boundary; release can occur in tour.
    held=subprocess.Popen([str(a.helper.resolve()),str(g.process.pid),'key','13','1300'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
    tick(.6)
    os_input('key',120,40)
    while held.poll() is None:tick(.032)
    out,err=held.communicate();records.append({'held_w_returncode':held.returncode,'stdout':out,'stderr':err})
    os_input('key',120,40)
    stopped=observe('held-switch-release-return')
    tick(.5)
    idle=observe('post-release-idle')
    check('no_stuck_steering_after_tour_release',idle['ship']['orientation']==stopped['ship']['orientation'])
    capture('gameplay-after-held-release')
    g.request({'op':'window','action':'minimize'})
    tick(.3)
    minimized=observe('minimized')
    check('native_window_minimized',minimized['window']['minimized'] is True)
    g.request({'op':'window','action':'restore'})
    os_input('focus')
    if g.request({'op':'inspect'})['paused']:os_input('key',36,40)
    restored=observe('restored')
    check('native_window_restored',restored['window']['minimized'] is False and not restored['paused'])
    before=restored['ship']['orientation'];tick(.5)
    check('restore_does_not_replay_steering',observe('restore-idle')['ship']['orientation']==before)
    capture('restored')
    observe('final')
    (art.directory/'observations.json').write_text(json.dumps(records,indent=2)+'\n')
art.close()
print('ARTIFACTS',art.directory,flush=True)
