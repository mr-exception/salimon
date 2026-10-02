"""Schema-only tests; exported asset validation is implemented by issue #78."""
import copy
import json
from pathlib import Path
import unittest

from jsonschema import Draft202012Validator

MODELS = Path(__file__).resolve().parents[1]


class ManifestSchemaTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.schema = json.loads((MODELS / 'asset-manifest.schema.json').read_text())
        cls.validator = Draft202012Validator(cls.schema)
        cls.examples = {
            kind: json.loads((MODELS / 'examples' / f'{kind}.manifest.json').read_text())
            for kind in ('ship', 'resource', 'item')
        }

    def test_schema_and_examples(self):
        Draft202012Validator.check_schema(self.schema)
        for kind, manifest in self.examples.items():
            with self.subTest(kind=kind):
                self.validator.validate(manifest)

    def test_missing_required_fields(self):
        for key in self.schema['required']:
            with self.subTest(field=key):
                manifest = copy.deepcopy(self.examples['resource'])
                del manifest[key]
                self.assertFalse(self.validator.is_valid(manifest))

    def test_invalid_contracts_and_configuration(self):
        cases = [
            ('schemaVersion', 2), ('assetId', 'models/assets/rock'),
            ('source', '/tmp/source.blend'), ('source', 'models/../source.blend'),
            ('source', 'models/source.obj'), ('source', r'models\source.blend'),
            ('runtime', 'outside/model.glb'), ('runtime', 'client/assets/../model.glb'),
            ('runtime', 'client/assets/rock/model.gltf'),
            ('budgets.maxTriangles', 0), ('budgets.maxGlbBytes', 1.5),
            ('coordinates.source.unitScale', 0.01), ('coordinates.runtime.up', '+Z'),
            ('contracts.nodes', ['Rock', 'Rock']),
            ('contracts.requiredExtras', [{'node': 'Rock', 'path': 'foreign.value'}]),
            ('collision', {'policy': 'authored-proxies', 'shapes': ['box']}),
            ('collision', {'policy': 'generated'}),
            ('collision', {'policy': 'none', 'shapes': ['box']}),
            ('profile', 'legacy-scout-v1'), ('extensions.item', {'data': {}}),
            ('lods', [{'level': -1, 'node': 'LOD0', 'maxTriangles': 10, 'selection': 'near'}]),
            ('typo', True),
        ]
        for path, value in cases:
            with self.subTest(path=path, value=value):
                manifest = copy.deepcopy(self.examples['resource'])
                target = manifest
                keys = path.split('.')
                for key in keys[:-1]:
                    target = target.setdefault(key, {})
                target[keys[-1]] = value
                self.assertFalse(self.validator.is_valid(manifest))

    def test_category_and_optional_features_do_not_require_ship_contracts(self):
        manifest = copy.deepcopy(self.examples['resource'])
        manifest['category'] = 'future-category'
        manifest['assetId'] = 'future.rock'
        manifest['collision'] = {'policy': 'generated', 'generator': 'bounds-v1'}
        manifest['lods'] = [
            {'level': 0, 'node': 'LOD0', 'maxTriangles': 500, 'selection': 'distance < 20m'},
            {'level': 1, 'node': 'LOD1', 'maxTriangles': 100, 'selection': 'distance >= 20m'},
        ]
        manifest['extensions'] = {'future': {'version': 1, 'data': {'custom': True}}}
        self.validator.validate(manifest)

    def test_scout_example_covers_current_named_contracts(self):
        repo = MODELS.parent
        gltf = json.loads((repo / 'client/assets/ship/export/salimon_phase0_ship.gltf').read_text())
        nodes = {node['name']: node for node in gltf['nodes']}
        contract = self.examples['ship']['contracts']
        for key in ('visualGroups', 'groups', 'nodes', 'sockets', 'markers', 'colliders'):
            self.assertTrue(set(contract[key]) <= nodes.keys(), key)
        self.assertEqual(set(contract['colliders']), {n for n in nodes if n.startswith('COLLIDER_')})
        self.assertEqual(set(contract['markers']), {n for n in nodes if n.startswith('MARKER_')})
        self.assertEqual(set(contract['materials']), {m['name'] for m in gltf['materials']})
        self.assertEqual({x['node'] for x in contract['requiredExtras']},
                         {n for n, node in nodes.items() if 'salimon' in node.get('extras', {})})


if __name__ == '__main__':
    unittest.main()
