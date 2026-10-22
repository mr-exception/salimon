import pathlib,subprocess,os,time,json,shutil
root=pathlib.Path('artifacts/native-20261022/376-crash-fixtures');d=root/'crash';d.mkdir(parents=True)
shutil.copy('artifacts/native-20261022/376-native/landed/before.sav',d/'checkpoint-0.sav')
(root/'manifest.json').write_text(json.dumps([{'case':'crash','fixture':True,'scope':'real SIGKILL after native verified Save Now'}]))
env=dict(os.environ,SALIMON_SAVE_DIR=str(d.resolve()),SALIMON_REFERENCE_SIZE='1',RUST_LOG='info')
with (root/'before-crash.stdout').open('w') as out,(root/'before-crash.stderr').open('w') as err:
 p=subprocess.Popen(['artifacts/native-20261022/Salimon348Fixed.app/Contents/MacOS/salimon-client'],env=env,stdout=out,stderr=err)
 try:
  deadline=time.monotonic()+30
  while 'renderer ready:' not in (root/'before-crash.stderr').read_text():
   if p.poll() is not None:raise RuntimeError(str(p.returncode))
   if time.monotonic()>deadline:raise TimeoutError('renderer')
   time.sleep(.1)
  records=[]
  for args in [('focus',),('key','53','100'),('key','45','100')]:
   r=subprocess.run(['artifacts/native-20261022/input-review',str(p.pid),*args],capture_output=True,text=True,check=True);records.append({'input':args,'stdout':r.stdout})
  time.sleep(1)
  p.kill();p.wait(timeout=5)
  (root/'crash.json').write_text(json.dumps({'pid':p.pid,'exit':p.returncode,'signal':'SIGKILL','input':records},indent=2)+'\n')
 finally:
  if p.poll() is None:p.kill();p.wait()
