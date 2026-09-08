"""Capture the internal Hall's bounded billion-sign fixture through production wgpu."""
import argparse
import json
from pathlib import Path
import subprocess

from planetary_environment import ROOT, digest, review_native
from salimon_test import result_summary, run


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output, binary = args.output.resolve(), args.binary.resolve()
    output.mkdir(parents=True, exist_ok=False)
    source = {str(p.relative_to(ROOT)): digest(p)
              for directory in ('client', 'scripts', 'scenarios')
              for p in sorted((ROOT / directory).rglob('*'))
              if p.is_file() and p.suffix in {'.rs', '.py', '.json', '.toml', '.wgsl'}}
    summary = {'status': 'failed', 'source_revision': subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        'source_sha256': source, 'binary_sha256': digest(binary),
        'scope': ['hall-prototype.json'], 'hardware_fps_validated': False, 'results': []}
    try:
        result = run(ROOT / 'scenarios/evidence/hall-prototype.json', [str(binary)], output, ['native'])
        summary['results'].append(result_summary(result))
        if result['status'] != 'passed':
            raise ValueError(result.get('error', 'Hall route failed'))
        summary['status'] = 'passed'
        review = {'routes': review_native(summary, summary['scope'])}
        evidence = json.loads(Path(result['result_file']).read_text())
        captures = [step for step in evidence['steps'] if 'screenshot' in step['step']]
        first, last = captures[0], captures[-1]
        if digest(Path(first['screenshot']['path'])) != digest(Path(last['screenshot']['path'])):
            raise ValueError('frozen arrival differs on revisit')
        if first['state']['hall_prototype'] != last['state']['hall_prototype']:
            raise ValueError('synthetic source or spatial window differs on revisit')
        review['revisit_pixels_identical'] = True
        review['synthetic_population'] = 1_000_000_000
        (output / 'review.json').write_text(json.dumps(review, indent=2) + '\n')
    except (OSError, ValueError, KeyError) as error:
        summary['status'], summary['error'] = 'failed', str(error)
    (output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({'status': summary['status'], 'report': str(output / 'summary.json'),
                      'error': summary.get('error')}))
    return summary['status'] != 'passed'


if __name__ == '__main__':
    raise SystemExit(main())
