"""Matched frozen native reference frames; never normal-play or optical FPS signoff."""
import argparse,hashlib,json,math,os,re,subprocess,sys,time
from pathlib import Path
sys.path.insert(0,str(Path('scripts').resolve()))
from salimon_test import Game,native_capture
from e2e_artifacts import Artifacts
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--binary',type=Path,required=True);p.add_argument('--helper',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
p.add_argument('--sample-seconds',type=float,default=3);p.add_argument('--repeats',type=int,default=3)
a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
os.environ.update(SALIMON_REFERENCE_SIZE='1',RUST_LOG='info,salimon_client::frame=trace,salimon_client::app=debug')
art=Artifacts({'workload':'frozen-scout-reference-v1','fixed_step':True,'normal_simulation':False},a.output,['native']);art.open_logs();rows=[];events=[]
from frame_data import frames,distribution,source_state,validate_window
try:
 with Game([str(a.binary.resolve())],{'scenario':'core-restart','seed':0,'step_ms':100},time.monotonic()+360,art,native_input=False) as g:
  def focus():
   r=subprocess.run([str(a.helper.resolve()),str(g.process.pid),'focus'],capture_output=True,text=True,timeout=8);events.append({'pid':g.process.pid,'stdout':r.stdout,'stderr':r.stderr,'returncode':r.returncode,'unix':time.time()})
   assert r.returncode==0 and f'frontmost={g.process.pid} target={g.process.pid}' in r.stdout
  focus();assert g.request({'op':'inspect'})['native_input_isolated'] is True
  for powered in [False,True]:
   if powered:
    g.request({'op':'interact'});g.request({'op':'step','frames':1})
   for view in ['cockpit','core','fabricator','aft-door']:
    g.request({'op':'scout_reference','view':view})
    for trial in range(a.repeats):
     focus();before=g.request({'op':'inspect'});assert before['ship']['power_available']==powered
     time.sleep(.75);start=len((art.directory/'stderr.log').read_text().splitlines());began=time.monotonic();time.sleep(a.sample_seconds)
     end=len((art.directory/'stderr.log').read_text().splitlines());after=g.request({'op':'inspect'});raw=(art.directory/'stderr.log').read_text().splitlines()[start:end];samples=frames(raw)
     row={'view':view,'powered':powered,'repeat':trial,'wall_seconds':time.monotonic()-began,'log_lines':[start,end],'before':before,'after':after,'samples':samples};rows.append(row)
     (a.output/'windows.json').write_text(json.dumps(rows,indent=2)+'\n')
     validate_window(row)
     row['metrics']={k:distribution(samples,k) for k in ['interval_ms','present_wall_ms','cpu_update_ms','cpu_render_ms','gpu_latest_ms','backlog_ms']}
     print(view,powered,trial,len(samples),flush=True)
    assert native_capture(g,art,f'{view}-{powered}',time.monotonic()+10)['status']=='captured'
  context=g.request({'op':'inspect'})['visual_context']
finally:
 (a.output/'windows.json').write_text(json.dumps(rows,indent=2)+'\n');(a.output/'focus.json').write_text(json.dumps(events,indent=2)+'\n');art.close()
(a.output/'summary.json').write_text(json.dumps({'schema':1,'workload':'frozen-scout-reference-v1','binary_sha256':hashlib.sha256(a.binary.read_bytes()).hexdigest(),'source':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'context':context,'normal_simulation':False,'cpu_frame_log_rounding_ms':.001,'logging_overhead_included':True,'windows':[{k:v for k,v in row.items() if k not in ['samples','before','after']} for row in rows],'approval_granted':False},indent=2)+'\n')
