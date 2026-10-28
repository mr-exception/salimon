//! Short live-controller forecast. Uses cloned paid power/contact/steering state;
//! never writes a command, ledger, assist or authoritative ship state.
use crate::{FlightState, ShipController};
use salimon_math::{length, sub};
use std::time::Duration;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Preview {
    pub target: &'static str,
    pub horizon_seconds: f64,
    pub closest_altitude_meters: f64,
    pub closest_seconds: f64,
    pub contact_window_seconds: Option<[f64; 2]>,
    pub convergence_meters: f64,
    pub power_exhausted: bool,
    pub endpoint_meters: [f64; 3],
}
impl ShipController {
    /// Assumes held inputs, current mass and existing on-board conversion. Does
    /// not anticipate runtime auxiliary demand, new cargo, refueling or commands.
    /// Half-step discrepancy is an indicator, not a certified uncertainty bound.
    pub fn navigation_preview(&self) -> Option<Preview> {
        let start = self.snapshot();
        if start.flight_state != FlightState::Flying {
            return None;
        }
        let (body, _) = self.terrain.nearest_body(
            start.pose.position_meters,
            crate::NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS,
            true,
        )?;
        let altitude = |ship: &ShipController| {
            length(sub(ship.pose.position_meters, body.center.meters())) - body.radius_meters
        };
        let mut coarse = self.clone();
        let mut fine = self.clone();
        let mut out = Preview {
            target: body.name,
            horizon_seconds: 8.192,
            closest_altitude_meters: altitude(self),
            closest_seconds: 0.,
            contact_window_seconds: None,
            convergence_meters: 0.,
            power_exhausted: !start.power_available,
            endpoint_meters: start.pose.position_meters,
        };
        for i in 1..=128 {
            coarse.advance(Duration::from_millis(64));
            for _ in 0..2 {
                fine.advance(Duration::from_millis(32));
            }
            let t = f64::from(i) * 0.064;
            let h = altitude(&fine);
            if h < out.closest_altitude_meters {
                out.closest_altitude_meters = h;
                out.closest_seconds = t;
            }
            out.convergence_meters = out.convergence_meters.max(length(sub(
                coarse.pose.position_meters,
                fine.pose.position_meters,
            )));
            out.power_exhausted |= !fine.snapshot().power_available;
            out.endpoint_meters = fine.pose.position_meters;
            if !out.closest_altitude_meters.is_finite()
                || !out.convergence_meters.is_finite()
                || !out.endpoint_meters.into_iter().all(f64::is_finite)
            {
                return None;
            }
            if fine.flight_state != FlightState::Flying {
                out.contact_window_seconds = Some([t - 0.064, t]);
                out.horizon_seconds = t;
                break;
            }
        }
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn falling(height: f64, speed: f64, throttle: i8) -> ShipController {
        let b = salimon_world::CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == salimon_world::CelestialBodyId::Earth)
            .unwrap();
        ShipController::flying_with_velocity(
            crate::ShipPose {
                position_meters: salimon_math::add(
                    b.center.meters(),
                    [0., b.radius_meters + height, 0.],
                ),
                orientation: [0., 0., 0., 1.],
            },
            throttle,
            [0., -speed, 0.],
        )
        .unwrap()
    }
    #[test]
    fn navigation_live_preview_matches_authoritative_and_never_mutates_power_or_controls() {
        let ship = falling(1000., 2., 25);
        let before = serde_json::to_vec(&ship).unwrap();
        let p = ship.navigation_preview().unwrap();
        let mut authoritative = ship.clone();
        for _ in 0..256 {
            authoritative.advance(Duration::from_millis(32));
        }
        assert_eq!(
            p.endpoint_meters,
            authoritative.snapshot().pose.position_meters
        );
        assert_eq!(serde_json::to_vec(&ship).unwrap(), before);
        assert_eq!(p.horizon_seconds, 8.192);
        assert!(p.contact_window_seconds.is_none());
        let mut reference = ship.clone();
        for _ in 0..512 {
            reference.advance(Duration::from_millis(16));
        }
        let discrepancy = length(sub(
            reference.snapshot().pose.position_meters,
            p.endpoint_meters,
        ));
        assert!(discrepancy <= p.convergence_meters + 1e-5);
    }
    #[test]
    fn navigation_live_contact_window_contains_actual_contact_and_power_loss_is_visible() {
        let ship = falling(100., 100., 0);
        let p = ship.navigation_preview().unwrap();
        let [lo, hi] = p.contact_window_seconds.unwrap();
        let mut actual = ship.clone();
        let mut seconds = 0.;
        while actual.snapshot().flight_state == FlightState::Flying && seconds < 8.2 {
            actual.advance(Duration::from_millis(16));
            seconds += 0.016;
        }
        assert!(
            seconds >= lo - 1e-9 && seconds <= hi + 1e-9,
            "actual {seconds}, predicted {lo}..{hi}"
        );
        let mut blackout = falling(1000., 2., 0);
        blackout.set_stored_core_energy_joules(0);
        assert!(blackout.navigation_preview().unwrap().power_exhausted);
        assert!(ShipController::default().navigation_preview().is_none());
        let far = ShipController::flying(
            crate::ShipPose {
                position_meters: [1e15; 3],
                orientation: [0., 0., 0., 1.],
            },
            0,
        );
        assert!(far.navigation_preview().is_none());
    }
}
