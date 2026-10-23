"""Parsing and fail-closed admission for frozen reference frames, never FPS approval."""
import re,math
pattern=re.compile(r'frame=(\d+) interval_ms=([\d.]+) present_wall_ms=([\d.]+) cpu_update_ms=([\d.]+) cpu_render_ms=([\d.]+) gpu_time=(.*?) ticks=(\d+) backlog_ms=([\d.]+) focused=(true|false) occluded=(true|false)')
def frames(lines):
 out=[]
 for line in lines:
  if 'salimon_client::frame]' not in line:continue
  m=pattern.search(line)
  if not m:raise RuntimeError('unrecognised frame record: '+line)
  n,interval,present,update,render,gpu,ticks,backlog,focused,occluded=m.groups();duration=re.fullmatch(r'Measured\(([\d.]+)(ns|µs|μs|ms|s)\)',gpu)
  if duration is None and gpu not in ['Pending','Unsupported']:raise RuntimeError('unrecognised GPU status: '+gpu)
  gpu_ms=None if duration is None else float(duration[1])*{'ns':1e-6,'µs':.001,'μs':.001,'ms':1,'s':1000}[duration[2]]
  out.append({'frame':int(n),'interval_ms':float(interval),'present_wall_ms':float(present),'cpu_update_ms':float(update),'cpu_render_ms':float(render),'gpu_latest_ms':gpu_ms,'gpu_status':gpu,'ticks':int(ticks),'backlog_ms':float(backlog),'focused':focused=='true','occluded':occluded=='true'})
 return out

def distribution(samples,key):
 v=sorted(s[key] for s in samples if s[key] is not None)
 return None if not v else {'samples':len(v),'p95':v[math.ceil(.95*len(v))-1],'max':v[-1]}

def source_state(s):return {k:s[k] for k in ['ship','player','survival','fabricator','world','journey','machinery','consumables','carrying']}

def validate_window(row):
 samples=row['samples'];assert len(samples)>=100
 assert [s['frame'] for s in samples]==list(range(samples[0]['frame'],samples[-1]['frame']+1))
 assert all(s['focused'] and not s['occluded'] and s['interval_ms']>0 and s['ticks']==0 for s in samples)
 assert source_state(row['before'])==source_state(row['after'])
 for state in [row['before'],row['after']]:
  c=state['visual_context'];assert c['backend']=='Metal' and c['adapter']=='Apple M1'
  assert c['viewport_pixels']==[1920,1080] and c['scale_factor']==2
  assert state['window']['minimized'] is False and state['native_input_isolated'] is True
  assert state['ship']['power_available']==row['powered']
