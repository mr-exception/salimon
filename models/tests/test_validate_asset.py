"""Generic validation fixtures are independent of Blender and the legacy scout."""
import copy
import json
from pathlib import Path
import shutil
import struct
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import export_asset
import validate_asset as validator

MODELS = Path(__file__).resolve().parents[1]


class ValidationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        (self.repo / 'models').mkdir()
        shutil.copy(MODELS / 'asset-manifest.schema.json', self.repo / 'models')
        self.manifest = json.loads((MODELS / 'examples/resource.manifest.json').read_text())
        self.path = self.repo / 'models/assets/resources/iron-fragment/manifest.json'
        self.path.parent.mkdir(parents=True)
        (self.repo / self.manifest['source']).write_bytes(b'fixture: source existence only')
        self.output = self.repo / self.manifest['runtime']
        self.output.parent.mkdir(parents=True)
        self.binary = struct.pack('<9f', 0, 0, 0, 1, 0, 0, 0, 1, 0)
        self.doc = {'asset': {'version': '2.0'}, 'scene': 0, 'scenes': [{'nodes': [0]}],
                    'nodes': [{'name': 'ASSET_iron-fragment', 'children': [1]},
                              {'name': 'Visual', 'children': [2]}, {'name': 'Iron_Fragment', 'mesh': 0}],
                    'buffers': [{'byteLength': len(self.binary)}],
                    'bufferViews': [{'buffer': 0, 'byteLength': len(self.binary)}],
                    'accessors': [{'bufferView': 0, 'componentType': 5126, 'type': 'VEC3', 'count': 3}],
                    'meshes': [{'primitives': [{'attributes': {'POSITION': 0}}]}]}

    def write(self):
        self.path.write_text(json.dumps(self.manifest))
        payload = json.dumps(self.doc).encode()
        payload += b' ' * (-len(payload) % 4)
        binary = self.binary + b'\0' * (-len(self.binary) % 4)
        self.output.write_bytes(struct.pack('<4sII', b'glTF', 2, 28 + len(payload) + len(binary)) +
                                struct.pack('<II', len(payload), 0x4E4F534A) + payload +
                                struct.pack('<II', len(binary), 0x004E4942) + binary)

    def run_validation(self, **registries):
        self.write()
        return validator.validate_asset(self.manifest['assetId'], self.repo, **registries)

    def rejects(self, text):
        with self.assertRaisesRegex(export_asset.ExportError, text):
            self.run_validation()

    def test_generic_resource_and_item_metrics(self):
        for category in ('resources', 'items', 'props', 'structures', 'vehicles'):
            self.manifest['category'] = category
            self.assertEqual(self.run_validation()['maxTriangles'], 1)

    def test_malformed_manifest_and_missing_contract(self):
        self.manifest['coordinates']['source']['unitScale'] = 0.01
        self.rejects('unitScale')
        self.manifest['coordinates']['source']['unitScale'] = 1
        self.manifest['contracts']['nodes'].append('Missing')
        self.rejects('resource.iron-fragment.*Missing')

    def test_source_runtime_paths_and_identity_uniqueness(self):
        source = self.repo / self.manifest['source']
        source.unlink()
        self.rejects('resource.iron-fragment.*missing Blender')
        source.symlink_to(self.repo / 'outside.blend')
        self.rejects('escapes models/assets')
        source.unlink()
        source.write_bytes(b'fixture')
        self.write()
        other = self.path.parent.parent / 'other/manifest.json'
        other.parent.mkdir()
        other.write_text(self.path.read_text())
        self.rejects('duplicate assetId')

    def test_texture_references_and_required_uvs(self):
        self.doc['materials'] = [{'pbrMetallicRoughness': {'baseColorTexture': {'index': 0}}}]
        self.doc['meshes'][0]['primitives'][0]['material'] = 0
        self.rejects('material 0.texture: invalid index')
        self.doc['textures'] = [{'source': 0}]
        self.rejects('texture 0.source: invalid index')
        self.doc['images'] = [{'bufferView': 0, 'mimeType': 'image/png'}]
        self.rejects('textured material needs TEXCOORD_0')
        self.doc['meshes'][0]['primitives'][0]['attributes']['TEXCOORD_0'] = 0
        self.rejects('TEXCOORD_0: expected float VEC2')

    def test_all_budgets(self):
        self.doc['materials'] = [{'name': 'a'}, {'name': 'b'}]
        self.doc['images'] = [{'bufferView': 0, 'mimeType': 'image/png'}]
        self.doc['meshes'][0]['primitives'] *= 2
        for budget in self.manifest['budgets']:
            original = self.manifest['budgets'][budget]
            self.manifest['budgets'][budget] = 1
            self.rejects(budget)
            self.manifest['budgets'][budget] = original

    def test_duplicate_nodes_materials_and_extras(self):
        self.doc['nodes'][2]['name'] = 'Visual'
        self.rejects('duplicate exported node')
        self.doc['nodes'][2]['name'] = 'Iron_Fragment'
        self.doc['materials'] = [{'name': 'same'}, {'name': 'same'}]
        self.rejects('duplicate material')
        self.doc.pop('materials')
        self.manifest['contracts']['requiredExtras'] = [{'node': 'Iron_Fragment', 'path': 'salimon.role'}]
        self.rejects('missing exported extras')
        self.doc['nodes'][2]['extras'] = {'salimon': {'role': 'fragment'}}
        self.run_validation()

    def test_bad_transforms(self):
        for field, value, message in (('translation', [float('nan'), 0, 0], 'finite'),
                                      ('scale', [-1, 1, 1], 'apply scale'),
                                      ('scale', [1, 2, 1], 'apply scale'),
                                      ('rotation', [0, 0, 0, 2], 'unit quaternion'),
                                      ('matrix', [0]*16, 'affine')):
            with self.subTest(field=field, value=value):
                self.doc['nodes'][2][field] = value
                self.rejects(message)
                del self.doc['nodes'][2][field]

    def test_hierarchy_and_visual_membership(self):
        self.doc['nodes'][0]['children'].append(2)
        self.rejects('multiple parents')
        self.doc['nodes'][0]['children'] = [1]
        self.doc['nodes'][2]['children'] = [0]
        self.rejects('root must have no parent')
        self.doc['nodes'][2].pop('children')
        self.doc['nodes'][1]['children'] = []
        self.rejects('all exported nodes')
        self.doc['nodes'][0]['children'].append(2)
        self.rejects('geometry outside visual')

    def add_spatial(self, role, name, group):
        i = len(self.doc['nodes'])
        self.doc['nodes'][0]['children'].append(i)
        self.doc['nodes'].extend([{'name': group, 'children': [i+1]}, {'name': name}])
        self.manifest['contracts'][role].append(name)
        return self.doc['nodes'][-1]

    def test_socket_marker_and_collision_contracts(self):
        self.add_spatial('sockets', 'SOCKET_Grip', 'Sockets')
        self.add_spatial('markers', 'MARKER_Use', 'Interaction_Markers')
        proxy = self.add_spatial('colliders', 'COLLIDER_Bounds', 'Collision_Proxies')
        self.manifest['collision'] = {'policy': 'authored-proxies', 'shapes': ['box']}
        self.rejects('shape is not declared')
        proxy['extras'] = {'salimon': {'shape': 'box'}, 'salimonProxyDimensions': [1, 2, 3]}
        self.run_validation()
        proxy['extras']['salimonProxyDimensions'] = [0, 2, 3]
        self.rejects('positive meters')
        proxy['extras']['salimonProxyDimensions'] = [1, 2, 3]
        self.manifest['contracts']['markers'].append('SOCKET_Grip')
        self.rejects('roles must not overlap')

    def test_lod_checks(self):
        self.doc['nodes'][1]['children'] = [3]
        self.doc['nodes'].append({'name': 'LOD0', 'children': [2]})
        self.manifest['lods'] = [{'level': 0, 'node': 'LOD0', 'maxTriangles': 1, 'selection': 'near'}]
        self.run_validation()
        self.manifest['lods'][0]['level'] = 2
        self.rejects('consecutively')
        self.manifest['lods'][0]['level'] = 0
        self.doc['nodes'][3]['translation'] = [1, 0, 0]
        self.rejects('shared identity pivot')

    def test_extension_and_collision_dispatch_fail_closed(self):
        self.manifest['extensions'] = {'resource': {'version': 1, 'data': {'expected': True}}}
        self.rejects('extensions.resource.*unsupported')
        called = []
        def check(context, data):
            called.append((context['metrics']['maxTriangles'], data['expected']))
        self.run_validation(extension_validators={('resources', 'resource', 1): check})
        self.assertEqual(called, [(1, True)])
        self.manifest.pop('extensions')
        self.manifest['collision'] = {'policy': 'generated', 'generator': 'fixture-v1'}
        self.rejects('collision.generator.*unsupported')
        self.run_validation(collision_validators={'fixture-v1': lambda context: called.append(context['manifest']['assetId'])})
        self.assertEqual(called[-1], 'resource.iron-fragment')

    def test_corrupt_glb_and_geometry(self):
        self.binary = struct.pack('<9f', float('inf'), *([0]*8))
        self.rejects('nonfinite geometry')
        self.binary = struct.pack('<9f', *([0]*9))
        self.doc['accessors'][0]['count'] = 4
        self.rejects('range exceeds bufferView')
        self.doc['accessors'][0]['count'] = 3
        self.write()
        self.output.write_bytes(self.output.read_bytes()[:-1])
        with self.assertRaisesRegex(validator.ValidationError, 'invalid GLB header'):
            validator.validate_asset(self.manifest['assetId'], self.repo)

    def test_failed_semantic_validation_preserves_export_destination(self):
        self.write()
        original = self.output.read_bytes()
        self.manifest['budgets']['maxGlbBytes'] = 1
        self.path.write_text(json.dumps(self.manifest))
        def fake_blender(command, **kwargs):
            Path(command[-1]).write_bytes(original)
            return type('Result', (), {'returncode': 0})()
        with patch.object(export_asset.shutil, 'which', return_value='/fake/blender'), patch.object(export_asset.subprocess, 'run', side_effect=fake_blender):
            with self.assertRaisesRegex(export_asset.ExportError, 'maxGlbBytes'):
                export_asset.export_asset(self.manifest['assetId'], repo=self.repo)
        self.assertEqual(self.output.read_bytes(), original)


if __name__ == '__main__':
    unittest.main()
