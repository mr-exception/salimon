import subprocess,json,pathlib,shutil,time,os
root=pathlib.Path('artifacts/native-20261022/376-offline');save=root/'save';save.mkdir(parents=True)
shutil.copy('artifacts/native-20261022/376-native/landed/before.sav',save/'checkpoint-0.sav')
before=json.loads((save/'checkpoint-0.sav').read_bytes()[28:])
with (root/'stdout.log').open('w') as out,(root/'stderr.log').open('w') as err:
 p=subprocess.Popen(['artifacts/native-20261022/Salimon348Fixed.app/Contents/MacOS/salimon-client'],env=dict(os.environ,SALIMON_SAVE_DIR=str(save.resolve()),SALIMON_REFERENCE_SIZE='1',RUST_LOG='info'),stdout=out,stderr=err)
 try:
  deadline=time.monotonic()+30
  while 'renderer ready:' not in (root/'stderr.log').read_text():
   if p.poll() is not None:raise RuntimeError(str(p.returncode))
   if time.monotonic()>deadline:raise TimeoutError('renderer')
   time.sleep(.1)
  path=max(save.glob('*.sav'),key=lambda p:int.from_bytes(p.read_bytes()[8:16],'little'));raw=path.read_bytes();(root/'startup.sav').write_bytes(raw);after=json.loads(raw[28:])
  diffs=[]
  def diff(a,b,path=''):
   if isinstance(a,dict) and isinstance(b,dict):
    for k in sorted(a.keys()|b.keys()):diff(a.get(k),b.get(k),path+'/'+k)
   elif a!=b:diffs.append({'path':path,'before':a,'after':b})
  diff(before,after);(root/'differences.json').write_text(json.dumps(diffs,indent=2)+'\n');print(json.dumps(diffs),flush=True)
 finally:
  p.kill();p.wait(timeout=5)
