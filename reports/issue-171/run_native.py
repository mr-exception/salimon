import subprocess,os,time,sys,socket
server=subprocess.Popen(['Xvfb',':78','-screen','0','1920x1080x24','-ac','-nolisten','unix','-nolisten','local','-listen','tcp'],stdout=open('/tmp/171-xserver2.log','w'),stderr=subprocess.STDOUT)
try:
 for i in range(20):
  try:
   with socket.create_connection(('127.0.0.1',6078),timeout=.2): break
  except OSError:time.sleep(.1)
 os.environ.update(DISPLAY='127.0.0.1:78',WGPU_BACKEND='vulkan',VK_DRIVER_FILES='/usr/share/vulkan/icd.d/lvp_icd.json')
 subprocess.run(['xdotool','getdisplaygeometry'],check=True)
 args=['python','scripts/packaged_smoke.py',*sys.argv[2:]] if sys.argv[1]=='smoke' else ['python','scripts/salimon-test',*sys.argv[1:]]
 sys.exit(subprocess.run(args).returncode)
finally:server.terminate();server.wait()
