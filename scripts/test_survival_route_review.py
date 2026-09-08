import unittest
from copy import deepcopy
from survival_route_review import STAGES, evaluate


def trial(seconds=360):
    return dict(schema=1, kind='physical_route', outcome='completed', revision='a' * 40,
                hardware='reference Apple M1 iMac', os='macOS', backend='Metal',
                body='Earth', layout='poor ice placement', obstacles='ship ramp and outcrop',
                operator='test fixture', recorded_at='2026-10-22', evidence='synthetic test input',
                fixture_placement_used=False, starter_reserves_used=False, seed=0,
                health_percent=10, fatigue_percent=10, hunger_percent=50,
                active_seconds={key: seconds / len(STAGES) for key in STAGES},
                elapsed_active_seconds=seconds,
                rations_produced_and_consumed=1, biomass_used_kg=.4, ice_used_kg=.1)


class PhysicalRoutes(unittest.TestCase):
    def test_supply_boundary_and_proposal_are_distinct_from_approval(self):
        for seconds, comfortable, negative in [(360, True, False), (720, False, False), (721, False, True)]:
            result = evaluate(trial(seconds))
            self.assertEqual(result['comfortable_proposal_met'], comfortable)
            self.assertEqual(result['net_negative'], negative)
            self.assertFalse(result['owner_approval'])
            self.assertFalse(result['native_signoff'])
        self.assertAlmostEqual(evaluate(trial(720))['unclamped_hunger_supply_margin'], 0)

    def test_fixture_shortcuts_and_unfunded_repeat_routes_fail(self):
        for change in [dict(kind='fixture'), dict(fixture_placement_used=True),
                       dict(starter_reserves_used=True), dict(biomass_used_kg=.39),
                       dict(rations_produced_and_consumed=2), dict(hardware='')]:
            with self.subTest(change=change), self.assertRaises(ValueError):
                evaluate({**trial(), **change})

    def test_missing_stages_and_nonfinite_or_negative_times_fail(self):
        for key in STAGES:
            value = trial()
            del value['active_seconds'][key]
            with self.subTest(missing=key), self.assertRaises(ValueError):
                evaluate(value)
        for bad in [float('nan'), float('inf'), -1, True, None]:
            value = deepcopy(trial())
            value['active_seconds']['discovery'] = bad
            with self.subTest(time=bad), self.assertRaises(ValueError):
                evaluate(value)

    def test_failed_scarce_routes_retain_unrun_stages_without_a_supply_pass(self):
        value = trial()
        value.update(outcome='no_feedstock', body='Moon', failure_reason='No biomass found')
        value['active_seconds'] = {key: 60 if key == 'discovery' else None for key in STAGES}
        value['elapsed_active_seconds'] = 60
        result = evaluate(value)
        self.assertEqual(result['status'], 'no_feedstock')
        self.assertIn('q_consumption', result['unrun_stages'])
        self.assertNotIn('break_even_met', result)
