import sys,os,time,json,re,subprocess,hashlib
from pathlib import Path
sys.path.insert(0,str(Path('scripts').resolve()))
from salimon_test import Game
from e2e_artifacts import Artifacts
os.environ['SALIMON_REFERENCE_SIZE']='1'
os.environ['RUST_LOG']='salimon_client::frame=debug'
results=[]
cases=[('baseline','artifacts/terrain-baseline/salimon-baseline','landed-earth'),('flat-control','artifacts/terrain-implementation/salimon-client','landed-earth')]+[(b,'artifacts/terrain-implementation/salimon-client','terrain-'+b) for b in ['earth','moon','mars']]
for label,binary,scene in cases:
 a=Artifacts({'label':label},'artifacts/terrain-reference');a.open_logs()
 with Game([str(Path(binary).resolve())],{'scenario':scene,'seed':181,'step_ms':16},time.monotonic()+90,a) as g:
  for i in range(30):
   time.sleep(.1); state=g.request({'op':'inspect'})
  start_frame=None
  log=(a.directory/'stderr.log').read_text()
  frames=re.findall(r'frame=(\d+)',log)
  if frames:start_frame=int(frames[-1])
  if label!='baseline':g.request({'op':'performance_reset'})
  started=time.monotonic()
  for i in range(100):time.sleep(.1);state=g.request({'op':'inspect'})
  rss=int(subprocess.check_output(['ps','-o','rss=','-p',str(g.process.pid)]))*1024
  result={'label':label,'scenario':scene,'warmup_seconds':3,'sample_seconds':time.monotonic()-started,'rss_bytes':rss,'binary_sha256':hashlib.sha256(Path(binary).read_bytes()).hexdigest(),'state':state,'log_directory':str(a.directory),'start_frame':start_frame}
 a.close()
 if label=='baseline':
  fields={k:[] for k in ['interval','cpu_update','cpu_render','gpu']}
  for line in (a.directory/'stderr.log').read_text().splitlines():
   m=re.search(r'frame=(\d+) interval_ms=([\d.]+).*cpu_update_ms=([\d.]+) cpu_render_ms=([\d.]+) gpu_time=Measured\(([\d.]+)(µs|ms|ns|s)\)',line)
   if m and int(m[1])>start_frame:
    for k,v in zip(fields,[float(m[2]),float(m[3]),float(m[4]),float(m[5])*{'µs':.001,'ms':1,'ns':.000001,'s':1000}[m[6]]]):fields[k].append(v)
  result['timings']={k:{'samples':len(v),'mean_ms':sum(v)/len(v),'p95_ms':sorted(v)[int(.95*len(v))],'max_ms':max(v)} if v else None for k,v in fields.items()}
 results.append(result)
 Path('artifacts/terrain-reference/measurements.json').write_text(json.dumps(results,indent=2))
 print(label,json.dumps(result.get('timings',state.get('performance'))),flush=True)
