import datetime, hashlib, json, os, platform, subprocess, sys, time
from pathlib import Path
root=Path.cwd()
out=root/'artifacts/m1-qualification'
base={**os.environ,'WGPU_BACKEND':'metal','SALIMON_SAVE_DIR':str(out/'isolated-saves'),'SALIMON_SETTINGS_FILE':str(out/'isolated-settings.json')}
binary=str(root/'target/release/salimon-client')
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
manifest={'source_revision':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'runtime_parent':subprocess.check_output(['git','rev-parse','HEAD^'],text=True).strip(),'binary_sha256':digest(Path(binary)),'platform':platform.platform(),'python':sys.version,'commands':[]}
for name,argv in [('rustc',['rustc','--version']),('model',['sysctl','-n','hw.model']),('cpu',['sysctl','-n','machdep.cpu.brand_string']),('memory_bytes',['sysctl','-n','hw.memsize']),('power',['pmset','-g','custom'])]:
 manifest[name]=subprocess.check_output(argv,text=True).strip()
def run(name,argv,extra=None,timeout=1200):
 env={**base,**(extra or {})}
 start=time.monotonic(); row={'name':name,'argv':argv,'environment':{k:env[k] for k in ['WGPU_BACKEND','SALIMON_SAVE_DIR','SALIMON_SETTINGS_FILE',*(extra or {})]},'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
 print('START',name,flush=True)
 try:
  with (out/f'{name}.log').open('w') as f:
   p=subprocess.run(argv,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=timeout)
  row['exit_code']=p.returncode
 except subprocess.TimeoutExpired:
  row['exit_code']=None;row['error']='campaign command deadline exceeded'
 row['elapsed_seconds']=time.monotonic()-start
 manifest['commands'].append(row)
 (out/'commands.json').write_text(json.dumps(manifest,indent=2)+'\n')
 print('END',name,row['exit_code'],round(row['elapsed_seconds'],2),flush=True)
run('universe',['python3','scripts/verify_universe_expansion.py','--binary',binary,'--output',str(out/'universe')])
run('construction',['python3','scripts/salimon-test','suite','--group','scout-construction','--binary',binary,'--artifacts',str(out/'construction'),'--screenshot-command','["native"]','--summary'])
run('metal-corpus',['python3','scripts/model_visual_tests.py','capture','--binary',binary,'--captures',str(out/'metal-corpus')])
run('metal-temporal',['python3','scripts/model_visual_tests.py','compare','--temporal-only','--captures',str(out/'metal-corpus'),'--output',str(out/'metal-temporal')])
run('os-smoke',['python3','scripts/packaged_smoke.py','--binary',binary,'--artifacts',str(out/'os-smoke'),'--timeout','120','--screenshot-command',json.dumps(['swift',str(root/'scripts/macos_window.swift'),'{pid}','{path}'])])
run('runtime-release-tests',['cargo','test','--release','--locked','-p','salimon-client'],{'SALIMON_BALANCE_REPORT':str(out/'balance.json'),'SALIMON_AVAILABILITY_REPORT':str(out/'availability.json')})
run('audio-device',['cargo','test','--release','--locked','-p','salimon-client','native_continuous_output_582','--','--ignored','--nocapture'],{'SALIMON_AUDIO_EVIDENCE':str(out/'audio-device')})
run('audio-export',['cargo','test','--release','--locked','-p','salimon-client','export_environment_audio_627','--','--ignored','--nocapture'],{'SALIMON_AUDIO_EVIDENCE':str(out/'audio')})
