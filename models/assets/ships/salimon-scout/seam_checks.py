"""Portable, phase-aware witnesses for articulated source seam regressions."""
import copy
import math
from geometry_quality import diagnose
from validate_asset import require
NAMES={'Exit_Door','Exit_Ramp_Tip','Door_Frame','Hull_Aft_Cap'}

def door_seams(document,binary):
    samples=[]
    for fraction in [0,.15,.35,.6,.75,1,.75,.35,0]:
        doc=copy.deepcopy(document)
        nodes={n['name']:n for n in doc['nodes']}
        for n in doc['nodes']:
            if n['name'] not in NAMES:n.pop('mesh',None)
        ramp=nodes['Exit_Door']['extras']['salimon'];pivot=ramp['pivot']
        h=min(fraction/.6,1);h=h*h*(3-2*h)
        e=max(0,min(1,(fraction-.6)/.4));e=e*e*(3-2*e)
        angle=ramp['openAngleRadians']*h;c,s=math.cos(angle),math.sin(angle)
        for name in ['Exit_Door','Exit_Ramp_Tip']:
            offset=[ramp['extensionFaceOffsetMeters']*e,ramp['extensionTravelMeters']*e] if name=='Exit_Ramp_Tip' else [0,0]
            x,y=offset[0]-pivot[0],offset[1]-pivot[1]
            nodes[name]['translation']=[pivot[0]+c*x-s*y,pivot[1]+s*x+c*y,0]
            nodes[name]['rotation']=[0,0,math.sin(angle/2),math.cos(angle/2)]
        result=diagnose(doc,binary,{})
        rows=[f for f in result['findings'] if f['code'] in ['surface-overlap','duplicate-face']
              and f['node']!=f['otherNode'] and 'Exit_Door' in (f['node'],f['otherNode'])]
        def material(name,face):
            return doc['meshes'][nodes[name]['mesh']]['primitives'][face[0]]['material']
        mixed=[r for r in rows if material(r['node'],r['face'])!=material(r['otherNode'],r['otherFace'])]
        # Full extension intentionally has same-material support overlap; never
        # allow competing material planes, nor any closed leaf/cap competition.
        require(not mixed,f'door has competing material planes at fraction {fraction}')
        if fraction==0:require(not rows,'closed door has coplanar leaf/frame/tray/cap surfaces')
        samples.append({'fraction':fraction,'overlapPairs':len(rows),'mixedMaterialPairs':len(mixed)})
    return samples
