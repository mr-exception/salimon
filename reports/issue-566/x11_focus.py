#!/usr/bin/env python3
"""Focus and independently admit one visible X11 PID; no physical-input claim."""
import json,os,shutil,subprocess,sys,time
pid=int(sys.argv[1]);assert sys.argv[2:] == ['focus']
xdo=os.environ.get('SALIMON_XDOTOOL') or shutil.which('xdotool');assert xdo
xenv=dict(os.environ)
if os.environ.get('SALIMON_XDO_LIBDIR'):xenv['LD_LIBRARY_PATH']=os.environ['SALIMON_XDO_LIBDIR']
def call(*args):return subprocess.check_output([xdo,*map(str,args)],text=True,env=xenv,timeout=2).strip()
errors=[]
for attempt in range(20):
 try:
  windows=call('search','--onlyvisible','--pid',pid).splitlines();assert len(windows)==1,windows
  window=windows[0];call('windowfocus','--sync',window)
  focused=call('getwindowfocus');actual=int(call('getwindowpid',focused));assert actual==pid,(actual,pid)
  assert focused==window,(focused,window)
  geometry=call('getwindowgeometry','--shell',window)
  info=subprocess.check_output(['xwininfo','-id',window],text=True,timeout=2)
  assert 'Map State: IsViewable' in info,info
  assert 'WIDTH=1280' in geometry and 'HEIGHT=800' in geometry,geometry
  print(f'frontmost={actual} target={pid}')
  print(json.dumps({'pid':pid,'window':window,'focused_window':focused,'geometry':geometry,'map_state':'IsViewable','display':os.environ.get('DISPLAY'),'attempt':attempt,'prior_errors':errors}))
  break
 except (AssertionError,subprocess.SubprocessError) as e:
  errors.append(str(e));time.sleep(.1)
else:
 print(json.dumps({'target':pid,'failed':True,'errors':errors}));sys.exit(1)
