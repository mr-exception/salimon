import json,sys,hashlib
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_fragments import summarize,distribution
root=Path('artifacts/native-20261020/313')
lanes={'native-cap-active':None,'native-mixed-active':500,'native-cap-settled':500,'serial-cap1000-active':None,'final-cap500-active':None,'final-cap1000-settled':None}
groups={};inputs={};failures=[]
for lane,only_count in lanes.items():
 for p in sorted((root/lane/'raw').glob('*.stdout')):
  try:j=json.loads(p.read_text())
  except ValueError:
   failures.append({'file':str(p.relative_to(root)),'reason':'empty/incomplete raw output'});continue
  if only_count is not None and j['config']['count']!=only_count:continue
  s=summarize(j)
  key=f"{j['config']['count']}-{j['config']['layout']}-{j['config']['initial_state']}"
  g=groups.setdefault(key,{'files':[],'window_summaries':[],'moving':[],'resting':[],'accounting':j['accounting'],'renderer':j['renderer']})
  assert g['renderer']==j['renderer']
  assert g['accounting']==j['accounting']
  rel=str(p.relative_to(root));inputs[rel]=hashlib.sha256(p.read_bytes()).hexdigest();g['files'].append(rel)
  g['window_summaries'].append({k:s[k] for k in ['frame_samples_retained','moving_samples','resting_samples','sleeping_observed','metrics']})
  for sample in j['samples']:
   if sample.get('frame_ms') is not None:g['moving' if sample['moving_objects']>0 else 'resting'].append(sample)
for key,g in groups.items():
 for lane in ['moving','resting']:
  rows=g.pop(lane);g[lane+'_completed_intervals']=len(rows)
  metrics=['frame_ms','physics_total_ms','physics_adapter_ms','cpu_render_ms','gpu_latest_ms','game_backlog_ms','matrix_allocations','projection_storage_bytes','substeps']
  g[lane+'_distributions']={k:distribution([r[k] for r in rows if r.get(k) is not None]) for k in metrics}
  g[lane+'_over_50ms']=sum(r['frame_ms']>50 for r in rows)
 p=g.pop('accounting')['fragments'];g['population']={'count':len(p),'actual_mass_kg':sum(f['mass_kg'] for f in p),'radius_range_meters':[min(f['radius_meters'] for f in p),max(f['radius_meters'] for f in p)],'unique_ids':len({f['id'] for f in p})}
print(json.dumps({k:{'moving_intervals':g['moving_completed_intervals'],'moving_frame':g['moving_distributions']['frame_ms'],'resting_frame':g['resting_distributions']['frame_ms']} for k,g in groups.items()},indent=2))
(root/'analysis.json').write_text(json.dumps({'source_revision':'fc634bf','binary_sha256':json.loads(Path('artifacts/native-20261020/build-current/build-info.json').read_text())['sha256'],'groups':groups,'raw_sha256':inputs,'incomplete_inputs':failures,'excluded':['overlapping rows recorded in 313-overlap-rejection.json','serial-cap500-active focus failures','serial-cap1000-settled focus failure','native-cap-settled/1000 interrupted during rejection'],'acceptance':'descriptive native evidence; no new budget or 60 FPS approval'},indent=2)+'\n')
