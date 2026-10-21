"""Exported near-corner rays and an intentional restored shoulder intrusion."""
from pathlib import Path
import copy
import struct
import sys
import unittest
REPO=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(REPO/'models/tools'),str(REPO/'models/assets/ships/salimon-scout')]
from validate_asset import ValidationError,read_document
from validate import validate_sightlines
class CockpitCornerTests(unittest.TestCase):
    def test_current_export_preserves_corner_lower_and_monitor_sightlines(self):
        validate_sightlines(*read_document(REPO/'client/assets/ship/export/salimon_phase0_ship.glb'))
    def test_reintroduced_port_shoulder_fails_corner_ray(self):
        doc,data=read_document(REPO/'client/assets/ship/export/salimon_phase0_ship.glb');doc=copy.deepcopy(doc);data=bytearray(data)
        node=next(n for n in doc['nodes']if n['name']=='Hull_Port_Side');changed=0
        for primitive in doc['meshes'][node['mesh']]['primitives']:
            a=doc['accessors'][primitive['attributes']['POSITION']];view=doc['bufferViews'][a['bufferView']]
            start=view.get('byteOffset',0)+a.get('byteOffset',0);stride=view.get('byteStride',12)
            for i in range(a['count']):
                offset=start+i*stride;x=struct.unpack_from('<f',data,offset)[0]
                if abs(x-4.98)<1e-5:struct.pack_into('<f',data,offset,5.2);changed+=1
        self.assertGreater(changed,0)
        with self.assertRaisesRegex(ValidationError,'canopy corner.*Hull_Port_Side'):validate_sightlines(doc,bytes(data))
if __name__=='__main__':unittest.main()
