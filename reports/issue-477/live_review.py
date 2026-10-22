"""Normal-clock native input recordings on isolated saved fixtures; no human signoff."""
import argparse, hashlib, json, os, shutil, subprocess, time, zlib
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--binary',type=Path,required=True);p.add_argument('--helper',type=Path,required=True)
p.add_argument('--move-helper',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
p.add_argument('--external',action='store_true');p.add_argument('--cases',nargs='+',default=['landed','moving','eva'])
a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
results=[]
for case in a.cases:
 out=a.output/case;out.mkdir();save=out/'save';save.mkdir()
 source=Path('reports/issue-376/evidence/initial')/case/'before.sav';shutil.copy(source,save/'checkpoint-0.sav')
 records=[];checks={};video=None
 def newest():
  b=max((q.read_bytes() for q in save.glob('checkpoint-*.sav')),key=lambda b:int.from_bytes(b[8:16],'little'))
  assert b[:8]==b'SALICK01' and zlib.crc32(b[8:24]+b[28:])==int.from_bytes(b[24:28],'little')
  return int.from_bytes(b[8:16],'little'),json.loads(b[28:]),b
 with (out/'stdout.log').open('w') as stdout,(out/'stderr.log').open('w') as stderr:
  proc=subprocess.Popen([str(a.binary.resolve())],stdout=stdout,stderr=stderr,env=dict(os.environ,SALIMON_SAVE_DIR=str(save.resolve()),SALIMON_REFERENCE_SIZE='1',RUST_LOG='info,salimon_client::app=trace'))
  def event(*args,move=False):
   r=subprocess.run([str((a.move_helper if move else a.helper).resolve()),str(proc.pid),*map(str,args)],capture_output=True,text=True,timeout=8)
   records.append({'input':args,'unix':time.time(),'returncode':r.returncode,'stdout':r.stdout,'stderr':r.stderr})
   if r.returncode:raise RuntimeError(r.stderr)
   return r.stdout
  def key(code,ms=100):event('key',code,ms)
  def snap(label,enter_pause=True):
   generation,_,_=newest()
   if enter_pause:key(53)
   requested=time.time();key(45);deadline=time.monotonic()+6
   while time.monotonic()<deadline:
    g,d,b=newest()
    if g>generation and d['saved_at_unix_seconds']>=int(requested):
     (out/(label+'.sav')).write_bytes(b)
     w=json.loads(event('window').splitlines()[0]);records.append({'window':w,'label':label})
     subprocess.run(['screencapture','-x','-l',str(w['kCGWindowNumber']),str(out/(label+'.png'))],check=True,timeout=8)
     return d
    time.sleep(.1)
   raise RuntimeError('fresh save missing '+label)
  try:
   deadline=time.monotonic()+30
   while 'renderer ready:' not in (out/'stderr.log').read_text():
    if proc.poll() is not None:raise RuntimeError('launch failed')
    if time.monotonic()>deadline:raise TimeoutError('renderer')
    time.sleep(.1)
   event('focus');before=snap('before')
   if a.external:
    event('move',-1880,200,move=True);event('focus');time.sleep(.5)
   key(36);event('focus')
   w=json.loads(event('window').splitlines()[0])
   video=subprocess.Popen(['screencapture','-x','-v','-V','22','-l',str(w['kCGWindowNumber']),str(out/'normal-play.mov')],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
   event('mouse',120,35);positive=snap('mouse-positive');key(36);event('focus')
   event('mouse',-120,-35);returned=snap('mouse-return');key(36);event('focus')
   key(13,2400);key(1,2400) if case!='moving' else key(2,900)
   time.sleep(.7)
   after=snap('after-controls');time.sleep(.5);idle=snap('paused-repeat',False)
   c=lambda d:d['state']['physical']['character']
   checks['positive_mouse_received']=abs(c(positive)['yaw_radians']-c(before)['yaw_radians'])>.01
   checks['inverse_mouse_restores']=abs(c(returned)['yaw_radians']-c(before)['yaw_radians'])<1e-6 and abs(c(returned)['pitch_radians']-c(before)['pitch_radians'])<1e-6
   checks['normal_clock_advanced']=after['simulated_time']!=before['simulated_time']
   checks['paused_state_frozen']=after['state']==idle['state'] and after['simulated_time']==idle['simulated_time']
   checks['native_movement_admitted']=c(after)['position']!=c(returned)['position'] if case!='moving' else after['state']['physical']['ship']['pose']['orientation']!=returned['state']['physical']['ship']['pose']['orientation']
   key(7);proc.wait(timeout=12);checks['clean_exit']=proc.returncode==0
   vo,ve=video.communicate(timeout=20);(out/'recording.log').write_text(vo+ve);checks['recording_created']=video.returncode==0 and (out/'normal-play.mov').stat().st_size>0
  finally:
   if proc.poll() is None:proc.kill();proc.wait()
   if video and video.poll() is None:video.terminate();video.communicate(timeout=5)
   (out/'input.json').write_text(json.dumps(records,indent=2)+'\n')
   (out/'checks.json').write_text(json.dumps({'checks':checks,'human_temporal_review':'pending','fixture':True,'fixed_step':False},indent=2)+'\n')
 results.append({'case':case,'checks':checks});print(json.dumps(results[-1]),flush=True)
(a.output/'results.json').write_text(json.dumps(results,indent=2)+'\n')
(a.output/'provenance.json').write_text(json.dumps({'binary_sha256':hashlib.sha256(a.binary.read_bytes()).hexdigest(),'source':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'external':a.external,'human_review':'pending'},indent=2)+'\n')

raise SystemExit(0 if all(all(r["checks"].values()) for r in results) else 1)
