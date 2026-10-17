"""Promote local native evidence after collection; never approve image baselines."""
import gzip
import hashlib
import json
import shutil
from pathlib import Path
from PIL import Image, ImageChops, ImageDraw

root=Path('reports/issue-279')
src=Path('artifacts/native-279')

def archive_file(source,destination):
 destination.parent.mkdir(parents=True,exist_ok=True)
 if source.suffix in ('.json','.jsonl','.log'):
  with gzip.open(str(destination)+'.gz','wb') as stream:stream.write(source.read_bytes())
 else:shutil.copy2(source,destination)

results=dict(schema=1,fixtures=[],routes=[],comparisons=[])
for folder in sorted((src/'candidates').iterdir()):
 if not folder.is_dir():continue
 dst=root/'captures'/folder.name;dst.mkdir(parents=True,exist_ok=True)
 frames=sorted(folder.glob('*.png'))
 for p in frames:shutil.copy2(p,dst/p.name)
 run=next((folder/'runs').glob('*/result.json'));r=json.loads(run.read_text())
 archive_file(run,dst/'result.json')
 for log in ['stderr.log']:
  p=run.parent/log
  if p.exists():archive_file(p,dst/log)
 temporal=json.loads((src/'temporal'/folder.name/'summary.json').read_text())
 (dst/'temporal.json').write_text(json.dumps(temporal,indent=2)+'\n')
 assert r['status']=='passed' and temporal['passed']
 results['fixtures'].append(dict(name=folder.name,status=r['status'],steps=r['steps_completed'],
  frames=len(frames),temporal_comparisons=len(temporal['results']),changed_pixels=sum(x['changed_pixels'] for x in temporal['results']),
  hashes={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in frames}))
 for p in frames:
  old=Path('reports/issue-229/captures')/folder.name/p.name
  if not old.exists():raise ValueError(f'missing locked historical image {old}')
  a=Image.open(old).convert('RGB');b=Image.open(p).convert('RGB');assert a.size==b.size
  diff=ImageChops.difference(a,b)
  magnitude=ImageChops.lighter(ImageChops.lighter(*diff.split()[:2]),diff.split()[2])
  pixels=sum(magnitude.histogram()[1:])
  results['comparisons'].append(dict(fixture=folder.name,frame=p.name,before=str(old),after=str(dst/p.name),changed_pixels=pixels,coverage='descriptive style difference; no baseline approval'))
for group in ['terrain','mining-emission','pile','window-smoke']:
 for run in sorted((src/group).glob('*/result.json')):
  r=json.loads(run.read_text());name=Path(r['scenario']).stem;dst=root/'evidence'/name
  dst.mkdir(parents=True,exist_ok=True);assert r['status']=='passed'
  for p in run.parent.iterdir():
   if p.suffix=='.png' or p.name in ['result.json','stderr.log']:archive_file(p,dst/p.name)
  results['routes'].append(dict(name=name,status=r['status'],steps=r['steps_completed'],evidence=str(dst),frames=len(list(dst.glob('*.png')))))
for group in ['before-onboard','before-spheres','after-spheres']:
 dst=root/'benchmarks'/group;dst.mkdir(parents=True,exist_ok=True)
 for p in (src/group).iterdir():
  if p.suffix=='.stdout':
   with gzip.open(dst/(p.name+'.json.gz'),'wb') as f:f.write(p.read_bytes())
  else:shutil.copy2(p,dst/p.name)
 p=dst/'summary.json';r=json.loads(p.read_text())
 for c in r['cases']:c['raw_file']+='.json.gz'
 p.write_text(json.dumps(r,indent=2)+'\n')
 shutil.copy2(src/(group+'-focus.json'),root/'benchmarks'/(group+'-focus.json'))
for group in ['terrain-before','terrain-after','terrain-comparison']:
 shutil.copytree(src/group,root/group,dirs_exist_ok=True)
for source,dest in [('artifacts/native-279-capture.log','capture.log'),('artifacts/native-279-temporal.log','temporal.log'),('artifacts/native-279-terrain.log','terrain-routes.log'),('artifacts/native-279-mining.log','mining.log'),('artifacts/native-279-pile.log','pile.log'),('artifacts/native-279-terrain-before.log','terrain-before.log'),('artifacts/native-279-terrain-after.log','terrain-after.log'),('artifacts/build/native-255/build-info.json','build-info.json')]:shutil.copy2(source,root/dest)
(root/'visual-results.json').write_text(json.dumps(results,indent=2)+'\n')
# Review sheet: selected exact historical/current poses, before left, current right.
selections=[]
for name in ['exterior','interior','core','resources','ramp','moving']:
 frames=sorted((src/'candidates'/name).glob('*repeat-00.png'));selections.append((name,frames[len(frames)//2].name))
sheet=Image.new('RGB',(1280,len(selections)*425),(24,29,38));draw=ImageDraw.Draw(sheet)
for i,(name,filename) in enumerate(selections):
 for col,folder in enumerate([Path('reports/issue-229/captures'),root/'captures']):
  im=Image.open(folder/name/filename).convert('RGB');im.thumbnail((640,400));x=col*640;y=i*425
  draw.text((x+8,y+6),name+' / '+('before' if col==0 else 'current'),fill='white');sheet.paste(im,(x,y+25))
sheet.save(root/'before-after.jpg',quality=90)
print('Archived',sum(x['frames'] for x in results['fixtures']),'fixture images and',len(results['routes']),'routes')
