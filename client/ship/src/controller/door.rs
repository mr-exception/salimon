//! Airlock permissions and hinge animation.

use super::ShipController;
use crate::{CockpitMessage, DOOR_HINGE_ANIMATION_DURATION, DoorState, FlightState};
use std::time::Duration;

impl ShipController {
    pub fn toggle_door(&mut self) {
        self.cockpit_message = None;
        match self.flight_state {
            state
                if matches!(state, FlightState::Landed { .. })
                    || (state == FlightState::Flying
                        && self.nearby_body_telemetry([0.0; 3]).is_none()) =>
            {
                self.door_state = match self.door_state {
                    DoorState::Closed => DoorState::Open,
                    DoorState::Open => DoorState::Closed,
                };
            }
            _ => {
                self.door_state = DoorState::Closed;
                self.cockpit_message = Some(CockpitMessage::DoorLockedWhileInFlight);
            }
        }
    }
    pub(super) fn advance_door(&mut self, delta: Duration) {
        let door_step = delta.as_secs_f64() / DOOR_HINGE_ANIMATION_DURATION.as_secs_f64();
        self.door_open_fraction = match self.door_state {
            DoorState::Open => (self.door_open_fraction + door_step).min(1.0),
            DoorState::Closed => (self.door_open_fraction - door_step).max(0.0),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS, ShipPose};
    use salimon_world::{CELESTIAL_BODIES, CelestialBodyId};

    #[test]
    fn landed_and_open_space_doors_toggle_but_nearby_flight_is_locked() {
        let mut landed = ShipController::default();
        landed.toggle_door();
        assert_eq!(landed.snapshot().door_state, DoorState::Open);

        let mut flying = ShipController::flying(
            ShipPose {
                position_meters: ShipController::default().snapshot().pose.position_meters,
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
    fn top_hinged_door_advances_reverses_and_blocks_takeoff_until_closed() {
        let mut ship = ShipController::default();
        ship.toggle_door();
        assert_eq!(ship.snapshot().door_open_fraction, 0.0);
        ship.advance(DOOR_HINGE_ANIMATION_DURATION / 2);
        assert!((ship.snapshot().door_open_fraction - 0.5).abs() < 1.0e-12);
        ship.toggle_door();
        ship.advance(DOOR_HINGE_ANIMATION_DURATION / 4);
        assert!((ship.snapshot().door_open_fraction - 0.25).abs() < 1.0e-12);
        ship.set_cockpit_control(true);
        ship.trigger_landing_action();
        assert_eq!(
            ship.snapshot().cockpit_message,
            Some(CockpitMessage::CloseDoorBeforeTakeoff)
        );
        ship.advance(DOOR_HINGE_ANIMATION_DURATION / 4);
        assert_eq!(ship.snapshot().door_open_fraction, 0.0);
    }

    #[test]
    fn space_airlock_uses_inclusive_nearby_threshold_and_assist_lock() {
        let earth = CELESTIAL_BODIES
            .iter()
            .find(|body| body.id == CelestialBodyId::Earth)
            .unwrap();
        for (distance, expected) in [
            (NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS, DoorState::Closed),
            (
                NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS + 1.0,
                DoorState::Open,
            ),
        ] {
            let mut position = earth.center.meters();
            position[1] += earth.radius_meters + distance;
            let mut ship = ShipController::flying(
                ShipPose {
                    position_meters: position,
                    orientation: [0.0, 0.0, 0.0, 1.0],
                },
                0,
            );
            ship.toggle_door();
            assert_eq!(ship.snapshot().door_state, expected);
            ship.toggle_door();
            assert_eq!(ship.snapshot().door_state, DoorState::Closed);
            for state in [
                FlightState::AssistedLanding {
                    body: CelestialBodyId::Earth,
                },
                FlightState::AssistedTakeoff {
                    body: CelestialBodyId::Earth,
                },
            ] {
                ship.flight_state = state;
                ship.toggle_door();
                assert_eq!(ship.door_state, DoorState::Closed);
            }
        }
    }
}
