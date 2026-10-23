"""Reanalyse native raw windows, preserving failed/incomplete populations."""
import argparse, gzip, hashlib, json, math, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[2]/'scripts'))
from benchmark_fragments import summarize

def read(path):
    return json.loads(gzip.decompress(path.read_bytes()) if path.suffix=='.gz' else path.read_bytes())

def dist(values):
    values=sorted(v for v in values if v is not None)
    if not values:return None
    return {'p95':values[math.ceil(.95*len(values))-1],'max':values[-1]}

def analyze(root):
    summary=read(root/'summary.json');groups={}
    focus=read(root.parent/(root.name+'-focus.json'))
    pids={f['pid'] for f in focus if f['returncode']==0 and f"frontmost={f['pid']} target={f['pid']}" in f['stdout']}
    for path in sorted(list(root.glob('*.stdout'))+list(root.glob('*.stdout.gz'))):
        if not path.stat().st_size:continue
        raw=read(path)
        runs=raw.get('runs',[raw])
        for run in runs:
            checked=summarize(run)
            assert run['accounting_preserved'] and run['presentation']['foreground_visible']
            assert run['presentation']['pid'] in pids
            assert run['mode']=='native_wall_clock'
            assert run['sampling_stride']==1
            config=run['config'];key=f"{config['count']}-{config['layout']}-{config['initial_state']}-{config['camera']}"
            g=groups.setdefault(key,{'windows':[],'moving':[],'quiet_awake':[],'sleeping':[]})
            assert len(run['accounting']['fragments'])==config['count']
            assert len({p['id'] for p in run['accounting']['fragments']})==config['count']
            assert all(p['mass_kg']>0 and p['radius_meters']>0 for p in run['accounting']['fragments'])
            if 'accounting' in g:assert g['accounting']==run['accounting']
            else:g['accounting']=run['accounting']
            g['windows'].append({'file':path.name,'presentation':run['presentation'],'config':config,'renderer':run['renderer'],'accounting_count':len(run['accounting']['fragments']),'frames':checked['frame_samples_retained']})
            for sample in run['samples']:
                if sample['frame_ms'] is None:continue
                mode='moving' if sample['moving_objects']>0 else 'sleeping' if sample['sleeping_objects']==config['count'] else 'quiet_awake'
                g[mode].append(sample)
    for g in groups.values():
        inventory=g.pop('accounting')
        g['accounting_sha256']=hashlib.sha256(json.dumps(inventory,sort_keys=True).encode()).hexdigest()
        g['mass_kg']=sum(p['mass_kg'] for p in inventory['fragments'])
        for mode in ['moving','quiet_awake','sleeping']:
            samples=g[mode];g[mode]={'completed_intervals':len(samples),'sufficient_active_pool':len(samples)>=100 if mode=='moving' else None,'metrics':{k:dist([s[k] for s in samples]) for k in ['frame_ms','physics_total_ms','physics_contact_ms','physics_integration_ms','physics_activation_ms','physics_adapter_ms','cpu_render_ms','gpu_latest_ms','game_backlog_ms','substeps','solver_passes','pair_visits','integrated_objects']},'frames_over_50ms':sum(s['frame_ms']>50 for s in samples)}
    return {'schema':1,'groups':groups,'failures':summary['failures'],'approval_granted':False,'note':'Descriptive timings; submitted-present intervals are not optical scanout. No stress limit or 60 FPS signoff.'}
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--root',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    a.output.write_text(json.dumps(analyze(a.root),indent=2)+'\n')
