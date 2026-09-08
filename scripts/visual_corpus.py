"""Versioned review records for independently approved renderer image corpora."""
import hashlib
import json
import math
from pathlib import Path
import re

SCHEMA = 1
COMPATIBLE_FIELDS = ('backend', 'adapter', 'driver', 'platform', 'viewport_pixels', 'scale_factor')


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def contract_hash(fixtures):
    # Include source actions/assertions, seeds and the exact reviewed masks/tolerances.
    payload = {name: {'fixture': fixture, 'scenario': json.loads(Path(fixture['scenario']).read_text())}
               for name, fixture in fixtures.items()}
    return hashlib.sha256(json.dumps(payload, sort_keys=True).encode()).hexdigest()


def write_json(path, data):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, allow_nan=False) + '\n')


def image_hashes(root, fixtures):
    from model_visual_tests import expanded_scenario
    images = {}
    for name, fixture in fixtures.items():
        if not re.fullmatch(r'[A-Za-z0-9_-]+', name):
            raise ValueError('invalid fixture directory')
        _, expected = expanded_scenario(fixture)
        actual = {p.name for p in (Path(root)/name).glob('*.png')}
        if actual != expected:
            raise ValueError(f'incomplete fixture evidence: {name}')
        images.update({f'{name}/{filename}': digest(Path(root)/name/filename) for filename in sorted(expected)})
    actual = {p.relative_to(root).as_posix() for p in Path(root).rglob('*.png')}
    if actual != set(images):
        raise ValueError('unexpected image outside fixture contract')
    return images


def validate_context(context):
    if any(not isinstance(context.get(k), str) or not context[k].strip()
           for k in ('backend', 'adapter', 'driver', 'platform')):
        raise ValueError('capture requires backend/adapter/driver/platform metadata')
    size = context.get('viewport_pixels')
    if not isinstance(size, list) or len(size) != 2 or any(type(v) is not int or v < 1 for v in size):
        raise ValueError('invalid physical viewport metadata')
    scale = context.get('scale_factor')
    if type(scale) not in (float, int) or not math.isfinite(scale) or scale <= 0:
        raise ValueError('invalid DPI scale factor')


def candidate(root, fixtures, context, revision, binary_sha256):
    validate_context(context)
    if not re.fullmatch('[0-9a-f]{40}', revision) or not re.fullmatch('[0-9a-f]{64}', binary_sha256):
        raise ValueError('invalid capture revision/binary digest')
    from PIL import Image
    images = image_hashes(root, fixtures)
    for name in images:
        with Image.open(Path(root)/name) as image:
            if list(image.size) != context['viewport_pixels']:
                raise ValueError('capture size does not match observed viewport')
    return {'schema': SCHEMA, 'status': 'candidate', 'context': context,
            'revision': revision, 'binary_sha256': binary_sha256,
            'fixture_contract_sha256': contract_hash(fixtures), 'images': images}


def reviewed_status(captures, baselines, fixtures):
    """Never promote or overwrite images. Pending coverage is not an appearance pass."""
    path = Path(baselines)/'corpus.json'
    if not path.is_file():
        return {'status': 'missing', 'passed': False, 'reason': 'no reviewed corpus metadata'}
    baseline = json.loads(path.read_text())
    if baseline.get('schema') != SCHEMA:
        raise ValueError('unsupported visual corpus schema')
    if baseline.get('status') in ('pending', 'candidate'):
        return {'status': baseline['status'], 'passed': False, 'reason': 'human image/mask/tolerance approval pending'}
    if baseline.get('status') != 'approved':
        raise ValueError('invalid visual corpus approval status')
    review = baseline.get('review', {})
    if (not all(isinstance(review.get(k), str) and review[k].strip() for k in ('reviewer', 'reviewed_at', 'evidence'))
            or review.get('masks_and_tolerances_reviewed') is not True):
        raise ValueError('approved corpus requires explicit image/mask/tolerance review')
    capture = json.loads((Path(captures)/'corpus.json').read_text())
    for root, metadata in ((baselines, baseline), (captures, capture)):
        validate_context(metadata.get('context', {}))
        if metadata.get('schema') != SCHEMA or metadata.get('fixture_contract_sha256') != contract_hash(fixtures):
            raise ValueError('visual fixture actions/seeds/tolerances changed; review a new corpus')
        if (not re.fullmatch('[0-9a-f]{40}', metadata.get('revision', ''))
                or not re.fullmatch('[0-9a-f]{64}', metadata.get('binary_sha256', ''))):
            raise ValueError('invalid corpus source provenance')
        if metadata.get('images') != image_hashes(root, fixtures):
            raise ValueError('corpus image hashes differ from reviewed/captured evidence')
        from PIL import Image
        for name in metadata['images']:
            with Image.open(Path(root)/name) as image:
                if list(image.size) != metadata['context']['viewport_pixels']:
                    raise ValueError('incompatible corpus image size/viewport metadata')
    for field in COMPATIBLE_FIELDS:
        if baseline['context'][field] != capture['context'][field]:
            raise ValueError(f'incompatible visual corpus {field}; use an independently reviewed corpus')
    return {'status': 'approved', 'passed': None, 'baseline_revision': baseline['revision'],
            'capture_revision': capture['revision']}
