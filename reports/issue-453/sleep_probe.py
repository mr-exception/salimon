"""Dedicated, owner-authorized physical sleep test on an isolated normal run.

No default sleep: --run-sleep is required. The operator must be ready to wake
the Mac manually. Does not change/cancel power schedules or request sudo.
"""
import argparse
from datetime import datetime
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
import zlib
p=argparse.ArgumentParser()
p.add_argument('--binary',type=Path,required=True)
p.add_argument('--helper',type=Path,required=True)
p.add_argument('--source-checkpoint',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--run-sleep',action='store_true',required=True)
a=p.parse_args()
a.output.mkdir(parents=True,exist_ok=False)
save=a.output/'save';save.mkdir();shutil.copy(a.source_checkpoint,save/'checkpoint-0.sav')
records=[];result={'status':'incomplete','physical_sleep_verified':False}
def read_save(label):
    path=max(save.glob('checkpoint-*.sav'),key=lambda x:int.from_bytes(x.read_bytes()[8:16],'little'))
    b=path.read_bytes();assert b[:8]==b'SALICK01' and int.from_bytes(b[24:28],'little')==zlib.crc32(b[8:24]+b[28:])
    (a.output/(label+'.sav')).write_bytes(b)
    return json.loads(b[28:])
with (a.output/'stdout.log').open('w') as stdout,(a.output/'stderr.log').open('w') as stderr:
    process=subprocess.Popen([str(a.binary.resolve())],stdout=stdout,stderr=stderr,env=dict(os.environ,SALIMON_SAVE_DIR=str(save.resolve()),SALIMON_REFERENCE_SIZE='1',RUST_LOG='info'))
    try:
        deadline=time.monotonic()+30
        while 'renderer ready:' not in (a.output/'stderr.log').read_text():
            if process.poll() is not None:raise RuntimeError(f'launch {process.returncode}')
            if time.monotonic()>deadline:raise TimeoutError('renderer')
            time.sleep(.1)
        def helper(*args):
            r=subprocess.run([str(a.helper.resolve()),str(process.pid),*map(str,args)],capture_output=True,text=True,timeout=8)
            records.append({'input':args,'wall_utc':time.time(),'stdout':r.stdout,'stderr':r.stderr,'returncode':r.returncode})
            if r.returncode:raise RuntimeError(r.stderr)
            return r.stdout
        def key(code):helper('key',code,100)
        def capture(label):
            w=json.loads(helper('window').splitlines()[0]);subprocess.run(['screencapture','-x','-l',str(w['kCGWindowNumber']),str(a.output/(label+'.png'))],check=True,timeout=8)
        def paused_save(label):
            key(53);key(45);time.sleep(.8);capture(label);return read_save(label)
        helper('focus');before=paused_save('before');before_wall=time.time()
        key(36)
        held=subprocess.Popen([str(a.helper.resolve()),str(process.pid),'key','13','5000'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        time.sleep(.8)
        wall_start=time.time();mono_start=time.monotonic()
        r=subprocess.run(['pmset','sleepnow'],capture_output=True,text=True,timeout=15)
        records.append({'operation':'pmset sleepnow','returncode':r.returncode,'stdout':r.stdout,'stderr':r.stderr,'wall_utc':time.time()})
        if r.returncode:raise RuntimeError('physical sleep command rejected: '+r.stderr)
        time.sleep(3)
        out,err=held.communicate(timeout=8)
        records.append({'held_w_returncode':held.returncode,'stdout':out,'stderr':err})
        if held.returncode:raise RuntimeError('held-W helper failed')
        wall_end=time.time();mono_end=time.monotonic()
        log=subprocess.check_output(['pmset','-g','log'],text=True,timeout=15)
        events=[]
        for line in log.splitlines():
            if not ('Entering Sleep state' in line or ('Wake from' in line and 'DarkWake' not in line)):continue
            try:stamp=datetime.strptime(line[:25],'%Y-%m-%d %H:%M:%S %z').timestamp()
            except ValueError:continue
            if stamp>=wall_start-1:events.append({'unix':stamp,'line':line})
        (a.output/'power-events.json').write_text(json.dumps(events,indent=2)+'\n')
        sleeps=[x for x in events if 'Entering Sleep state' in x['line']]
        wakes=[x for x in events if 'Wake from' in x['line']]
        result.update(wall_gap_seconds=wall_end-wall_start,monotonic_gap_seconds=mono_end-mono_start,physical_sleep_verified=bool(sleeps and wakes))
        helper('focus');key(36)
        after=paused_save('after-wake')
        key(36);time.sleep(1)
        idle=paused_save('after-idle')
        result['position_stable_without_fresh_input']=after['state']['physical']['character']['position']==idle['state']['physical']['character']['position']
        def seconds(d):return d['secs']+d['nanos']/1e9
        result['simulation_advance_seconds']=seconds(after['simulated_time'])-seconds(before['simulated_time'])
        after_wall=time.time()
        result['world_clock_wall_seconds']=after_wall-before_wall
        sleep_seconds=(wakes[-1]['unix']-sleeps[0]['unix']) if sleeps and wakes else 0
        # pmset event timestamps are integer seconds (two rounded endpoints).
        # Three seconds bounds that rounding plus checkpoint/tick quantization;
        # it is not a gameplay timing/performance budget.
        result['os_sleep_seconds']=sleep_seconds
        result['awake_wall_upper_bound_seconds']=after_wall-before_wall-sleep_seconds+3
        result['no_sleep_time_replayed']=bool(sleeps and wakes) and result['simulation_advance_seconds']<=result['awake_wall_upper_bound_seconds']
        result['status']='passed' if result['physical_sleep_verified'] and result['position_stable_without_fresh_input'] and result['no_sleep_time_replayed'] else 'failed'
        key(7);process.wait(timeout=12)
    finally:
        (a.output/'input.json').write_text(json.dumps(records,indent=2)+'\n')
        (a.output/'result.json').write_text(json.dumps(result,indent=2)+'\n')
        if process.poll() is None:process.kill();process.wait(timeout=5)
print(json.dumps(result))
if result['status']!='passed':raise SystemExit(1)
