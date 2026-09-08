"""Exact optimization, preservation, visual budgets and failure-safe staging."""
import copy
import json
from pathlib import Path
import shutil
import struct
import sys
import subprocess
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import process_glb as pipeline
from glb_costs import check_visual_budget, measure
from validate_asset import read_document

REPO = Path(__file__).resolve().parents[2]
VALIDATOR = (REPO / 'models/tools/gltf/node_modules/gltf-validator').is_dir() and shutil.which('node')


class ProcessingTests(unittest.TestCase):
    def setUp(self):
        self.manifest = json.loads((REPO / 'models/assets/items/mining-tool/manifest.json').read_text())
        self.source = REPO / self.manifest['runtime']
        self.document, self.binary = read_document(self.source)

    def test_exact_repack_preserves_all_semantics_and_is_idempotent(self):
        # Force duplication: distinct view metadata and indices must survive.
        doc = copy.deepcopy(self.document)
        doc['bufferViews'].append(copy.deepcopy(doc['bufferViews'][0]))
        doc['bufferViews'][-1]['name'] = 'Keep_This_View'
        after, data = pipeline.exact_repack(doc, self.binary)
        self.assertEqual(pipeline.semantic_snapshot(doc, self.binary), pipeline.semantic_snapshot(after, data))
        self.assertEqual(after['bufferViews'][-1]['byteOffset'], after['bufferViews'][0]['byteOffset'])
        self.assertEqual(after['bufferViews'][-1]['name'], 'Keep_This_View')
        self.assertLess(len(data), len(self.binary))
        self.assertEqual(pipeline.encode_glb(after, data), pipeline.encode_glb(*pipeline.exact_repack(after, data)))

    def test_snapshot_detects_contract_and_visual_changes(self):
        baseline = pipeline.semantic_snapshot(self.document, self.binary)
        for mutate in (
            lambda d: d['nodes'][0].update(name='renamed'),
            lambda d: d['nodes'][0].update(translation=[1, 0, 0]),
            lambda d: d['nodes'][0].update(extras={'collider': 'lost'}),
            lambda d: d['materials'][0].update(alphaMode='BLEND'),
            lambda d: d['materials'][0].update(emissiveFactor=[1, 0, 0]),
            lambda d: d['accessors'][0].update(byteOffset=4),
            lambda d: d.update(animations=[{'name': 'retained'}]),
        ):
            doc = copy.deepcopy(self.document)
            mutate(doc)
            self.assertNotEqual(baseline, pipeline.semantic_snapshot(doc, self.binary))
        damaged = bytearray(self.binary)
        damaged[0] ^= 1
        self.assertNotEqual(baseline, pipeline.semantic_snapshot(self.document, damaged))

    def test_extensions_rejected_without_guessing_byte_references(self):
        self.document['extensionsUsed'] = ['EXT_meshopt_compression']
        with self.assertRaisesRegex(ValueError, 'extension-bearing'):
            pipeline.exact_repack(self.document, self.binary)

    def test_visual_budgets_are_separate_from_spatial_and_storage(self):
        metrics = measure(self.document, self.binary, self.source.stat().st_size, self.manifest)
        self.assertEqual(metrics['textureMemoryBytes'], sum(4 * (256 >> level)**2 for level in range(9)))
        self.assertEqual(metrics['drawCalls'], 1)
        for key in ('maxNodes', 'maxTextureMemoryBytes', 'maxDrawCalls'):
            manifest = copy.deepcopy(self.manifest)
            manifest['visualBudget'][key] = 0
            with self.assertRaisesRegex(ValueError, key):
                pipeline.validate_manifest(manifest, self.source)
        for style in ('option-a', 'option-b'):
            manifest = copy.deepcopy(self.manifest)
            manifest['visualBudget']['style'] = style
            check_visual_budget(metrics, manifest)
            self.assertEqual(manifest['quality'], self.manifest['quality'])

    def test_failing_compliance_contract_or_semantic_gate_keeps_previous_candidate(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / 'candidate.glb'
            target.write_bytes(b'previous candidate')
            ok = {'issues': {'numErrors': 0, 'numWarnings': 0, 'truncated': False}}
            bad = {'issues': {'numErrors': 1, 'numWarnings': 0, 'truncated': False}}
            with patch.object(pipeline, 'compliance', return_value=bad):
                with self.assertRaisesRegex(ValueError, 'compliance errors'):
                    pipeline.process(self.manifest, self.source, target)
            with patch.object(pipeline, 'compliance', return_value=ok), \
                 patch.object(pipeline, 'validate_manifest', side_effect=ValueError('bad quality')):
                with self.assertRaisesRegex(ValueError, 'bad quality'):
                    pipeline.process(self.manifest, self.source, target)
            with patch.object(pipeline, 'compliance', return_value=ok), \
                 patch.object(pipeline, 'validate_manifest'), \
                 patch.object(pipeline, 'semantic_snapshot', side_effect=[1, 2]):
                with self.assertRaisesRegex(ValueError, 'semantics'):
                    pipeline.process(self.manifest, self.source, target)
            self.assertEqual(target.read_bytes(), b'previous candidate')
            self.assertFalse(list(Path(directory).glob('.glb-process-*')))
        with self.assertRaisesRegex(ValueError, 'overwrite source'):
            pipeline.process(self.manifest, self.source, self.source)

    @unittest.skipUnless(VALIDATOR, 'npm ci --prefix models/tools/gltf for exporter integration')
    def test_opt_in_generic_export_and_failed_processing_preserves_runtime(self):
        import export_asset
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory)
            manifest_path = repo / 'models/assets/items/mining-tool/manifest.json'
            manifest_path.parent.mkdir(parents=True)
            manifest_path.write_text(json.dumps(self.manifest))
            (repo / self.manifest['source']).write_bytes(b'fixture source, Blender mocked')
            shutil.copy(REPO / 'models/asset-manifest.schema.json', repo / 'models')
            output = repo / self.manifest['runtime']
            output.parent.mkdir(parents=True)
            output.write_bytes(b'previous export')
            real_run = subprocess.run
            def run(command, **kwargs):
                if command[0] == '/fake/blender':
                    shutil.copyfile(self.source, command[-1])
                    return subprocess.CompletedProcess(command, 0)
                return real_run(command, **kwargs)
            with patch.object(export_asset.shutil, 'which', return_value='/fake/blender'), \
                 patch.object(export_asset.subprocess, 'run', side_effect=run):
                export_asset.export_asset(self.manifest['assetId'], repo=repo, glb_profile='exact-v1')
                report_path = output.with_suffix('.processing.json')
                report = json.loads(report_path.read_text())
                self.assertEqual(output.stat().st_size, report['after']['glbBytes'])
                previous = output.read_bytes(), report_path.read_bytes()
                with patch.object(pipeline, 'compliance', side_effect=ValueError('invalid compliance')):
                    with self.assertRaisesRegex(export_asset.ExportError, 'invalid compliance'):
                        export_asset.export_asset(self.manifest['assetId'], repo=repo, glb_profile='exact-v1')
                self.assertEqual(previous, (output.read_bytes(), report_path.read_bytes()))

    @unittest.skipUnless(VALIDATOR, 'npm ci --prefix models/tools/gltf for Khronos integration')
    def test_real_compliance_and_deterministic_disable_round_trip(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / 'candidate.glb'
            a = pipeline.process(self.manifest, self.source, target, 'exact-v1')
            first = target.read_bytes()
            b = pipeline.process(self.manifest, self.source, target, 'exact-v1')
            self.assertEqual(a, b)
            self.assertEqual(first, target.read_bytes())
            self.assertLess(a['after']['glbBytes'], a['before']['glbBytes'])
            pipeline.process(self.manifest, self.source, target, 'none')
            self.assertEqual(target.read_bytes(), self.source.read_bytes())
            # Generic validation accepts min/max-less positions; Khronos must catch it.
            doc = copy.deepcopy(self.document)
            position = doc['meshes'][0]['primitives'][0]['attributes']['POSITION']
            doc['accessors'][position].pop('min', None)
            doc['accessors'][position].pop('max', None)
            target.write_bytes(pipeline.encode_glb(doc, self.binary))
            self.assertGreater(pipeline.compliance(target)['issues']['numErrors'], 0)


if __name__ == '__main__':
    unittest.main()
