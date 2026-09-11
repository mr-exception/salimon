//! Portable Phase 0 ship state and interaction rules.
//!
//! This crate owns direct-speed flight, smoothed steering, cockpit authority,
//! solid-body collision correction, and the landed-only door contract without
//! depending on platform input or GPU presentation.

use std::time::Duration;

use salimon_world::{
    BodyRole, CELESTIAL_BODIES, CelestialBodyId, PHASE_ZERO_REFERENCE_MAX_SPEED_METERS_PER_SECOND,
};

/// Lowest point of the uniformly enlarged Task 10 runtime mesh in ship-local Y.
pub const LOWEST_LOCAL_Y_METERS: f64 = -0.20;
/// Phase 0 steering rate after the short presentation ramp has settled.
pub const STEERING_RATE_RADIANS_PER_SECOND: f64 = 5.0_f64.to_radians();
/// Time for a steering axis to ramp fully on or off. This is not inertia.
pub const STEERING_RAMP_SECONDS: f64 = 0.12;
/// Conservative bounding sphere for the 20.3 x 7.44 x 16.6 meter ship asset.
pub const COLLISION_RADIUS_METERS: f64 = 13.7;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SteeringInput {
    /// `-1` pitches down, `1` pitches up.
    pub pitch: f64,
    /// `-1` yaws left, `1` yaws right.
    pub yaw: f64,
    /// `-1` rolls left, `1` rolls right.
    pub roll: f64,
}

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
    pub steering: SteeringInput,
}

#[derive(Clone, Debug)]
pub struct ShipController {
    pose: ShipPose,
    flight_state: FlightState,
    door_state: DoorState,
    cockpit_control_active: bool,
    thruster_percentage: u8,
    cockpit_message: Option<CockpitMessage>,
    steering_target: SteeringInput,
    steering: SteeringInput,
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
            steering_target: SteeringInput::default(),
            steering: SteeringInput::default(),
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
            steering_target: SteeringInput::default(),
            steering: SteeringInput::default(),
        }
    }

    pub fn set_cockpit_control(&mut self, active: bool) {
        self.cockpit_control_active = active;
        if !active {
            self.steering_target = SteeringInput::default();
        }
    }

    pub fn set_steering_input(&mut self, input: SteeringInput) {
        self.steering_target = if self.cockpit_control_active {
            SteeringInput {
                pitch: input.pitch.clamp(-1.0, 1.0),
                yaw: input.yaw.clamp(-1.0, 1.0),
                roll: input.roll.clamp(-1.0, 1.0),
            }
        } else {
            SteeringInput::default()
        };
    }

    pub fn adjust_thruster(&mut self, percentage_points: i8) {
        if !self.cockpit_control_active {
            return;
        }
        self.thruster_percentage = i16::from(self.thruster_percentage)
            .saturating_add(i16::from(percentage_points))
            .clamp(0, 100) as u8;
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
        let seconds = delta.as_secs_f64().min(0.1);
        self.advance_steering(seconds);
        if self.flight_state != FlightState::Flying {
            return;
        }
        let distance = self.speed_meters_per_second() * seconds;
        let forward = self.pose.axes()[0];
        for (position, direction) in self.pose.position_meters.iter_mut().zip(forward) {
            *position += direction * distance;
        }
        self.resolve_solid_body_collisions();
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
            steering: self.steering,
        }
    }

    fn advance_steering(&mut self, seconds: f64) {
        let step = seconds / STEERING_RAMP_SECONDS;
        self.steering.pitch = approach(self.steering.pitch, self.steering_target.pitch, step);
        self.steering.yaw = approach(self.steering.yaw, self.steering_target.yaw, step);
        self.steering.roll = approach(self.steering.roll, self.steering_target.roll, step);
        if self.flight_state != FlightState::Flying {
            return;
        }
        let angles = [
            self.steering.roll * STEERING_RATE_RADIANS_PER_SECOND * seconds,
            self.steering.yaw * STEERING_RATE_RADIANS_PER_SECOND * seconds,
            self.steering.pitch * STEERING_RATE_RADIANS_PER_SECOND * seconds,
        ];
        for (axis, angle) in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
            .into_iter()
            .zip(angles)
        {
            self.pose.orientation =
                quaternion_multiply(self.pose.orientation, axis_angle_quaternion(axis, angle));
        }
        self.pose.orientation = normalized_quaternion(self.pose.orientation);
    }

    fn resolve_solid_body_collisions(&mut self) {
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

fn approach(current: f64, target: f64, maximum_delta: f64) -> f64 {
    current + (target - current).clamp(-maximum_delta, maximum_delta)
}

fn vector_length(vector: [f64; 3]) -> f64 {
    vector.iter().map(|value| value * value).sum::<f64>().sqrt()
}

fn axis_angle_quaternion(axis: [f64; 3], angle: f64) -> [f64; 4] {
    let half = angle * 0.5;
    let sine = half.sin();
    [axis[0] * sine, axis[1] * sine, axis[2] * sine, half.cos()]
}

fn quaternion_multiply(left: [f64; 4], right: [f64; 4]) -> [f64; 4] {
    let [lx, ly, lz, lw] = left;
    let [rx, ry, rz, rw] = right;
    [
        lw * rx + lx * rw + ly * rz - lz * ry,
        lw * ry - lx * rz + ly * rw + lz * rx,
        lw * rz + lx * ry - ly * rx + lz * rw,
        lw * rw - lx * rx - ly * ry - lz * rz,
    ]
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
        ship.adjust_thruster(1);
        assert_eq!(ship.snapshot().thruster_percentage, 100);
        ship.adjust_thruster(-1);
        assert_eq!(ship.snapshot().thruster_percentage, 99);
        ship.set_cockpit_control(false);
        ship.adjust_thruster(-1);
        assert_eq!(ship.snapshot().thruster_percentage, 99);
    }

    #[test]
    fn steering_ramps_then_holds_the_fixed_five_degree_rate() {
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
        );
        ship.set_steering_input(SteeringInput {
            pitch: 1.0,
            ..SteeringInput::default()
        });
        ship.advance(Duration::from_millis(60));
        assert_eq!(ship.snapshot().steering.pitch, 0.5);
        ship.advance(Duration::from_millis(60));
        assert_eq!(ship.snapshot().steering.pitch, 1.0);
        let before = ship.snapshot().pose.axes()[0];
        ship.advance(Duration::from_millis(100));
        let after = ship.snapshot().pose.axes()[0];
        let angle = before
            .iter()
            .zip(after)
            .map(|(left, right)| left * right)
            .sum::<f64>()
            .clamp(-1.0, 1.0)
            .acos();
        assert!((angle - 0.5_f64.to_radians()).abs() < 1.0e-10);
        assert!(after[1] > before[1], "positive pitch must raise the nose");
    }

    #[test]
    fn steering_target_clears_when_cockpit_is_left_without_changing_heading() {
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
        );
        ship.set_steering_input(SteeringInput {
            yaw: 1.0,
            ..SteeringInput::default()
        });
        ship.advance(Duration::from_millis(100));
        ship.set_cockpit_control(false);
        ship.advance(Duration::from_millis(100));
        ship.advance(Duration::from_millis(100));
        let settled = ship.snapshot();
        assert_eq!(settled.steering, SteeringInput::default());
        let orientation = settled.pose.orientation;
        ship.advance(Duration::from_secs(1));
        assert_eq!(ship.snapshot().pose.orientation, orientation);
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
