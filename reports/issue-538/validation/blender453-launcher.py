#!/workspace/tools/blender-python/bin/python
import sys, runpy, bpy
args=sys.argv[1:]
if '--version' in args:
 print('Blender '+bpy.app.version_string+' (bpy wheel)');sys.exit(0)
for a in args:
 if a.endswith('.blend'):bpy.ops.wm.open_mainfile(filepath=a)
if '--python' in args:
 script=args[args.index('--python')+1]
 sys.argv=[script]+(args[args.index('--'):] if '--' in args else [])
 runpy.run_path(script,run_name='__main__')
