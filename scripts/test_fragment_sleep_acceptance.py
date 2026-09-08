"""Controller look inputs retain ship axes and distant-anchor precision."""
import copy
import math
import unittest

from fragment_sleep_acceptance import ship_point_look_delta


class ShipPointAimTests(unittest.TestCase):
    def state(self):
        return {'ship': {'orientation': [0, 0, 0, 1]}, 'player': {
            'ship_local_eye_position_meters': [0, 0, 0],
            'eye_position_meters': [0, 0, 0],
            'look_target_meters': [1, 0, 0], 'up': [0, 1, 0]}}

    def test_straight_ahead_needs_no_look_input(self):
        self.assertEqual(ship_point_look_delta(self.state(), [2, 0, 0]), (0, 0))

    def test_right_angle_and_downward_pitch_have_controller_signs(self):
        dx, dy = ship_point_look_delta(self.state(), [0, -1, 1])
        self.assertAlmostEqual(dx * .0022, math.pi / 2)
        self.assertAlmostEqual(dy * .0022, math.pi / 4)

    def test_rotated_ship_at_distant_anchor_uses_local_target_difference(self):
        state = self.state()
        state['ship']['orientation'] = [0, math.sqrt(.5), 0, math.sqrt(.5)]
        state['player']['eye_position_meters'] = [1e12, -2e12, 3e12]
        state['player']['look_target_meters'] = [1e12, -2e12, 3e12 - 1]
        original = copy.deepcopy(state)
        dx, dy = ship_point_look_delta(state, [0, -1, 1])
        self.assertAlmostEqual(dx * .0022, math.pi / 2)
        self.assertAlmostEqual(dy * .0022, math.pi / 4)
        self.assertEqual(state, original)

    def test_rolled_ship_does_not_assume_world_y_up(self):
        state = self.state()
        state['ship']['orientation'] = [math.sqrt(.5), 0, 0, math.sqrt(.5)]
        state['player']['up'] = [0, 0, 1]
        dx, dy = ship_point_look_delta(state, [0, 0, 1])
        self.assertAlmostEqual(dx * .0022, math.pi / 2)
        self.assertAlmostEqual(dy, 0)

    def test_degenerate_target_is_rejected(self):
        with self.assertRaises(ValueError):
            ship_point_look_delta(self.state(), [0, 0, 0])


if __name__ == '__main__':
    unittest.main()
