import subprocess,json,sys,os
from pathlib import Path
root=Path('artifacts/native-20261020');helper=Path('artifacts/native-20261020/input-313')
def matrix(label,binary,counts,layouts,states,sample=3,warm=0,repeats=1,timeout=90,settle=0):
 out=root/label;out.mkdir(parents=True,exist_ok=True)
 cmd=['python3','reports/issue-229/foreground_matrix.py','--helper',str(helper),'--records',str(out/'focus.json'),'--','python3','scripts/benchmark_fragments.py','--binary',str(binary),'--machine-label','apple-m1-imac-16gb-macos26.2','--counts',*map(str,counts),'--layouts',*layouts,'--states',*states,'--cameras','facing','--minimum-samples','100','--warmup-seconds',str(warm),'--sample-seconds',str(sample),'--repeats',str(repeats),'--timeout',str(timeout),'--output',str(out/'raw')]
 if settle:cmd+=['--settle-steps',str(settle)]
 (out/'command.json').write_text(json.dumps(cmd,indent=2)+'\n')
 print('START',label,flush=True)
 with (out/'runner.log').open('w') as f:r=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT,env=dict(os.environ,SALIMON_REFERENCE_SIZE='1',RUST_LOG='info'))
 print('END',label,'exit',r.returncode,flush=True)
 return r.returncode
if __name__=='__main__':
 b=root/'SalimonReview.app/Contents/MacOS/salimon-client'
 matrix('313/native-cap-active',b,[500,1000],['current-cap'],['active'])
 matrix('313/native-mixed-active',b,[500,1000],['current-mixed'],['active'],timeout=60)
 matrix('313/native-cap-settled',b,[500,1000],['current-cap'],['settled'],warm=2,sample=10,settle=1200,timeout=90)
