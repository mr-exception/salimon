"""Production GPU opening, packet and physical-site evidence for #470/#471.

Fixtures declare placement and (only for recovered packets) 2 kg each iron and
silicate. They do not claim walked acquisition, flight or actual OS input.
"""
import argparse
import hashlib
import json
from pathlib import Path
import time
from e2e_artifacts import Artifacts
from salimon_test import Game, native_capture


def run(binary, output, scenarios=None):
    output.mkdir(parents=True, exist_ok=False)
    evidence = []
    for scenario in scenarios or ('canonical-journey', 'narrative-distress', 'narrative-warning', 'narrative-recovered', 'narrative-chart'):
        setup = dict(scenario=scenario, seed=0, step_ms=16)
        art = Artifacts(setup, output / scenario, ['native'])
        art.open_logs()
        deadline = time.monotonic() + 240
        try:
            with Game([str(binary.resolve())], setup, deadline, art, native_input=False) as game:
                def request(**kw):
                    return game.request(kw)
                def inspect():
                    return request(op='inspect')
                def press(key):
                    request(op='key', key=key, pressed=True)
                    request(op='key', key=key, pressed=False)
                def capture(name):
                    # Three produced frames ensure the source presentation was consumed.
                    for _ in range(3):
                        request(op='step', frames=1)
                    result = native_capture(game, art, name, deadline)
                    if result['status'] != 'captured':
                        raise AssertionError(result)
                    state = inspect()
                    evidence.append(dict(scenario=scenario, name=name,
                                         visual_context=state['visual_context'], terrain=state.get('terrain'), player=state['player'],
                                         narrative=state['narrative'], chart=state['chart'],
                                         recovery=state['recovery'], archive=state['archive'],
                                         tutorial=state['tutorial'], audio_output=state['audio_output'],
                                         png=result['path'], sha256=hashlib.sha256(Path(result['path']).read_bytes()).hexdigest()))
                    return state
                if scenario in ('canonical-journey', 'narrative-distress', 'narrative-warning'):
                    ready_deadline = min(deadline, time.monotonic() + 180)
                    while not inspect()['terrain']['presentation_ready']:
                        if time.monotonic() >= ready_deadline:
                            raise AssertionError('canonical terrain presentation did not become ready within 180 seconds')
                        request(op='step', frames=1)
                state = inspect()
                if scenario == 'canonical-journey':
                    assert state['chart']['entry_count'] == 0
                    assert 'Earth is home' in state['tutorial']['prompt']
                    assert state['recovery']['module'] is None
                    capture('awakening-outside-scout')
                    press('skip_tutorial')
                    assert inspect()['tutorial']['prompt'] is None
                elif scenario == 'narrative-chart':
                    assert state['chart']['entry_count'] == 2
                    press('chart')
                    assert inspect()['chart']['mode']
                    for i in range(2):
                        if i:
                            press('scanner_next')
                        press('chart_route')
                        current = inspect()['chart']
                        assert current['route_count'] == i + 1
                        assert 'UNVERIFIED ROUTE' in current['note']
                        assert current['source']['routes'][-1]['lead_source'] is not None
                        assert all(e['observed'] is None for e in current['source']['entries'])
                        capture(f'uncertain-route-{i+1}')
                elif scenario == 'narrative-recovered':
                    assert state['chart']['entry_count'] == 2
                    assert state['recovery']['spent_joules'] == [3e9, 5e9]
                    assert state['chart']['route_count'] == 0
                    press('interact')
                    assert inspect()['archive']['open']
                    for _ in range(5):
                        press('thruster_down')
                    assert 'Distress:' in inspect()['archive']['text']
                    capture('distress-payload')
                    press('thruster_down')
                    assert 'trap' in inspect()['archive']['text']
                    capture('warning-payload')
                else:
                    i = 0 if scenario == 'narrative-distress' else 1
                    assert state['narrative']['hint'] is not None
                    capture('physical-relay')
                    press('interact')
                    opened = inspect()
                    assert 'UNVERIFIED CLAIM' in opened['narrative']['text']
                    assert opened['narrative']['source']['inspected_utc_millis'][i] is not None
                    assert opened['chart']['route_count'] == 0
                    capture('inspected-clue')
                    press('interact')
                    assert inspect()['narrative']['text'] is None
                    press('interact')
                    assert inspect()['narrative']['source'] == opened['narrative']['source']
        finally:
            art.close()
    (output / 'evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(json.dumps(dict(status='passed', captures=len(evidence), output=str(output))))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--scenario', action='append', choices=('canonical-journey', 'narrative-distress', 'narrative-warning', 'narrative-recovered', 'narrative-chart'))
    args = parser.parse_args()
    run(args.binary, args.output, args.scenario)


if __name__ == '__main__':
    main()
