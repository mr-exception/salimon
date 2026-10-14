"""Opt-in native loaded sleep acceptance using production protocol-1 inputs."""
import argparse
import json
import math
import subprocess
import time
from pathlib import Path

from benchmark_fragments import binary_hash
from e2e_artifacts import Artifacts
from salimon_test import Game, ScenarioError


def ship_point_look_delta(state, point):
    """Aim through protocol look inputs, without installing player/object poses.

    Match automation.rs's aim_fragment mouse conversion. Subtract in ship-local
    coordinates before rotating, retaining precision at the 1e12 m world anchor.
    """
    def dot(a, b):
        return sum(x * y for x, y in zip(a, b))
    def cross(a, b):
        return [a[1]*b[2] - a[2]*b[1], a[2]*b[0] - a[0]*b[2], a[0]*b[1] - a[1]*b[0]]
    def normalize(vector):
        magnitude = math.sqrt(dot(vector, vector))
        if not math.isfinite(magnitude) or magnitude < 1e-9:
            raise ValueError('cannot aim along a degenerate direction')
        return [value / magnitude for value in vector]
    player = state['player']
    local = [goal - eye for goal, eye in zip(point, player['ship_local_eye_position_meters'])]
    q = state['ship']['orientation']
    twice_cross = [2 * value for value in cross(q[:3], local)]
    correction = cross(q[:3], twice_cross)
    target = normalize([local[i] + q[3] * twice_cross[i] + correction[i] for i in range(3)])
    current = normalize([goal - eye for goal, eye in zip(player['look_target_meters'], player['eye_position_meters'])])
    up = normalize(player['up'])
    def planar(direction):
        vertical = dot(direction, up)
        return normalize([direction[i] - up[i] * vertical for i in range(3)])
    a, b = planar(current), planar(target)
    yaw = math.atan2(-dot(cross(a, b), up), dot(a, b))
    pitch = math.asin(max(-1, min(1, dot(target, up)))) - math.asin(max(-1, min(1, dot(current, up))))
    return yaw / .0022, -pitch / .0022


def population(state):
    return sorted((p['id'], p['resource'], p['mass_kg'], p['source_deposit_id'])
                  for p in state['world']['fragments'])


def checkpoint(state):
    return {'ship': {key: state['ship'][key] for key in
                    ('flight_state', 'position_meters', 'orientation', 'speed_meters_per_second')},
            'carried_id': state['carrying']['object_id'],
            'sleeping': state['world']['sleeping_fragment_count'],
            'mass_kg': state['world']['fragment_mass_kg'],
            'fragments': [{key: p[key] for key in ('id', 'mass_kg', 'resource', 'sleeping',
                          'reference_frame', 'ship_local_position_meters', 'orientation_xyzw')}
                          for p in state['world']['fragments']]}


def run(binary, count, layout, settle_steps, timeout, output, carrying, request_timeout=120):
    artifacts = Artifacts({'source': 'fragment_sleep_acceptance'}, output)
    artifacts.open_logs()
    result = {'count': count, 'layout': layout, 'status': 'failed',
              'artifact_directory': str(artifacts.directory), 'checkpoints': {},
              'request_timeout_seconds': request_timeout}
    setup = {'scenario': 'coasting-orbit', 'seed': 155, 'step_ms': 16,
             'fragment_count': count, 'fragment_layout': layout,
             'fragment_settle_steps': settle_steps}
    started = time.monotonic()
    try:
        with Game([str(binary)], setup, started + timeout, artifacts) as game:
            def request(op, **fields):
                # Loaded support removal rebuilds and solves genuine contacts.
                # Game also caps this deadline by the entire run's deadline.
                result = game.request({'op': op, **fields}, time.monotonic() + request_timeout)
                return result if op == 'inspect' else game.request({'op': 'inspect'}, time.monotonic() + request_timeout)
            def key(name, pressed):
                return game.request({'op': 'key', 'key': name, 'pressed': pressed},
                                    time.monotonic() + request_timeout)
            def press(name):
                key(name, True)
                key(name, False)
                return request('inspect')
            def face_point(state, point):
                dx, dy = ship_point_look_delta(state, point)
                return request('look', dx=dx, dy=dy)
            def walk_to(state, x, z, batches=80):
                for _ in range(batches):
                    eye = state['player']['ship_local_eye_position_meters']
                    if math.hypot(x - eye[0], z - eye[2]) < .16:
                        break
                    state = face_point(state, [x, eye[1], z])
                    key('forward', True)
                    state = request('step', frames=2)
                    key('forward', False)
                    if math.dist(eye, state['player']['ship_local_eye_position_meters']) < .005:
                        break
                return state
            initial = request('inspect')
            identity = population(initial)
            assert len(identity) == count and len({p[0] for p in identity}) == count
            expected_mass = sum(piece[2] for piece in identity)
            assert initial['world']['fragment_mass_kg'] == expected_mass
            anchors = {p['id']: p['ship_local_position_meters'] for p in initial['world']['fragments']}
            def check(state, name, asleep=True):
                assert population(state) == identity, f'{name}: identity/material/mass changed'
                assert state['world']['fragment_mass_kg'] == expected_mass, f'{name}: lost mass'
                if asleep:
                    assert state['world']['sleeping_fragment_count'] == count, f'{name}: cargo woke'
                    error = max(math.dist(p['ship_local_position_meters'], anchors[p['id']])
                                for p in state['world']['fragments'])
                    assert error < .001, f'{name}: local drift {error} m'
                    result.setdefault('maximum_rigid_motion_drift_meters', 0)
                    result['maximum_rigid_motion_drift_meters'] = max(result['maximum_rigid_motion_drift_meters'], error)
                result['checkpoints'][name] = checkpoint(state)
            check(initial, 'initial_sleep')
            key('left', True)
            state = request('step', frames=100)
            key('left', False)
            assert abs(state['ship']['speed_meters_per_second'] - 25000) < .001
            check(state, 'fast_rotating_flight')
            for name, expected in [('landing', 'Landed { body: Earth }'), ('takeoff', 'Flying')]:
                state = request('landing')
                for _ in range(100):
                    if state['ship']['flight_state'] == expected:
                        break
                    state = request('step', frames=100)
                    check(state, name + '_progress')
                assert state['ship']['flight_state'] == expected, f'{name}: did not complete'
                check(state, name)
            if carrying:
                state = request('interact')
                assert state['player']['location'] == 'InsideShip'
                before = {p['id']: p for p in state['world']['fragments']}
                def dependent_on(upper, lower):
                    if upper['environment_supported'] or upper['ship_local_position_meters'][1] - lower['ship_local_position_meters'][1] <= .05:
                        return False
                    lower_contacts = [id for id in upper['contact_fragment_ids']
                                      if before[id]['ship_local_position_meters'][1] < upper['ship_local_position_meters'][1] - .02]
                    return lower_contacts == [lower['id']]
                candidates = sorted((p for p in before.values() if p['environment_supported']
                    and any(dependent_on(other, p) for other in before.values())),
                    key=lambda p: p['distance_to_player_meters'])
                picked = None
                for piece in candidates:
                    eye = state['player']['ship_local_eye_position_meters']
                    if piece['ship_local_position_meters'][0] < -2.24 and eye[0] > -2.24:
                        # Authored Core clearance: take the outside aisle before
                        # walking aft, rather than cutting through its pedestal.
                        aisle = -1.8 if eye[2] <= 0 else 1.8
                        state = walk_to(state, eye[0], aisle)
                        state = walk_to(state, -2.5, aisle)
                    # Walk through the normal controller; no player/cargo pose
                    # overrides are available through this acceptance harness.
                    for _ in range(40):
                        state = request('aim_fragment', fragment_id=piece['id'])
                        if state['carrying']['target_id'] == piece['id']:
                            break
                        eye = state['player']['local_ship_position_meters']
                        position = piece['ship_local_position_meters']
                        if math.hypot(eye[0] - position[0], eye[2] - position[2]) < .55:
                            state = request('restore_look')
                            break
                        key('forward', True)
                        state = request('step', frames=4)
                        key('forward', False)
                        state = request('restore_look')
                        if math.dist(eye, state['player']['local_ship_position_meters']) < .005:
                            break
                    if state['carrying']['target_id'] != piece['id']:
                        state = request('aim_fragment', fragment_id=piece['id'])
                    # A root can be obstructed from its center-distance cutoff.
                    # Walk around it through the ordinary strafe controller and
                    # accept only the actual production reticle target.
                    for attempt in range(16):
                        if state['carrying']['target_id'] == piece['id']:
                            break
                        direction = 'left' if attempt < 8 else 'right'
                        key(direction, True)
                        state = request('step', frames=8)
                        key(direction, False)
                        request('restore_look')
                        state = request('aim_fragment', fragment_id=piece['id'])
                    if state['carrying']['target_id'] != piece['id']:
                        request('restore_look')
                        continue
                    state = press('grab_drop')
                    picked = state['carrying']['object_id']
                    if picked is not None:
                        break
                    request('restore_look')
                assert picked is not None, 'no reachable loaded bottom support'
                dependents = {p['id'] for p in before.values() if dependent_on(p, before[picked])}
                component, pending = set(), [picked]
                while pending:
                    id = pending.pop()
                    if id not in component:
                        component.add(id)
                        pending.extend(before[id]['contact_fragment_ids'])
                assert sum(p['carried'] for p in state['world']['fragments']) == 1
                press('slot_1')
                assert request('inspect')['mining']['equipped'] is False
                fallen = []
                for _ in range(40):
                    state = request('step', frames=1)
                    fallen = [p['id'] for p in state['world']['fragments'] if p['id'] in dependents
                              and before[p['id']]['ship_local_position_meters'][1]
                              - p['ship_local_position_meters'][1] > .01]
                    if fallen:
                        break
                assert fallen, 'bottom removal did not make a dependent body fall'
                unrelated = [p['id'] for p in state['world']['fragments']
                             if p['id'] not in component and p['sleeping']]
                assert unrelated, 'no unrelated sleeping island remained'
                check(state, 'support_removed', asleep=False)
                result['removed_support_id'] = picked
                result['fallen_ids'] = fallen
                result['dependent_ids_before_pickup'] = sorted(dependents)
                result['unrelated_sleeping_ids'] = unrelated
                request('restore_look')
                state = request('inspect')
                eye = state['player']['ship_local_eye_position_meters']
                aisle = -1.8 if eye[2] <= 0 else 1.8
                state = walk_to(state, eye[0], aisle)
                state = walk_to(state, .76, aisle)
                # The authored gap between Core and pilot chair is clear floor.
                # Face it explicitly; the pre-pickup view need not point toward
                # that floor. Every release still uses the production validator.
                release_point = [.65 if aisle < 0 else 1.24, .8, 0]
                for _ in range(12):
                    # The carried object sits left of the ray. Aim across that
                    # offset so its footprint, rather than just the reticle,
                    # fits between the body-expanded furniture proxies.
                    state = face_point(state, release_point)
                    state = press('grab_drop')
                    if state['carrying']['object_id'] is None:
                        break
                    key('forward', True)
                    state = request('step', frames=1)
                    key('forward', False)
                assert state['carrying']['object_id'] is None, 'production placement rejected release'
                for cycle in range(3):
                    state = request('step', frames=1)
                    request('aim_fragment', fragment_id=picked)
                    state = press('grab_drop')
                    assert state['carrying']['object_id'] == picked, 'repeat pickup changed identity'
                    state = face_point(state, release_point)
                    state = press('grab_drop')
                    assert state['carrying']['object_id'] is None, 'repeat release rejected'
                    check(state, f'pickup_drop_{cycle + 1}', asleep=False)
                result['repeated_pickup_drop_cycles'] = 3
            result['status'] = 'passed'
    except (AssertionError, ScenarioError, OSError, ValueError, KeyError) as exc:
        result['error'] = str(exc)
    finally:
        artifacts.close()
    result['duration_seconds'] = time.monotonic() - started
    artifacts.write_json('acceptance.json', result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--counts', type=int, nargs='+', default=[500, 1000])
    parser.add_argument('--layout', choices=['scattered', 'dense'], default='dense')
    parser.add_argument('--settle-steps', type=int, default=1200)
    parser.add_argument('--timeout', type=int, default=7200)
    parser.add_argument('--request-timeout', type=int, default=120,
                        help='per-request deadline in seconds, also bounded by --timeout')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--carrying', action='store_true', help='also require real F bottom removal and repeated pickup/drop')
    args = parser.parse_args()
    if any(not 1 <= count <= 1000 for count in args.counts) or not 1 <= args.settle_steps <= 37500 or args.timeout <= 0 or args.request_timeout <= 0:
        parser.error('invalid population, preparation limit or deadline')
    binary = args.binary.resolve()
    args.output.mkdir(parents=True, exist_ok=True)
    result = {'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
              'binary_sha256': binary_hash(binary), 'setup_seed': 155, 'step_ms': 16,
              'settle_steps_limit': args.settle_steps,
              'runs': [run(binary, count, args.layout, args.settle_steps, args.timeout, args.output, args.carrying, args.request_timeout)
                       for count in args.counts]}
    (args.output / 'summary.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({key: value for key, value in result.items() if key != 'runs'} |
                     {'runs': [{key: value for key, value in run.items() if key != 'checkpoints'} for run in result['runs']]}))
    return int(any(run['status'] != 'passed' for run in result['runs']))


if __name__ == '__main__':
    raise SystemExit(main())
