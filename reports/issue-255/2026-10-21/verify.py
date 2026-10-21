"""Verify serial native admission/accounting, active samples and unchanged budgets."""
import argparse,gzip,json,sys
from pathlib import Path
REPO=next(p for p in Path(__file__).resolve().parents if (p/'Cargo.toml').exists())
sys.path.insert(0,str(REPO/'scripts'))
sys.path.insert(0,str(REPO/'reports/issue-255'))
from benchmark_fragments import summarize,distribution,compare
from analyze import analyze
import importlib.util
spec=importlib.util.spec_from_file_location('proposal',REPO/'reports/issue-288/verify.py');proposal=importlib.util.module_from_spec(spec);spec.loader.exec_module(proposal)
def load(p):
 return json.loads(gzip.decompress(p.read_bytes()) if p.suffix=='.gz' else p.read_bytes())
def rows(root,lane):
 d=root/lane;s=load(d/'raw/summary.json');focus=load(d/'focus.json');pids={f['pid'] for f in focus if f['returncode']==0 and f"frontmost={f['pid']} target={f['pid']}" in f['stdout']}
 result=[];paths=[]
 for c in s['cases']:
  p=d/'raw'/c['raw_file']
  if not p.exists():p=p.with_suffix(p.suffix+'.gz')
  if p in paths:continue
  paths.append(p)
  raw=load(p)
  for run in raw.get('runs',[raw]):
   summarize(run);assert run['presentation']['pid'] in pids
   assert run['mode']=='native_wall_clock' and run['sampling_stride']==1
   result.append(run)
 return s,paths,result

def verify(root):
 out={'scope':'legacy onboard 500 x .032kg, separate from #313 current spheres','accepted':{},'failed_or_incomplete':{}}
 allruns={}
 for lane in ['empty-stable','settled','active','empty-launch','stress','historical-before','historical-after']:
  summary,paths,runs=rows(root,lane);allruns[lane]=runs
  out['accepted'][lane]={'binary_sha256':summary['binary_sha256'],'windows':[summarize(x) for x in runs]}
  if summary['failures']:out['failed_or_incomplete'][lane]=summary['failures']
  if lane=='active':out['active_analysis']=analyze(paths)
 for lane,n in [('empty-stable',3),('settled',3),('stress',1)]:
  assert len(allruns[lane])==n
  assert all(summarize(x)['sleeping_observed'] and summarize(x)['frame_samples_retained']>=100 for x in allruns[lane])
 # Validate exact current population and same physical settings, without equating historical physics.
 active=allruns['active'];empty=allruns['empty-launch'];assert len(active)==10 and len(empty)==10
 for x in active:
  assert x['accounting']==active[0]['accounting'];assert x['renderer']==active[0]['renderer']
 fragments=active[0]['accounting']['fragments'];assert len(fragments)==500 and len({f['id'] for f in fragments})==500
 assert abs(sum(f['mass_kg'] for f in fragments)-16)<1e-8
 assert all(abs(f['mass_kg']-.032)<1e-12 for f in fragments)
 prefix=[]
 for a,e in zip(active,empty,strict=True):
  assert dict(a['config'],count=0)==e['config'] and a['renderer']==e['renderer']
  complete=[x for x in a['samples'] if x['frame_ms'] is not None];ids=[i for i,x in enumerate(complete) if x['moving_objects']>0]
  assert ids==list(range(len(ids))),'moving intervals are not a contiguous launch prefix'
  er=[x for x in e['samples'] if x['frame_ms'] is not None];assert len(er)>=len(ids);prefix.extend(er[:len(ids)])
 out['matched_empty_prefix']={'intervals':len(prefix),'frame_ms':distribution([x['frame_ms'] for x in prefix]),'frames_over_50ms':sum(x['frame_ms']>50 for x in prefix)}
 before=load(root/'historical-before/raw/summary.json');after=load(root/'historical-after/raw/summary.json')
 assert len(allruns['historical-before'])==len(allruns['historical-after'])==3
 for a,b in zip(allruns['historical-before'],allruns['historical-after'],strict=True):assert a['accounting']==b['accounting'] and a['renderer']==b['renderer']
 out['fresh_historical_relative_failures']=compare(after,before,.2)
 budget=load(REPO/'reports/issue-288/proposed-budget.json')
 ev={'stages':{'final':out['active_analysis']},'matched_empty_prefix':out['matched_empty_prefix'],'whole_window_relative_gate_failures':out['fresh_historical_relative_failures']}
 out['proposal_evaluation']=proposal.evaluate(ev,budget)
 out['current_cadence']={lane:[{'frame_p95_ms':summarize(x)['metrics']['frame_ms']['p95'],'within_17_5ms_guidance':summarize(x)['metrics']['frame_ms']['p95']<=17.5,'sleeping':summarize(x)['sleeping_observed'],'frames':summarize(x)['frame_samples_retained']} for x in allruns[lane]] for lane in ['empty-stable','settled','stress']}
 out['acceptance_granted']=False
 return out
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--root',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.write_text(json.dumps(verify(a.root),indent=2)+'\n')
