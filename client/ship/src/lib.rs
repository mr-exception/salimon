//! Portable Phase 0 ship state and interaction rules.
//!
//! Flight steering and assisted landing/takeoff are owned by later tasks. This
//! crate establishes persistent motion, cockpit authority, and the landed-only
//! door contract without depending on platform input or GPU presentation.

use std::time::Duration;

use salimon_world::{
    CELESTIAL_BODIES, CelestialBodyId, PHASE_ZERO_REFERENCE_MAX_SPEED_METERS_PER_SECOND,
};

/// Lowest point of the uniformly enlarged Task 10 runtime mesh in ship-local Y.
pub const LOWEST_LOCAL_Y_METERS: f64 = -0.20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoorState {
    Closed,
    Open,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlightState {
    Landed { body: CelestialBodyId },
    Flying,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CockpitMessage {
    DoorLockedWhileInFlight,
}

impl CockpitMessage {
    #[must_use]
    pub const fn text(self) -> &'static str {
        match self {
            Self::DoorLockedWhileInFlight => "Door locked while in flight",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShipPose {
    pub position_meters: [f64; 3],
    /// Unit quaternion `[x, y, z, w]` rotating ship-local coordinates to world.
    pub orientation: [f64; 4],
}

impl ShipPose {
    #[must_use]
    pub fn axes(self) -> [[f64; 3]; 3] {
        let [x, y, z, w] = normalized_quaternion(self.orientation);
        [
            [
                1.0 - 2.0 * (y * y + z * z),
                2.0 * (x * y + z * w),
                2.0 * (x * z - y * w),
            ],
            [
                2.0 * (x * y - z * w),
                1.0 - 2.0 * (x * x + z * z),
                2.0 * (y * z + x * w),
            ],
            [
                2.0 * (x * z + y * w),
                2.0 * (y * z - x * w),
                1.0 - 2.0 * (x * x + y * y),
            ],
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShipSnapshot {
    pub pose: ShipPose,
    pub flight_state: FlightState,
    pub door_state: DoorState,
    pub cockpit_control_active: bool,
    pub thruster_percentage: u8,
    pub speed_meters_per_second: f64,
    pub cockpit_message: Option<CockpitMessage>,
}

#[derive(Clone, Debug)]
pub struct ShipController {
    pose: ShipPose,
    flight_state: FlightState,
    door_state: DoorState,
    cockpit_control_active: bool,
    thruster_percentage: u8,
    cockpit_message: Option<CockpitMessage>,
}

impl Default for ShipController {
    fn default() -> Self {
        let earth = CELESTIAL_BODIES
            .iter()
            .find(|body| body.id == CelestialBodyId::Earth)
            .expect("world catalog always contains Earth");
        let mut position_meters = earth.center.meters();
        position_meters[1] += earth.radius_meters - LOWEST_LOCAL_Y_METERS;
        Self {
            pose: ShipPose {
                position_meters,
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            flight_state: FlightState::Landed {
                body: CelestialBodyId::Earth,
            },
            door_state: DoorState::Closed,
            cockpit_control_active: false,
            thruster_percentage: 0,
            cockpit_message: None,
        }
    }
}

impl ShipController {
    #[must_use]
    pub fn flying(pose: ShipPose, thruster_percentage: u8) -> Self {
        Self {
            pose,
            flight_state: FlightState::Flying,
            door_state: DoorState::Closed,
            cockpit_control_active: true,
            thruster_percentage: thruster_percentage.min(100),
            cockpit_message: None,
        }
    }

    pub fn set_cockpit_control(&mut self, active: bool) {
        self.cockpit_control_active = active;
    }

    pub fn toggle_door(&mut self) {
        self.cockpit_message = None;
        match self.flight_state {
            FlightState::Landed { .. } => {
                self.door_state = match self.door_state {
                    DoorState::Closed => DoorState::Open,
                    DoorState::Open => DoorState::Closed,
                };
            }
            FlightState::Flying => {
                self.door_state = DoorState::Closed;
                self.cockpit_message = Some(CockpitMessage::DoorLockedWhileInFlight);
            }
        }
    }

    pub fn clear_cockpit_message(&mut self) {
        self.cockpit_message = None;
    }

    pub fn advance(&mut self, delta: Duration) {
        if self.flight_state != FlightState::Flying {
            return;
        }
        let distance = self.speed_meters_per_second() * delta.as_secs_f64().min(0.1);
        let forward = self.pose.axes()[0];
        for (position, direction) in self.pose.position_meters.iter_mut().zip(forward) {
            *position += direction * distance;
        }
    }

    #[must_use]
    pub fn speed_meters_per_second(&self) -> f64 {
        PHASE_ZERO_REFERENCE_MAX_SPEED_METERS_PER_SECOND * f64::from(self.thruster_percentage)
            / 100.0
    }

    #[must_use]
    pub fn snapshot(&self) -> ShipSnapshot {
        ShipSnapshot {
            pose: self.pose,
            flight_state: self.flight_state,
            door_state: self.door_state,
            cockpit_control_active: self.cockpit_control_active,
            thruster_percentage: self.thruster_percentage,
            speed_meters_per_second: self.speed_meters_per_second(),
            cockpit_message: self.cockpit_message,
        }
    }
}

fn normalized_quaternion(quaternion: [f64; 4]) -> [f64; 4] {
    let length = quaternion
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    if length <= 1.0e-12 || !length.is_finite() {
        [0.0, 0.0, 0.0, 1.0]
    } else {
        quaternion.map(|value| value / length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ship_is_landed_on_earth_with_closed_door() {
        let snapshot = ShipController::default().snapshot();
        assert_eq!(
            snapshot.flight_state,
            FlightState::Landed {
                body: CelestialBodyId::Earth
            }
        );
        assert_eq!(snapshot.door_state, DoorState::Closed);
        assert_eq!(snapshot.thruster_percentage, 0);
        let earth = CELESTIAL_BODIES
            .iter()
            .find(|body| body.id == CelestialBodyId::Earth)
            .unwrap();
        assert_eq!(snapshot.pose.orientation, [0.0, 0.0, 0.0, 1.0]);
        assert!(
            (snapshot.pose.position_meters[1] + LOWEST_LOCAL_Y_METERS
                - (earth.center.meters()[1] + earth.radius_meters))
                .abs()
                < 1.0e-9
        );
    }

    #[test]
    fn landed_door_toggles_but_flying_door_is_locked_with_monitor_message() {
        let mut landed = ShipController::default();
        landed.toggle_door();
        assert_eq!(landed.snapshot().door_state, DoorState::Open);

        let mut flying = ShipController::flying(
            ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            40,
        );
        flying.toggle_door();
        let snapshot = flying.snapshot();
        assert_eq!(snapshot.door_state, DoorState::Closed);
        assert_eq!(
            snapshot.cockpit_message.map(CockpitMessage::text),
            Some("Door locked while in flight")
        );
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
        assert_eq!(
            after.speed_meters_per_second,
            before.speed_meters_per_second
        );
        assert!(after.pose.position_meters[0] > before.pose.position_meters[0]);
        assert!(!after.cockpit_control_active);
    }

    #[test]
    fn axes_rotate_local_forward_with_pose() {
        let half_turn_about_y = ShipPose {
            position_meters: [0.0; 3],
            orientation: [0.0, 1.0, 0.0, 0.0],
        };
        let axes = half_turn_about_y.axes();
        assert!((axes[0][0] + 1.0).abs() < 1.0e-12);
        assert!(axes[0][1].abs() < 1.0e-12);
        assert!(axes[0][2].abs() < 1.0e-12);
    }
}
