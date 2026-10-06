//! Direct-speed flight and solid-body collision correction.

use super::ShipController;
use crate::{COLLISION_RADIUS_METERS, FlightState};
use salimon_math::{length as vector_length, scale};
use salimon_world::{BodyRole, CELESTIAL_BODIES, PHASE_ZERO_REFERENCE_MAX_SPEED_METERS_PER_SECOND};

impl ShipController {
    pub(super) fn advance_flight(&mut self, seconds: f64) {
        let distance = self.speed_meters_per_second() * seconds;
        let forward = self.pose.axes()[0];
        for (position, direction) in self.pose.position_meters.iter_mut().zip(forward) {
            *position += direction * distance;
        }
        self.resolve_solid_body_collisions();
    }

    pub fn adjust_thruster(&mut self, percentage_points: i8) {
        if !self.cockpit_control_active || self.flight_state != FlightState::Flying {
            return;
        }
        self.thruster_percentage = i16::from(self.thruster_percentage)
            .saturating_add(i16::from(percentage_points))
            .clamp(0, 100) as u8;
    }

    #[must_use]
    pub fn speed_meters_per_second(&self) -> f64 {
        PHASE_ZERO_REFERENCE_MAX_SPEED_METERS_PER_SECOND * f64::from(self.thruster_percentage)
            / 100.0
    }

    #[must_use]
    pub fn velocity_meters_per_second(&self) -> [f64; 3] {
        match self.flight_state {
            FlightState::Flying => scale(self.pose.axes()[0], self.speed_meters_per_second()),
            FlightState::AssistedLanding { .. } | FlightState::AssistedTakeoff { .. } => {
                let assist = self.assist.as_ref().expect("active assistance has a path");
                scale(assist.surface_normal, assist.radial_speed_meters_per_second)
            }
            FlightState::Landed { .. } => [0.0; 3],
        }
    }

    pub(super) fn resolve_solid_body_collisions(&mut self) {
        for body in CELESTIAL_BODIES
            .iter()
            .filter(|body| body.role == BodyRole::Solid)
        {
            let center = body.center.meters();
            let offset = [
                self.pose.position_meters[0] - center[0],
                self.pose.position_meters[1] - center[1],
                self.pose.position_meters[2] - center[2],
            ];
            let distance = vector_length(offset);
            let minimum = body.radius_meters + COLLISION_RADIUS_METERS;
            if distance < minimum {
                let normal = if distance > 1.0e-9 {
                    offset.map(|value| value / distance)
                } else {
                    self.pose.axes()[0].map(|value| -value)
                };
                self.pose.position_meters = [
                    center[0] + normal[0] * minimum,
                    center[1] + normal[1] * minimum,
                    center[2] + normal[2] * minimum,
                ];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ShipPose;
    use salimon_world::CelestialBodyId;
    use std::time::Duration;

    #[test]
    fn leaving_cockpit_does_not_change_flying_motion_or_thruster() {
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: [10.0, 20.0, 30.0],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            37,
        );
        let before = ship.snapshot();
        ship.set_cockpit_control(false);
        ship.advance(Duration::from_millis(100));
        let after = ship.snapshot();
        assert_eq!(after.thruster_percentage, before.thruster_percentage);
        assert_eq!(after.pose.orientation, before.pose.orientation);
        assert_eq!(
            after.speed_meters_per_second,
            before.speed_meters_per_second
        );
        assert!(after.pose.position_meters[0] > before.pose.position_meters[0]);
        assert!(!after.cockpit_control_active);
    }

    #[test]
    fn thruster_changes_by_exact_points_only_with_cockpit_authority() {
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            99,
        );
        ship.adjust_thruster(1);
        assert_eq!(ship.snapshot().thruster_percentage, 100);
        assert_eq!(
            ship.snapshot().speed_meters_per_second,
            PHASE_ZERO_REFERENCE_MAX_SPEED_METERS_PER_SECOND
        );
        ship.adjust_thruster(1);
        assert_eq!(ship.snapshot().thruster_percentage, 100);
        ship.adjust_thruster(-1);
        assert_eq!(ship.snapshot().thruster_percentage, 99);
        ship.set_cockpit_control(false);
        ship.adjust_thruster(-1);
        assert_eq!(ship.snapshot().thruster_percentage, 99);
    }

    #[test]
    fn solid_body_penetration_is_corrected_to_nearest_valid_position() {
        let earth = CELESTIAL_BODIES
            .iter()
            .find(|body| body.id == CelestialBodyId::Earth)
            .unwrap();
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: earth.center.meters(),
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
        );
        ship.advance(Duration::ZERO);
        let position = ship.snapshot().pose.position_meters;
        let distance = vector_length([
            position[0] - earth.center.meters()[0],
            position[1] - earth.center.meters()[1],
            position[2] - earth.center.meters()[2],
        ]);
        assert!((distance - earth.radius_meters - COLLISION_RADIUS_METERS).abs() < 1.0e-3);
    }
}
