import subprocess,sys,time
windows=subprocess.check_output(['xdotool','search','--name','Salimon']).decode().split()
for window in windows:
 subprocess.run(['xdotool','windowsize',window,'1920','1080'],check=True)
time.sleep(.5)
subprocess.run([sys.executable,'scripts/capture_settled.py',sys.argv[1]],check=True)
