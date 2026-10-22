"""Bounded M1 Fabricator OS-input probe; run from repository root."""
import argparse
import json
import math
from pathlib import Path
import subprocess
import sys
import time
sys.path.insert(0, str(Path('scripts').resolve()))
from salimon_test import Game, native_capture
from e2e_artifacts import Artifacts
p = argparse.ArgumentParser()
p.add_argument('--binary', type=Path, required=True)
p.add_argument('--helper', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
art = Artifacts({'native_fabricator': True}, a.output, ['native'])
art.open_logs()
records, checks = [], {}
try:
    with Game([str(a.binary.resolve())], {'scenario':'landed-earth','seed':0,'step_ms':16}, time.monotonic()+120, art) as g:
        def tick(n=4):
            for _ in range(n):
                g.request({'op':'step','frames':1})
                time.sleep(.016)
        def observe(name):
            st=g.request({'op':'inspect'})
            records.append({'name':name,'state':st})
            return st
        def oskey(code, ms=50):
            proc=subprocess.Popen([str(a.helper.resolve()),str(g.process.pid),'key',str(code),str(ms)],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
            while proc.poll() is None: tick(2)
            out,err=proc.communicate()
            records.append({'key':code,'ms':ms,'returncode':proc.returncode,'stdout':out,'stderr':err})
            if proc.returncode: raise RuntimeError(err)
            tick(8)
        def capture(name):
            r=native_capture(g,art,name,time.monotonic()+10)
            if r['status']!='captured': raise RuntimeError(str(r))
        # Production walking/look commands prepare location; only E/Q/modal keys use OS input.
        for key,n in [('backward',60),('right',65)]:
            g.request({'op':'key','key':key,'pressed':True})
            g.request({'op':'step','frames':n})
            g.request({'op':'key','key':key,'pressed':False})
        st=observe('walked')
        x,y,z=st['player']['local_ship_position_meters']
        dx,dy,dz=-3.8-x,2.0-y,3.69-z
        g.request({'op':'look','dx':math.atan2(dz,dx)/.0022,'dy':-math.atan2(dy,math.hypot(dx,dz))/.0022})
        observe('aimed');capture('physical-screen')
        proc=subprocess.run([str(a.helper.resolve()),str(g.process.pid),'focus'],capture_output=True,text=True,timeout=5)
        records.append({'focus_returncode':proc.returncode,'stdout':proc.stdout,'stderr':proc.stderr})
        if proc.returncode: raise RuntimeError(proc.stderr)
        tick(8)
        if g.request({'op':'inspect'})['paused']: oskey(36)
        oskey(14)
        st=observe('native-e')
        checks['native_e_opens_machine']=st['fabricator']['ui_open']
        capture('crafting-modal')
        oskey(14)
        checks['native_e_closes_machine']=not observe('native-e-close')['fabricator']['ui_open']
        oskey(12,500)
        checks['q_release_closes_wheel']=not observe('native-q-release')['consumable_wheel']['open']
        checks['no_stale_wheel_hold']=not g.request({'op':'inspect'})['consumable_wheel']['held']
        checks['empty_inventory_preserved']=g.request({'op':'inspect'})['consumables']==st['consumables']
finally:
    (art.directory/'observations.json').write_text(json.dumps(records,indent=2)+'\n')
    (art.directory/'checks.json').write_text(json.dumps(checks,indent=2)+'\n')
    art.close()
print(json.dumps({'artifacts':str(art.directory),'checks':checks}))
if not checks or not all(checks.values()): sys.exit(1)
