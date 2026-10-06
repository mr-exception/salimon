//! Captured landing/takeoff paths and uncancellable timing.

use super::ShipController;
use crate::orientation::{normalize_or, quaternion_slerp, surface_aligned_orientation};
use crate::{
    COLLISION_RADIUS_METERS, CockpitMessage, DoorState, FlightState, LANDING_ASSIST_DURATION,
    LANDING_RANGE_RADIUS_FRACTION, LOWEST_LOCAL_Y_METERS, ShipPose, SteeringInput,
    TAKEOFF_ASSIST_DURATION,
};
use salimon_math::{add, length as vector_length, scale, sub};
use salimon_world::{BodyRole, CELESTIAL_BODIES, CelestialBody, CelestialBodyId};
use std::time::Duration;

const LANDING_ALIGNMENT_SECONDS: f64 = 2.0;
const LANDING_APPROACH_SECONDS: f64 = 4.0;
const LANDING_TOUCHDOWN_SECONDS: f64 = 2.0;
const TAKEOFF_LIFT_SECONDS: f64 = 2.0;
const TAKEOFF_CLEARANCE_SECONDS: f64 = 4.0;
/// Captured path data prevents accumulated position or frame-rate-dependent timing.
#[derive(Clone, Debug)]
pub(super) struct AssistTransition {
    elapsed: Duration,
    start_pose: ShipPose,
    pub(super) surface_normal: [f64; 3],
    aligned_orientation: [f64; 4],
    start_distance: f64,
    intermediate_distance: f64,
    destination_distance: f64,
    pub(super) radial_speed_meters_per_second: f64,
}

impl AssistTransition {
    fn new(pose: ShipPose, body: &CelestialBody, landing: bool) -> Self {
        let offset = sub(pose.position_meters, body.center.meters());
        let start_distance = vector_length(offset);
        let surface_normal = normalize_or(offset, pose.axes()[1]);
        let destination_distance = if landing {
            body.radius_meters - LOWEST_LOCAL_Y_METERS
        } else {
            body.radius_meters * (1.0 + LANDING_RANGE_RADIUS_FRACTION) + COLLISION_RADIUS_METERS
        };
        let intermediate_distance = if landing {
            // Reserve visible local motion for touchdown even on the largest body.
            destination_distance
                + ((start_distance - destination_distance).max(0.0) * 0.25).min(15.0)
        } else {
            (start_distance + 15.0).min(destination_distance)
        };
        Self {
            elapsed: Duration::ZERO,
            start_pose: pose,
            surface_normal,
            aligned_orientation: surface_aligned_orientation(surface_normal, pose.axes()[0]),
            start_distance,
            intermediate_distance,
            destination_distance,
            radial_speed_meters_per_second: 0.0,
        }
    }
}

pub(super) fn body_definition(id: CelestialBodyId) -> &'static CelestialBody {
    CELESTIAL_BODIES
        .iter()
        .find(|body| body.id == id)
        .expect("flight state only stores catalog body identifiers")
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
                    self.assist = Some(AssistTransition::new(self.pose, body, true));
                    self.steering_target = SteeringInput::default();
                    self.steering = SteeringInput::default();
                    self.flight_state = FlightState::AssistedLanding { body: body.id };
                }
            }
            FlightState::Landed { body } => {
                if self.door_state == DoorState::Open || self.door_open_fraction > 0.0 {
                    self.cockpit_message = Some(CockpitMessage::CloseDoorBeforeTakeoff);
                } else {
                    self.assist = Some(AssistTransition::new(
                        self.pose,
                        body_definition(body),
                        false,
                    ));
                    self.flight_state = FlightState::AssistedTakeoff { body };
                }
            }
            FlightState::AssistedLanding { .. } | FlightState::AssistedTakeoff { .. } => {}
        }
    }

    pub(super) fn landing_target(&self) -> Option<&'static CelestialBody> {
        CELESTIAL_BODIES.iter().find(|body| {
            body.role == BodyRole::Solid
                && vector_length(sub(self.pose.position_meters, body.center.meters()))
                    <= body.radius_meters * (1.0 + LANDING_RANGE_RADIUS_FRACTION)
        })
    }

    pub(super) fn advance_landing(&mut self, body_id: CelestialBodyId, delta: Duration) {
        let assist = self
            .assist
            .as_mut()
            .expect("landing captures an assist path");
        assist.elapsed = assist
            .elapsed
            .saturating_add(delta)
            .min(LANDING_ASSIST_DURATION);
        let seconds = assist.elapsed.as_secs_f64();
        let alignment = smoothstep(seconds / LANDING_ALIGNMENT_SECONDS);
        self.pose.orientation = quaternion_slerp(
            assist.start_pose.orientation,
            assist.aligned_orientation,
            alignment,
        );
        // Hold the exact starting position while leveling the hull.
        if seconds <= LANDING_ALIGNMENT_SECONDS {
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
        assist.radial_speed_meters_per_second = speed;
        self.pose.position_meters = add(
            body_definition(body_id).center.meters(),
            scale(assist.surface_normal, distance),
        );
        if assist.elapsed == LANDING_ASSIST_DURATION {
            self.flight_state = FlightState::Landed { body: body_id };
            self.thruster_percentage = 0;
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
        assist.radial_speed_meters_per_second = speed;
        self.pose.position_meters = add(
            body_definition(body_id).center.meters(),
            scale(assist.surface_normal, distance),
        );
        if assist.elapsed == TAKEOFF_ASSIST_DURATION {
            self.flight_state = FlightState::Flying;
            self.assist = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orientation::axis_angle_quaternion;
    use salimon_math::dot;

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

                    let destination = body.radius_meters - LOWEST_LOCAL_Y_METERS;
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

                    ship.trigger_landing_action();
                    assert_eq!(ship.snapshot().pose, landed.pose);
                    ship.advance(Duration::from_secs(1));
                    let lifted = ship.snapshot();
                    let lift = vector_length(sub(
                        lifted.pose.position_meters,
                        landed.pose.position_meters,
                    ));
                    assert!(
                        (lift - 7.5).abs() < 1.0e-3,
                        "the first second must show local lift"
                    );
                    assert_eq!(lifted.pose.orientation, landed.pose.orientation);
                    let clearance = body.radius_meters * (1.0 + LANDING_RANGE_RADIUS_FRACTION)
                        + COLLISION_RADIUS_METERS;
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
                    assert!(
                        (vector_length(sub(flying.pose.position_meters, body.center.meters()))
                            - clearance)
                            .abs()
                            < 1.0e-3
                    );
                    assert_eq!(ship.contextual_cockpit_message(), None);
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
                    assert_eq!(partitioned.snapshot(), reference.snapshot());
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
