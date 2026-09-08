use crate::session::{GameplaySession, ShipControlInput};
use salimon_ship::{FlightState, ShipSnapshot};
use std::time::Duration;

impl GameplaySession {
    pub(super) fn advance_powered_domains(&mut self, delta: Duration) -> ShipSnapshot {
        self.advance_surface_age(delta);
        self.camera_prototype.advance(delta);
        self.ship
            .set_steering_input(self.input.steering().steering());
        let pre_flight = self.ship.snapshot().flight_state;
        self.advance_fabrication(delta);
        if !matches!(pre_flight, FlightState::GroundedAfterCrash { .. })
            && matches!(
                self.ship.snapshot().flight_state,
                FlightState::GroundedAfterCrash { .. }
            )
        {
            self.interrupt_construction_after_crash();
        }
        self.advance_recovery(delta);
        self.advance_construction(delta);
        self.advance_scanner(delta);
        let ship = self.ship.snapshot();
        if !ship.power_available
            || !ship.cockpit_control_active
            || ship.flight_state != FlightState::Flying
        {
            self.input.steering = ShipControlInput::default();
        }
        self.character
            .set_ship_gravity_enabled(ship.power_available);
        self.mining.ship_gravity_disabled = !ship.power_available;
        ship
    }
}
