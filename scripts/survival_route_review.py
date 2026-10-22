#!/usr/bin/env python3
"""Evaluate recorded physical ration routes without granting balance acceptance."""
import argparse
import hashlib
import json
import math
import re
from pathlib import Path

STAGES = ('discovery', 'setup', 'outbound', 'gather', 'return', 'intake',
          'fabricator', 'collection', 'q_consumption')


def number(value, name, minimum=0):
    if type(value) not in (float, int) or not math.isfinite(value) or value < minimum:
        raise ValueError(f'{name} must be finite and >= {minimum}')
    return value


def evaluate(trial):
    if not isinstance(trial, dict) or trial.get('schema') != 1 or trial.get('kind') != 'physical_route':
        raise ValueError('requires a schema-1 physical_route, not fixture travel budgets')
    for field in ('revision', 'hardware', 'os', 'backend', 'body', 'layout', 'obstacles',
                  'operator', 'recorded_at', 'evidence'):
        if not isinstance(trial.get(field), str) or not trial[field].strip():
            raise ValueError(f'missing physical-route context: {field}')
    if trial.get('fixture_placement_used') is not False:
        raise ValueError('fixture placement cannot establish a physical route')
    if not re.fullmatch('[0-9a-f]{40}', trial['revision']):
        raise ValueError('record the exact source revision')
    if type(trial.get('seed')) is not int or trial['seed'] < 0:
        raise ValueError('seed must be a nonnegative integer')
    for field in ('health_percent', 'fatigue_percent', 'hunger_percent'):
        if number(trial.get(field), field) > 100:
            raise ValueError(f'{field} exceeds 100')
    stages = trial.get('active_seconds')
    if not isinstance(stages, dict) or set(stages) != set(STAGES):
        raise ValueError('record every physical route stage, including discovery and Q')
    outcome = trial.get('outcome')
    if outcome not in ('completed', 'failed', 'no_feedstock'):
        raise ValueError('record completed, failed or no_feedstock outcome')
    elapsed = sum(number(value, key) for key, value in stages.items() if value is not None)
    if abs(number(trial.get('elapsed_active_seconds'), 'elapsed_active_seconds') - elapsed) > .01:
        raise ValueError('exclusive stage durations must match the independently recorded active duration')
    if elapsed <= 0:
        raise ValueError('route duration must be positive')
    if outcome != 'completed':
        if not isinstance(trial.get('failure_reason'), str) or not trial['failure_reason'].strip():
            raise ValueError('retain the reason for a failed or unavailable route')
        return {'schema': 1, 'status': outcome, 'failure_reason': trial['failure_reason'],
                'active_seconds': elapsed, 'unrun_stages': [key for key in STAGES if stages[key] is None],
                'budget_status': 'proposed_pending_owner_review', 'native_signoff': False,
                'owner_approval': False}
    if any(value is None for value in stages.values()):
        raise ValueError('completed routes require all stage timings')
    if type(trial.get('rations_produced_and_consumed')) is not int or trial['rations_produced_and_consumed'] < 1:
        raise ValueError('record at least one newly produced and consumed ration')
    count = trial['rations_produced_and_consumed']
    for field, kg in (('biomass_used_kg', .4), ('ice_used_kg', .1)):
        if number(trial.get(field), field) + 1e-9 < kg * count:
            raise ValueError('newly gathered inputs must fund every recorded ration')
    if trial.get('starter_reserves_used') is not False:
        raise ValueError('starter reserves cannot prove a repeatable physical route')
    per_ration = elapsed / count
    # Supply margin before meter clamping or concurrent food modifiers; observed
    # before/after state belongs in evidence, not an invented actual hunger delta.
    return {'schema': 1, 'status': 'completed', 'active_seconds': elapsed, 'seconds_per_ration': per_ration,
            'unclamped_hunger_supply_margin': count * 40 - elapsed * 100 / 1800,
            'net_negative': per_ration > 720,
            'comfortable_proposal_met': per_ration <= 360,
            'break_even_met': per_ration <= 720,
            'budget_status': 'proposed_pending_owner_review',
            'native_signoff': False, 'owner_approval': False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('trial', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    try:
        raw = args.trial.read_bytes()
        result = evaluate(json.loads(raw))
        result['input_sha256'] = hashlib.sha256(raw).hexdigest()
        args.output.parent.mkdir(parents=True, exist_ok=True)
        with args.output.open('x') as stream:
            json.dump(result, stream, indent=2, allow_nan=False)
            stream.write('\n')
    except (OSError, ValueError, TypeError) as error:
        parser.exit(1, f'route review failed: {error}\n')
    print(json.dumps(result))
    return 1 if result['status'] != 'completed' or result['net_negative'] else 0


if __name__ == '__main__':
    raise SystemExit(main())
