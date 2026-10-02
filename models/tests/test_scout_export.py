"""Migration regressions for the Blender-owned scout and its retained contracts."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import sys
import unittest
from unittest.mock import patch

REPO = Path(__file__).resolve().parents[2]
ASSET = REPO / 'models/assets/ships/salimon-scout'
sys.path.insert(0, str(REPO / 'models/tools'))
from validate_asset import ValidationError, read_document, validate_asset

spec = importlib.util.spec_from_file_location('scout_export', ASSET / 'export.py')
scout = importlib.util.module_from_spec(spec)
spec.loader.exec_module(scout)


class ScoutExportTests(unittest.TestCase):
    def setUp(self):
        self.manifest = json.loads((ASSET / 'manifest.json').read_text())
        self.output = REPO / self.manifest['runtime']
        self.document, _ = read_document(self.output)

    def test_checked_in_export_provenance_and_shared_budgets(self):
        report = json.loads((ASSET / 'export-report.json').read_text())
        self.assertEqual(report['sourceSha256'], hashlib.sha256((ASSET / 'source.blend').read_bytes()).hexdigest())
        self.assertEqual(report['runtimeSha256'], hashlib.sha256(self.output.read_bytes()).hexdigest())
        metrics = validate_asset('ship.salimon-scout', extension_validators={
            ('ships', 'ship', 1): scout.validate_preservation,
        })
        self.assertEqual(report['metrics'], metrics)
        self.assertIn('Blender', self.document['asset']['generator'])
        self.assertIn('Blender', self.document['asset']['extras']['salimon']['sourceWorkflow'])

    def test_runtime_contract_changes_are_rejected(self):
        for name, mutate in (
            ('Exit_Door', lambda n: n.update(extras={})),
            ('MARKER_PlayerStart', lambda n: n.update(translation=[0, 0, 0])),
            ('COLLIDER_AftDoor', lambda n: n['extras']['salimon'].update(sizeMeters=[1, 1, 1])),
            ('Monitor_Port', lambda n: n.update(rotation=[0, 0, 1, 0])),
        ):
            with self.subTest(node=name):
                doc = copy.deepcopy(self.document)
                mutate(next(n for n in doc['nodes'] if n['name'] == name))
                with self.assertRaises(ValidationError):
                    scout.validate_preservation({'document': doc}, {})
        doc = copy.deepcopy(self.document)
        next(m for m in doc['materials'] if m['name'] == 'Cockpit Glass')['alphaMode'] = 'OPAQUE'
        with self.assertRaises(ValidationError):
            scout.validate_preservation({'document': doc}, {})

    def test_failed_staged_validation_preserves_all_published_files(self):
        paths = [self.output.with_suffix(s) for s in ('.glb', '.bin', '.gltf')]
        paths.append(ASSET / 'export-report.json')
        before = {p: p.read_bytes() for p in paths}
        manifest = copy.deepcopy(self.manifest)
        manifest['budgets']['maxGlbBytes'] = 1
        # Simulate a successful Blender process producing the current GLB,
        # then make semantic validation fail before publication.
        def blender(command, **kwargs):
            shutil.copyfile(self.output, command[-1])
        with patch.object(scout, 'resolve_manifest', return_value=(ASSET / 'manifest.json', manifest)), \
             patch.object(scout.shutil, 'which', return_value='/mock/blender'), \
             patch.object(scout.subprocess, 'run', side_effect=blender):
            with self.assertRaisesRegex(ValidationError, 'maxGlbBytes'):
                scout.export()
        self.assertEqual(before, {p: p.read_bytes() for p in paths})


if __name__ == '__main__':
    unittest.main()
