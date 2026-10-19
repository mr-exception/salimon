//! Typed logical character state. Caller rebinds the immutable terrain query.
use super::CharacterController;
use salimon_persistence::{SaveError, Validate, invalid};
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct CharacterSaveState(CharacterController);
impl Validate for CharacterSaveState {
    fn validate(&self) -> Result<(), SaveError> {
        let c = &self.0;
        let position = match c.position {
            crate::state::PositionState::Cockpit => [0.; 3],
            crate::state::PositionState::Inside { local } => local,
            crate::state::PositionState::Doorway { world, .. }
            | crate::state::PositionState::Surface { world }
            | crate::state::PositionState::Space { world } => world,
        };
        if position
            .iter()
            .chain(c.interior_velocity.iter())
            .chain(c.eva_inherited_velocity.iter())
            .chain(c.eva_control_velocity.iter())
            .chain(c.eva_axes.iter().flatten())
            .any(|v| !v.is_finite())
            || !c.yaw_radians.is_finite()
            || !c.pitch_radians.is_finite()
            || c.pitch_radians.abs() > std::f64::consts::FRAC_PI_2
            || !c.vertical_speed.is_finite()
        {
            return Err(invalid("character pose/motion"));
        }
        for axis in c.eva_axes {
            let n: f64 = axis.iter().map(|v| v * v).sum();
            if (n - 1.).abs() > 1e-9 {
                return Err(invalid("EVA basis"));
            }
        }
        if let Some(s) = c.nearby_surface
            && (!s.radius_meters.is_finite()
                || s.radius_meters <= 0.
                || s.body_center_meters.iter().any(|v| !v.is_finite()))
        {
            return Err(invalid("nearby character surface"));
        }
        Ok(())
    }
}
impl CharacterSaveState {
    pub fn nearby_surface(&self) -> Option<crate::SurfaceFrame> {
        self.0.nearby_surface
    }
}
impl CharacterController {
    pub fn save_state(&self) -> CharacterSaveState {
        CharacterSaveState(self.clone())
    }
    pub fn from_save(
        state: CharacterSaveState,
        query: Option<crate::GroundQuery>,
    ) -> Result<Self, SaveError> {
        state.validate()?;
        let mut c = state.0;
        c.jump_was_down = false;
        c.set_movement_scale(1.0);
        if let Some(s) = &mut c.nearby_surface {
            s.query = query;
        }
        Ok(c)
    }
}
