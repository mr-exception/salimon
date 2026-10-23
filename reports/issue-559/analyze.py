"""Compare deterministic prefixes; never turn missing samples into performance passes."""
import argparse,gzip,json,struct
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('before',type=Path);p.add_argument('after',type=Path);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
results=[]
for old in sorted(a.before.glob('*.jsonl')):
 new=a.after/old.name
 before=[json.loads(s) for s in old.read_text().splitlines()];after=[json.loads(s) for s in new.read_text().splitlines()]
 populations=lambda rows:[r for r in rows if r['type']=='population']
 assert populations(before)==populations(after)[:len(populations(before))],old.name
 samples=lambda rows:{(r['launch'],r['step']):r for r in rows if r['type']=='sample'}
 b=samples(before);c=samples(after)
 deterministic=['moving','sleeping','substeps','passes','pair_visits','contacts','grid_queries','preparation']
 assert all(k in c and all(v[key]==c[k][key] for key in deterministic) for k,v in b.items()),old.name
 states=lambda rows:[r for r in rows if r['type']=='state']
 # A partial timeout has no final state; compare every state actually completed.
 assert states(before)==states(after)[:len(states(before))],old.name
 results.append({'case':old.stem,'matched_sample_prefix':len(b),'matched_terminal_states':len(states(before)),'before_outcomes':[r for r in before if r['type']=='outcome'],'after_outcomes':[r for r in after if r['type']=='outcome']})
# Independently reconcile archived source inventory with the effective solver bits.
source=Path('reports/issue-455/evidence/native-data/after-mixed/native-500-current-mixed-active-facing.stdout.gz')
raw=json.loads(gzip.decompress(source.read_bytes()))['accounting']['fragments']
bits=lambda v:struct.unpack('<Q',struct.pack('<d',v))[0]
inv=populations([json.loads(s) for s in (a.after/'500-mixed-moving.jsonl').read_text().splitlines()])[0]['metadata']['population']
assert len(raw)==len(inv)==500
for x,y in zip(raw,inv):
 assert all(x[k]==y[k] for k in ['id','source','resource'])
 assert bits(x['mass_kg'])==y['solver_mass_bits'] and bits(x['radius_meters'])==y['solver_radius_bits']
a.output.write_text(json.dumps({'archived_inventory_exact':True,'cases':results},indent=2)+'\n')
