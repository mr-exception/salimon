"""Compare terrain_style_reference GPU evidence without promoting baselines.

Requires Pillow from scripts/visual-requirements.txt. Timing samples describe the
recorded adapter and pass only; they are not hardware FPS acceptance.
"""
import argparse
import csv
import json
from pathlib import Path
from PIL import Image, ImageChops, ImageDraw


def compare(before, after, output):
    def rows(folder):
        with (folder / 'costs.csv').open() as stream:
            return {r['scene']: r for r in csv.DictReader(stream)}
    old, new = rows(before), rows(after)
    if old.keys() != new.keys() or not old:
        raise ValueError('comparison scene sets differ or are empty')
    output.mkdir(parents=True, exist_ok=True)
    measurements = []
    for scene in old:
        a = Image.open(before / (scene + '.png')).convert('RGB')
        b = Image.open(after / (scene + '.png')).convert('RGB')
        if a.size != b.size:
            raise ValueError(f'{scene}: drawable sizes differ')
        mask_a = a.point(lambda x: 255 if x else 0).convert('L').point(lambda x: 255 if x else 0)
        mask_b = b.point(lambda x: 255 if x else 0).convert('L').point(lambda x: 255 if x else 0)
        changed_coverage = sum(ImageChops.difference(mask_a, mask_b).histogram()[1:])
        if changed_coverage:
            raise ValueError(f'{scene}: {changed_coverage} silhouette/coverage pixels changed')
        for field in ('draws', 'triangles', 'patch_gpu_bytes', 'uniform_upload_bytes'):
            if old[scene][field] != new[scene][field]:
                raise ValueError(f'{scene}: geometry/buffer cost {field} changed')
        pixels_a, pixels_b = a.tobytes(), b.tobytes()
        changed_rgb = sum(pixels_a[i:i+3] != pixels_b[i:i+3] for i in range(0, len(pixels_a), 3))
        measurements.append({'scene': scene, 'changedCoveragePixels': changed_coverage,
                             'changedColorPixels': changed_rgb, 'before': old[scene], 'after': new[scene]})
    for body in ('earth', 'moon', 'mars'):
        sheet = Image.new('RGB', (1024, 1236), (25, 30, 38))
        draw = ImageDraw.Draw(sheet)
        for row, distance in enumerate(('near', 'far', 'orbit')):
            for col, (label, folder) in enumerate((('before', before), ('after', after))):
                x, y = col * 512, row * 412
                draw.text((x + 10, y + 6), f'{body} / {distance} / {label}', fill='white')
                sheet.paste(Image.open(folder / f'{body}-{distance}.png').convert('RGB'), (x, y + 28))
        sheet.save(output / (body + '.jpg'), quality=90)
    result = {'schema': 1, 'scope': '512x384 fixed synthetic hill/crater and analytic-body GPU fixtures; software Vulkan timings are not native FPS', 'scenes': measurements}
    (output / 'comparison.json').write_text(json.dumps(result, indent=2) + '\n')
    return result


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--before', type=Path, required=True)
    p.add_argument('--after', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    compare(a.before, a.after, a.output)
