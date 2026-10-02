"""Export failure handling and optional real-Blender round trips."""
import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import export_asset as exporter

MODELS = Path(__file__).resolve().parents[1]
BLENDER = shutil.which(os.environ.get('BLENDER', 'blender'))


class ExportTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.repo = Path(self.temporary.name)
        (self.repo / 'models').mkdir()
        shutil.copy(MODELS / 'asset-manifest.schema.json', self.repo / 'models')
        self.manifest = json.loads((MODELS / 'examples/resource.manifest.json').read_text())
        self.path = self.repo / 'models/assets/resources/iron-fragment/manifest.json'
        self.path.parent.mkdir(parents=True)
        self.save()
        self.source = self.repo / self.manifest['source']
        self.source.write_bytes(b'test placeholder')
        self.output = self.repo / self.manifest['runtime']
        self.output.parent.mkdir(parents=True)
        self.output.write_bytes(b'original runtime artifact')

    def save(self):
        self.path.write_text(json.dumps(self.manifest))

    def test_id_directory_and_manifest_lookup(self):
        for asset in (self.manifest['assetId'], str(self.path), str(self.path.parent)):
            self.assertEqual(exporter.resolve_manifest(asset, self.repo)[1], self.manifest)
        with self.assertRaisesRegex(exporter.ExportError, 'no asset manifest'):
            exporter.resolve_manifest('missing.asset', self.repo)

    def test_bad_schema_and_duplicate_identity(self):
        self.manifest['coordinates']['source']['unitScale'] = 0.01
        self.save()
        with self.assertRaisesRegex(exporter.ExportError, 'unitScale'):
            exporter.resolve_manifest(str(self.path), self.repo)
        self.manifest['coordinates']['source']['unitScale'] = 1
        self.save()
        other = self.path.parent.parent / 'copy/manifest.json'
        other.parent.mkdir()
        other.write_text(self.path.read_text())
        for asset in (self.manifest['assetId'], str(self.path)):
            with self.assertRaisesRegex(exporter.ExportError, 'duplicate assetId'):
                exporter.resolve_manifest(asset, self.repo)

    def test_missing_source_and_blender(self):
        with self.assertRaisesRegex(exporter.ExportError, 'Blender executable not found'):
            exporter.export_asset(str(self.path), '/missing/blender', self.repo)
        self.source.unlink()
        with self.assertRaisesRegex(exporter.ExportError, 'missing Blender source'):
            exporter.export_asset(str(self.path), repo=self.repo)

    def test_symlink_escape_and_examples_rejected(self):
        with tempfile.TemporaryDirectory() as outside:
            self.source.unlink()
            self.source.symlink_to(Path(outside) / 'source.blend')
            with self.assertRaisesRegex(exporter.ExportError, 'path escapes models/assets'):
                exporter.export_asset(str(self.path), repo=self.repo)
        example = self.repo / 'models/examples/manifest.json'
        example.parent.mkdir()
        example.write_text(self.path.read_text())
        with self.assertRaisesRegex(exporter.ExportError, 'examples are not assets'):
            exporter.resolve_manifest(str(example), self.repo)

    def test_failed_process_and_bad_output_preserve_runtime(self):
        with patch.object(exporter.shutil, 'which', return_value='/fake/blender'):
            for returncode in (1, 0):
                with patch.object(exporter.subprocess, 'run', return_value=subprocess.CompletedProcess([], returncode)):
                    with self.assertRaises(exporter.ExportError):
                        exporter.export_asset(str(self.path), repo=self.repo)
                self.assertEqual(self.output.read_bytes(), b'original runtime artifact')
        self.assertFalse(list(self.output.parent.glob('.salimon-export-*')))

    @unittest.skipUnless(BLENDER, 'set BLENDER to run real Blender export integration')
    def test_real_blender_categories_names_extras_axes_and_proxies(self):
        for category in ('resources', 'items'):
            with self.subTest(category=category):
                manifest = copy.deepcopy(self.manifest)
                manifest['assetId'] = f'{category}.fixture'
                manifest['category'] = category
                manifest['contracts']['sockets'] = ['SOCKET_Grip']
                manifest['contracts']['colliders'] = ['COLLIDER_Bounds']
                manifest['collision'] = {'policy': 'authored-proxies', 'shapes': ['box']}
                manifest['contracts']['requiredExtras'] = [{'node': 'Iron_Fragment', 'path': 'salimon.role'}]
                self.path.write_text(json.dumps(manifest))
                fixture = self.repo / 'create_fixture.py'
                fixture.write_text('''import bpy
bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete(use_global=False)
scene = bpy.context.scene
scene.unit_settings.system = 'METRIC'
scene.unit_settings.scale_length = 1
root = bpy.data.objects.new('ASSET_iron-fragment', None)
scene.collection.objects.link(root)
visual = bpy.data.objects.new('Visual', None)
scene.collection.objects.link(visual)
visual.parent = root
bpy.ops.mesh.primitive_cube_add(size=1)
mesh = bpy.context.object
mesh.name = 'Iron_Fragment'
mesh.parent = visual
mesh['salimon'] = {'role': 'fixture'}
marker = bpy.data.objects.new('SOCKET_Grip', None)
scene.collection.objects.link(marker)
marker.parent = root
marker.location = (1, 2, 3)
marker['salimon'] = {'purpose': 'grip'}
bpy.ops.mesh.primitive_cube_add(size=2)
proxy = bpy.context.object
proxy.name = 'COLLIDER_Bounds'
proxy.parent = root
proxy['salimon'] = {'shape': 'box'}
bpy.ops.mesh.primitive_cube_add(size=9)
bpy.context.object.name = 'Authoring_Helper'
bpy.ops.wm.save_as_mainfile(filepath=''' + repr(str(self.source)) + ''')
''')
                subprocess.run([BLENDER, '--background', '--factory-startup', '--python-exit-code', '1', '--python', str(fixture)], check=True, stdout=subprocess.DEVNULL)
                exporter.export_asset(manifest['assetId'], BLENDER, self.repo)
                doc = exporter.read_glb(self.output)
                nodes = {n['name']: n for n in doc['nodes']}
                self.assertNotIn('Authoring_Helper', nodes)
                self.assertEqual(nodes['Iron_Fragment']['extras']['salimon']['role'], 'fixture')
                for actual, expected in zip(nodes['SOCKET_Grip']['translation'], (1, 3, -2)):
                    self.assertAlmostEqual(actual, expected, places=5)
                self.assertNotIn('mesh', nodes['COLLIDER_Bounds'])
                self.assertEqual(nodes['COLLIDER_Bounds']['extras']['salimon']['shape'], 'box')
                self.assertEqual(nodes['COLLIDER_Bounds']['extras']['salimonProxyDimensions'], [2, 2, 2])
                # A failed preservation check must not overwrite a valid previous export.
                original = self.output.read_bytes()
                manifest['contracts']['nodes'].append('Missing_Contract')
                self.path.write_text(json.dumps(manifest))
                with self.assertRaisesRegex(exporter.ExportError, 'Missing_Contract'):
                    exporter.export_asset(str(self.path), BLENDER, self.repo)
                self.assertEqual(self.output.read_bytes(), original)


if __name__ == '__main__':
    unittest.main()
