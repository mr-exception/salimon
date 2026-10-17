import json,os,plistlib,shutil,subprocess
from pathlib import Path
root=Path('artifacts/native-279')
app=Path('artifacts/SalimonBeforeArt.app/Contents');(app/'MacOS').mkdir(parents=True,exist_ok=True)
plist=plistlib.loads(Path('artifacts/SalimonAcceptance.app/Contents/Info.plist').read_bytes());plist['CFBundleIdentifier']='org.salimon.before-art';plist['CFBundleName']='Salimon Before Art'
(app/'Info.plist').write_bytes(plistlib.dumps(plist))
shutil.copy2('artifacts/build/native-236/salimon-client',app/'MacOS/salimon-client')
env=dict(os.environ,SALIMON_REFERENCE_SIZE='1',RUST_LOG='info')
for label,binary,layouts,seconds in [('before-onboard',str(app/'MacOS/salimon-client'),['onboard'],30),('before-spheres',str(app/'MacOS/salimon-client'),['sphere-deck','sphere-pile'],8),('after-spheres','artifacts/SalimonAcceptance.app/Contents/MacOS/salimon-client',['sphere-deck','sphere-pile'],8)]:
 cmd=['python3','reports/issue-229/foreground_matrix.py','--helper','artifacts/native-input','--records',str(root/(label+'-focus.json')),'--','python3','scripts/benchmark_fragments.py','--binary',binary,'--machine-label','apple-m1-imac-16gb-macos26.2','--counts','500','--layouts',*layouts,'--states','settled','--cameras','facing','--repeats','3','--minimum-samples','100','--warmup-seconds','2','--sample-seconds',str(seconds),'--settle-steps','2400','--timeout','180','--output',str(root/label)]
 print(label,flush=True);subprocess.run(cmd,env=env,check=True)
print('DONE',flush=True)
