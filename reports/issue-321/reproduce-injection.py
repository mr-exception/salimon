"""Inject a persistent patch in disposable production captures; never approve a corpus.

Run from the repository root after capture. This exercises pixel comparison
against the candidate images; human approval remains a separate requirement.
"""
import json
from pathlib import Path
import shutil
import sys
from PIL import Image, ImageDraw
sys.path.insert(0, str(Path.cwd()/'scripts'))
from model_visual_tests import evaluate

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
(output/'injection.json').write_text(json.dumps(summary, indent=2)+'\n')
print(json.dumps(summary))
