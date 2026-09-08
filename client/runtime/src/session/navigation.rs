//! Disposable observation cache; no save or simulation ownership.
use super::GameplaySession;
use std::time::Duration;
pub(crate) struct Cached {
    at: Duration,
    source: salimon_ship::ShipSnapshot,
    preview: Option<salimon_ship::preview::Preview>,
}
impl GameplaySession {
    pub(crate) fn navigation_prediction(&self) -> Option<salimon_renderer::PredictionInstruments> {
        let source = self.ship.snapshot();
        let mut cache = self.navigation_cache.borrow_mut();
        let expired = cache.as_ref().is_none_or(|v| {
            self.elapsed < v.at
                || self.elapsed - v.at >= Duration::from_millis(500)
                || source.flight_state != v.source.flight_state
                || source.terrain != v.source.terrain
                || source.thruster_percentage != v.source.thruster_percentage
                || source.engines_enabled != v.source.engines_enabled
                || source.total_mass_kg != v.source.total_mass_kg
                || source.power_available != v.source.power_available
        });
        if expired {
            *cache = Some(Cached {
                at: self.elapsed,
                source,
                preview: self.ship.navigation_preview_at(self.motion.as_ref()),
            });
        }
        let cached = cache.as_ref()?;
        let age = (self.elapsed - cached.at).as_secs_f64();
        cached
            .preview
            .map(|v| salimon_renderer::PredictionInstruments {
                horizon_seconds: (v.horizon_seconds - age).max(0.),
                closest_altitude_meters: v.closest_altitude_meters,
                closest_seconds: (v.closest_seconds - age).max(0.),
                contact_window_seconds: v
                    .contact_window_seconds
                    .map(|window| window.map(|t| (t - age).max(0.))),
                convergence_meters: v.convergence_meters,
                age_seconds: age,
                warning: if v.contact_window_seconds.is_some() {
                    "CONTACT ESTIMATE - HOLD INPUTS"
                } else if v.power_exhausted {
                    "POWER LIMITED - HOLD INPUTS"
                } else {
                    "HOLD INPUTS - NO AUTOPILOT"
                },
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn navigation_cache_is_disposable_and_controls_invalidate_without_changing_saves() {
        let mut s = GameplaySession::new_canonical_run().unwrap();
        let base = s.ship.snapshot();
        s.ship = salimon_ship::ShipController::flying_with_velocity(
            salimon_ship::ShipPose {
                position_meters: salimon_math::add(base.pose.position_meters, [0., 1000., 0.]),
                ..base.pose
            },
            0,
            [0., -2., 0.],
        )
        .unwrap();
        s.ship.configure_terrain(
            salimon_world::terrain::TerrainMode::Canonical,
            crate::terrain::landing_footprint(),
        );
        s.advance_automation(Duration::from_millis(16));
        let bytes = s.save_bytes(10).unwrap();
        let first = s.navigation_prediction().unwrap();
        assert_eq!(s.save_bytes(10).unwrap(), bytes);
        assert_eq!(s.navigation_prediction(), Some(first));
        s.advance_automation(Duration::from_millis(16));
        let aged = s.navigation_prediction().unwrap();
        assert_eq!(aged.age_seconds, 0.016);
        assert!((first.horizon_seconds - aged.horizon_seconds - 0.016).abs() < 1e-12);
        assert!((first.closest_seconds - aged.closest_seconds - 0.016).abs() < 1e-12);
        s.command(super::super::Command::ToggleEngines);
        assert_eq!(s.navigation_prediction().unwrap().age_seconds, 0.);
        let bytes = s.save_bytes(10).unwrap();
        s.restore_bytes(&bytes).unwrap();
        assert_eq!(s.navigation_prediction().unwrap().age_seconds, 0.);
        assert_eq!(s.save_bytes(10).unwrap(), bytes);
        s.advance_automation(Duration::from_millis(512));
        assert_eq!(s.navigation_prediction().unwrap().age_seconds, 0.);
    }
}
