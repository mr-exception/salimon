"""Review explicit native fixtures through real normal-launch/save/restart paths.

Requires the owner's authorization for the reviewed OS-input helper. Never point
--fixtures at a player's save directory. The generator creates these test saves.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
import zlib
p = argparse.ArgumentParser()
p.add_argument('--binary', type=Path, required=True)
p.add_argument('--helper', type=Path, required=True)
p.add_argument('--fixtures', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
manifest = json.loads((a.fixtures/'manifest.json').read_text())
results = []
def checkpoint(directory):
    candidates = []
    for path in directory.glob('checkpoint-*.sav'):
        data = path.read_bytes()
        if len(data) < 28 or data[:8] != b'SALICK01':
            continue
        if int.from_bytes(data[16:24], 'little') != len(data)-28:
            continue
        if int.from_bytes(data[24:28], 'little') != zlib.crc32(data[8:24]+data[28:]):
            continue
        candidates.append((int.from_bytes(data[8:16], 'little'),data))
    generation,data = max(candidates)
    return generation, json.loads(data[28:]), data
for case in manifest:
    name=case['case']; directory=a.fixtures/name; out=a.output/name
    out.mkdir()
    generation,before,raw=checkpoint(directory)
    (out/'before.sav').write_bytes(raw)
    env=dict(os.environ, SALIMON_SAVE_DIR=str(directory.resolve()), SALIMON_REFERENCE_SIZE='1', RUST_LOG='info')
    records=[]
    with (out/'stdout.log').open('w') as stdout, (out/'stderr.log').open('w') as stderr:
        process=subprocess.Popen([str(a.binary.resolve())],env=env,stdout=stdout,stderr=stderr)
        try:
            deadline=time.monotonic()+30
            while 'renderer ready:' not in (out/'stderr.log').read_text():
                if process.poll() is not None: raise RuntimeError(f'launch failed: {process.returncode}')
                if time.monotonic()>deadline: raise TimeoutError('renderer not ready')
                time.sleep(.1)
            def helper(*args):
                r=subprocess.run([str(a.helper.resolve()),str(process.pid),*map(str,args)],capture_output=True,text=True,timeout=8)
                records.append({'input':args,'stdout':r.stdout,'stderr':r.stderr,'returncode':r.returncode})
                if r.returncode: raise RuntimeError(r.stderr)
                return r.stdout
            def key(code): helper('key',code,100)
            def capture(label):
                w=json.loads(helper('window').splitlines()[0]); records.append({'window':w})
                subprocess.run(['screencapture','-x','-l',str(w['kCGWindowNumber']),str(out/(label+'.png'))],check=True,timeout=8)
            helper('focus')
            if name=='terminal':
                capture('journey-ended')
                key(15)  # physical R: verified archive/two-slot restart
                time.sleep(1)
                key(53); key(45); time.sleep(.8)
                capture('restart-paused')
                _,after,raw=checkpoint(directory)
                assert after['state']['survival']['run']['serial']==before['state']['survival']['run']['serial']+1
                archive = directory/'journeys/v2'/f"run-{before['state']['survival']['run']['serial']}.json"
                report=json.loads(archive.read_text())
                assert report['serial']==before['state']['survival']['run']['serial']
                shutil.copy(archive,out/'archived-run.json')
            else:
                key(4)  # H: actual status-details panel
                capture('status-details')
                key(4); key(53); key(45);time.sleep(.8)
                capture('paused-save')
                _,after,raw=checkpoint(directory)
                # Launch/render time may step physics/work/effects. Identity is exact.
                assert after['state']['fabrication']['inventory']==before['state']['fabrication']['inventory']
                assert after['state']['physical']['seed']==before['state']['physical']['seed']
                assert after['state']['survival']['run']['serial']==before['state']['survival']['run']['serial']
            # A second real Save Now must not tick any owner while paused.
            paused=after
            time.sleep(1.2);key(45);time.sleep(.6)
            _,after,raw=checkpoint(directory)
            assert after['state']==paused['state'], 'paused Save Now changed domain state'
            assert after['simulated_time']==paused['simulated_time']
            assert after['pending_time']==paused['pending_time']
            (out/'pause-freeze.json').write_text(json.dumps({'passed':True,'real_wait_seconds':1.2,'compared':['state','simulated_time','pending_time']})+'\n')
            (out/'after.sav').write_bytes(raw)
            (out/'summary.json').write_text(json.dumps({'case':name,'fixture':True,'before':before,'after':after,'records':records},separators=(',',':'))+'\n')
            key(7)  # X while paused: verified save and exit
            process.wait(timeout=12)
            if process.returncode: raise RuntimeError(f'exit {process.returncode}')
            results.append({'case':name,'status':'passed','scope':'native launch/render/H-or-terminal/R/save/exit; fixture, not travel or natural death'})
            print(json.dumps(results[-1]),flush=True)
        finally:
            (out/'input.json').write_text(json.dumps(records,indent=2)+'\n')
            if process.poll() is None:
                process.kill();process.wait(timeout=5)
            (a.output/'results.json').write_text(json.dumps(results,indent=2)+'\n')
