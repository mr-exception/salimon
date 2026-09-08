"""Assemble review contact sheets from reference PNGs (Pillow, offline only)."""
import argparse
from pathlib import Path
import shutil
from PIL import Image, ImageDraw


def collect(source, destination):
    destination.mkdir(parents=True,exist_ok=True)
    groups={
        'ship': ['current-exterior','proposed-exterior','current-exterior-far','proposed-exterior-far'],
        'interior':['current-interior','proposed-interior','current-core','proposed-core'],
        'resources-tool':['terrain-resources','resource-closeup','tool-near','tool-far'],
    }
    for name,views in groups.items():
        sheet=Image.new('RGB',(1280,1016),(25,30,38))
        draw=ImageDraw.Draw(sheet)
        for i,view in enumerate(views):
            x,y=(i%2)*640,(i//2)*508
            draw.text((x+12,y+6),view,fill='white')
            with Image.open(source/(view+'.png')) as image:
                sheet.paste(image.convert('RGB'),(x,y+28))
        sheet.save(destination/(name+'.jpg'),quality=85,optimize=True)
    shutil.copyfile(source/'reference-costs.json',destination/'reference-costs.json')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('source',type=Path)
    p.add_argument('destination',type=Path)
    a=p.parse_args()
    collect(a.source,a.destination)
