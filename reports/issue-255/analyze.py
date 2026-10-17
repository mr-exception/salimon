"""Summarize admitted native windows and pool only completed moving intervals."""
import argparse
import gzip
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'scripts'))
from benchmark_fragments import METRICS, distribution, summarize


def analyze(paths):
    windows = []
    active = []
    completed = []
    signatures = []
    for path in paths:
        opener = gzip.open if path.suffix == '.gz' else open
        with opener(path, 'rt') as stream:
            raw = json.load(stream)
        for run in raw.get('runs', [raw]):
            summary = summarize(run)
            if run['mode'] != 'native_wall_clock' or run.get('sampling_stride', 1) != 1:
                raise ValueError('requires native, undecimated raw windows')
            samples = [s for s in run['samples'] if s.get('frame_ms') is not None]
            completed.extend(samples)
            moving = [s for s in samples if s['moving_objects'] > 0]
            signature = (run['config'], run['renderer'])
            signatures.append(signature)
            windows.append(dict(file=str(path), summary=summary,
                                active_intervals=len(moving), completed_intervals=len(samples)))
            active.extend(moving)
    if not windows:
        raise ValueError('no input windows')
    # Pool fresh launches of one identical case only; never combine workloads.
    if any(s != signatures[0] for s in signatures):
        raise ValueError('cannot pool different configurations/renderers')
    return dict(windows=windows, active_intervals=len(active),
                active_tail_sample_requirement_met=len(active) >= 100,
                active_metrics={key: distribution([s[key] for s in active if s.get(key) is not None])
                                for key in (*METRICS, 'game_backlog_ms', 'solver_passes')},
                completed_game_backlog_ms=distribution([s['game_backlog_ms'] for s in completed]),
                active_frames_over_50ms=sum(s['frame_ms'] > 50 for s in active),
                active_frames_over_16_67ms=sum(s['frame_ms'] > 1000/60 for s in active))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('raw', type=Path, nargs='+')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.write_text(json.dumps(analyze(args.raw), indent=2)+'\n')
