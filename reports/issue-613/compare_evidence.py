"""Recheck retained original images and production-native outcomes; no baseline promotion."""
from pathlib import Path
import json
from PIL import Image, ImageChops

ROOT = Path(__file__).resolve().parent


def compare_pair(before, after):
    a = Image.open(before).convert('RGB')
    b = Image.open(after).convert('RGB')
    assert a.size == b.size
    ap, bp = list(a.get_flattened_data()), list(b.get_flattened_data())
    coverage = sum((x != (0, 0, 0)) != (y != (0, 0, 0)) for x, y in zip(ap, bp))
    changed = sum(x != y for x, y in zip(ap, bp))
    maximum = max(v[1] for v in ImageChops.difference(a, b).getextrema())
    assert coverage == 0 and maximum <= 1, (before, coverage, maximum)
    return {'image': before.name, 'pixels': list(a.size), 'changed_coverage_pixels': coverage,
            'changed_color_pixels': changed, 'max_channel_delta': maximum}


neutral = [compare_pair(p, ROOT / 'gpu-neutral' / p.name)
           for p in sorted((ROOT / 'gpu-parent').glob('*.png'))]
assert len(neutral) == 21
vesper = [compare_pair(p, ROOT / 'native-vesper' / p.name)
          for p in sorted((ROOT / 'native-parent-vesper').glob('*.png'))]
assert len(vesper) == 4 and all(r['pixels'] == [1280, 800] for r in vesper)
native = []
for name, count in [('native-parent-vesper', 22), ('native-vesper', 22),
                    ('native-earth', 17), ('native-moon', 17), ('native-mars', 17)]:
    data = json.loads((ROOT / name / 'result.json').read_text())
    assert data['status'] == 'passed' and data['steps_completed'] == count
    terrain = [s['state']['terrain'] for s in data['steps'] if s.get('state', {}).get('terrain')]
    contexts = [s['state']['visual_context'] for s in data['steps']
                if s.get('state', {}).get('visual_context')]
    assert all(c['viewport_pixels'] == [1280, 800] for c in contexts[1:])
    gpu = max(t['gpu_cache_bytes'] for t in terrain)
    upload = max(t['peak_upload_bytes'] for t in terrain)
    assert gpu <= 32 * 1024 * 1024 and upload <= 2 * 1024 * 1024
    native.append({'route': name, 'steps': count, 'max_observed_gpu_cache_bytes': gpu,
                   'peak_upload_bytes': upload,
                   'peak_preparation_micros': max(t['preparation_peak_micros'] for t in terrain),
                   'peak_generation_micros': max(t['generation_peak_micros'] for t in terrain)})
result = {'neutral': neutral, 'vesper': vesper, 'native': native,
          'interpretation': 'Identical coverage; any changed color is at most one 8-bit level. Software-renderer measurements are not hardware FPS acceptance.'}
(ROOT / 'comparison.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'neutral_cases': len(neutral), 'vesper_cases': len(vesper),
                  'native_steps': sum(r['steps'] for r in native)}))
