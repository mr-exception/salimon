"""Shared source parameters for the #540 berth and generated consumer geometry."""
import json
import math
from pathlib import Path

def definition():
    return json.loads((Path(__file__).parent/'habitation-geometry.json').read_text())

def rust_source():
    d=definition();x,y,z=d['bunkHingeMeters'];w,h,t=d['frameSizeMeters'];margin=d['clearanceMeters']
    radius=math.hypot(h,t/2)
    bounds=[x-w/2-margin,x+w/2+margin,y-t/2-margin,y+radius+margin,
            z+d['cushionOffsetMeters']-d['cushionSizeMeters'][2]/2-margin,z+radius+margin]
    return ('// Generated from models/assets/ships/salimon-scout/habitation-geometry.json.\n'
            f'pub const BUNK_HINGE_METERS: [f64; 3] = {d["bunkHingeMeters"]!r};\n'
            f'pub const BUNK_SWEEP_BOUNDS: [f64; 6] = {bounds!r};\n')
