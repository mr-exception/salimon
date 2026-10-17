"""Reviewed Option A recipe; edits owning libraries, never assembly links.

Run Blender 4.5.3 in background, then normal Scout export/verify commands.
Idempotent visual-only pass: no vertex, transform, metadata or UV edits.
"""
from pathlib import Path
import sys

import bpy

ASSET=Path(__file__).resolve().parent
sys.path.insert(0,str(ASSET.parents[3]/'models/tools'))
from style_palette import apply


def main():
    if bpy.app.version[:3] != (4,5,3):
        raise ValueError('authoring requires Blender 4.5.3')
    bpy.context.preferences.filepaths.save_version=0
    shared=ASSET/'assembly/shared.blend'
    bpy.ops.wm.open_mainfile(filepath=str(shared))
    apply([m for m in bpy.data.materials if not m.library])
    bpy.ops.wm.save_as_mainfile(filepath=str(shared))
    for source in sorted((ASSET/'components').glob('*/source.blend')):
        bpy.ops.wm.open_mainfile(filepath=str(source))
        if 'Hull Ceramic' not in bpy.data.materials:
            with bpy.data.libraries.load('//../../assembly/shared.blend', link=True, relative=True) as (available, loaded):
                loaded.materials=['Hull Ceramic']
        changed=False
        for obj in bpy.data.objects:
            if obj.library or obj.type!='MESH' or obj.data.library:
                continue
            # Large decorative cyan areas become secondary slate planes.
            # Status, lamps and emissive materials retain their exact roles.
            if obj.name in ('Roof_Service_Panels','Ceiling_Teal_Inset_Panels'):
                obj.data.materials[0]=bpy.data.materials['Hull Ceramic']
                changed=True
            if any(m and m.name=='Cockpit Glass' for m in obj.data.materials):
                continue
            # Keep soft seat cushions intentionally smoother; do not touch UVs.
            if obj.name in ('Pilot_Seat_Cushions','Pilot_Seat_Bolsters','Pilot_Seat_Headrest'):
                continue
            if any(p.use_smooth for p in obj.data.polygons):
                for polygon in obj.data.polygons:
                    polygon.use_smooth=False
                if obj.data.has_custom_normals:
                    obj.data.normals_split_custom_set([(0,0,0)]*len(obj.data.loops))
                changed=True
        if changed:
            bpy.ops.wm.save_as_mainfile(filepath=str(source))


if __name__=='__main__':main()
