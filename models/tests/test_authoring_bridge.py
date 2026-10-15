import json
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import authoring_bridge as bridge


class BridgeTests(unittest.TestCase):
    def test_protocol_and_notifications(self):
        operation = lambda *args: {'objects': []}
        self.assertIsNone(bridge.respond({'method': 'notifications/initialized'}, operation))
        for method in ('initialize', 'tools/list', 'ping'):
            result = bridge.respond({'id': 1, 'method': method}, operation)
            self.assertIn('result', result)
            json.dumps(result)
        self.assertEqual(bridge.respond({'id': 1, 'method': 'eval'}, operation)['error']['code'], -32601)

    def test_no_eval_or_untyped_arguments(self):
        for name, args in [('execute_code', {'code': 'bad'}), ('inspect_asset', {'asset': 'x', 'code': 'bad'})]:
            with self.assertRaises(ValueError):
                bridge.run_operation(name, args, 'blender', '/tmp')

    def test_writes_confined_to_scratch(self):
        with patch.object(bridge, 'resolve_manifest', return_value=(None, {'assetId': 'test'})):
            with self.assertRaisesRegex(ValueError, 'artifacts'):
                bridge.run_operation('inspect_asset', {'asset': 'x'}, 'blender', bridge.REPO / 'models')

    def test_failed_operation_is_mcp_error(self):
        def fail(*args):
            raise ValueError('bad asset')
        result = bridge.respond({'id': 3, 'method': 'tools/call', 'params': {
            'name': 'inspect_asset', 'arguments': {'asset': 'x'}}}, fail)
        self.assertTrue(result['result']['isError'])

    def test_headless_inspection_preserves_saved_source(self):
        import hashlib
        import os
        blender = os.environ.get('BLENDER')
        if not blender:
            self.skipTest('BLENDER required for source inspection')
        source = bridge.REPO / 'models/assets/items/cargo-container/source.blend'
        before = hashlib.sha256(source.read_bytes()).hexdigest()
        result = bridge.run_operation('inspect_asset', {'asset': 'item.cargo-container'},
                                      blender, bridge.REPO / 'artifacts/authoring-test')
        self.assertTrue(result['objects'])
        self.assertEqual(hashlib.sha256(source.read_bytes()).hexdigest(), before)
