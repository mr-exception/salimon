"""Inject a persistent patch in disposable production captures; never approve a corpus.

Run from the repository root after capture. This exercises pixel comparison
against the candidate images; human approval remains a separate requirement.
"""
import json
from pathlib import Path
import shutil
import subprocess
import sys
from PIL import Image, ImageDraw
sys.path.insert(0, str(Path.cwd()/'scripts'))
from model_visual_tests import evaluate
from visual_corpus import candidate, write_json

source = Path('artifacts/model-visual/candidates')
output = Path('artifacts/model-visual/injected-change')
output.mkdir(parents=True, exist_ok=True)
fixtures = json.loads(Path('scenarios/visual/fixtures.json').read_text())
fixture = fixtures['core']
changed = output/'core'
if changed.exists():
    shutil.rmtree(changed)
shutil.copytree(source/'core', changed)
for path in changed.glob('*.png'):
    with Image.open(path) as original:
        image = original.convert('RGB')
    x, y = image.width//2, image.height//2
    ImageDraw.Draw(image).rectangle((x-60, y-60, x+59, y+59), fill=(255, 0, 255))
    image.save(path)
result = evaluate(changed, source/'core', output/'diffs', fixture)
temporal = all(r['passed'] for r in result['results'] if r['kind']=='temporal')
assert temporal, 'injection must remain temporally stable'
assert not result['passed'], 'persistent injected change must fail appearance'
assert all(not r['passed'] for r in result['results'] if r['kind']=='baseline')
summary = {'status': 'passed', 'test': 'persistent post-capture 120x120 pixel patch',
           'fixture': 'core', 'temporal_passed': temporal,
           'appearance_passed': result['passed'],
           'approval': 'pending; direct pixel comparison does not certify reviewed appearance',
           'results': result['results']}
baseline = Path('scenarios/visual/baselines/software-vulkan-option-a-v1')
if json.loads((baseline/'corpus.json').read_text()).get('status') == 'approved':
    # Refresh injected image hashes so the reviewed CLI must detect the pixel
    # regression, rather than stopping at a stale provenance/hash error.
    injected = output/'reviewed-captures'
    if injected.exists():
        shutil.rmtree(injected)
    shutil.copytree(source, injected)
    for path in changed.glob('*.png'):
        shutil.copyfile(path, injected/'core'/path.name)
    original = json.loads((source/'corpus.json').read_text())
    metadata = candidate(injected, fixtures, original['context'],
                         original['revision'], original['binary_sha256'])
    metadata['test_injection'] = 'post-capture persistent 120x120 Core patch'
    write_json(injected/'corpus.json', metadata)
    checked = subprocess.run([sys.executable, 'scripts/model_visual_tests.py', 'compare',
        '--reviewed', '--captures', str(injected), '--baselines', str(baseline),
        '--output', str(output/'reviewed-diffs')], capture_output=True, text=True)
    assert checked.returncode == 1, checked.stdout + checked.stderr
    coverage = json.loads((output/'reviewed-diffs/appearance.json').read_text())
    assert coverage['status'] == 'failed' and coverage['passed'] is False
    summary.update(reviewed_exit_code=checked.returncode, reviewed_coverage=coverage,
                   approval='approved baseline; injected corpus is disposable test evidence')
(output/'injection.json').write_text(json.dumps(summary, indent=2)+'\n')
print(json.dumps(summary))
