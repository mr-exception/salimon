#!/usr/bin/env python3
"""Salimon authoring protocol v1: opt-in stdio MCP, no sockets or Python eval.

Each operation starts a fresh Blender process. Only scratch output is writable;
tracked assets and linked libraries are never saved by this bridge.
"""
import argparse
import json
from pathlib import Path
import subprocess
import sys

from export_asset import REPO, resolve_manifest

VERSION = '1.0.0'
TOOLS = [
    {'name': name, 'description': description, 'inputSchema': {
        'type': 'object', 'properties': {'asset': {'type': 'string'}},
        'required': ['asset'], 'additionalProperties': False}}
    for name, description in (
        ('inspect_asset', 'List source objects, metric bounds, transforms and library paths.'),
        ('preview_asset', 'Render front, side and three-quarter source views to scratch.'),
        ('sample_palette', 'Write an isolated Option A palette edit; never save production source.'))
]


def run_operation(name, arguments, blender, output):
    if name not in {t['name'] for t in TOOLS}:
        raise ValueError('unknown operation')
    if set(arguments) != {'asset'} or not isinstance(arguments['asset'], str):
        raise ValueError('expected only an asset string')
    _, manifest = resolve_manifest(arguments['asset'], REPO)
    output = Path(output).resolve()
    if not output.is_relative_to((REPO / 'artifacts').resolve()):
        raise ValueError('output must be below repository artifacts/')
    destination = output / manifest['assetId'] / name
    destination.mkdir(parents=True, exist_ok=True)
    request = destination / 'request.json'
    request.write_text(json.dumps({'operation': name, 'manifest': manifest,
                                  'output': str(destination)}))
    command = [blender, '--background', '--factory-startup', '--disable-autoexec',
               str(REPO / manifest['source']), '--python-exit-code', '1',
               '--python', str(REPO / 'models/tools/blender_authoring.py'), '--', str(request)]
    subprocess.run(command, check=True, timeout=180, stdout=sys.stderr, stderr=sys.stderr)
    return json.loads((destination / 'inspection.json').read_text())


def respond(request, operation):
    method = request.get('method')
    if 'id' not in request:
        return None
    response = {'jsonrpc': '2.0', 'id': request['id']}
    try:
        if method == 'initialize':
            result = {'protocolVersion': '2024-11-05', 'capabilities': {'tools': {}},
                      'serverInfo': {'name': 'salimon-authoring', 'version': VERSION}}
        elif method == 'ping':
            result = {}
        elif method == 'tools/list':
            result = {'tools': TOOLS}
        elif method == 'tools/call':
            params = request['params']
            try:
                value = operation(params['name'], params.get('arguments', {}))
                result = {'content': [{'type': 'text', 'text': json.dumps(value, sort_keys=True)}]}
            except (ValueError, OSError, subprocess.SubprocessError) as error:
                result = {'isError': True, 'content': [{'type': 'text', 'text': str(error)}]}
        else:
            response['error'] = {'code': -32601, 'message': 'Unknown method'}
            return response
        response['result'] = result
    except (KeyError, TypeError) as error:
        response['error'] = {'code': -32602, 'message': str(error)}
    return response


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--blender', required=True)
    parser.add_argument('--output', default=str(REPO / 'artifacts/authoring'))
    parser.add_argument('--operation', choices=[t['name'] for t in TOOLS])
    parser.add_argument('--asset')
    args = parser.parse_args()
    operation = lambda name, arguments: run_operation(name, arguments, args.blender, args.output)
    if args.operation:
        print(json.dumps(operation(args.operation, {'asset': args.asset}), indent=2, sort_keys=True))
        return
    for line in sys.stdin:
        try:
            reply = respond(json.loads(line), operation)
        except (ValueError, TypeError):
            reply = {'jsonrpc': '2.0', 'id': None,
                     'error': {'code': -32700, 'message': 'Invalid JSON request'}}
        if reply is not None:
            print(json.dumps(reply), flush=True)


if __name__ == '__main__':
    main()
