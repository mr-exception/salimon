"""Probe the comparator with persistent image defects; does not approve a baseline."""
import json,sys
from pathlib import Path
from PIL import Image,ImageDraw
sys.path.insert(0,str(Path('scripts').resolve()))
from model_visual_tests import compare
root=Path('artifacts/native-20261020/314');reference=root/'isolated-final-candidates/interior/dashboard-zero-repeat-00.png';out=root/'isolated-final-injected';out.mkdir(parents=True,exist_ok=True)
results={}
for name in ['silhouette','material','layout']:
 image=Image.open(reference).convert('RGB');draw=ImageDraw.Draw(image)
 if name=='silhouette':draw.polygon([(390,250),(640,160),(890,250),(890,450),(390,450)],fill=(5,5,5))
 elif name=='material':draw.rectangle((260,240,1000,420),fill=(235,25,180))
 else:
  region=image.crop((300,500,980,780));draw.rectangle((300,500,980,780),fill=(30,40,50));image.paste(region,(410,380))
 path=out/(name+'.png');image.save(path)
 appearance=compare(reference,path,out/name,channel_tolerance=8,changed_fraction=.001,masks=[])
 temporal=compare(path,path,out/(name+'-repeat'),channel_tolerance=8,changed_fraction=.001,masks=[])
 assert appearance['passed'] is False and temporal['passed'] is True
 results[name]={'appearance':appearance,'frozen_wrong_repeat':temporal}
(out/'results.json').write_text(json.dumps({'reference_kind':'unapproved current Metal candidate; comparator self-test only','results':results},indent=2)+'\n')
print(json.dumps({name:{'appearance_rejected':not r['appearance']['passed'],'wrong_frozen_repeat_passes':r['frozen_wrong_repeat']['passed']} for name,r in results.items()}))
