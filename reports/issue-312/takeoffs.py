from pathlib import Path
import sys,json,os
sys.path.insert(0,'scripts')
from salimon_test import run,result_summary
root=Path('artifacts/native-20261020/312/takeoffs-current');root.mkdir(parents=True,exist_ok=True)
results=[]
for name in ['earth','moon','mars']:
 for count in [0,500]:
  s=json.loads(Path(f'scenarios/terrain-{name}-touchdown.json').read_text())
  s['timeout_ms']=240000
  s['steps']=[step for step in s['steps'] if step.get('action',{}).get('op') != 'interact' and step.get('assert',{}).get('path') != 'player.location']
  if count:s['setup'].update(fragment_count=count,fragment_layout='sphere-deck',fragment_settle_steps=1200)
  s['steps'] += [{'action':{'op':'landing'}},{'assert':{'path':'ship.flight_state','equals':f'AssistedTakeoff {{ body: {name.title()} }}'}},{'action':{'op':'landing'}},{'assert':{'path':'ship.flight_state','equals':f'AssistedTakeoff {{ body: {name.title()} }}'}},{'action':{'op':'step','frames':375}},{'assert':{'path':'ship.flight_state','equals':'Flying'}},{'assert':{'path':'ship.speed_meters_per_second','equals':0}},{'assert':{'path':'ship.thruster_percentage','equals':0}},{'screenshot':f'{name}-{count}-handoff'}]
  p=root/f'{name}-{count}.json';p.write_text(json.dumps(s,indent=2)+'\n')
  r=run(p,[str(Path('artifacts/native-20261020/build/salimon-client').resolve())],root/f'{name}-{count}',['native']);results.append(result_summary(r));print(json.dumps(result_summary(r)),flush=True)
(root/'summary.json').write_text(json.dumps(results,indent=2)+'\n')
