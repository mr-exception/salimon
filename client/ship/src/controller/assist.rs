//! Captured landing/takeoff paths and uncancellable timing.

use super::ShipController;
use crate::orientation::{normalize_or, quaternion_slerp, surface_aligned_orientation};
use crate::{
    COLLISION_RADIUS_METERS, CockpitMessage, DoorState, FlightState, LANDED_ORIGIN_HEIGHT_METERS,
    LANDING_ASSIST_DURATION, LANDING_RANGE_RADIUS_FRACTION, ShipPose, SteeringInput,
    TAKEOFF_ASSIST_DURATION,
};
use salimon_math::{add, length as vector_length, scale, sub};
#[cfg(test)]
use salimon_world::CELESTIAL_BODIES;
use salimon_world::{BodyRole, CelestialBody, CelestialBodyId};
use std::time::Duration;

/// Starter eligibility tuning. Future upgrades remain documentation only.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssistConfig {
    pub landing_range_radius_fraction: f64,
    pub maximum_landing_speed_meters_per_second: f64,
}
impl Default for AssistConfig {
    fn default() -> Self {
        Self {
            landing_range_radius_fraction: LANDING_RANGE_RADIUS_FRACTION,
            maximum_landing_speed_meters_per_second: 20.0,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidAssistConfig;
impl std::fmt::Display for InvalidAssistConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("assist range must be finite in (0, 1]; speed must be finite and nonnegative")
    }
}
impl std::error::Error for InvalidAssistConfig {}

const RECOVERY_LIFT_SECONDS: f64 = 2.0;
const RECOVERY_REPOSITION_SECONDS: f64 = 2.0;
const RECOVERY_DURATION: Duration = Duration::from_secs(12);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RecoveryPath {
    start_normal: [f64; 3],
    start_distance: f64,
    clearance_distance: f64,
}

const LANDING_ALIGNMENT_SECONDS: f64 = 2.0;
const LANDING_APPROACH_SECONDS: f64 = 4.0;
const LANDING_TOUCHDOWN_SECONDS: f64 = 2.0;
// Local surface clearance, independent of planetary radius or landing eligibility.
// Eased stages peak at 7.5/15 m/s; each has at most 15 m/s² acceleration.
const TAKEOFF_LIFT_METERS: f64 = 10.0;
const TAKEOFF_TOTAL_RISE_METERS: f64 = 50.0;
const TAKEOFF_LIFT_SECONDS: f64 = 2.0;
const TAKEOFF_CLEARANCE_SECONDS: f64 = 4.0;
/// Captured path data prevents accumulated position or frame-rate-dependent timing.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AssistTransition {
    elapsed: Duration,
    duration: Duration,
    recovery: Option<RecoveryPath>,
    pub(super) velocity_meters_per_second: [f64; 3],
    reservation: Option<crate::energy::AssistEnergyReservation>,
    energy_watts: f64,
    start_pose: ShipPose,
    pub(super) surface_normal: [f64; 3],
    aligned_orientation: [f64; 4],
    landing_pose: Option<ShipPose>,
    start_distance: f64,
    intermediate_distance: f64,
    destination_distance: f64,
}

impl AssistTransition {
    fn new(pose: ShipPose, body: &CelestialBody, landing: bool) -> Self {
        let offset = sub(pose.position_meters, body.center.meters());
        let start_distance = vector_length(offset);
        let surface_normal = normalize_or(offset, pose.axes()[1]);
        let destination_distance = if landing {
            body.radius_meters + LANDED_ORIGIN_HEIGHT_METERS
        } else {
            start_distance + TAKEOFF_TOTAL_RISE_METERS
        };
        let intermediate_distance = if landing {
            // Reserve visible local motion for touchdown even on the largest body.
            destination_distance
                + ((start_distance - destination_distance).max(0.0) * 0.25).min(15.0)
        } else {
            start_distance + TAKEOFF_LIFT_METERS
        };
        Self {
            elapsed: Duration::ZERO,
            duration: if landing {
                LANDING_ASSIST_DURATION
            } else {
                TAKEOFF_ASSIST_DURATION
            },
            recovery: None,
            velocity_meters_per_second: [0.0; 3],
            reservation: None,
            energy_watts: 0.0,
            landing_pose: None,
            start_pose: pose,
            surface_normal,
            aligned_orientation: surface_aligned_orientation(surface_normal, pose.axes()[0]),
            start_distance,
            intermediate_distance,
            destination_distance,
        }
    }
}

pub(super) fn body_definition(id: CelestialBodyId) -> &'static CelestialBody {
    salimon_world::canonical::body(id).expect("flight state only stores catalog body identifiers")
}

pub(crate) fn smoothstep(amount: f64) -> f64 {
    let amount = amount.clamp(0.0, 1.0);
    amount * amount * (3.0 - 2.0 * amount)
}

/// Position and its analytic time derivative, with zero velocity at both ends.
fn eased_radial_segment(start: f64, end: f64, seconds: f64, duration: f64) -> (f64, f64) {
    let amount = (seconds / duration).clamp(0.0, 1.0);
    (
        start + (end - start) * smoothstep(amount),
        (end - start) * 6.0 * amount * (1.0 - amount) / duration,
    )
}

impl ShipController {
    pub fn configure_assistance(
        &mut self,
        config: AssistConfig,
    ) -> Result<(), InvalidAssistConfig> {
        if !config.landing_range_radius_fraction.is_finite()
            || config.landing_range_radius_fraction <= 0.0
            || config.landing_range_radius_fraction > 1.0
            || !config.maximum_landing_speed_meters_per_second.is_finite()
            || config.maximum_landing_speed_meters_per_second < 0.0
        {
            return Err(InvalidAssistConfig);
        }
        self.assist_config = config;
        Ok(())
    }

    pub(super) fn assist_remaining(&self) -> Option<Duration> {
        self.assist
            .as_ref()
            .map(|assist| assist.duration.saturating_sub(assist.elapsed))
    }

    /// Performs the contextual `L` action. Automatic sequences deliberately do
    /// nothing here, making them uncancellable once started.
    pub fn trigger_landing_action(&mut self) {
        if !self.cockpit_control_active || !self.energy_core.powered() || !self.engines_enabled {
            return;
        }
        self.clear_throttle_input();
        self.cockpit_message = None;
        match self.flight_state {
            FlightState::Flying => {
                if let Some(body) = self.landing_target() {
                    self.start_landing(body, false);
                }
            }
            FlightState::GroundedAfterCrash { body } => {
                if self.speed_meters_per_second()
                    <= self.assist_config.maximum_landing_speed_meters_per_second
                {
                    self.start_landing(body_definition(body), true);
                }
            }
            FlightState::Landed { body } => {
                if self.door_state == DoorState::Open || self.door_open_fraction > 0.0 {
                    self.cockpit_message = Some(CockpitMessage::CloseDoorBeforeTakeoff);
                } else {
                    let mut assist = AssistTransition::new(self.pose, body_definition(body), false);
                    if !self.reserve_assist_energy(&mut assist, TAKEOFF_ASSIST_DURATION) {
                        return;
                    }
                    self.assist = Some(assist);
                    self.flight_state = FlightState::AssistedTakeoff { body };
                }
            }
            FlightState::AssistedLanding { .. }
            | FlightState::AssistedTakeoff { .. }
            | FlightState::Bouncing { .. } => {}
        }
    }

    fn start_landing(&mut self, body: &CelestialBody, recovering: bool) {
        let mut assist = AssistTransition::new(self.pose, body, true);
        if self.terrain != salimon_world::terrain::TerrainMode::Flat {
            let Some(footprint) = &self.landing_footprint else {
                return;
            };
            let pose = if recovering {
                crate::terrain_landing::nearby_landing_pose(
                    *body,
                    assist.surface_normal,
                    self.terrain,
                    footprint,
                )
            } else {
                crate::terrain_landing::landing_pose(
                    *body,
                    assist.surface_normal,
                    self.pose.axes()[0],
                    self.terrain,
                    footprint,
                )
            };
            let Some(pose) = pose else {
                self.cockpit_message = Some(CockpitMessage::UnsafeLandingSite);
                return;
            };
            assist.destination_distance =
                vector_length(sub(pose.position_meters, body.center.meters()));
            assist.aligned_orientation = pose.orientation;
            assist.landing_pose = Some(pose);
        }
        if recovering {
            let start_normal = assist.surface_normal;
            // A spherical shell above the maximum terrain and conservative ship
            // radius keeps the unchanged crash orientation clear during relocation.
            let clearance_distance = (body.radius_meters
                + self.terrain.elevation_bounds(*body)[1]
                + COLLISION_RADIUS_METERS
                + 15.0)
                .max(assist.start_distance);
            assist.recovery = Some(RecoveryPath {
                start_normal,
                start_distance: assist.start_distance,
                clearance_distance,
            });
            if let Some(pose) = assist.landing_pose {
                assist.surface_normal = normalize_or(
                    sub(pose.position_meters, body.center.meters()),
                    start_normal,
                );
            }
            assist.start_distance = clearance_distance;
            assist.duration = RECOVERY_DURATION;
        }
        assist.intermediate_distance = assist.destination_distance
            + ((assist.start_distance - assist.destination_distance).max(0.0) * 0.25).min(15.0);
        let duration = assist.duration;
        if !self.reserve_assist_energy(&mut assist, duration) {
            return;
        }
        self.assist = Some(assist);
        self.bounce = None;
        self.velocity = [0.0; 3];
        self.thruster_percentage = 0;
        self.steering_target = SteeringInput::default();
        self.steering = SteeringInput::default();
        self.flight_state = FlightState::AssistedLanding { body: body.id };
    }

    pub(super) fn active_assist_watts(&self) -> f64 {
        self.assist
            .as_ref()
            .map_or(0.0, |assist| assist.energy_watts)
    }

    fn reserve_assist_energy(&mut self, assist: &mut AssistTransition, duration: Duration) -> bool {
        // Captured kinematic paths use a conservative full forward-power estimate.
        // Idle consumers stay on the ordinary bus and cannot spend this budget.
        assist.energy_watts = self.energy_core.config().forward_watts;
        assist.reservation = self
            .energy_core
            .reserve_assist(assist.energy_watts * duration.as_secs_f64());
        if assist.reservation.is_none() {
            self.cockpit_message = Some(CockpitMessage::InsufficientAssistEnergy);
            return false;
        }
        true
    }

    pub(super) fn settle_assist_energy(&mut self, delta: Duration) {
        let Some(assist) = self.assist.as_mut() else {
            return;
        };
        let remaining = assist.duration.saturating_sub(assist.elapsed);
        if let Some(budget) = assist.reservation.as_mut() {
            budget.spend(assist.energy_watts * delta.min(remaining).as_secs_f64());
        }
    }

    fn finish_assist_energy(&mut self) {
        if let Some(budget) = self
            .assist
            .as_mut()
            .and_then(|assist| assist.reservation.take())
        {
            self.energy_core.refund_assist(budget);
        }
    }

    pub(super) fn landing_target(&self) -> Option<&'static CelestialBody> {
        if self.speed_meters_per_second()
            > self.assist_config.maximum_landing_speed_meters_per_second
        {
            return None;
        }
        self.terrain.bodies().iter().find(|body| {
            body.role == BodyRole::Solid
                && vector_length(sub(self.pose.position_meters, body.center.meters()))
                    <= body.radius_meters
                        + body.radius_meters * self.assist_config.landing_range_radius_fraction
        })
    }

    pub(super) fn advance_landing(&mut self, body_id: CelestialBodyId, delta: Duration) {
        let assist = self
            .assist
            .as_mut()
            .expect("landing captures an assist path");
        assist.elapsed = assist.elapsed.saturating_add(delta).min(assist.duration);
        let elapsed = assist.elapsed.as_secs_f64();
        if let Some(recovery) = &assist.recovery {
            if elapsed <= RECOVERY_LIFT_SECONDS {
                let (distance, speed) = eased_radial_segment(
                    recovery.start_distance,
                    recovery.clearance_distance,
                    elapsed,
                    RECOVERY_LIFT_SECONDS,
                );
                self.pose.position_meters = add(
                    body_definition(body_id).center.meters(),
                    scale(recovery.start_normal, distance),
                );
                assist.velocity_meters_per_second = scale(recovery.start_normal, speed);
                return;
            }
            if elapsed <= RECOVERY_LIFT_SECONDS + RECOVERY_REPOSITION_SECONDS {
                let amount = ((elapsed - RECOVERY_LIFT_SECONDS) / RECOVERY_REPOSITION_SECONDS)
                    .clamp(0.0, 1.0);
                let difference = sub(assist.surface_normal, recovery.start_normal);
                let direction = add(recovery.start_normal, scale(difference, smoothstep(amount)));
                let normal = normalize_or(direction, assist.surface_normal);
                let derivative = scale(
                    sub(
                        difference,
                        scale(normal, salimon_math::dot(normal, difference)),
                    ),
                    6.0 * amount * (1.0 - amount)
                        / (RECOVERY_REPOSITION_SECONDS * vector_length(direction)),
                );
                self.pose.position_meters = add(
                    body_definition(body_id).center.meters(),
                    scale(normal, recovery.clearance_distance),
                );
                assist.velocity_meters_per_second = scale(derivative, recovery.clearance_distance);
                return;
            }
        }
        let seconds = elapsed
            - if assist.recovery.is_some() {
                RECOVERY_LIFT_SECONDS + RECOVERY_REPOSITION_SECONDS
            } else {
                0.0
            };
        let alignment = smoothstep(seconds / LANDING_ALIGNMENT_SECONDS);
        self.pose.orientation = quaternion_slerp(
            assist.start_pose.orientation,
            assist.aligned_orientation,
            alignment,
        );
        // Hold the exact starting position while leveling the hull.
        if seconds <= LANDING_ALIGNMENT_SECONDS {
            if assist.recovery.is_some() {
                self.pose.position_meters = add(
                    body_definition(body_id).center.meters(),
                    scale(assist.surface_normal, assist.start_distance),
                );
            }
            assist.velocity_meters_per_second = [0.0; 3];
            return;
        }
        let approach_elapsed = seconds - LANDING_ALIGNMENT_SECONDS;
        let (distance, speed) = if approach_elapsed <= LANDING_APPROACH_SECONDS {
            eased_radial_segment(
                assist.start_distance,
                assist.intermediate_distance,
                approach_elapsed,
                LANDING_APPROACH_SECONDS,
            )
        } else {
            eased_radial_segment(
                assist.intermediate_distance,
                assist.destination_distance,
                approach_elapsed - LANDING_APPROACH_SECONDS,
                LANDING_TOUCHDOWN_SECONDS,
            )
        };
        assist.velocity_meters_per_second = scale(assist.surface_normal, speed);
        self.pose.position_meters = add(
            body_definition(body_id).center.meters(),
            scale(assist.surface_normal, distance),
        );
        if assist.elapsed == assist.duration {
            if let Some(pose) = assist.landing_pose {
                self.pose = pose;
            }
            self.velocity = [0.0; 3];
            self.flight_state = FlightState::Landed { body: body_id };
            self.thruster_percentage = 0;
            self.finish_assist_energy();
            self.assist = None;
        }
    }

    pub(super) fn advance_takeoff(&mut self, body_id: CelestialBodyId, delta: Duration) {
        if delta.is_zero() {
            return;
        }
        let assist = self
            .assist
            .as_mut()
            .expect("takeoff captures an assist path");
        assist.elapsed = assist
            .elapsed
            .saturating_add(delta)
            .min(TAKEOFF_ASSIST_DURATION);
        let seconds = assist.elapsed.as_secs_f64();
        let (distance, speed) = if seconds <= TAKEOFF_LIFT_SECONDS {
            eased_radial_segment(
                assist.start_distance,
                assist.intermediate_distance,
                seconds,
                TAKEOFF_LIFT_SECONDS,
            )
        } else {
            eased_radial_segment(
                assist.intermediate_distance,
                assist.destination_distance,
                seconds - TAKEOFF_LIFT_SECONDS,
                TAKEOFF_CLEARANCE_SECONDS,
            )
        };
        assist.velocity_meters_per_second = scale(assist.surface_normal, speed);
        self.pose.position_meters = add(
            body_definition(body_id).center.meters(),
            scale(assist.surface_normal, distance),
        );
        if assist.elapsed == TAKEOFF_ASSIST_DURATION {
            self.velocity = assist.velocity_meters_per_second;
            self.thruster_percentage = 0;
            self.flight_state = FlightState::Flying;
            self.finish_assist_energy();
            self.assist = None;
        }
    }
}

impl AssistTransition {
    pub(super) fn validate_save(&self) -> Result<(), salimon_persistence::SaveError> {
        use salimon_persistence::invalid;
        if self.duration.is_zero()
            || self.elapsed > self.duration
            || !self.energy_watts.is_finite()
            || self.energy_watts < 0.
            || [
                self.start_distance,
                self.intermediate_distance,
                self.destination_distance,
            ]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.)
            || self
                .surface_normal
                .iter()
                .chain(self.velocity_meters_per_second.iter())
                .any(|v| !v.is_finite())
        {
            return Err(invalid("assist path"));
        }
        for pose in [Some(self.start_pose), self.landing_pose]
            .into_iter()
            .flatten()
        {
            let n: f64 = pose.orientation.iter().map(|v| v * v).sum();
            if pose.position_meters.iter().any(|v| !v.is_finite())
                || !n.is_finite()
                || (n - 1.).abs() > 1e-9
            {
                return Err(invalid("assist pose"));
            }
        }
        let n: f64 = self.aligned_orientation.iter().map(|v| v * v).sum();
        if !n.is_finite() || (n - 1.).abs() > 1e-9 {
            return Err(invalid("assist aligned orientation"));
        }
        if let Some(r) = &self.reservation {
            r.validate_save()?;
        }
        if let Some(r) = &self.recovery
            && (r.start_normal.iter().any(|v| !v.is_finite())
                || !r.start_distance.is_finite()
                || !r.clearance_distance.is_finite()
                || r.start_distance < 0.
                || r.clearance_distance < 0.)
        {
            return Err(invalid("recovery path"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orientation::axis_angle_quaternion;
    use salimon_math::dot;

    #[test]
    fn reservation_rejection_and_bus_blackout_do_not_restart_or_cancel_assist() {
        let mut ship = ShipController::default();
        ship.set_cockpit_control(true);
        ship.set_stored_core_energy_joules(599_999_999);
        let before = ship.snapshot();
        ship.trigger_landing_action();
        assert_eq!(ship.snapshot().flight_state, before.flight_state);
        assert_eq!(ship.snapshot().energy_core, before.energy_core);
        assert_eq!(
            ship.snapshot().cockpit_message,
            Some(CockpitMessage::InsufficientAssistEnergy)
        );
        ship.set_stored_core_energy_joules(600_000_000);
        ship.trigger_landing_action();
        assert_eq!(ship.snapshot().energy_core.stored_joules, 0);
        assert!(!ship.snapshot().power_available);
        ship.trigger_landing_action();
        ship.set_cockpit_control(false);
        ship.advance(TAKEOFF_ASSIST_DURATION);
        assert_eq!(ship.snapshot().flight_state, FlightState::Flying);
        assert_eq!(ship.snapshot().energy_core.stored_joules, 0);
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

    fn approaching_ship(body: &CelestialBody, normal: [f64; 3], altitude: f64) -> ShipController {
        ShipController::flying(
            ShipPose {
                position_meters: add(
                    body.center.meters(),
                    scale(normal, body.radius_meters + altitude),
                ),
                orientation: axis_angle_quaternion([0.0, 0.0, 1.0], 1.2),
            },
            100,
        )
    }

    fn advance_for(ship: &mut ShipController, duration: Duration, step: Duration) {
        let mut remaining = duration;
        while !remaining.is_zero() {
            let delta = remaining.min(step);
            ship.advance(delta);
            remaining -= delta;
        }
    }

    #[test]
    fn assists_have_readable_phases_and_exact_endpoints_on_every_solid_body() {
        let normals = [
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            normalize_or([0.31, 0.72, -0.62], [0.0, 1.0, 0.0]),
        ];
        for body in CELESTIAL_BODIES
            .iter()
            .filter(|body| body.role == BodyRole::Solid)
        {
            for normal in normals {
                for altitude in [COLLISION_RADIUS_METERS, body.radius_meters * 0.149] {
                    let mut ship = approaching_ship(body, normal, altitude);
                    let start = ship.snapshot().pose;
                    ship.trigger_landing_action();
                    assert_eq!(
                        ship.snapshot().pose,
                        start,
                        "activation cannot snap the pose"
                    );
                    assert_eq!(ship.velocity_meters_per_second(), [0.0; 3]);
                    ship.advance(Duration::from_secs(1));
                    let halfway = ship.snapshot().pose;
                    assert_eq!(halfway.position_meters, start.position_meters);
                    assert_ne!(halfway.orientation, start.orientation);
                    assert!(dot(halfway.axes()[1], normal) < 1.0 - 1.0e-6);
                    ship.advance(Duration::from_secs(1));
                    assert!(dot(ship.snapshot().pose.axes()[1], normal) > 1.0 - 1.0e-10);
                    assert_eq!(ship.snapshot().pose.position_meters, start.position_meters);

                    let destination = body.radius_meters + LANDED_ORIGIN_HEIGHT_METERS;
                    let mut previous_distance = body.radius_meters + altitude;
                    for _ in 0..599 {
                        ship.advance(Duration::from_millis(10));
                        let snapshot = ship.snapshot();
                        let offset = sub(snapshot.pose.position_meters, body.center.meters());
                        let distance = vector_length(offset);
                        assert!(distance <= previous_distance + 1.0e-3);
                        assert!(distance >= destination - 1.0e-3);
                        assert!(dot(normalize_or(offset, normal), normal) > 1.0 - 1.0e-10);
                        assert!(dot(snapshot.velocity_meters_per_second, normal) <= 0.0);
                        assert_eq!(
                            snapshot.flight_state,
                            FlightState::AssistedLanding { body: body.id }
                        );
                        previous_distance = distance;
                    }
                    ship.advance(Duration::from_millis(10));
                    let landed = ship.snapshot();
                    assert_eq!(landed.flight_state, FlightState::Landed { body: body.id });
                    assert_eq!(landed.thruster_percentage, 0);
                    assert_eq!(landed.velocity_meters_per_second, [0.0; 3]);
                    assert!(
                        (vector_length(sub(landed.pose.position_meters, body.center.meters()))
                            - destination)
                            .abs()
                            < 1.0e-3
                    );

                    let height =
                        vector_length(sub(landed.pose.position_meters, body.center.meters()))
                            - body.radius_meters;
                    assert!(
                        (height + crate::LOWEST_LOCAL_Y_METERS
                            - crate::LANDED_HULL_CLEARANCE_METERS)
                            .abs()
                            < 1e-3
                    );
                    assert!((height + crate::LANDING_FOOT_LOCAL_Y_METERS).abs() < 1e-3);
                    // Feet at the four authored support positions are tangent to the
                    // nominal sphere; catalog curvature/anchor quantization is <1 mm.
                    for x in [-5.6, 4.8] {
                        for z in [-3.6, 3.6] {
                            let axes = landed.pose.axes();
                            let foot = add(
                                landed.pose.position_meters,
                                add(
                                    scale(axes[0], x),
                                    add(
                                        scale(axes[1], crate::LANDING_FOOT_LOCAL_Y_METERS),
                                        scale(axes[2], z),
                                    ),
                                ),
                            );
                            assert!(
                                (vector_length(sub(foot, body.center.meters()))
                                    - body.radius_meters)
                                    .abs()
                                    < 1e-3
                            );
                        }
                    }
                    ship.trigger_landing_action();
                    assert_eq!(ship.snapshot().pose, landed.pose);
                    ship.advance(Duration::from_secs(1));
                    let lifted = ship.snapshot();
                    let lift = vector_length(sub(
                        lifted.pose.position_meters,
                        landed.pose.position_meters,
                    ));
                    assert!(
                        (lift - 5.0).abs() < 1.0e-3,
                        "the first second must show local lift"
                    );
                    assert_eq!(lifted.pose.orientation, landed.pose.orientation);
                    let clearance = body.radius_meters + LANDED_ORIGIN_HEIGHT_METERS + 50.0;
                    for _ in 0..499 {
                        ship.advance(Duration::from_millis(10));
                        let snapshot = ship.snapshot();
                        let distance =
                            vector_length(sub(snapshot.pose.position_meters, body.center.meters()));
                        assert!(distance >= previous_distance - 1.0e-3);
                        assert!(distance <= clearance + 1.0e-3);
                        assert_eq!(
                            snapshot.flight_state,
                            FlightState::AssistedTakeoff { body: body.id }
                        );
                        assert!(dot(snapshot.velocity_meters_per_second, normal) >= 0.0);
                        previous_distance = distance;
                    }
                    ship.advance(Duration::from_millis(10));
                    let flying = ship.snapshot();
                    assert_eq!(flying.flight_state, FlightState::Flying);
                    assert_eq!(flying.velocity_meters_per_second, [0.0; 3]);
                    assert_eq!(flying.thruster_percentage, 0);
                    assert!(
                        (vector_length(sub(flying.pose.position_meters, body.center.meters()))
                            - clearance)
                            .abs()
                            < 1.0e-3
                    );
                    assert_eq!(
                        ship.contextual_cockpit_message(),
                        Some(CockpitMessage::PressToLand)
                    );
                }
            }
        }
    }

    #[test]
    fn assist_timing_and_pose_are_independent_of_frame_partition_and_large_deltas() {
        let body = body_definition(CelestialBodyId::Earth);
        let mut landing = approaching_ship(body, [0.0, 1.0, 0.0], body.radius_meters * 0.1);
        landing.trigger_landing_action();
        let mut takeoff = ShipController::default();
        takeoff.set_cockpit_control(true);
        takeoff.trigger_landing_action();
        for (starting, duration) in [
            (landing, LANDING_ASSIST_DURATION),
            (takeoff, TAKEOFF_ASSIST_DURATION),
        ] {
            for elapsed in [
                Duration::ZERO,
                Duration::from_millis(1375),
                Duration::from_millis(5123),
                duration - Duration::from_nanos(1),
                duration,
                duration + Duration::from_secs(10),
            ] {
                let mut reference = starting.clone();
                reference.advance(elapsed);
                for step in [
                    Duration::from_millis(16),
                    Duration::from_millis(33),
                    Duration::from_millis(250),
                    Duration::from_millis(700),
                ] {
                    let mut partitioned = starting.clone();
                    advance_for(&mut partitioned, elapsed, step);
                    if elapsed <= duration {
                        assert_eq!(partitioned.snapshot(), reference.snapshot());
                    } else {
                        // Inertial continuation adds world-space f64 positions
                        // at a 1e12 m anchor: up to 625 additions can accumulate
                        // less than 0.1 m of coordinate quantization.
                        let mut observed = partitioned.snapshot();
                        let expected = reference.snapshot();
                        assert!(
                            vector_length(sub(
                                observed.pose.position_meters,
                                expected.pose.position_meters
                            )) < 0.1
                        );
                        observed.pose.position_meters = expected.pose.position_meters;
                        assert_eq!(observed, expected);
                    }
                }
                if elapsed < duration {
                    assert_eq!(
                        reference.snapshot().flight_state,
                        starting.snapshot().flight_state
                    );
                } else {
                    assert!(matches!(
                        reference.snapshot().flight_state,
                        FlightState::Landed { .. } | FlightState::Flying
                    ));
                }
            }
        }
    }

    #[test]
    fn assisted_velocity_matches_path_derivative_and_settles_at_phase_boundaries() {
        let body = body_definition(CelestialBodyId::Earth);
        let mut landing = approaching_ship(body, [0.0, 1.0, 0.0], body.radius_meters * 0.1);
        landing.trigger_landing_action();
        let mut takeoff = ShipController::default();
        takeoff.set_cockpit_control(true);
        takeoff.trigger_landing_action();
        for (starting, phases, samples) in [
            (landing, [0, 2, 6, 8], [0.7, 2.1, 3.2, 6.1, 7.4]),
            (takeoff, [0, 2, 6, 6], [0.7, 1.9, 2.1, 3.2, 5.4]),
        ] {
            for seconds in phases {
                let mut ship = starting.clone();
                ship.advance(Duration::from_secs(seconds));
                assert_eq!(ship.velocity_meters_per_second(), [0.0; 3]);
            }
            for seconds in samples {
                let epsilon = Duration::from_millis(1);
                let mut before = starting.clone();
                before.advance(Duration::from_secs_f64(seconds) - epsilon);
                let mut at = before.clone();
                at.advance(epsilon);
                let mut after = at.clone();
                after.advance(epsilon);
                let measured = scale(
                    sub(after.pose.position_meters, before.pose.position_meters),
                    (2.0 * epsilon.as_secs_f64()).recip(),
                );
                let velocity = at.velocity_meters_per_second();
                assert!(
                    vector_length(sub(measured, velocity)) < 0.2,
                    "t={seconds}: measured {measured:?}, reported {velocity:?}"
                );
                assert!(
                    (at.snapshot()
                        .nearby_body
                        .unwrap()
                        .radial_speed_meters_per_second
                        - velocity[1])
                        .abs()
                        < 1.0e-6
                );
            }
        }
    }

    #[test]
    fn repeated_actions_and_manual_input_cannot_restart_or_override_assists() {
        let body = body_definition(CelestialBodyId::Earth);
        let mut landing = approaching_ship(body, [0.0, 1.0, 0.0], 100.0);
        landing.trigger_landing_action();
        let mut takeoff = ShipController::default();
        takeoff.trigger_landing_action();
        assert!(matches!(
            takeoff.snapshot().flight_state,
            FlightState::Landed { .. }
        ));
        takeoff.set_cockpit_control(true);
        takeoff.trigger_landing_action();
        for (mut ship, duration) in [
            (landing, LANDING_ASSIST_DURATION),
            (takeoff, TAKEOFF_ASSIST_DURATION),
        ] {
            let mut reference = ship.clone();
            ship.advance(Duration::from_secs(1));
            ship.trigger_landing_action();
            ship.adjust_thruster(-20);
            ship.set_steering_input(SteeringInput {
                pitch: 1.0,
                yaw: -1.0,
                roll: 1.0,
            });
            ship.toggle_door();
            assert_eq!(ship.snapshot().door_state, DoorState::Closed);
            ship.clear_cockpit_message();
            ship.set_cockpit_control(false);
            ship.advance(duration - Duration::from_secs(1));
            reference.set_cockpit_control(false);
            reference.advance(duration);
            assert_eq!(ship.snapshot(), reference.snapshot());
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
}

#[cfg(test)]
mod safety_tests {
    use super::*;
    use crate::orientation::axis_angle_quaternion;
    use salimon_math::{dot, length};

    #[test]
    fn full_speed_including_tangential_motion_range_and_authority_gate_landing() {
        let body = body_definition(CelestialBodyId::Earth);
        let pose = ShipPose {
            position_meters: add(body.center.meters(), [0.0, body.radius_meters * 1.15, 0.0]),
            // Deliberately point sideways: heading never gates activation.
            orientation: axis_angle_quaternion([0.0, 0.0, 1.0], 1.3),
        };
        for velocity in [[20.0, 0.0, 0.0], [0.0, -20.0, 0.0], [12.0, 16.0, 0.0]] {
            let mut ship = ShipController::flying_with_velocity(pose, 100, velocity).unwrap();
            ship.trigger_landing_action();
            assert!(matches!(
                ship.flight_state,
                FlightState::AssistedLanding { .. }
            ));
        }
        for velocity in [[20.001, 0.0, 0.0], [0.0, -20.001, 0.0], [15.0, 15.0, 0.0]] {
            let mut ship = ShipController::flying_with_velocity(pose, 0, velocity).unwrap();
            let before = ship.snapshot();
            ship.trigger_landing_action();
            assert_eq!(ship.snapshot(), before);
            assert_eq!(ship.contextual_cockpit_message(), None);
        }
        let mut ship = ShipController::flying(pose, 0);
        ship.set_cockpit_control(false);
        ship.trigger_landing_action();
        assert_eq!(ship.flight_state, FlightState::Flying);
        ship.set_cockpit_control(true);
        ship.pose.position_meters[1] += 0.01;
        ship.trigger_landing_action();
        assert_eq!(ship.flight_state, FlightState::Flying);
        ship.configure_assistance(AssistConfig {
            landing_range_radius_fraction: 0.2,
            ..Default::default()
        })
        .unwrap();
        ship.trigger_landing_action();
        assert!(matches!(
            ship.flight_state,
            FlightState::AssistedLanding { .. }
        ));
        assert!(
            ship.configure_assistance(AssistConfig {
                maximum_landing_speed_meters_per_second: f64::NAN,
                ..Default::default()
            })
            .is_err()
        );
    }

    #[test]
    fn recovery_is_powered_reserved_uncancellable_and_safe_on_all_solids() {
        for body in CELESTIAL_BODIES
            .iter()
            .filter(|b| b.role == BodyRole::Solid)
        {
            let normal = normalize_or([0.31, 0.72, -0.62], [0.0, 1.0, 0.0]);
            let pose = ShipPose {
                position_meters: add(
                    body.center.meters(),
                    scale(normal, body.radius_meters + COLLISION_RADIUS_METERS),
                ),
                orientation: axis_angle_quaternion([1.0, 0.0, 0.0], 1.7),
            };
            let mut ship = ShipController::flying(pose, 0);
            ship.flight_state = FlightState::GroundedAfterCrash { body: body.id };
            ship.set_stored_core_energy_joules(0);
            ship.trigger_landing_action();
            assert_eq!(ship.pose, pose);
            assert!(matches!(
                ship.flight_state,
                FlightState::GroundedAfterCrash { .. }
            ));
            ship.set_stored_core_energy_joules(1_199_999_999);
            ship.trigger_landing_action();
            assert!(matches!(
                ship.flight_state,
                FlightState::GroundedAfterCrash { .. }
            ));
            assert_eq!(ship.snapshot().energy_core.stored_joules, 1_199_999_999);
            ship.set_stored_core_energy_joules(1_200_000_000);
            ship.trigger_landing_action();
            assert_eq!(ship.snapshot().energy_core.stored_joules, 0);
            assert_eq!(ship.pose, pose);
            assert!(matches!(
                ship.flight_state,
                FlightState::AssistedLanding { .. }
            ));
            let mut whole = ship.clone();
            whole.advance(RECOVERY_DURATION);
            ship.trigger_landing_action();
            ship.set_cockpit_control(false);
            for _ in 0..400 {
                ship.advance(Duration::from_millis(10));
                assert_eq!(ship.pose.orientation, pose.orientation);
                assert!(
                    length(sub(ship.pose.position_meters, body.center.meters()))
                        >= body.radius_meters + COLLISION_RADIUS_METERS - 0.001
                );
            }
            for _ in 0..800 {
                ship.advance(Duration::from_millis(10));
            }
            assert_eq!(ship.pose, whole.pose);
            assert_eq!(ship.flight_state, FlightState::Landed { body: body.id });
            assert_eq!(ship.velocity_meters_per_second(), [0.0; 3]);
            assert_eq!(ship.thruster_percentage, 0);
            assert!(dot(ship.pose.axes()[1], normal) > 1.0 - 1e-10);
            assert!(ship.assist.is_none());
        }
    }

    #[test]
    fn takeoff_hands_velocity_to_inertial_flight_and_consumes_leftover_time() {
        let mut ship = ShipController::default();
        ship.set_cockpit_control(true);
        ship.trigger_landing_action();
        let mut whole = ship.clone();
        ship.advance(TAKEOFF_ASSIST_DURATION);
        let finish = ship.snapshot();
        assert_eq!(finish.speed_meters_per_second, 0.0);
        assert_eq!(finish.thruster_percentage, 0);
        ship.advance(Duration::from_secs(1));
        let expected = add(
            finish.pose.position_meters,
            finish.velocity_meters_per_second,
        );
        assert!(length(sub(ship.pose.position_meters, expected)) < 0.001);
        whole.advance(TAKEOFF_ASSIST_DURATION + Duration::from_secs(1));
        assert_eq!(whole.snapshot(), ship.snapshot());
    }
    #[test]
    #[ignore = "opt-in deterministic takeoff trace; no native hardware claim"]
    fn takeoff_trace() {
        eprintln!(
            "body,mass_kg,t_seconds,altitude_m,speed_mps,throttle,stored_joules,pose_m,velocity_mps"
        );
        for body in CELESTIAL_BODIES
            .iter()
            .filter(|b| b.role == BodyRole::Solid)
        {
            for mass in [100_000.0, 110_000.0] {
                let mut ship = ShipController::flying(
                    ShipPose {
                        position_meters: body
                            .center
                            .translated([0.0, body.radius_meters + 100.0, 0.0])
                            .meters(),
                        orientation: [0.0, 0.0, 0.0, 1.0],
                    },
                    0,
                );
                ship.set_total_mass_kg(mass).unwrap();
                ship.set_cockpit_control(true);
                ship.trigger_landing_action();
                ship.advance(LANDING_ASSIST_DURATION);
                assert!(matches!(
                    ship.snapshot().flight_state,
                    FlightState::Landed { .. }
                ));
                for seconds in -1..=7 {
                    if seconds == 0 {
                        ship.trigger_landing_action();
                    }
                    if seconds > 0 {
                        ship.advance(Duration::from_secs(1));
                    }
                    if seconds == 3 {
                        assert!(
                            !ship.adjust_thruster(100),
                            "assist captures flight controls"
                        );
                    }
                    let snapshot = ship.snapshot();
                    eprintln!(
                        "{},{mass},{seconds},{:.6},{:.6},{},{},{:?},{:?}",
                        body.name,
                        vector_length(sub(snapshot.pose.position_meters, body.center.meters()))
                            - body.radius_meters,
                        snapshot.speed_meters_per_second,
                        snapshot.thruster_percentage,
                        snapshot.energy_core.stored_joules,
                        snapshot.pose.position_meters,
                        snapshot.velocity_meters_per_second
                    );
                }
            }
        }
    }
    #[test]
    fn local_takeoff_bounds_speed_acceleration_mass_and_reserved_energy_on_all_bodies() {
        for body in CELESTIAL_BODIES
            .iter()
            .filter(|b| b.role == BodyRole::Solid)
        {
            for mass in [100_000.0, 110_000.0, 1_000_000.0] {
                let mut ship = ShipController::flying(
                    ShipPose {
                        position_meters: body
                            .center
                            .translated([0.0, body.radius_meters + 100.0, 0.0])
                            .meters(),
                        orientation: [0.0, 0.0, 0.0, 1.0],
                    },
                    0,
                );
                ship.set_total_mass_kg(mass).unwrap();
                ship.trigger_landing_action();
                ship.advance(LANDING_ASSIST_DURATION);
                let initial = ship.snapshot();
                assert!(matches!(initial.flight_state, FlightState::Landed { .. }));
                // Landing eligibility range must not become a takeoff destination.
                ship.configure_assistance(AssistConfig {
                    landing_range_radius_fraction: 0.9,
                    ..Default::default()
                })
                .unwrap();
                ship.trigger_landing_action();
                assert_eq!(ship.snapshot().pose, initial.pose);
                assert_eq!(
                    initial.energy_core.stored_joules - ship.snapshot().energy_core.stored_joules,
                    600_000_000
                );
                let mut velocity = [0.0; 3];
                let mut peak = 0.0_f64;
                for _ in 0..600 {
                    assert!(
                        !ship.adjust_thruster(100),
                        "captured control cannot change the path"
                    );
                    ship.advance(Duration::from_millis(10));
                    let snapshot = ship.snapshot();
                    let acceleration =
                        vector_length(sub(snapshot.velocity_meters_per_second, velocity)) / 0.01;
                    assert!(
                        acceleration <= 15.000001,
                        "bounded local acceleration: {acceleration}"
                    );
                    assert!(snapshot.speed_meters_per_second <= 15.000001);
                    assert!(
                        vector_length(sub(
                            snapshot.pose.position_meters,
                            initial.pose.position_meters
                        )) <= 50.001
                    );
                    velocity = snapshot.velocity_meters_per_second;
                    peak = peak.max(snapshot.speed_meters_per_second);
                }
                let end = ship.snapshot();
                assert_eq!(end.flight_state, FlightState::Flying);
                assert_eq!(end.velocity_meters_per_second, [0.0; 3]);
                assert_eq!(end.total_mass_kg, mass);
                assert!((peak - 15.0).abs() < 0.000001);
                assert!(
                    (vector_length(sub(end.pose.position_meters, initial.pose.position_meters))
                        - 50.0)
                        .abs()
                        < 0.001
                );
                assert_eq!(
                    initial.energy_core.stored_joules - end.energy_core.stored_joules,
                    600_660_000
                );
                ship.advance(Duration::from_secs(1));
                assert_eq!(
                    ship.snapshot().pose,
                    end.pose,
                    "no hidden momentum at handoff"
                );
            }
        }
    }

    #[test]
    fn powered_flight_after_takeoff_obeys_dry_and_realistic_cargo_force_and_power_budgets() {
        for mass in [100_000.0, 110_000.0, 200_000.0] {
            let mut ship = ShipController::default();
            ship.set_total_mass_kg(mass).unwrap();
            ship.set_cockpit_control(true);
            ship.trigger_landing_action();
            ship.advance(TAKEOFF_ASSIST_DURATION);
            let before = ship.snapshot();
            assert_eq!(before.speed_meters_per_second, 0.0);
            assert!(ship.adjust_thruster(100));
            ship.advance(Duration::from_secs(1));
            let powered = ship.snapshot();
            assert!((powered.speed_meters_per_second - 100_000_000.0 / mass).abs() < 1e-9);
            assert_eq!(
                before.energy_core.stored_joules - powered.energy_core.stored_joules,
                100_110_000
            );
            assert!(ship.adjust_thruster(-100));
            assert!(ship.adjust_thruster(-100));
            ship.advance(Duration::from_secs(2));
            let stopped = ship.snapshot();
            assert!(stopped.speed_meters_per_second < 1e-8);
            assert_eq!(
                powered.energy_core.stored_joules - stopped.energy_core.stored_joules,
                100_220_000
            );
        }
    }
}
