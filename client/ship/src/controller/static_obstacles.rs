//! Conservative authored-hull clearance against fixed organism parts; physics owns sweeps.
use super::*;
use salimon_physics::static_boxes::{StaticBox, StaticCollisionError, StaticField};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub(super) struct ExternalObstacles {
    pub field: Arc<StaticField>,
    pub hull_radius: f64,
}
impl ShipController {
    /// Caller supplies the full authored hull bound, independent of render LOD.
    /// Admission rejects an already intersecting restore/spawn; it never removes organisms.
    pub fn set_surface_obstacles(
        &mut self,
        field: Arc<StaticField>,
        hull: StaticBox,
    ) -> Result<(), StaticCollisionError> {
        let b = hull.bounds();
        let radius = (0..3)
            .map(|i| b[2 * i].abs().max(b[2 * i + 1].abs()).powi(2))
            .sum::<f64>()
            .sqrt();
        if !radius.is_finite() {
            return Err(StaticCollisionError::InvalidBounds);
        }
        if field.overlaps(self.pose.position_meters, [radius; 3])? {
            return Err(StaticCollisionError::InvalidBounds);
        }
        self.surface_obstacles = if field.boxes().is_empty() {
            None
        } else {
            Some(ExternalObstacles {
                field,
                hull_radius: radius,
            })
        };
        Ok(())
    }
    /// Safety hold preserves the pose/source field instead of inventing a clear landing.
    pub fn hold_for_surface_obstruction(&mut self) {
        self.velocity = [0.; 3];
        self.thruster_percentage = 0;
        self.engines_enabled = false;
        self.assist = None;
        self.bounce = None;
        self.clear_flight_control_input();
        self.flight_state = FlightState::Flying;
        self.cockpit_message = Some(CockpitMessage::UnsafeLandingSite);
    }
    pub(crate) fn external_site_clear(&self, pose: ShipPose) -> bool {
        self.surface_obstacles.as_ref().is_none_or(|o| {
            !o.field
                .overlaps(pose.position_meters, [o.hull_radius; 3])
                .expect("finite validated pose and hull")
        })
    }
    pub(super) fn constrain_surface_obstacles(&mut self, start: [f64; 3]) {
        let Some(o) = &self.surface_obstacles else {
            return;
        };
        let end = self.pose.position_meters;
        if start == end {
            return;
        }
        if let Some(hit) = o
            .field
            .sweep(start, end, [o.hull_radius; 3])
            .expect("finite production ship segment")
        {
            let movement = salimon_math::sub(end, start);
            let distance = salimon_math::length(movement);
            let margin = start
                .iter()
                .map(|v| v.abs() * f64::EPSILON * 8.)
                .fold(0.001, f64::max);
            self.pose.position_meters = salimon_math::add(
                start,
                salimon_math::scale(movement, (hit.fraction - margin / distance).max(0.)),
            );
            self.hold_for_surface_obstruction();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ship_hull_sweeps_fixed_organisms_and_unsafe_landing_is_rejected() {
        let pose = ShipPose {
            position_meters: [-100., 10000., 0.],
            orientation: [0., 0., 0., 1.],
        };
        let mut ship = ShipController::flying_with_velocity(pose, 0, [200., 0., 0.]).unwrap();
        let field = Arc::new(
            StaticField::new(vec![
                StaticBox::new([0., 1., 9990., 10010., -10., 10.]).unwrap(),
            ])
            .unwrap(),
        );
        ship.set_surface_obstacles(field, StaticBox::new([-2., 2., -2., 2., -2., 2.]).unwrap())
            .unwrap();
        assert!(!ship.external_site_clear(ShipPose {
            position_meters: [0., 10000., 0.],
            ..pose
        }));
        ship.pose.position_meters = [100., 10000., 0.];
        ship.constrain_surface_obstacles(pose.position_meters);
        assert!(ship.pose.position_meters[0] < -3.);
        assert_eq!(ship.velocity, [0.; 3]);
        assert!(!ship.engines_enabled);
        assert_eq!(ship.flight_state, FlightState::Flying);
    }
}
