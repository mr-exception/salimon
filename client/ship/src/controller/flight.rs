//! Inertial force-based flight and solid-body collision correction.

use super::ShipController;
use crate::{COLLISION_RADIUS_METERS, FlightState};
use salimon_math::{add, length as vector_length, scale};
use salimon_world::{BodyRole, CELESTIAL_BODIES};

impl ShipController {
    pub(super) fn advance_flight(&mut self, seconds: f64) {
        self.integrate_thrust(seconds, self.thruster_percentage as i8);
        self.resolve_solid_body_collisions();
    }

    fn integrate_thrust(&mut self, seconds: f64, throttle: i8) {
        let acceleration = scale(
            self.pose.axes()[0],
            self.propulsion
                .acceleration_meters_per_second_squared(throttle),
        );
        self.pose.position_meters = add(
            self.pose.position_meters,
            add(
                scale(self.velocity, seconds),
                scale(acceleration, 0.5 * seconds * seconds),
            ),
        );
        self.velocity = add(self.velocity, scale(acceleration, seconds));
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
        let [x, y, z] = self.velocity_meters_per_second();
        x.hypot(y).hypot(z)
    }

    #[must_use]
    pub fn velocity_meters_per_second(&self) -> [f64; 3] {
        match self.flight_state {
            FlightState::Flying => self.velocity,
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
            let minimum = body.radius_meters
                + self.terrain.elevation_bounds(*body)[1]
                + COLLISION_RADIUS_METERS;
            if self.terrain != salimon_world::terrain::TerrainMode::Flat {
                if distance < minimum {
                    // Extrema are only the broad bound. Contact projects the
                    // conservative ship sphere against the shared local query.
                    for _ in 0..4 {
                        let local = salimon_math::sub(self.pose.position_meters, center);
                        let direction =
                            crate::orientation::normalize_or(local, self.pose.axes()[1]);
                        let ground = self.terrain.sample(*body, direction);
                        let gap = salimon_math::dot(
                            salimon_math::sub(local, ground.body_local_position_meters),
                            ground.outward_normal,
                        );
                        if gap >= COLLISION_RADIUS_METERS {
                            break;
                        }
                        self.pose.position_meters = salimon_math::add(
                            self.pose.position_meters,
                            scale(
                                ground.outward_normal,
                                COLLISION_RADIUS_METERS - gap + 0.0002,
                            ),
                        );
                    }
                }
                continue;
            }
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
    fn long_direct_updates_do_not_discard_travel() {
        let pose = ShipPose {
            position_meters: [0.0; 3],
            orientation: [0.0, 0.0, 0.0, 1.0],
        };
        let mut whole = ShipController::flying(pose, 10);
        let mut split = whole.clone();
        whole.advance(Duration::from_secs(2));
        for _ in 0..20 {
            split.advance(Duration::from_millis(100));
        }
        assert!(
            (whole.snapshot().pose.position_meters[0] - split.snapshot().pose.position_meters[0])
                .abs()
                < 1e-9
        );
        assert!(whole.snapshot().pose.position_meters[0] > 0.0);
    }

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
        assert!(after.speed_meters_per_second > before.speed_meters_per_second);
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
        assert_eq!(ship.snapshot().speed_meters_per_second, 0.0);
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

#[cfg(test)]
mod inertial_tests {
    use super::*;
    use crate::{Propulsion, ShipPose, SteeringInput};
    use std::time::Duration;

    fn ship(throttle: u8, velocity: [f64; 3]) -> ShipController {
        ShipController::flying_with_velocity(
            ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            throttle,
            velocity,
        )
        .unwrap()
    }

    #[test]
    fn turning_while_coasting_preserves_world_momentum_and_true_speed() {
        let mut ship = ship(0, [300.0, -400.0, 0.0]);
        ship.set_steering_input(SteeringInput {
            yaw: 1.0,
            ..Default::default()
        });
        for _ in 0..100 {
            ship.advance(Duration::from_millis(16));
        }
        let state = ship.snapshot();
        assert_ne!(state.pose.orientation, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(state.velocity_meters_per_second, [300.0, -400.0, 0.0]);
        assert_eq!(state.speed_meters_per_second, 500.0);
        assert!((state.pose.position_meters[0] - 480.0).abs() < 1e-9);
    }

    #[test]
    fn forward_reverse_and_mass_obey_force_over_mass() {
        let mut forward = ship(100, [0.0; 3]);
        forward.advance(Duration::from_secs(1));
        assert_eq!(
            forward.snapshot().velocity_meters_per_second,
            [40.0, 0.0, 0.0]
        );
        assert_eq!(forward.snapshot().pose.position_meters, [20.0, 0.0, 0.0]);
        let mut reverse = ship(0, [0.0; 3]);
        reverse.integrate_thrust(1.0, -100);
        assert_eq!(
            reverse.snapshot().velocity_meters_per_second,
            [-20.0, 0.0, 0.0]
        );
        assert_eq!(reverse.snapshot().pose.position_meters, [-10.0, 0.0, 0.0]);
        forward.set_propulsion(Propulsion::new(200_000.0, 4_000_000.0, 2_000_000.0).unwrap());
        forward.advance(Duration::from_secs(1));
        assert_eq!(
            forward.snapshot().velocity_meters_per_second,
            [60.0, 0.0, 0.0]
        );
        assert_eq!(forward.snapshot().total_mass_kg, 200_000.0);
    }

    #[test]
    fn arbitrary_partitions_and_high_speed_keep_all_travel() {
        let mut whole = ship(100, [10_000_000.0, 0.0, 0.0]);
        let mut split = whole.clone();
        whole.advance(Duration::from_millis(1003));
        for ms in [1, 307, 90, 5, 600] {
            split.advance(Duration::from_millis(ms));
        }
        assert!(
            (whole.snapshot().pose.position_meters[0] - split.snapshot().pose.position_meters[0])
                .abs()
                < 1e-7
        );
        assert!(
            (whole.snapshot().speed_meters_per_second - split.snapshot().speed_meters_per_second)
                .abs()
                < 1e-7
        );
        let velocity = whole.velocity_meters_per_second();
        whole.adjust_thruster(-100);
        whole.set_cockpit_control(false);
        whole.advance(Duration::from_secs(1));
        assert_eq!(whole.velocity_meters_per_second(), velocity);
        assert!(ShipController::flying_with_velocity(whole.pose, 0, [f64::NAN; 3]).is_err());
    }
}
