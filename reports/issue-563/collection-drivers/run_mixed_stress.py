import datetime,json,os,subprocess,time
from pathlib import Path
root=Path.cwd();out=root/'artifacts/m1-qualification'
cmd=['python3','scripts/benchmark_fragments.py','--binary',str(root/'target/release/salimon-client'),'--mode','native','--machine-label','Apple-M1-iMac-16GB-macOS26.2-Metal-1920x1080-scale2-AC','--counts','1000','--layouts','current-mixed','--states','active','--cameras','facing','--repeats','1','--warmup-seconds','0','--sample-seconds','3','--minimum-samples','100','--timeout','90','--output',str(out/'mixed-stress')]
manifest={'argv':cmd,'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'focus_method':'PID-scoped System Events frontmost, no gameplay input','focus_attempts':[]}
started=time.monotonic()
with (out/'mixed-stress-driver.log').open('w') as log:
 p=subprocess.Popen(cmd,stdout=log,stderr=subprocess.STDOUT,env={**os.environ,'WGPU_BACKEND':'metal'})
 seen=set()
 while p.poll() is None:
  children=subprocess.run(['pgrep','-P',str(p.pid)],capture_output=True,text=True).stdout.split()
  for pid in children:
   if pid in seen:continue
   name=subprocess.run(['ps','-p',pid,'-o','comm='],capture_output=True,text=True).stdout.strip()
   if not name.endswith('salimon-client'):continue
   # Allow the native shell to create the window, then explicitly activate only this PID.
   time.sleep(1)
   result=subprocess.run(['osascript','-e',f'tell application "System Events" to set frontmost of first application process whose unix id is {int(pid)} to true'],capture_output=True,text=True,timeout=10)
   manifest['focus_attempts'].append({'pid':int(pid),'exit_code':result.returncode,'error':result.stderr})
   seen.add(pid)
  time.sleep(.2)
manifest['exit_code']=p.returncode;manifest['elapsed_seconds']=time.monotonic()-started
(out/'mixed-stress-command.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('stress exit',p.returncode,flush=True)
