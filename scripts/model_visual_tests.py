#!/usr/bin/env python3
"""Capture production GPU fixtures and compare reviewed image sequences offline."""
import argparse
import json
from pathlib import Path
import shutil


def expanded_scenario(fixture):
    """Use the same checkpoint contract for capture and offline verification."""
    if fixture['mode'] not in ('static', 'moving'):
        raise ValueError('invalid fixture mode')
    repeat = fixture.get('repeat', 1)
    if type(repeat) is not int or repeat < 1:
        raise ValueError('repeat must be a positive integer')
    scenario = json.loads(Path(fixture['scenario']).read_text())
    steps = []
    names = []
    for step in scenario['steps']:
        if 'screenshot' in step:
            name = step['screenshot']
            if not isinstance(name, str) or not name or Path(name).name != name or name in ('.', '..'):
                raise ValueError(f'invalid screenshot name: {name!r}')
            names.extend(f'{name}-repeat-{i:02}' for i in range(repeat))
            steps.extend({'screenshot': f'{name}-repeat-{i:02}'} for i in range(repeat))
        else:
            steps.append(step)
    if not names or len(names) != len(set(names)):
        raise ValueError('fixture must have unique screenshot names')
    if (fixture['mode'] == 'moving' and repeat < 2) or len(names) < 2:
        raise ValueError('fixture requires repeated frames for temporal comparison')
    scenario['steps'] = steps
    return scenario, {name + '.png' for name in names}


def compare(expected, actual, output, *, channel_tolerance=8, changed_fraction=0.001,
            masks=()):
    # Optional dependency: ordinary runner/build tooling remains standard-library only.
    from PIL import Image, ImageChops, ImageDraw
    if not 0 <= channel_tolerance <= 255 or not 0 <= changed_fraction <= 1:
        raise ValueError('invalid image tolerance')
    with Image.open(expected) as image:
        reference = image.convert('RGB')
    with Image.open(actual) as image:
        candidate = image.convert('RGB')
    if reference.size != candidate.size:
        raise ValueError(f'image dimensions differ: {reference.size} != {candidate.size}')
    valid = Image.new('L', reference.size, 255)
    draw = ImageDraw.Draw(valid)
    for box in masks:
        if (len(box) != 4 or any(type(v) is not int for v in box)
                or not 0 <= box[0] < box[2] <= reference.width
                or not 0 <= box[1] < box[3] <= reference.height):
            raise ValueError(f'invalid mask: {box}')
        draw.rectangle((box[0], box[1], box[2]-1, box[3]-1), fill=0)
    diff = ImageChops.difference(reference, candidate)
    channels = diff.split()
    magnitude = ImageChops.lighter(ImageChops.lighter(channels[0], channels[1]), channels[2])
    changed = ImageChops.multiply(magnitude.point(lambda v: 255 if v > channel_tolerance else 0), valid)
    count = changed.histogram()[255]
    pixels = valid.histogram()[255]
    if not pixels:
        raise ValueError('masks exclude every pixel')
    output = Path(output)
    output.mkdir(parents=True, exist_ok=True)
    diff.save(output / 'difference.png')
    candidate.paste((255, 0, 255), mask=changed)
    candidate.save(output / 'highlight.png')
    result = dict(changed_pixels=count, compared_pixels=pixels,
                  changed_fraction=count/pixels, peak_channel_delta=magnitude.getextrema()[1],
                  passed=count/pixels <= changed_fraction)
    (output / 'metrics.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


def evaluate(captures, baselines, output, fixture, temporal_only=False):
    """Moving frames compare corresponding poses; static repeats also test time stability."""
    results = []
    frames = sorted(Path(captures).glob('*.png'))
    if not frames:
        raise ValueError('no captured frames')
    if 'scenario' in fixture:
        _, expected = expanded_scenario(fixture)
        actual = {frame.name for frame in frames}
        if actual != expected:
            raise ValueError(f'incomplete fixture evidence: missing={sorted(expected - actual)}, '
                             f'unexpected={sorted(actual - expected)}')
    options = fixture.get('tolerances', {})
    if not temporal_only:
        missing = {p.name for p in Path(baselines).glob('*.png')} - {p.name for p in frames}
        if missing:
            raise ValueError(f'missing captured frames: {sorted(missing)}')
    for frame in ([] if temporal_only else frames):
        baseline = Path(baselines) / frame.name
        if not baseline.is_file():
            raise ValueError(f'unapproved baseline: {baseline}; capture candidates for human review')
        result = compare(baseline, frame, Path(output)/frame.stem, **options)
        results.append(dict(frame=frame.name, kind='baseline', **result))
    groups = {}
    for frame in frames:
        group = 'all' if fixture['mode'] == 'static' else frame.stem.rsplit('-repeat-', 1)[0]
        groups.setdefault(group, []).append(frame)
    for group in groups.values():
        for frame in group[1:]:
            result = compare(group[0], frame, Path(output)/('temporal-'+frame.stem), **options)
            results.append(dict(frame=frame.name, kind='temporal', **result))
    if temporal_only and not results:
        raise ValueError('no repeated frames for temporal comparison')
    summary = dict(passed=all(r['passed'] for r in results), results=results)
    Path(output).mkdir(parents=True, exist_ok=True)
    (Path(output)/'summary.json').write_text(json.dumps(summary, indent=2)+'\n')
    return summary


def capture(fixture, binary, output):
    from salimon_test import run
    output = Path(output)
    output.mkdir(parents=True, exist_ok=True)
    scenario, _ = expanded_scenario(fixture)
    expanded = output/'expanded.json'
    expanded.write_text(json.dumps(scenario))
    result = run(expanded, [str(Path(binary).resolve())], output/'runs', ['native'])
    if result['status'] != 'passed':
        raise ValueError(f"production capture failed: {result.get('error', result)}")
    for previous in output.glob('*.png'):
        previous.unlink()
    for step in result['steps']:
        shot = step.get('screenshot')
        if shot:
            shutil.copyfile(shot['path'], output/(step['step']['screenshot']+'.png'))
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['capture', 'compare'])
    parser.add_argument('--fixtures', type=Path, default=Path('scenarios/visual/fixtures.json'))
    parser.add_argument('--binary', type=Path)
    parser.add_argument('--temporal-only', action='store_true', help='check repeat stability only; does not approve visual appearance')
    parser.add_argument('--captures', type=Path, default=Path('artifacts/model-visual/candidates'))
    parser.add_argument('--baselines', type=Path, default=Path('scenarios/visual/baselines/option-a'))
    parser.add_argument('--output', type=Path, default=Path('artifacts/model-visual/diffs'))
    args = parser.parse_args()
    if args.command == 'capture' and not args.binary:
        parser.error('capture requires --binary')
    try:
        results = []
        for name, fixture in json.loads(args.fixtures.read_text()).items():
            if args.command == 'capture':
                capture(fixture, args.binary, args.captures/name)
            else:
                results.append(evaluate(args.captures/name, args.baselines/name, args.output/name, fixture, args.temporal_only))
        return 0 if all(r['passed'] for r in results) else 1
    except (OSError, ValueError) as exc:
        parser.exit(1, f'{exc}\n')


if __name__ == '__main__':
    raise SystemExit(main())
