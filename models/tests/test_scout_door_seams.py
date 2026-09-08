"""Production export regression and intentional duplicated-tray failure."""
import copy
from pathlib import Path
import sys
import unittest
REPO=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(REPO/'models/tools'))
sys.path.insert(0,str(REPO/'models/assets/ships/salimon-scout'))
from validate_asset import read_document,ValidationError
from seam_checks import door_seams
class DoorSeamsTests(unittest.TestCase):
    def test_closed_opening_extension_and_reversal(self):
        doc,b=read_document(REPO/'client/assets/ship/export/salimon_phase0_ship.glb')
        rows=door_seams(doc,b)
        self.assertEqual(rows[0]['overlapPairs'],0)
        self.assertTrue(all(r['mixedMaterialPairs']==0 for r in rows))
    def test_duplicate_leaf_on_tray_fails_instead_of_becoming_exempt(self):
        doc,b=read_document(REPO/'client/assets/ship/export/salimon_phase0_ship.glb')
        doc=copy.deepcopy(doc);nodes={n['name']:n for n in doc['nodes']}
        nodes['Exit_Ramp_Tip']['mesh']=nodes['Exit_Door']['mesh']
        with self.assertRaises(ValidationError):door_seams(doc,b)
if __name__=='__main__':unittest.main()
