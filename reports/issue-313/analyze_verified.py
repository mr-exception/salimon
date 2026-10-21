import sys,json,hashlib
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_fragments import summarize,distribution
r=Path('artifacts/native-20261020/313-verified');groups={};invalid=[];hashes={}
for lane in ['cap500-active','cap500-extra','cap1000-active','mixed-active','cap-settled']:
 folder=r/lane/'raw';summary=json.loads((folder/'summary.json').read_text());invalid += [{'lane':lane,**f} for f in summary['failures']]
 focus=json.loads((r/lane/'focus.json').read_text());pids={x['pid'] for x in focus if x['returncode']==0 and f"frontmost={x['pid']} target={x['pid']}" in x['stdout']}
 accepted={c['raw_file'] for c in summary['cases']}
 for name in accepted:
  p=folder/name;j=json.loads(p.read_text());assert j['presentation']['pid'] in pids
  z=summarize(j);key=f"{j['config']['count']}-{j['config']['layout']}-{j['config']['initial_state']}"
  g=groups.setdefault(key,{'files':[],'windows':[],'accounting':j['accounting'],'renderer':j['renderer'],'moving':[],'nonmoving':[]})
  assert g['accounting']==j['accounting'] and g['renderer']==j['renderer']
  rel=str(p.relative_to(r));hashes[rel]=hashlib.sha256(p.read_bytes()).hexdigest();g['files'].append(rel)
  g['windows'].append({k:z[k] for k in ['frame_samples_retained','moving_samples','resting_samples','sleeping_observed']})
  for x in j['samples']:
   if x.get('frame_ms') is not None:g['moving' if x['moving_objects']>0 else 'nonmoving'].append(x)
for key,g in groups.items():
 for state in ['moving','nonmoving']:
  samples=g.pop(state);g[state+'_intervals']=len(samples)
  g[state+'_over_50ms']=sum(x['frame_ms']>50 for x in samples)
  g[state+'_metrics']={m:distribution([x[m] for x in samples if x.get(m) is not None]) for m in ['frame_ms','physics_total_ms','physics_adapter_ms','cpu_render_ms','gpu_latest_ms','game_backlog_ms','matrix_allocations','projection_storage_bytes']}
 fs=g.pop('accounting')['fragments'];g['population']={'count':len(fs),'unique_ids':len({f['id'] for f in fs}),'actual_mass_kg':sum(f['mass_kg'] for f in fs),'radius_range_meters':[min(f['radius_meters'] for f in fs),max(f['radius_meters'] for f in fs)]}
 print(key,'moving',g['moving_intervals'],'frame',g['moving_metrics']['frame_ms'],'physics',g['moving_metrics']['physics_total_ms'],'nonmoving',g['nonmoving_metrics']['frame_ms'])
out={'binary_sha256':json.loads(Path('artifacts/native-20261020/build-verified/build-info.json').read_text())['sha256'],'production_isolation':'target-production; release physics coefficient regressions passed','groups':groups,'rejected_or_incomplete_cases':invalid,'raw_sha256':hashes,'acceptance':'descriptive; no new budget/owner approval/physical scanout claim'}
(r/'analysis.json').write_text(json.dumps(out,indent=2)+'\n')
