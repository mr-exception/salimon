//! Portable Phase 0 ship state and interaction rules.
//!
//! This crate owns direct-speed flight, smoothed steering, cockpit authority,
//! solid-body collision correction, and the landed-only door contract without
//! depending on platform input or GPU presentation.

use std::time::Duration;

use salimon_world::{
    BodyRole, CELESTIAL_BODIES, CelestialBody, CelestialBodyId,
    LANDING_RANGE_ALTITUDE_RADIUS_FACTOR, PHASE_ZERO_REFERENCE_MAX_SPEED_METERS_PER_SECOND,
    WorldPosition, nearest_celestial_body_within_surface_distance,
};

/// Lowest point of the revised 4 m-tall Task 10 runtime mesh in ship-local Y.
pub const LOWEST_LOCAL_Y_METERS: f64 = -0.107_526_881_720_430_11;
/// Phase 0 steering rate after the short presentation ramp has settled.
pub const STEERING_RATE_RADIANS_PER_SECOND: f64 = 5.0_f64.to_radians();
/// Time for a steering axis to ramp fully on or off. This is not inertia.
pub const STEERING_RAMP_SECONDS: f64 = 0.12;
/// Conservative bounding sphere for the wider 20.3 x 4.0 x 20.0 meter ship asset.
pub const COLLISION_RADIUS_METERS: f64 = 15.0;
/// Height above a solid surface at which Phase 0 landing assistance is offered.
pub const LANDING_RANGE_RADIUS_FRACTION: f64 = LANDING_RANGE_ALTITUDE_RADIUS_FACTOR;
/// Deliberately readable fixed speeds for the short automatic sequences.
pub const LANDING_ASSIST_SPEED_METERS_PER_SECOND: f64 = 40.0;
pub const TAKEOFF_ASSIST_SPEED_METERS_PER_SECOND: f64 = 60.0;
/// Inclusive nominal-surface distance for the cockpit's nearby-body sensor.
pub const NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS: f64 = 3_000_000.0;
/// Noncanonical Phase 0 fixture values used to exercise live Core telemetry.
pub const DEFAULT_CORE_ENERGY_CAPACITY_JOULES: u64 = 1_000_000_000_000;
pub const DEFAULT_CORE_ENERGY_STORED_JOULES: u64 = 750_000_000_000;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SteeringInput {
    /// `-1` pitches down, `1` pitches up.
    pub pitch: f64,
    /// `-1` yaws right, `1` yaws left in the ship's rendered frame.
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
    AssistedLanding { body: CelestialBodyId },
    AssistedTakeoff { body: CelestialBodyId },
    Flying,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CockpitMessage {
    DoorLockedWhileInFlight,
    PressToLand,
    PressToTakeOff,
    CloseDoorBeforeTakeoff,
    LandingInProgress,
    TakeoffInProgress,
}

impl CockpitMessage {
    #[must_use]
    pub const fn text(self) -> &'static str {
        match self {
            Self::DoorLockedWhileInFlight => "Door locked while in flight",
            Self::PressToLand => "Press L to land",
            Self::PressToTakeOff => "Press L to take off",
            Self::CloseDoorBeforeTakeoff => "Close door before takeoff",
            Self::LandingInProgress => "Assisted landing in progress",
            Self::TakeoffInProgress => "Assisted takeoff in progress",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShipPose {
    pub position_meters: [f64; 3],
    /// Unit quaternion `[x, y, z, w]` rotating ship-local coordinates to world.
    pub orientation: [f64; 4],
}

/// Bounded Energy Core telemetry fixture. Phase 0 does not consume or generate it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnergyCoreState {
    pub capacity_joules: u64,
    pub stored_joules: u64,
}

impl Default for EnergyCoreState {
    fn default() -> Self {
        Self {
            capacity_joules: DEFAULT_CORE_ENERGY_CAPACITY_JOULES,
            stored_joules: DEFAULT_CORE_ENERGY_STORED_JOULES,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NearbyBodyTelemetry {
    pub name: &'static str,
    pub surface_distance_meters: f64,
    /// Negative approaches the body, positive recedes, and zero is stationary.
    pub radial_speed_meters_per_second: f64,
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
    pub velocity_meters_per_second: [f64; 3],
    pub energy_core: EnergyCoreState,
    pub nearby_body: Option<NearbyBodyTelemetry>,
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
    energy_core: EnergyCoreState,
    cockpit_message: Option<CockpitMessage>,
    steering_target: SteeringInput,
    steering: SteeringInput,
    assist_surface_normal: [f64; 3],
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
            energy_core: EnergyCoreState::default(),
            cockpit_message: None,
            steering_target: SteeringInput::default(),
            steering: SteeringInput::default(),
            assist_surface_normal: [0.0, 1.0, 0.0],
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
            energy_core: EnergyCoreState::default(),
            cockpit_message: None,
            steering_target: SteeringInput::default(),
            steering: SteeringInput::default(),
            assist_surface_normal: [0.0, 1.0, 0.0],
        }
    }

    pub fn set_cockpit_control(&mut self, active: bool) {
        self.cockpit_control_active = active;
        if !active {
            self.steering_target = SteeringInput::default();
        }
    }

    pub fn set_steering_input(&mut self, input: SteeringInput) {
        self.steering_target =
            if self.cockpit_control_active && self.flight_state == FlightState::Flying {
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
        if !self.cockpit_control_active || self.flight_state != FlightState::Flying {
            return;
        }
        self.thruster_percentage = i16::from(self.thruster_percentage)
            .saturating_add(i16::from(percentage_points))
            .clamp(0, 100) as u8;
    }

    /// Changes only the Phase 0 telemetry fixture; no power simulation is run.
    pub fn set_stored_core_energy_joules(&mut self, stored_joules: u64) {
        self.energy_core.stored_joules = stored_joules.min(self.energy_core.capacity_joules);
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
            FlightState::Flying
            | FlightState::AssistedLanding { .. }
            | FlightState::AssistedTakeoff { .. } => {
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
        match self.flight_state {
            FlightState::Flying => {
                let distance = self.speed_meters_per_second() * seconds;
                let forward = self.pose.axes()[0];
                for (position, direction) in self.pose.position_meters.iter_mut().zip(forward) {
                    *position += direction * distance;
                }
                self.resolve_solid_body_collisions();
            }
            FlightState::AssistedLanding { body } => self.advance_landing(body, seconds),
            FlightState::AssistedTakeoff { body } => self.advance_takeoff(body, seconds),
            FlightState::Landed { .. } => {}
        }
    }

    /// Performs the contextual `L` action. Automatic sequences deliberately do
    /// nothing here, making them uncancellable once started.
    pub fn trigger_landing_action(&mut self) {
        if !self.cockpit_control_active {
            return;
        }
        self.cockpit_message = None;
        match self.flight_state {
            FlightState::Flying => {
                if let Some(body) = self.landing_target() {
                    let center = body.center.meters();
                    self.assist_surface_normal =
                        normalize_or(sub(self.pose.position_meters, center), self.pose.axes()[1]);
                    self.pose.orientation = surface_aligned_orientation(
                        self.assist_surface_normal,
                        self.pose.axes()[0],
                    );
                    self.steering_target = SteeringInput::default();
                    self.steering = SteeringInput::default();
                    self.flight_state = FlightState::AssistedLanding { body: body.id };
                }
            }
            FlightState::Landed { body } => {
                if self.door_state == DoorState::Open {
                    self.cockpit_message = Some(CockpitMessage::CloseDoorBeforeTakeoff);
                } else {
                    let center = body_definition(body).center.meters();
                    self.assist_surface_normal =
                        normalize_or(sub(self.pose.position_meters, center), self.pose.axes()[1]);
                    self.flight_state = FlightState::AssistedTakeoff { body };
                }
            }
            FlightState::AssistedLanding { .. } | FlightState::AssistedTakeoff { .. } => {}
        }
    }

    #[must_use]
    pub fn contextual_cockpit_message(&self) -> Option<CockpitMessage> {
        if let Some(message) = self.cockpit_message {
            return Some(message);
        }
        match self.flight_state {
            FlightState::AssistedLanding { .. } => Some(CockpitMessage::LandingInProgress),
            FlightState::AssistedTakeoff { .. } => Some(CockpitMessage::TakeoffInProgress),
            FlightState::Landed { .. } if self.cockpit_control_active => {
                Some(CockpitMessage::PressToTakeOff)
            }
            FlightState::Flying
                if self.cockpit_control_active && self.landing_target().is_some() =>
            {
                Some(CockpitMessage::PressToLand)
            }
            _ => None,
        }
    }

    fn landing_target(&self) -> Option<&'static CelestialBody> {
        CELESTIAL_BODIES.iter().find(|body| {
            body.role == BodyRole::Solid
                && vector_length(sub(self.pose.position_meters, body.center.meters()))
                    <= body.radius_meters * (1.0 + LANDING_RANGE_RADIUS_FRACTION)
        })
    }

    fn advance_landing(&mut self, body_id: CelestialBodyId, seconds: f64) {
        let body = body_definition(body_id);
        let destination_distance = body.radius_meters - LOWEST_LOCAL_Y_METERS;
        let current_distance = vector_length(sub(self.pose.position_meters, body.center.meters()));
        let speed = landing_assist_speed(body);
        let next_distance = (current_distance - speed * seconds).max(destination_distance);
        self.pose.position_meters = add(
            body.center.meters(),
            scale(self.assist_surface_normal, next_distance),
        );
        if next_distance <= destination_distance + 1.0e-9 {
            self.flight_state = FlightState::Landed { body: body_id };
            self.thruster_percentage = 0;
        }
    }

    fn advance_takeoff(&mut self, body_id: CelestialBodyId, seconds: f64) {
        let body = body_definition(body_id);
        let clear_distance =
            body.radius_meters * (1.0 + LANDING_RANGE_RADIUS_FRACTION) + COLLISION_RADIUS_METERS;
        let current_distance = vector_length(sub(self.pose.position_meters, body.center.meters()));
        let speed = takeoff_assist_speed(body);
        let next_distance = (current_distance + speed * seconds).min(clear_distance);
        self.pose.position_meters = add(
            body.center.meters(),
            scale(self.assist_surface_normal, next_distance),
        );
        if next_distance >= clear_distance - 1.0e-9 {
            self.flight_state = FlightState::Flying;
        }
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
            FlightState::AssistedLanding { body } => scale(
                self.assist_surface_normal,
                -landing_assist_speed(body_definition(body)),
            ),
            FlightState::AssistedTakeoff { body } => scale(
                self.assist_surface_normal,
                takeoff_assist_speed(body_definition(body)),
            ),
            FlightState::Landed { .. } => [0.0; 3],
        }
    }

    fn nearby_body_telemetry(
        &self,
        velocity_meters_per_second: [f64; 3],
    ) -> Option<NearbyBodyTelemetry> {
        let position = WorldPosition::new(
            self.pose.position_meters[0],
            self.pose.position_meters[1],
            self.pose.position_meters[2],
        );
        nearest_celestial_body_within_surface_distance(
            CELESTIAL_BODIES,
            position,
            NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS,
        )
        .map(|(body, surface_distance_meters)| NearbyBodyTelemetry {
            name: body.name,
            surface_distance_meters,
            radial_speed_meters_per_second: body
                .radial_speed_meters_per_second(position, velocity_meters_per_second),
        })
    }

    #[must_use]
    pub fn snapshot(&self) -> ShipSnapshot {
        let velocity_meters_per_second = self.velocity_meters_per_second();
        ShipSnapshot {
            pose: self.pose,
            flight_state: self.flight_state,
            door_state: self.door_state,
            cockpit_control_active: self.cockpit_control_active,
            thruster_percentage: self.thruster_percentage,
            speed_meters_per_second: self.speed_meters_per_second(),
            velocity_meters_per_second,
            energy_core: self.energy_core,
            nearby_body: self.nearby_body_telemetry(velocity_meters_per_second),
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

fn body_definition(id: CelestialBodyId) -> &'static CelestialBody {
    CELESTIAL_BODIES
        .iter()
        .find(|body| body.id == id)
        .expect("flight state only stores catalog body identifiers")
}

fn landing_assist_speed(body: &CelestialBody) -> f64 {
    LANDING_ASSIST_SPEED_METERS_PER_SECOND.max(body.radius_meters * 0.10)
}

fn takeoff_assist_speed(body: &CelestialBody) -> f64 {
    TAKEOFF_ASSIST_SPEED_METERS_PER_SECOND.max(body.radius_meters * 0.15)
}

fn add(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn sub(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn scale(vector: [f64; 3], amount: f64) -> [f64; 3] {
    vector.map(|value| value * amount)
}

fn dot(left: [f64; 3], right: [f64; 3]) -> f64 {
    left.into_iter().zip(right).map(|(a, b)| a * b).sum()
}

fn cross(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn normalize_or(vector: [f64; 3], fallback: [f64; 3]) -> [f64; 3] {
    let length = vector_length(vector);
    if length > 1.0e-12 {
        scale(vector, length.recip())
    } else {
        fallback
    }
}

fn surface_aligned_orientation(up: [f64; 3], prior_forward: [f64; 3]) -> [f64; 4] {
    let tangent = sub(prior_forward, scale(up, dot(prior_forward, up)));
    let forward = normalize_or(
        tangent,
        normalize_or(cross([0.0, 0.0, 1.0], up), [1.0, 0.0, 0.0]),
    );
    let port = normalize_or(cross(forward, up), [0.0, 0.0, 1.0]);
    quaternion_from_axes(forward, up, port)
}

fn quaternion_from_axes(forward: [f64; 3], up: [f64; 3], port: [f64; 3]) -> [f64; 4] {
    let m00 = forward[0];
    let m01 = up[0];
    let m02 = port[0];
    let m10 = forward[1];
    let m11 = up[1];
    let m12 = port[1];
    let m20 = forward[2];
    let m21 = up[2];
    let m22 = port[2];
    let trace = m00 + m11 + m22;
    let quaternion = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        [(m21 - m12) / s, (m02 - m20) / s, (m10 - m01) / s, 0.25 * s]
    } else if m00 > m11 && m00 > m22 {
        let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
        [0.25 * s, (m01 + m10) / s, (m02 + m20) / s, (m21 - m12) / s]
    } else if m11 > m22 {
        let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
        [(m01 + m10) / s, 0.25 * s, (m12 + m21) / s, (m02 - m20) / s]
    } else {
        let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
        [(m02 + m20) / s, (m12 + m21) / s, 0.25 * s, (m10 - m01) / s]
    };
    normalized_quaternion(quaternion)
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

    #[test]
    fn every_solid_body_supports_uncancellable_arbitrary_point_landing() {
        for body in CELESTIAL_BODIES
            .iter()
            .filter(|body| body.role == BodyRole::Solid)
        {
            let normal = normalize_or([0.31, 0.72, -0.62], [0.0, 1.0, 0.0]);
            let approach_distance = body.radius_meters * 1.10 + COLLISION_RADIUS_METERS;
            let mut ship = ShipController::flying(
                ShipPose {
                    position_meters: add(body.center.meters(), scale(normal, approach_distance)),
                    orientation: axis_angle_quaternion([0.0, 0.0, 1.0], 1.2),
                },
                100,
            );
            assert_eq!(
                ship.contextual_cockpit_message(),
                Some(CockpitMessage::PressToLand)
            );
            ship.trigger_landing_action();
            assert_eq!(
                ship.snapshot().flight_state,
                FlightState::AssistedLanding { body: body.id }
            );
            ship.set_cockpit_control(false);
            ship.trigger_landing_action();
            for _ in 0..200 {
                ship.advance(Duration::from_millis(100));
            }
            let snapshot = ship.snapshot();
            assert_eq!(snapshot.flight_state, FlightState::Landed { body: body.id });
            assert_eq!(snapshot.thruster_percentage, 0);
            let landed_normal = normalize_or(
                sub(snapshot.pose.position_meters, body.center.meters()),
                [0.0, 1.0, 0.0],
            );
            assert!(dot(landed_normal, normal) > 1.0 - 1.0e-10);
            assert!(dot(snapshot.pose.axes()[1], normal) > 1.0 - 1.0e-10);
        }
    }

    #[test]
    fn landing_requires_cockpit_authority_and_exact_range() {
        let mars = body_definition(CelestialBodyId::Mars);
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: add(mars.center.meters(), [mars.radius_meters * 1.16, 0.0, 0.0]),
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
        );
        assert_eq!(ship.contextual_cockpit_message(), None);
        ship.pose.position_meters[0] = mars.center.meters()[0] + mars.radius_meters * 1.149_999;
        ship.set_cockpit_control(false);
        ship.trigger_landing_action();
        assert_eq!(ship.snapshot().flight_state, FlightState::Flying);
        ship.set_cockpit_control(true);
        ship.trigger_landing_action();
        assert_eq!(
            ship.snapshot().flight_state,
            FlightState::AssistedLanding {
                body: CelestialBodyId::Mars
            }
        );
    }

    #[test]
    fn takeoff_is_door_interlocked_and_completes_without_cockpit_authority() {
        let mut ship = ShipController::default();
        ship.set_cockpit_control(true);
        ship.toggle_door();
        ship.trigger_landing_action();
        assert_eq!(
            ship.contextual_cockpit_message(),
            Some(CockpitMessage::CloseDoorBeforeTakeoff)
        );
        assert!(matches!(
            ship.snapshot().flight_state,
            FlightState::Landed { .. }
        ));
        ship.toggle_door();
        ship.trigger_landing_action();
        assert!(matches!(
            ship.snapshot().flight_state,
            FlightState::AssistedTakeoff { .. }
        ));
        ship.set_cockpit_control(false);
        ship.trigger_landing_action();
        for _ in 0..200 {
            ship.advance(Duration::from_millis(100));
        }
        assert_eq!(ship.snapshot().flight_state, FlightState::Flying);
    }

    #[test]
    fn core_energy_fixture_is_bounded_and_updates_live_snapshots() {
        let mut ship = ShipController::default();
        assert_eq!(ship.snapshot().energy_core, EnergyCoreState::default());

        ship.set_stored_core_energy_joules(125_000_000_000);
        assert_eq!(ship.snapshot().energy_core.stored_joules, 125_000_000_000);
        ship.set_stored_core_energy_joules(u64::MAX);
        assert_eq!(
            ship.snapshot().energy_core.stored_joules,
            DEFAULT_CORE_ENERGY_CAPACITY_JOULES
        );
    }

    #[test]
    fn nearby_body_telemetry_crosses_range_and_reports_radial_direction() {
        let earth = body_definition(CelestialBodyId::Earth);
        let position = add(
            earth.center.meters(),
            [
                earth.radius_meters + NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS,
                0.0,
                0.0,
            ],
        );
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: position,
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            10,
        );

        let receding = ship.snapshot().nearby_body.unwrap();
        assert_eq!(receding.name, "Earth");
        assert_eq!(
            receding.surface_distance_meters,
            NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS
        );
        assert!(receding.radial_speed_meters_per_second > 0.0);

        ship.pose.orientation = [0.0, 1.0, 0.0, 0.0];
        assert!(
            ship.snapshot()
                .nearby_body
                .unwrap()
                .radial_speed_meters_per_second
                < 0.0
        );

        ship.thruster_percentage = 0;
        assert_eq!(
            ship.snapshot()
                .nearby_body
                .unwrap()
                .radial_speed_meters_per_second,
            0.0
        );

        ship.pose.position_meters[0] += 1.0;
        assert!(ship.snapshot().nearby_body.is_none());
    }
}
