"""Bounded native input/Metal capture on the attached 75 Hz display; no FPS claim."""
import argparse,json,subprocess,sys,time
from pathlib import Path
sys.path.insert(0,str(Path('scripts').resolve()))
from salimon_test import Game,native_capture
from e2e_artifacts import Artifacts
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--binary',type=Path,required=True);p.add_argument('--helper',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
a=p.parse_args();art=Artifacts({'display':'attached external nominal75Hz','clock':'fixed E2E; interpolation disabled'},a.output,['native']);art.open_logs();records=[]
with Game([str(a.binary.resolve())],{'scenario':'flight-regression','seed':314,'step_ms':16},time.monotonic()+180,art) as g:
 def tick(seconds):
  end=time.monotonic()+seconds
  while time.monotonic()<end:
   start=time.monotonic();g.request({'op':'step','frames':1});time.sleep(max(0,.016-(time.monotonic()-start)))
 def event(*args):
  proc=subprocess.Popen([str(a.helper.resolve()),str(g.process.pid),*map(str,args)],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
  while proc.poll() is None:tick(.032)
  out,err=proc.communicate();records.append({'input':args,'returncode':proc.returncode,'stdout':out,'stderr':err});(art.directory/'observations.json').write_text(json.dumps(records,indent=2)+'\n')
  if proc.returncode:raise RuntimeError(err)
 def inspect(name):
  st=g.request({'op':'inspect'});records.append({'name':name,'state':st});(art.directory/'observations.json').write_text(json.dumps(records,indent=2)+'\n');return st
 def context(scale):
  end=time.monotonic()+8
  while time.monotonic()<end:
   st=g.request({'op':'inspect'})
   if st['visual_context']['scale_factor']==scale:return st
   time.sleep(.05)
  raise RuntimeError('target display scale was not observed')
 def shot(name):
  r=native_capture(g,art,name,time.monotonic()+10)
  if r['status']!='captured':raise RuntimeError(str(r))
 checks={}
 try:
  event('move',40,100);event('focus');event('key',122,40)
  event('move',-1880,200);context(1.0)
  g.request({'op':'window','action':'resize','width':1280,'height':800});event('focus')
  if g.request({'op':'inspect'})['paused']:event('key',36,40)
  initial=inspect('external-75hz-initial');shot('external-75hz-initial')
  event('key',0,700);steered=inspect('external-75hz-steered');shot('external-75hz-steered')
  event('mouse',120,35);looked=inspect('external-75hz-look');shot('external-75hz-look')
  def direction(st):
   q=st['player'];return [q['look_target_meters'][i]-q['eye_position_meters'][i] for i in range(3)]
  checks={'external_scale1':initial['visual_context']['scale_factor']==1,'native_steering':steered['ship']['orientation']!=initial['ship']['orientation'],
   'native_mouse_delivery':max(abs(x-y) for x,y in zip(direction(steered),direction(looked)))>.05}
 finally:
  event('key',122,40);event('move',40,100);context(2.0)
  restored=inspect('returned-to-built-in-60hz')
  checks['returned_primary_scale2']=restored['visual_context']['scale_factor']==2
  (art.directory/'checks.json').write_text(json.dumps({'checks':checks,'coverage':{'frame_rate_or_live_interpolation_certified':False}},indent=2)+'\n')
art.close()
status='passed' if all(checks.values()) else 'partial'
print(json.dumps({'status':status,'checks':checks,'artifacts':str(art.directory)}))
raise SystemExit(0 if status=='passed' else 2)
