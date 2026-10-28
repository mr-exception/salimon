"""Verify original 1280x800 scanner captures; never edits/promotes images."""
import json
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parent
ROI = (520, 240, 760, 475)
metrics = {}
for label in ('undiscovered', 'identified', 'blackout'):
    with Image.open(ROOT / f'{label}.png') as image:
        assert image.size == (1280, 800), (label, image.size)
        region = image.convert('RGB').crop(ROI)
        pixels = [region.getpixel((x, y)) for y in range(region.height) for x in range(region.width)]
    metrics[label] = {
        'cyan_pixels': sum(g-r > 40 and b-r > 40 and g > 130 and b > 140 for r, g, b in pixels),
        'selected_amber_pixels': sum(r > 220 and g > 130 and b < 120 for r, g, b in pixels),
    }
assert metrics['undiscovered']['cyan_pixels'] == 0
assert metrics['undiscovered']['selected_amber_pixels'] == 0
assert metrics['identified']['cyan_pixels'] >= 50
assert metrics['identified']['selected_amber_pixels'] >= 20
assert metrics['blackout']['cyan_pixels'] == 0
assert metrics['blackout']['selected_amber_pixels'] == 0
print(json.dumps({'viewport': [1280, 800], 'roi': ROI, 'metrics': metrics}, indent=2))
