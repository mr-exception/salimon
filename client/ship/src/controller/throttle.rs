//! Persistent signed selection and simulation-time held-key repeats.
use super::ShipController;
use crate::FlightState;
use std::time::Duration;

pub const THROTTLE_REPEAT_INTERVAL: Duration = Duration::from_millis(100);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThrottleDirection {
    Up,
    Down,
}

#[derive(Clone, Debug, Default)]
pub(super) struct HeldThrottle {
    up: bool,
    down: bool,
    elapsed: Duration,
}
impl ShipController {
    fn can_adjust_thruster(&self) -> bool {
        self.cockpit_control_active
            && self.flight_state == FlightState::Flying
            && self.energy_core.powered()
    }

    /// One explicit percentage-point command; accepted even when already clamped.
    pub fn adjust_thruster(&mut self, points: i8) -> bool {
        if !self.can_adjust_thruster() {
            return false;
        }
        self.apply_thruster_points(i16::from(points));
        true
    }
    fn apply_thruster_points(&mut self, points: i16) {
        self.thruster_percentage =
            (i16::from(self.thruster_percentage) + points).clamp(-100, 100) as i8;
    }

    /// Real edges only. Repeated press events cannot increment or restart cadence.
    pub fn set_throttle_key(&mut self, direction: ThrottleDirection, pressed: bool) {
        if !self.can_adjust_thruster() {
            self.clear_throttle_input();
            return;
        }
        let held = match direction {
            ThrottleDirection::Up => &mut self.throttle_input.up,
            ThrottleDirection::Down => &mut self.throttle_input.down,
        };
        if *held == pressed {
            return;
        }
        *held = pressed;
        self.throttle_input.elapsed = Duration::ZERO;
        if pressed {
            self.adjust_thruster(match direction {
                ThrottleDirection::Up => 1,
                ThrottleDirection::Down => -1,
            });
        }
    }
    pub fn clear_throttle_input(&mut self) {
        self.throttle_input = HeldThrottle::default();
    }

    pub(super) fn throttle_boundary(&self) -> Option<Duration> {
        let up = self.throttle_input.up && !self.throttle_input.down;
        let down = self.throttle_input.down && !self.throttle_input.up;
        (self.can_adjust_thruster()
            && ((up && self.thruster_percentage < 100)
                || (down && self.thruster_percentage > -100)))
            .then(|| THROTTLE_REPEAT_INTERVAL.saturating_sub(self.throttle_input.elapsed))
    }
    pub(super) fn advance_throttle(&mut self, delta: Duration) {
        if !self.can_adjust_thruster() {
            self.clear_throttle_input();
            return;
        }
        let direction =
            i16::from(i8::from(self.throttle_input.up) - i8::from(self.throttle_input.down));
        if direction == 0 {
            self.throttle_input.elapsed = Duration::ZERO;
            return;
        }
        let elapsed = self.throttle_input.elapsed.saturating_add(delta).as_nanos();
        let interval = THROTTLE_REPEAT_INTERVAL.as_nanos();
        let repeats = (elapsed / interval).min(200) as i16;
        self.throttle_input.elapsed = Duration::from_nanos((elapsed % interval) as u64);
        self.apply_thruster_points(direction * repeats);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LANDING_ASSIST_DURATION, ShipPose};
    fn ship() -> ShipController {
        ShipController::flying(
            ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
        )
    }
    #[test]
    fn signed_selection_clamps_and_reverse_thrust_moves_backward() {
        let mut ship = ship();
        assert!(ship.adjust_thruster(-100));
        ship.adjust_thruster(-127);
        assert_eq!(ship.snapshot().thruster_percentage, -100);
        ship.advance(Duration::from_secs(1));
        assert_eq!(ship.velocity_meters_per_second(), [-500.0, 0.0, 0.0]);
        ship.adjust_thruster(127);
        ship.adjust_thruster(127);
        assert_eq!(ship.snapshot().thruster_percentage, 100);
    }
    #[test]
    fn held_keys_use_real_edges_and_partition_independent_cadence() {
        let mut whole = ship();
        let mut split = ship();
        for ship in [&mut whole, &mut split] {
            ship.set_throttle_key(ThrottleDirection::Down, true);
            ship.set_throttle_key(ThrottleDirection::Down, true);
            assert_eq!(ship.snapshot().thruster_percentage, -1);
        }
        whole.advance_throttle(Duration::from_millis(1003));
        for ms in [13, 200, 1, 389, 400] {
            split.advance_throttle(Duration::from_millis(ms));
        }
        assert_eq!(whole.snapshot().thruster_percentage, -11);
        assert_eq!(
            whole.snapshot().thruster_percentage,
            split.snapshot().thruster_percentage
        );
        whole.set_throttle_key(ThrottleDirection::Down, false);
        whole.advance_throttle(Duration::from_secs(2));
        assert_eq!(whole.snapshot().thruster_percentage, -11);
    }
    #[test]
    fn authority_power_and_opposing_keys_clear_repeat_without_resetting_selection() {
        let mut ship = ship();
        ship.set_throttle_key(ThrottleDirection::Up, true);
        ship.set_throttle_key(ThrottleDirection::Down, true);
        ship.advance_throttle(Duration::from_secs(1));
        assert_eq!(ship.snapshot().thruster_percentage, 0);
        ship.set_throttle_key(ThrottleDirection::Down, false);
        ship.advance_throttle(THROTTLE_REPEAT_INTERVAL);
        assert_eq!(ship.snapshot().thruster_percentage, 1);
        ship.set_cockpit_control(false);
        assert!(!ship.adjust_thruster(-1));
        ship.set_cockpit_control(true);
        ship.advance_throttle(Duration::from_secs(1));
        assert_eq!(ship.snapshot().thruster_percentage, 1);
        ship.set_stored_core_energy_joules(0);
        assert!(!ship.adjust_thruster(-1));
        ship.set_throttle_key(ThrottleDirection::Down, true);
        ship.advance_throttle(Duration::from_secs(1));
        ship.set_stored_core_energy_joules(1);
        ship.advance_throttle(Duration::from_secs(1));
        assert_eq!(ship.snapshot().thruster_percentage, 1);
    }
    #[test]
    fn power_loss_clears_hold_even_if_restored_before_next_tick() {
        let mut ship = ship();
        ship.set_throttle_key(ThrottleDirection::Up, true);
        ship.set_stored_core_energy_joules(0);
        ship.set_stored_core_energy_joules(1);
        ship.advance_throttle(Duration::from_secs(1));
        assert_eq!(ship.snapshot().thruster_percentage, 1);
    }

    #[test]
    fn contact_resets_selection_and_clears_held_repeat() {
        let earth = salimon_world::CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == salimon_world::CelestialBodyId::Earth)
            .unwrap();
        let pose = ShipPose {
            position_meters: salimon_math::add(
                earth.center.meters(),
                [
                    0.0,
                    earth.radius_meters + crate::COLLISION_RADIUS_METERS + 1.0,
                    0.0,
                ],
            ),
            orientation: [0.0, 0.0, 0.0, 1.0],
        };
        let mut ship = ShipController::flying_with_velocity(pose, 10, [0.0, -100.0, 0.0]).unwrap();
        ship.set_throttle_key(ThrottleDirection::Up, true);
        ship.advance(Duration::from_millis(16));
        assert_eq!(ship.snapshot().thruster_percentage, 0);
        ship.advance(Duration::from_millis(100));
        assert_eq!(ship.snapshot().thruster_percentage, 0);
    }

    #[test]
    fn landing_locks_input_and_finishes_with_zero_selection() {
        let mut pose = ShipController::default().snapshot().pose;
        pose.position_meters[1] += 100.0;
        let mut ship = ShipController::flying(pose, -25);
        ship.set_throttle_key(ThrottleDirection::Down, true);
        ship.trigger_landing_action();
        assert!(!ship.adjust_thruster(1));
        ship.advance(LANDING_ASSIST_DURATION);
        assert_eq!(ship.snapshot().thruster_percentage, 0);
        ship.advance_throttle(Duration::from_secs(1));
        assert_eq!(ship.snapshot().thruster_percentage, 0);
    }
}
