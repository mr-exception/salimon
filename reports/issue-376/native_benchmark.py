import subprocess,os,time,pathlib,json
out=pathlib.Path('artifacts/native-20261022/376-benchmark');out.mkdir()
for count in [0,500]:
 args=['artifacts/native-20261022/Salimon348Fixed.app/Contents/MacOS/salimon-client','--benchmark','--mode','native','--count',str(count),'--layout','onboard','--state','settled','--camera','facing','--warmup-seconds','1','--sample-seconds','5','--minimum-samples','100','--repeats','3','--settle-steps','120']
 d=out/str(count);d.mkdir()
 (d/'command.json').write_text(json.dumps(args))
 with (d/'stdout.json').open('w') as stdout,(d/'stderr.log').open('w') as stderr:
  p=subprocess.Popen(args,stdout=stdout,stderr=stderr,env=dict(os.environ,SALIMON_REFERENCE_SIZE='1',RUST_LOG='info'))
  try:
   deadline=time.monotonic()+120
   while 'renderer ready:' not in (d/'stderr.log').read_text():
    if p.poll() is not None:raise RuntimeError('benchmark startup exit '+str(p.returncode))
    if time.monotonic()>deadline:raise TimeoutError('benchmark preparation')
    time.sleep(.1)
   r=subprocess.run(['artifacts/native-20261022/input-review',str(p.pid),'focus'],capture_output=True,text=True,check=True)
   (d/'focus.txt').write_text(r.stdout)
   r=subprocess.run(['artifacts/native-20261022/input-review',str(p.pid),'window'],capture_output=True,text=True,check=True)
   (d/'window.txt').write_text(r.stdout)
   p.wait(timeout=40)
   if p.returncode:raise RuntimeError('benchmark exit '+str(p.returncode))
   print('passed',count,flush=True)
  finally:
   if p.poll() is None:p.kill();p.wait()
