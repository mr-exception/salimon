import subprocess,time,json
from pathlib import Path
root=Path('artifacts/native-20261022/455/diagnostic');root.mkdir(exist_ok=False)
command=['artifacts/native-20261022/455/build-optimized/salimon-client','--benchmark','--count','1000','--layout','current-mixed','--state','active','--camera','facing','--mode','physics','--seed','155','--warmup-seconds','0','--sample-seconds','3','--minimum-samples','100']
(root/'command.json').write_text(json.dumps(command,indent=2)+'\n');records=[]
with (root/'stdout.json').open('w') as out,(root/'stderr.log').open('w') as err:
 p=subprocess.Popen(command,stdout=out,stderr=err)
 try:
  for i in range(3):
   time.sleep(5)
   if p.poll() is not None:break
   r=subprocess.run(['sample',str(p.pid),'3','-file',str(root/f'stack-{i}.txt')],capture_output=True,text=True,timeout=10)
   records.append({'index':i,'returncode':r.returncode,'stdout':r.stdout,'stderr':r.stderr})
 finally:
  incomplete=p.poll() is None
  if incomplete:p.kill()
  p.wait(timeout=5)
(root/'result.json').write_text(json.dumps({'diagnostic_only':True,'instrumentation':'three independent 3s macOS sample windows; not native frame evidence','bounded_incomplete':incomplete,'exit':p.returncode,'samples':records},indent=2)+'\n')
