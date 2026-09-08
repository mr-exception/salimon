"""Run the final native universe/offline matrix with original captures and raw timings."""
import argparse
import json
import os
from pathlib import Path
import subprocess

from planetary_environment import ROOT, digest, review_native
from salimon_test import result_summary, run
from scenario_suites import SCANNER, UNIVERSE_FLIGHT, UNIVERSE_RENDER

ROUTES = ('offline-resume-progress.json', 'offline-clock-recovery.json',
          *UNIVERSE_FLIGHT, *UNIVERSE_RENDER, *SCANNER, 'player-chart.json',
          'planetary-night.json', 'planetary-vacuum.json', 'environment-audio.json')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--scenario', choices=ROUTES, action='append',
                        help='explicit focused rerun; default runs the complete matrix')
    args = parser.parse_args()
    output, binary = args.output.resolve(), args.binary.resolve()
    output.mkdir(parents=True, exist_ok=False)
    names = args.scenario or ROUTES
    summary = {
        'status': 'running', 'source_revision': subprocess.check_output(
            ['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        'source_dirty': bool(subprocess.check_output(
            ['git', 'status', '--porcelain'], cwd=ROOT, text=True).strip()),
        'binary_sha256': digest(binary), 'scope': list(names),
        'source_sha256': {str(p.relative_to(ROOT)): digest(p)
                          for directory in ('client', 'scripts', 'scenarios')
                          for p in sorted((ROOT / directory).rglob('*'))
                          if p.is_file() and p.suffix in {'.rs', '.py', '.json', '.toml', '.wgsl'}},
        'hardware_fps_validated': False, 'results': []}
    try:
        for name in names:
            # Exercise real settings IO only against a declared fresh test file;
            # E2E deliberately leaves personal preferences unopened otherwise.
            original_settings = os.environ.get('SALIMON_SETTINGS_FILE')
            os.environ['SALIMON_SETTINGS_FILE'] = str(output / (Path(name).stem + '-settings.json'))
            try:
                result = run(ROOT / 'scenarios/evidence' / name, [str(binary)], output, ['native'])
            finally:
                if original_settings is None:
                    os.environ.pop('SALIMON_SETTINGS_FILE', None)
                else:
                    os.environ['SALIMON_SETTINGS_FILE'] = original_settings
            summary['results'].append(result_summary(result))
            (output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
            print(json.dumps(result_summary(result)), flush=True)
            if result['status'] != 'passed':
                raise ValueError(f'failed route: {name}')
        summary['status'] = 'passed'
        review = review_native(summary, names)
        (output / 'review.json').write_text(json.dumps(review, indent=2) + '\n')
    except (OSError, ValueError, KeyError) as error:
        summary['status'] = 'failed'
        summary['error'] = str(error)
    (output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({'status': summary['status'], 'report': str(output / 'summary.json')}))
    return summary['status'] != 'passed'


if __name__ == '__main__':
    raise SystemExit(main())
