"""Regressions for the Blender-owned scout and its authored contracts."""
import copy
import hashlib
import importlib.util
import json
import os
import subprocess
from pathlib import Path, PureWindowsPath
import shutil
import sys
import unittest
from unittest.mock import Mock, patch

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
        self.assertEqual(report['sourceSha256'], hashlib.sha256((REPO / self.manifest['source']).read_bytes()).hexdigest())
        self.assertEqual(report['authoringSha256'], {
            p.relative_to(ASSET).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(ASSET.rglob('*.blend'))})
        self.assertEqual(report['runtimeSha256'], hashlib.sha256(self.output.read_bytes()).hexdigest())
        metrics = validate_asset('ship.salimon-scout', extension_validators={
            ('ships', 'ship', 1): scout.validate_preservation,
        })
        self.assertEqual(report['metrics'], metrics)
        self.assertIn('Blender', self.document['asset']['generator'])
        self.assertIn('Blender', self.document['asset']['extras']['salimon']['sourceWorkflow'])

    def test_authoring_hash_keys_are_portable_on_windows(self):
        asset = Mock()
        source = Mock()
        asset.rglob.return_value = [source]
        source.relative_to.return_value = PureWindowsPath('components/hull/source.blend')
        source.read_bytes.return_value = b'authored component'
        self.assertEqual(scout.authoring_hashes(asset), {
            'components/hull/source.blend': hashlib.sha256(b'authored component').hexdigest(),
        })

    def test_linked_component_edit_propagation_and_saved_source(self):
        blender = shutil.which(os.environ.get('BLENDER', 'blender'))
        if not blender:
            self.skipTest('Blender required for linked component verification')
        for script in ('verify_modular.py', 'verify_source.py'):
            result = subprocess.run([blender, '--background', '--factory-startup',
                                     '--disable-autoexec', '--python-exit-code', '1',
                                     '--python', str(ASSET / script)],
                                    capture_output=True, text=True, timeout=60)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_non_spatial_legacy_contract_changes_are_rejected(self):
        for name, mutate in (
            ('Exit_Door', lambda n: n.update(extras={})),
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

    def test_authored_spatial_contract_changes_round_trip(self):
        preservation = json.loads((ASSET / 'preservation.json').read_text())
        doc = copy.deepcopy(self.document)
        marker = next(n for n in doc['nodes'] if n['name'] == 'MARKER_PlayerStart')
        marker['translation'] = [0.75, 2.0, -2.0]
        collider = next(n for n in doc['nodes'] if n['name'] == 'COLLIDER_AftDoor')
        collider['extras']['salimon']['sizeMeters'] = [0.4, 2.4, 3.0]
        nose_floor = next(n for n in doc['nodes'] if n['name'] == 'COLLIDER_CockpitNoseFloor')
        nose_floor['translation'][0] += 0.1
        scout.validate_preservation({'document': doc}, {})
        spatial = scout.build_spatial_contracts(doc, preservation)
        self.assertEqual(spatial['markers']['MARKER_PlayerStart']['positionMeters'], [0.75, 2.0, -2.0])
        self.assertEqual(spatial['colliders']['COLLIDER_AftDoor']['sizeMeters'], [0.4, 2.4, 3.0])
        self.assertAlmostEqual(spatial['colliders']['COLLIDER_CockpitNoseFloor']['centerMeters'][0],
                               nose_floor['translation'][0])
        self.assertNotIn('cargoRoom', spatial)

    def test_failed_staged_validation_preserves_all_published_files(self):
        paths = [self.output.with_suffix(s) for s in ('.glb', '.bin', '.gltf')]
        paths.extend([
            ASSET / 'export-report.json',
            REPO / 'client/assets/ship/spatial-contracts.json',
            REPO / 'client/character/src/spatial_contracts.rs',
        ])
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
