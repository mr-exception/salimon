//! Closed-game power only. Kinematic/region chronology belongs to its host.
use super::*;
impl ShipController {
    /// Settle conversion, ordinary service and Fabricator demand without thrust,
    /// steering, door/assist motion or changing any selected engine authority.
    /// Captured assist joules remain isolated from the ordinary bus.
    pub fn advance_offline_power(&mut self, delta: Duration) {
        self.fabricator_powered_seconds = 0.;
        self.auxiliary_watts = 0.;
        let converting = self.conversion.remaining_duration().min(delta);
        let incoming = self.conversion.advance(converting);
        for (active, input) in [(converting, incoming), (delta - converting, 0.)] {
            let powered = self
                .energy_core
                .advance_with_load(active, 0, false, input, self.fabricator_watts)
                .expect("validated offline power demand and conversion");
            if self.fabricator_watts > 0. {
                self.fabricator_powered_seconds += powered;
            }
        }
    }
    /// A trusted chronological region owner supplies a verified motion boundary.
    /// This boundary grants no charge, thrust, assist, engine switch or jump.
    pub fn apply_verified_offline_motion(
        &mut self,
        pose: crate::ShipPose,
        velocity: [f64; 3],
        flight: crate::FlightState,
    ) -> Result<(), crate::InvalidFlightMotion> {
        let norm: f64 = pose.orientation.iter().map(|v| v * v).sum();
        if self.assist.is_some()
            || self.bounce.is_some()
            || !matches!(
                flight,
                FlightState::Flying
                    | FlightState::Landed { .. }
                    | FlightState::GroundedAfterCrash { .. }
            )
            || pose
                .position_meters
                .iter()
                .chain(velocity.iter())
                .any(|v| !v.is_finite())
            || !norm.is_finite()
            || (norm - 1.).abs() > 1e-9
            || (!matches!(flight, FlightState::Flying) && velocity != [0.; 3])
        {
            return Err(crate::InvalidFlightMotion);
        }
        self.pose = pose;
        self.velocity = velocity;
        self.flight_state = flight;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closed_power_cannot_generate_impulse_or_change_engine_selection() {
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: [0.; 3],
                orientation: [0., 0., 0., 1.],
            },
            100,
        );
        let before = ship.snapshot();
        ship.advance_offline_power(Duration::from_secs(30));
        let after = ship.snapshot();
        assert_eq!(after.pose, before.pose);
        assert_eq!(
            after.velocity_meters_per_second,
            before.velocity_meters_per_second
        );
        assert_eq!(after.engines_enabled, before.engines_enabled);
        assert_eq!(after.thruster_percentage, before.thruster_percentage);
        let service =
            ship.energy_config().gravity_idle_watts + ship.energy_config().cockpit_idle_watts;
        assert_eq!(
            before.energy_core.stored_joules - after.energy_core.stored_joules,
            (30. * service) as u64
        );
    }
}
