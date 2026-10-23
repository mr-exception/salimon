"""Serial CPU stress evidence; no rendering/FPS or hardware budget admission."""
import argparse,hashlib,json,math,os,platform,subprocess,time
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
results=[]
for count in [500,1000]:
 for mixed in [False,True]:
  for settled in [False,True]:
   name=f'{count}-{"mixed" if mixed else "cap"}-{"natural" if settled else "moving"}'
   path=(a.output/(name+'.jsonl')).resolve();timeout=45 if settled else 120
   env=dict(os.environ,SALIMON_STRESS_OUTPUT=str(path),SALIMON_STRESS_COUNT=str(count),SALIMON_STRESS_MIXED=str(int(mixed)),SALIMON_STRESS_SETTLED=str(int(settled)))
   command=[str(a.binary.resolve()),'sphere_workloads::current_sphere_stress_cpu','--ignored','--exact','--nocapture']
   began=time.monotonic()
   with (a.output/(name+'.log')).open('w') as log:
    try:r=subprocess.run(command,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=timeout);status='finished';code=r.returncode
    except subprocess.TimeoutExpired:status='timeout';code=None
   raw=[json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []
   samples=[s for s in raw if s['type']=='sample' and not s['preparation'] and (settled or s['moving']>0)]
   def dist(key):
    v=sorted(s[key] for s in samples)
    return None if not v else {'n':len(v),'median':v[len(v)//2],'p95':v[math.ceil(.95*len(v))-1],'p99':v[math.ceil(.99*len(v))-1],'max':v[-1]}
   row={'case':name,'process_status':status,'returncode':code,'timeout_seconds':timeout,'wall_seconds':time.monotonic()-began,'outcome':[s for s in raw if s['type']=='outcome'],'raw_samples':sum(s['type']=='sample' for s in raw),'metrics':{k:dist(k) for k in ['total_ms','contact_ms','grid_query_ms','grid_queries','activation_ms','substeps','pair_visits','contacts']}}
   results.append(row);print(name,status,row['outcome'],flush=True)
   (a.output/'summary.json').write_text(json.dumps({'binary_sha256':hashlib.sha256(a.binary.read_bytes()).hexdigest(),'platform':platform.platform(),'cpu':platform.machine(),'mode':'CPU fixed-16ms production adapter; historical layout checkout','rows':results},indent=2)+'\n')
