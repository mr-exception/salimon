//! Typed static-world collision adapter. The common sweep belongs to physics.
use super::*;
use salimon_physics::static_boxes::StaticField;
use std::sync::Arc;
impl CharacterController {
    /// Derived world geometry, rebuilt before movement after restore. Never saved as player state.
    pub fn set_surface_obstacles(&mut self, field: Arc<StaticField>) {
        self.surface_obstacles = if field.boxes().is_empty() {
            None
        } else {
            Some(field)
        };
    }
    pub(super) fn constrain_surface_obstacles(
        &mut self,
        before: PositionState,
        surface: SurfaceFrame,
    ) {
        let Some(field) = &self.surface_obstacles else {
            return;
        };
        let (start, end) = match (before, self.position) {
            (
                PositionState::Surface { world: a } | PositionState::Space { world: a },
                PositionState::Surface { world: b } | PositionState::Space { world: b },
            ) => (a, b),
            _ => return,
        };
        if start == end {
            return;
        }
        let up = normalize(sub(start, surface.body_center_meters));
        let offset = scale(
            up,
            PLAYER_EYE_HEIGHT_METERS - PLAYER_BODY_HEIGHT_METERS * 0.5,
        );
        let a = sub(start, offset);
        let b = sub(end, offset);
        let half = std::array::from_fn(|i| {
            PLAYER_RADIUS_METERS
                + up[i].abs() * (PLAYER_BODY_HEIGHT_METERS * 0.5 - PLAYER_RADIUS_METERS)
        });
        let hit = field
            .sweep(a, b, half)
            .expect("validated field and finite production character motion");
        if let Some(hit) = hit {
            let delta = sub(end, start);
            let length = salimon_math::length(delta);
            let quantization = start
                .iter()
                .map(|v| v.abs() * f64::EPSILON * 8.)
                .fold(0.001, f64::max);
            let t = (hit.fraction - quantization / length).max(0.);
            let world = add(start, scale(delta, t));
            self.position = match self.position {
                PositionState::Surface { .. } => PositionState::Surface { world },
                PositionState::Space { .. } => PositionState::Space { world },
                _ => return,
            };
            self.vertical_speed = 0.;
            let normal = hit.normal;
            for velocity in [
                &mut self.eva_inherited_velocity,
                &mut self.eva_control_velocity,
            ] {
                let incoming = dot(*velocity, normal).min(0.);
                *velocity = sub(*velocity, scale(normal, incoming));
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use salimon_physics::static_boxes::StaticBox;
    #[test]
    fn character_static_organisms_block_swept_body_and_allow_retreat() {
        let surface = SurfaceFrame {
            query: None,
            body_center_meters: [0.; 3],
            radius_meters: 1000.,
        };
        let mut c = CharacterController::on_surface(surface, [0., 1000., 0.]).unwrap();
        let field = Arc::new(
            StaticField::new(vec![
                StaticBox::new([1., 2., 999., 1004., -1., 1.]).unwrap(),
            ])
            .unwrap(),
        );
        c.set_surface_obstacles(field);
        let a = c.position;
        c.position = PositionState::Surface {
            world: [10., 1001.75, 0.],
        };
        c.constrain_surface_obstacles(a, surface);
        let PositionState::Surface { world } = c.position else {
            panic!()
        };
        assert!(
            world[0] < 1. - PLAYER_RADIUS_METERS && world[0] > 0.99 * (1. - PLAYER_RADIUS_METERS)
        );
        let before = c.position;
        c.position = PositionState::Surface {
            world: [-2., 1001.75, 0.],
        };
        c.constrain_surface_obstacles(before, surface);
        let PositionState::Surface { world } = c.position else {
            panic!()
        };
        assert_eq!(world[0], -2.);
    }
}
