import json,sys,time,os
from pathlib import Path
sys.path.insert(0,str(Path('scripts').resolve()))
from salimon_test import Game
from e2e_artifacts import Artifacts
root=Path('artifacts/native-m1/input-'+sys.argv[1]).resolve();root.mkdir(exist_ok=True); (root/'stop').unlink(missing_ok=True)
art=Artifacts({'native_input_probe':sys.argv[1]},root,['native']);art.open_logs()
setup={'scenario':sys.argv[1],'seed':198,'step_ms':16}
with Game([str(Path('artifacts/SalimonAcceptance.app/Contents/MacOS/salimon-client').resolve())],setup,time.monotonic()+300,art) as g:
 print('READY '+str(art.directory),flush=True)
 log=(art.directory/'observations.jsonl').open('w');last=0;seq=0
 while not (root/'stop').exists():
  start=time.monotonic();g.request({'op':'step','frames':1})
  if start-last>=0.5:
   st=g.request({'op':'inspect'});log.write(json.dumps({'elapsed':start,'state':st})+'\n');log.flush();last=start
  if (root/'capture').exists():
   (root/'capture').unlink();seq+=1
   p=art.directory/f'native-input-{seq}.png'
   g.request({'op':'capture','path':str(p)})
  time.sleep(max(0,0.016-(time.monotonic()-start)))
 log.close()
art.close()
