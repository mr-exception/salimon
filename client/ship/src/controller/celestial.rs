//! Active and closed canonical motion in explicitly translating, nonrotating
//! Vesper axes. Relative acceleration is celestial gravity minus frame acceleration.
use super::ShipController;
use crate::FlightState;
use salimon_math::{add, length, scale, sub};
use salimon_universe::{
    Error,
    coordinates::GalacticPosition,
    gravity::{Config, Selector},
    starting_motion::{self, Clock},
};
use std::time::Duration;

impl ShipController {
    /// Closing never preserves a live manual engine or held pilot command. An
    /// already funded committed assist remains a committed physical path.
    pub fn shut_down_for_close(&mut self) {
        self.engines_enabled = false;
        self.thruster_percentage = 0;
        self.clear_flight_control_input();
    }
    /// Atomically advance physical sources and their exact motion clock. Both
    /// hosts use this operation; closed mode settles power without live engines.
    pub fn advance_celestial(
        &mut self,
        clock: &mut Clock,
        delta: Duration,
        closed: bool,
    ) -> Result<(), Error> {
        clock.validate()?;
        if self.terrain != salimon_world::terrain::TerrainMode::Canonical {
            return Err(error("celestial advance requires the canonical body frame"));
        }
        if starting_motion::DEFINITION.surface_pressure_pascals != 0.0 {
            return Err(error(
                "canonical atmosphere requires a published drag model",
            ));
        }
        let final_clock = clock.advanced(delta)?;
        let mut candidate = self.clone();
        let mut at = clock.clone();
        if closed {
            candidate.shut_down_for_close();
        }
        let mut remaining = delta;
        let mut funded_seconds = 0.;
        while !remaining.is_zero() {
            let mut step = remaining;
            if matches!(candidate.flight_state, FlightState::Flying) {
                // Far flight uses the one-second orbital reference grid. Near
                // terrain the chord sag for the <=12 m/s² published surface
                // field is <0.4 mm over 16 ms, below the existing 1 mm sweep.
                let body = salimon_world::canonical::BODIES[0];
                let radius = length(candidate.pose.position_meters);
                let speed = length(candidate.velocity);
                let outer = body.radius_meters
                    + candidate.terrain.elevation_bounds(body)[1]
                    + crate::COLLISION_RADIUS_METERS;
                let near = radius - outer <= speed + 32.;
                step = step.min(if near {
                    Duration::from_millis(16)
                } else {
                    Duration::from_secs(1)
                });
                // Anchor one-second steps to the model's exact epoch grid so
                // ordinary second-resolution closed-interval partitions agree.
                if !near {
                    let nanos = at.epoch.nanosecond_of_day();
                    let until_second = 1_000_000_000 - nanos % 1_000_000_000;
                    step = step.min(Duration::from_nanos(until_second));
                }
            } else if matches!(
                candidate.flight_state,
                FlightState::AssistedLanding { .. }
                    | FlightState::AssistedTakeoff { .. }
                    | FlightState::Bouncing { .. }
            ) {
                step = step.min(Duration::from_millis(16));
            }
            let next_clock = at.advanced(step)?;
            let relative_start = candidate.pose.position_meters;
            let flying = candidate.flight_state == FlightState::Flying;
            if flying {
                let gravity = relative_gravity(at.epoch, candidate.pose.position_meters)?;
                candidate.velocity =
                    add(candidate.velocity, scale(gravity, step.as_secs_f64() * 0.5));
            }
            if closed {
                candidate.advance_offline_power(step);
                candidate.settle_assist_energy(step);
                let obstacle_start = candidate.pose.position_meters;
                match candidate.flight_state {
                    FlightState::Flying => candidate.advance_powered_flight(step, 0., 0.),
                    FlightState::AssistedLanding { body } => candidate.advance_landing(body, step),
                    FlightState::AssistedTakeoff { body } => candidate.advance_takeoff(body, step),
                    FlightState::Bouncing { body } => candidate.advance_bounce(body, step),
                    FlightState::Landed { .. } | FlightState::GroundedAfterCrash { .. } => {}
                }
                candidate.constrain_surface_obstacles(obstacle_start);
            } else {
                candidate.advance(step);
            }
            if !candidate.contact_verified {
                return Err(error(
                    "canonical collision query exhausted its verification budget",
                ));
            }
            if flying {
                verify_stellar_exterior(
                    at.epoch,
                    next_clock.epoch,
                    relative_start,
                    candidate.pose.position_meters,
                    step,
                )?;
            }
            funded_seconds += candidate.fabricator_powered_seconds;
            if flying && candidate.flight_state == FlightState::Flying {
                let gravity = relative_gravity(next_clock.epoch, candidate.pose.position_meters)?;
                candidate.velocity =
                    add(candidate.velocity, scale(gravity, step.as_secs_f64() * 0.5));
            }
            if candidate
                .pose
                .position_meters
                .into_iter()
                .chain(candidate.velocity)
                .any(|v| !v.is_finite())
            {
                return Err(error("canonical trajectory nonfinite state"));
            }
            at = next_clock;
            remaining -= step;
        }
        candidate.fabricator_powered_seconds = funded_seconds;
        *self = candidate;
        *clock = final_clock;
        Ok(())
    }
}
fn relative_gravity(epoch: salimon_universe::Epoch, position: [f64; 3]) -> Result<[f64; 3], Error> {
    let mut selector = Selector::new(Config::default())?;
    let gravity = selector
        .sample(
            &starting_motion::sources(epoch)?,
            GalacticPosition::new([0; 3], position)?,
            0.,
        )?
        .meters_per_second_squared;
    Ok(sub(
        gravity,
        starting_motion::planet(epoch)?.acceleration_mps2,
    ))
}
fn error(message: &str) -> Error {
    Error::Invalid(message.into())
}
fn verify_stellar_exterior(
    start: salimon_universe::Epoch,
    end: salimon_universe::Epoch,
    local_start: [f64; 3],
    local_end: [f64; 3],
    delta: Duration,
) -> Result<(), Error> {
    let a = add(starting_motion::planet(start)?.position_meters, local_start);
    let b = add(starting_motion::planet(end)?.position_meters, local_end);
    let movement = sub(b, a);
    let travel = length(movement);
    let closest = if travel > 0. {
        let direction = scale(movement, travel.recip());
        let along = (-salimon_math::dot(a, direction)).clamp(0., travel);
        length(add(a, scale(direction, along)))
    } else {
        length(a)
    };
    let p = &*starting_motion::DEFINITION;
    // Exterior gravity is globally bounded by the two surface accelerations.
    // Refuse even an unresolved chord/curve near the stellar model boundary.
    let acceleration = 6.67430e-11
        * (p.star_mass_kg / p.star_radius_meters.powi(2)
            + p.planet_mass_kg / p.planet_radius_meters.powi(2));
    let uncertainty = acceleration * delta.as_secs_f64().powi(2) / 8.;
    if !closest.is_finite()
        || closest <= p.star_radius_meters + crate::COLLISION_RADIUS_METERS + uncertainty
    {
        return Err(error(
            "coasting path reaches the stellar interior model boundary; resume remains unverified",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn orbit() -> (ShipController, Clock) {
        let p = &*starting_motion::DEFINITION;
        let r = p.planet_radius_meters + 500_000.;
        let v = (6.67430e-11 * p.planet_mass_kg / r).sqrt();
        let mut ship = ShipController::flying_with_velocity(
            crate::ShipPose {
                position_meters: [r, 0., 0.],
                orientation: [0., 0., 0., 1.],
            },
            0,
            [0., 0., v],
        )
        .unwrap();
        ship.terrain = salimon_world::terrain::TerrainMode::Canonical;
        (
            ship,
            Clock::at(starting_motion::REFERENCE_UNIX_SECONDS).unwrap(),
        )
    }
    #[test]
    fn online_and_closed_gravity_share_production_motion_and_close_removes_thrust() {
        let (mut active, mut at) = orbit();
        let mut closed = active.clone();
        let mut ct = at.clone();
        active.shut_down_for_close();
        let before = closed.snapshot();
        for _ in 0..1000 {
            active
                .advance_celestial(&mut at, Duration::from_millis(16), false)
                .unwrap();
            closed
                .advance_celestial(&mut ct, Duration::from_millis(16), true)
                .unwrap();
        }
        assert_eq!(active.snapshot().pose, closed.snapshot().pose);
        assert_eq!(active.velocity, closed.velocity);
        assert_eq!(at, ct);
        assert_ne!(closed.velocity, before.velocity_meters_per_second);
        assert!(!closed.snapshot().engines_enabled);
        assert_eq!(closed.snapshot().thruster_percentage, 0);
    }
    #[test]
    fn invalid_clock_rolls_back_motion_energy_and_engine_state() {
        let (mut ship, mut clock) = orbit();
        let original = serde_json::to_vec(&ship).unwrap();
        clock.definition_sha256 = "wrong".into();
        assert!(
            ship.advance_celestial(&mut clock, Duration::from_secs(10), true)
                .is_err()
        );
        assert_eq!(serde_json::to_vec(&ship).unwrap(), original);
    }
}

#[cfg(test)]
mod reference_tests {
    use super::*;
    #[test]
    fn long_coast_stays_within_the_25m_finer_step_reference_budget() {
        let p = &*starting_motion::DEFINITION;
        let radius = p.planet_radius_meters + 500_000.;
        let speed = (6.67430e-11 * p.planet_mass_kg / radius).sqrt();
        let mut whole = ShipController::flying_with_velocity(
            crate::ShipPose {
                position_meters: [radius, 0., 0.],
                orientation: [0., 0., 0., 1.],
            },
            0,
            [0., 0., speed],
        )
        .unwrap();
        whole.terrain = salimon_world::terrain::TerrainMode::Canonical;
        let mut reference = whole.clone();
        let mut at = Clock::at(starting_motion::REFERENCE_UNIX_SECONDS).unwrap();
        let mut rt = at.clone();
        whole
            .advance_celestial(&mut at, Duration::from_secs(3600), true)
            .unwrap();
        for _ in 0..14400 {
            reference
                .advance_celestial(&mut rt, Duration::from_millis(250), true)
                .unwrap();
        }
        let position_error = length(sub(
            whole.pose.position_meters,
            reference.pose.position_meters,
        ));
        let velocity_error = length(sub(whole.velocity, reference.velocity));
        assert!(
            position_error < 25.,
            "position difference {position_error} m"
        );
        assert!(
            velocity_error < 0.05,
            "velocity difference {velocity_error} m/s"
        );
        assert_eq!(at, rt);
        assert!(matches!(whole.flight_state, FlightState::Flying));
    }
}
