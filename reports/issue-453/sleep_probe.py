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
mode=p.add_mutually_exclusive_group(required=True)
mode.add_argument('--run-sleep',action='store_true')
mode.add_argument('--manual-sleep',action='store_true')
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
    process=subprocess.Popen([str(a.binary.resolve())],stdout=stdout,stderr=stderr,env=dict(os.environ,SALIMON_SAVE_DIR=str(save.resolve()),SALIMON_REFERENCE_SIZE='1',RUST_LOG='info,salimon_client::app=trace'))
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
            previous=max(int.from_bytes(x.read_bytes()[8:16],'little') for x in save.glob('checkpoint-*.sav'))
            key(53);requested=time.time();key(45)
            deadline=time.monotonic()+5
            while time.monotonic()<deadline:
                candidates=[x.read_bytes() for x in save.glob('checkpoint-*.sav')]
                if any(int.from_bytes(b[8:16],'little')>previous and json.loads(b[28:])['saved_at_unix_seconds']>=int(requested) for b in candidates):
                    capture(label);return read_save(label)
                time.sleep(.1)
            raise RuntimeError('Save Now did not publish a fresh checkpoint: '+label)
        helper('focus');before=paused_save('before');before_wall=time.time()
        key(36);helper('key',13,800)
        positive=paused_save('positive-admission')
        result['native_steering_admitted']=positive['state']['physical']['ship']['pose']['orientation']!=before['state']['physical']['ship']['pose']['orientation']
        if not result['native_steering_admitted']:raise RuntimeError('native W steering did not change orientation')
        key(36)
        stop=a.output/'release-held-input'
        held=subprocess.Popen([str(a.helper.resolve()),str(process.pid),'hold','13',str(stop.resolve())],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        time.sleep(.8)
        wall_start=time.time();mono_start=time.monotonic()
        if a.run_sleep:
            r=subprocess.run(['pmset','sleepnow'],capture_output=True,text=True,timeout=15)
            records.append({'operation':'pmset sleepnow','returncode':r.returncode,'stdout':r.stdout,'stderr':r.stderr,'wall_utc':time.time()})
            if r.returncode:raise RuntimeError('physical sleep command rejected: '+r.stderr)
        else:
            records.append({'operation':'waiting for owner manual sleep','wall_utc':time.time()})
            print('WAITING_FOR_MANUAL_SLEEP: native W positively admitted; hold is active; put Mac to sleep, then wake and unlock.',flush=True)
        # Sleep entry may wait tens of seconds for another application's ack.
        # A successful request or dark display is not a completed sleep/wake.
        events=[];sleeps=[];wakes=[]
        wake_deadline=time.monotonic()+270
        while time.monotonic()<wake_deadline:
            log=subprocess.check_output(['pmset','-g','log'],text=True,timeout=15)
            events=[]
            for line in log.splitlines():
                if not ('Entering Sleep state' in line or ('Wake from' in line and 'DarkWake' not in line)):continue
                try:stamp=datetime.strptime(line[:25],'%Y-%m-%d %H:%M:%S %z').timestamp()
                except ValueError:continue
                if stamp>=wall_start-1:events.append({'unix':stamp,'line':line})
            sleeps=[x for x in events if 'Entering Sleep state' in x['line']]
            wakes=[x for x in events if 'Wake from' in x['line']]
            if sleeps and wakes:break
            time.sleep(1)
        (a.output/'power-events.json').write_text(json.dumps(events,indent=2)+'\n')
        result['power_stats']=subprocess.check_output(['pmset','-g','stats'],text=True,timeout=5)
        stop.touch()
        out,err=held.communicate(timeout=8)
        records.append({'held_w_returncode':held.returncode,'stdout':out,'stderr':err})
        if held.returncode:raise RuntimeError('held-W helper failed')
        down=float(out.split('held_down_unix=')[1].splitlines()[0])
        released=float(out.split('released_unix=')[1].splitlines()[0])
        result['held_input_overlaps_actual_sleep']=bool(sleeps and wakes) and down<sleeps[0]['unix']<wakes[-1]['unix']<released
        wall_end=time.time();mono_end=time.monotonic()
        result.update(wall_gap_seconds=wall_end-wall_start,monotonic_gap_seconds=mono_end-mono_start,physical_sleep_verified=bool(sleeps and wakes))
        if not result['physical_sleep_verified']:raise RuntimeError('no actual Sleep/full-Wake pair; request alone is not acceptance')
        focus_deadline=time.monotonic()+60
        while True:
            try:helper('focus');break
            except RuntimeError:
                if time.monotonic()>=focus_deadline:raise
                time.sleep(1)  # Allow the owner to unlock the dedicated desktop.
        key(36);time.sleep(2)
        after=paused_save('after-wake')
        key(36);time.sleep(1)
        idle=paused_save('after-idle')
        result['position_stable_without_fresh_input']=after['state']['physical']['character']['position']==idle['state']['physical']['character']['position']
        result['steering_stable_without_fresh_input']=after['state']['physical']['ship']['pose']['orientation']==idle['state']['physical']['ship']['pose']['orientation']
        def seconds(d):return d['secs']+d['nanos']/1e9
        result['simulation_advance_seconds']=seconds(idle['simulated_time'])-seconds(before['simulated_time'])
        after_wall=time.time()
        result['world_clock_wall_seconds']=after_wall-before_wall
        sleep_seconds=(wakes[-1]['unix']-sleeps[0]['unix']) if sleeps and wakes else 0
        # pmset event timestamps are integer seconds (two rounded endpoints).
        # Three seconds bounds that rounding plus checkpoint/tick quantization;
        # it is not a gameplay timing/performance budget.
        result['os_sleep_seconds']=sleep_seconds
        result['awake_wall_upper_bound_seconds']=after_wall-before_wall-sleep_seconds+3
        result['no_sleep_time_replayed']=bool(sleeps and wakes) and result['simulation_advance_seconds']<=result['awake_wall_upper_bound_seconds']
        result['status']='passed' if result['native_steering_admitted'] and result['held_input_overlaps_actual_sleep'] and result['steering_stable_without_fresh_input'] and result['physical_sleep_verified'] and result['position_stable_without_fresh_input'] and result['no_sleep_time_replayed'] else 'failed'
        key(7);process.wait(timeout=12)
    finally:
        (a.output/'input.json').write_text(json.dumps(records,indent=2)+'\n')
        (a.output/'result.json').write_text(json.dumps(result,indent=2)+'\n')
        if 'stop' in locals():stop.touch()
        if 'held' in locals() and held.poll() is None:
            held.communicate(timeout=8)
        if process.poll() is None:process.kill();process.wait(timeout=5)
print(json.dumps(result))
if result['status']!='passed':raise SystemExit(1)
