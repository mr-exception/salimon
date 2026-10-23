"""Validate complete matched frozen windows; summarize raw tails without FPS approval."""
import argparse,hashlib,json,struct,sys
from pathlib import Path
sys.path.insert(0,str(Path('reports/issue-483').resolve()))
from frame_data import distribution,source_state,validate_window
p=argparse.ArgumentParser(description=__doc__);p.add_argument('control',type=Path);p.add_argument('current',type=Path);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
lanes={}
for name,root in [('control',a.control),('current',a.current)]:
 summary=json.loads((root/'summary.json').read_text());rows=json.loads((root/'windows.json').read_text())
 assert summary['admission_profile']=='cloud-llvmpipe-1280' and len(rows)==24
 assert summary['hardware_gpu_metrics'] is None and summary['approval_granted'] is False
 events=json.loads((root/'focus.json').read_text());assert len(events)==25
 assert all(e['returncode']==0 and f"frontmost={e['pid']} target={e['pid']}" in e['stdout'] for e in events)
 for event in events:
  proof=json.loads(event['stdout'].splitlines()[-1]);assert proof['map_state']=='IsViewable' and proof['window']==proof['focused_window']
  geometry=dict(line.split('=',1) for line in proof['geometry'].splitlines())
  assert {k:int(geometry[k]) for k in ['X','Y','WIDTH','HEIGHT','SCREEN']}==dict(X=0,Y=0,WIDTH=1280,HEIGHT=800,SCREEN=0)
 for row in rows:
  validate_window(row,'cloud-llvmpipe-1280')
  assert row['before']['construction']==row['after']['construction']
  matches=list(root.glob(f"run-*/{row['view']}-{row['powered']}-{row['repeat']}.png"));assert len(matches)==1
  data=matches[0].read_bytes();assert struct.unpack('!II',data[16:24])==(1280,800)
  assert hashlib.sha256(data).hexdigest()==row['image_sha256']
 lanes[name]={(r['view'],r['powered'],r['repeat']):r for r in rows}
assert lanes['control'].keys()==lanes['current'].keys()
for key,before in lanes['control'].items():
 after=lanes['current'][key]
 assert source_state(before['before'])==source_state(after['before']),key
 assert before['before']['construction']==after['before']['construction'],key
results=[]
for view,powered in sorted({(k[0],k[1]) for k in lanes['control']}):
 row={'view':view,'powered':powered,'lanes':{}}
 for name,records in lanes.items():
  selected=[v for k,v in records.items() if k[:2]==(view,powered)]
  assert len({v['image_sha256'] for v in selected})==1
  root=a.control if name=='control' else a.current
  canonical=list(root.glob(f'run-*/{view}-{powered}.png'));assert len(canonical)==1
  assert hashlib.sha256(canonical[0].read_bytes()).hexdigest()==selected[0]['image_sha256']
  samples=[s for v in selected for s in v['samples']]
  row['lanes'][name]={'frames':len(samples),'metrics':{k:distribution(samples,k) for k in ['interval_ms','present_wall_ms','cpu_update_ms','cpu_render_ms','gpu_latest_ms','backlog_ms']},'image_sha256':selected[0]['image_sha256']}
 results.append(row)
a.output.write_text(json.dumps({'schema':1,'workload':'frozen-scout-reference-v1','windows_per_lane':24,'matched_source_and_context':True,'repeat_hashes_exact':True,'hardware_gpu_metrics':None,'normal_play_fps':None,'approval_granted':False,'rows':results},indent=2)+'\n')
