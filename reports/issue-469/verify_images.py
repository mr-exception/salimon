"""Read-only verification of original chart captures and matching native states."""
import json
from pathlib import Path
from PIL import Image
ROOT = Path(__file__).resolve().parent
ROI = (520, 240, 760, 475)
metrics = {}
for label in ('empty', 'observed', 'resumed', 'historical'):
    with Image.open(ROOT / f'{label}.png') as image:
        assert image.size == (1280, 800)
        region = image.convert('RGB').crop(ROI)
        pixels = [region.getpixel((x, y)) for y in range(region.height) for x in range(region.width)]
    metrics[label] = {
        'cyan_pixels': sum(g-r > 40 and b-r > 40 and g > 130 and b > 140 for r, g, b in pixels),
        'selected_amber_pixels': sum(r > 220 and g > 130 and b < 120 for r, g, b in pixels),
    }
assert metrics['empty']['cyan_pixels'] == 0
assert metrics['empty']['selected_amber_pixels'] == 0
for label in ('observed', 'resumed', 'historical'):
    assert metrics[label]['cyan_pixels'] >= 50
    assert metrics[label]['selected_amber_pixels'] >= 20
states = json.loads((ROOT / 'state-summaries.json').read_text())
assert states['empty']['chart']['entry_count'] == 0
for label in ('observed', 'resumed', 'historical'):
    assert states[label]['chart']['entry_count'] == 1
    assert states[label]['chart']['route_count'] == 1
assert states['historical']['scanner']['active_count'] == 0
assert states['historical']['chart']['mode']
assert states['observed']['chart']['source']['routes'] == states['historical']['chart']['source']['routes']
print(json.dumps({'viewport': [1280, 800], 'roi': ROI, 'metrics': metrics, 'source_checks': 'passed'}, indent=2))
