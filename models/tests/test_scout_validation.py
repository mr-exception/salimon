"""Mutation regressions for the direct Blender scout category validator."""
import copy
import importlib.abc
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from test_scout_export import ASSET, REPO, scout
from validate import REGISTRIES, main
from validate_asset import ValidationError, accessor_values, read_document, validate_manifest


class ScoutValidationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.manifest = json.loads((ASSET / 'manifest.json').read_text())
        cls.output = REPO / cls.manifest['runtime']
        cls.document, cls.binary = read_document(cls.output)

    def validate(self, document=None, binary=None, manifest=None):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'scout.glb'
            path.write_bytes(scout.encode_glb(document or self.document,
                                             self.binary if binary is None else binary))
            return validate_manifest(manifest or self.manifest, path, REPO,
                                     extension_validators=REGISTRIES)

    def vertex_edit(self, name, semantic, edit):
        document = copy.deepcopy(self.document)
        binary = bytearray(self.binary)
        node = next(n for n in document['nodes'] if n['name'] == name)
        primitive = document['meshes'][node['mesh']]['primitives'][0]
        accessor = document['accessors'][primitive['attributes'][semantic]]
        view = document['bufferViews'][accessor['bufferView']]
        width = {'VEC2': 2, 'VEC3': 3}[accessor['type']]
        start = view.get('byteOffset', 0) + accessor.get('byteOffset', 0)
        stride = view.get('byteStride', width * 4)
        for row in range(accessor['count']):
            offset = start + row * stride
            values = struct.unpack_from('<' + 'f' * width, binary, offset)
            struct.pack_into('<' + 'f' * width, binary, offset, *edit(values))
        # Deliberately keep min/max unchanged: actual vertices must be measured.
        return document, bytes(binary)

    def test_direct_validator_without_geometry_generator(self):
        class RejectGenerator(importlib.abc.MetaPathFinder):
            def find_spec(self, fullname, path, target=None):
                if fullname == 'generate_salimon_phase0_ship':
                    raise AssertionError('validator tried to load procedural geometry')
        with patch.object(sys, 'meta_path', [RejectGenerator(), *sys.meta_path]):
            main()
        self.assertNotIn('generate_salimon_phase0_ship', sys.modules)

    def test_uv_normals_center_assembly_and_real_dimensions(self):
        cases = (
            ('Monitor_Center', 'TEXCOORD_0', lambda p: (p[0], 1-p[1])),
            ('Monitor_Port', 'NORMAL', lambda p: tuple(-v for v in p)),
            ('Cockpit_Monitor_Housings', 'POSITION', lambda p: (p[0] + .05, p[1], p[2])),
            ('Wing_Port', 'POSITION', lambda p: (p[0], p[1], p[2] + 20)),
        )
        for name, semantic, edit in cases:
            with self.subTest(name=name):
                document, binary = self.vertex_edit(name, semantic, edit)
                with self.assertRaises(ValidationError):
                    self.validate(document, binary)

    def test_window_sightline_regression(self):
        # Reposition an existing opaque wall in the seated forward ray while
        # keeping the ship envelope and material/node contract unchanged.
        document, binary = self.vertex_edit('Wall_Port_Inner', 'POSITION',
                                            lambda p: (6.5, p[1], p[0]))
        with self.assertRaisesRegex(ValidationError, 'obstructed|blocked'):
            self.validate(document, binary)

    def test_collision_anchor_material_and_budget_regressions(self):
        for name, mutate in (
            ('collision', lambda d: next(n for n in d['nodes'] if n['name'] == 'COLLIDER_AftDoor')['extras']['salimon'].update(sizeMeters=[.1, .1, .1])),
            ('marker', lambda d: next(n for n in d['nodes'] if n['name'] == 'MARKER_CockpitSeat').update(translation=[0, 0, 0])),
            ('material', lambda d: next(m for m in d['materials'] if m['name'] == 'Cockpit Glass').update(alphaMode='OPAQUE')),
            ('stale metrics', lambda d: d['extras']['salimon'].update(triangleCount=1)),
        ):
            with self.subTest(name=name):
                document = copy.deepcopy(self.document)
                mutate(document)
                with self.assertRaises(ValidationError):
                    self.validate(document)
        manifest = copy.deepcopy(self.manifest)
        manifest['budgets']['maxTriangles'] = 1
        with self.assertRaisesRegex(ValidationError, 'maxTriangles'):
            self.validate(manifest=manifest)

    def test_validation_is_active_with_optimized_python(self):
        with tempfile.TemporaryDirectory() as directory:
            export = Path(directory)
            document = copy.deepcopy(self.document)
            document['extras']['salimon']['triangleCount'] = 1
            (export / 'salimon_phase0_ship.glb').write_bytes(scout.encode_glb(document, self.binary))
            code = f'import sys; sys.path.insert(0, {str(ASSET)!r}); from validate import main; from pathlib import Path; main(Path({directory!r}))'
            result = subprocess.run([sys.executable, '-O', '-c', code], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('ValidationError', result.stderr)

    def test_ship_semantic_failure_preserves_published_export(self):
        paths = [self.output.with_suffix(s) for s in ('.glb', '.bin', '.gltf')]
        paths.extend([ASSET / 'export-report.json', REPO / 'client/assets/ship/spatial-contracts.json'])
        paths.extend(REPO / 'client/character/src' / n for n in ('cargo_layout.rs', 'thruster_collision.rs', 'ship_anchors.rs'))
        before = {p: p.read_bytes() for p in paths}
        def blender(command, **kwargs):
            document, binary = self.vertex_edit('Monitor_Center', 'TEXCOORD_0', lambda p: (p[0], 1-p[1]))
            Path(command[-1]).write_bytes(scout.encode_glb(document, binary))
        with patch.object(scout.shutil, 'which', return_value='/mock/blender'), \
             patch.object(scout.subprocess, 'run', side_effect=blender):
            with self.assertRaises(ValidationError):
                scout.export()
        self.assertEqual(before, {p: p.read_bytes() for p in paths})

    def test_shared_accessor_offsets_strides_and_uint32_indices(self):
        binary = struct.pack('<IffIff', 99, 1, 2, 88, 3, 4)
        document = {'bufferViews': [{'byteOffset': 0, 'byteStride': 12}],
                    'accessors': [{'bufferView': 0, 'byteOffset': 4, 'count': 2, 'type': 'VEC2', 'componentType': 5126},
                                  {'bufferView': 0, 'count': 2, 'type': 'SCALAR', 'componentType': 5125}]}
        self.assertEqual(accessor_values(document, binary, 0), [(1, 2), (3, 4)])
        self.assertEqual(accessor_values(document, binary, 1), [(99,), (88,)])


if __name__ == '__main__':
    unittest.main()
