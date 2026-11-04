"""Verify the retained production framebuffer observations without re-rendering."""
import json
import pathlib
import struct
import sys

root = pathlib.Path(sys.argv[1])
for method in ('keyboard', 'mouse'):
    data = json.loads((root / f'{method}-frames.json').read_text())
    capture = (root / f'{method}-visible.png').read_bytes()
    width, height = struct.unpack('>II', capture[16:24])
    moving = 0
    previous = None
    observations = 0
    for frame in data['frames']:
        assert '4 temporary contacts' in frame['pulse']
        ship = frame['ship']
        assert ship is not None
        if previous is not None and ship != previous:
            moving += 1
        previous = ship
        for body in frame['bodies']:
            assert body['present'] and body['bright'] >= 10
            assert all(abs(a - b) < 1e-5 for a, b in zip(body['pixel'], body['actualPixel']))
            observations += 1
        # Fixture: front orthographic view, +X heading and stationary velocity.
        heading, velocity = frame['vectors']
        assert velocity['display'] == 'none'
        if ship[0] < width and 0 <= ship[1] <= height:
            assert heading['display'] != 'none'
            expected = [max(0, ship[0] / width * 100), ship[1] / height * 100, 100, ship[1] / height * 100]
            assert all(abs(a - b) < 1e-5 for a, b in zip(heading['start'] + heading['end'], expected))
        else:
            assert heading['display'] == 'none'
    assert moving > 5 and observations > 20
    assert {body['id'] for frame in data['frames'] for body in frame['bodies']} == {'star', 'planet', 'moon', 'asteroid'}
    print(f'{method}: {len(data["frames"])} frames, {moving} movement frames, {observations} body observations; mesh/pixel/projection/vector/scanner checks pass')
