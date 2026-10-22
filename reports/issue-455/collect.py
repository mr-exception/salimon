import os,subprocess,json,time
from pathlib import Path
root=Path('artifacts/native-20261022/455');helper='artifacts/native-20261020/input-313'
before='artifacts/native-20261022/Salimon453Final.app/Contents/MacOS/salimon-client';after=str(root/'Salimon455.app/Contents/MacOS/salimon-client')
lanes=[('before-cap',before,['--counts','500','1000','--layouts','current-cap','--states','active','--warmup-seconds','0','--sample-seconds','3','--repeats','5','--timeout','90']),('after-cap',after,['--counts','500','1000','--layouts','current-cap','--states','active','--warmup-seconds','0','--sample-seconds','3','--repeats','5','--timeout','90']),('after-mixed',after,['--counts','500','1000','--layouts','current-mixed','--states','active','--warmup-seconds','0','--sample-seconds','3','--repeats','1','--timeout','120']),('after-sleeping',after,['--counts','500','1000','--layouts','current-cap','current-mixed','--states','settled','--settle-steps','300','--warmup-seconds','2','--sample-seconds','5','--repeats','1','--timeout','45'])]
results=[]
for name,binary,extra in lanes:
 command=['python3','reports/issue-229/foreground_matrix.py','--helper',helper,'--records',str(root/(name+'-focus.json')),'--','python3','scripts/benchmark_fragments.py','--binary',binary,'--machine-label','apple-m1-imac-16gb-macos26.2','--minimum-samples','100','--cameras','facing','--output',str(root/name),*extra]
 (root/(name+'-command.json')).write_text(json.dumps(command,indent=2)+'\n');print('START '+name,flush=True)
 with (root/(name+'.log')).open('w') as log:r=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,env=dict(os.environ,SALIMON_REFERENCE_SIZE='1',RUST_LOG='info'))
 results.append({'lane':name,'returncode':r.returncode});(root/'collection.json').write_text(json.dumps(results,indent=2)+'\n');print('END '+name+' '+str(r.returncode),flush=True)
