import sys,json
from pathlib import Path
sys.path.insert(0,'scripts')
from salimon_test import run,result_summary
root=Path('artifacts/native-20261020/313/visual');root.mkdir(parents=True,exist_ok=True)
results=[]
for name,binary in [('before','build-damping-before'),('current','build-current')]:
 s={'setup':{'scenario':'landed-earth','seed':303,'step_ms':16,'fragment_count':50,'fragment_layout':'current-mixed'},'timeout_ms':120000,'steps':[{'action':{'op':'look','dx':1428,'dy':220}},{'screenshot':'initial'}]}
 for n in [8,16,32]:s['steps'] += [{'action':{'op':'step','frames':n}},{'assert':{'path':'world.fragment_count','equals':50}},{'screenshot':f'after-{n}-more-ticks'}]
 p=root/f'{name}-damping.json';p.write_text(json.dumps(s,indent=2)+'\n')
 r=run(p,[str(Path('artifacts/native-20261020')/binary/'salimon-client')],root/name,['native']);results.append(result_summary(r));print(json.dumps(result_summary(r)),flush=True)
for count in [500,1000]:
 s={'setup':{'scenario':'landed-earth','seed':303,'step_ms':16,'fragment_count':count,'fragment_layout':'current-cap','fragment_settle_steps':1200},'timeout_ms':120000,'steps':[{'assert':{'path':'world.fragment_count','equals':count}},{'assert':{'path':'world.fragment_mass_kg','approx':count*10,'tolerance':1e-6}},{'action':{'op':'look','dx':1428,'dy':220}},{'screenshot':f'current-cap-{count}-settled'}]}
 p=root/f'cap-{count}.json';p.write_text(json.dumps(s,indent=2)+'\n')
 r=run(p,[str(Path('artifacts/native-20261020/build-current/salimon-client').resolve())],root/f'cap-{count}',['native']);results.append(result_summary(r));print(json.dumps(result_summary(r)),flush=True)
(root/'summary.json').write_text(json.dumps(results,indent=2)+'\n')
