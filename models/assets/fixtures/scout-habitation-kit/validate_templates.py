"""Validate library templates separately: co-location is not a composed scene.

Each selected named node is passed to the ordinary geometry-quality inspector,
with no overlap exemptions or widened tolerances. The full GLB still passes the
ordinary manifest, units, material, topology and budget checks first.
"""
from pathlib import Path
import copy,json,sys
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'models/tools'))
from validate_asset import validate_asset,read_document
from geometry_quality import diagnose

def check():
    metrics=validate_asset('fixture.scout-habitation-kit')
    doc,binary=read_document(ROOT/'client/assets/fixtures/scout-habitation-kit/model.glb')
    results={}
    for name in ['Table','ScreenPanel','Shutter','LockerShell','LockerDoor','SeatBase','Harness','SeatPad','SeatBack','ShutterHandle']:
        template=copy.deepcopy(doc)
        for node in template['nodes']:
            if node.get('name')!=name:node.pop('mesh',None)
        report=diagnose(template,binary,{'toleranceMeters':1e-5,'areaToleranceSquareMeters':1e-10})
        errors=[f for f in report['findings'] if f['severity']=='error']
        if errors:raise ValueError(f'{name}: {errors}')
        results[name]=report
    print(json.dumps({'metrics':metrics,'templates':results},sort_keys=True))
if __name__=='__main__':check()
