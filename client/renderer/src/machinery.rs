//! Immutable presentation inputs; no machinery simulation or clock lives here.
use crate::RendererError;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MachineryState {
    #[default]
    Idle,
    Working,
    Blocked,
    Unpowered,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MachineryFeedback {
    /// Signed requested thrust, gated by actual powered engine operation.
    pub thrust: f32,
    /// Actual ramped yaw input, for opposed rudder pedal travel.
    pub rudder: f32,
    pub converting: bool,
    pub core_charge: f32,
    pub fabricator: MachineryState,
    pub fabrication_progress: f32,
    /// Four-second active-time cycle [0,1); stable during pause and capture.
    pub phase: f32,
    pub cockpit_active: bool,
}
impl MachineryFeedback {
    pub(crate) fn uniforms(self) -> Result<[f32; 8], RendererError> {
        if !self.rudder.is_finite()
            || !(-1.0..=1.0).contains(&self.rudder)
            || !self.thrust.is_finite()
            || !(-1.0..=1.0).contains(&self.thrust)
            || [self.core_charge, self.fabrication_progress, self.phase]
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err(RendererError::new(
                "invalid Scout machinery feedback",
                "expected finite normalized presentation inputs",
            ));
        }
        Ok([
            f32::from(self.converting),
            self.thrust,
            match self.fabricator {
                MachineryState::Idle => 0.,
                MachineryState::Working => 1.,
                MachineryState::Blocked => 2.,
                MachineryState::Unpowered => 3.,
            },
            self.phase,
            self.core_charge,
            self.fabrication_progress,
            f32::from(self.cockpit_active),
            self.rudder,
        ])
    }
}

/// Only named functional surfaces acquire feedback; structural meshes stay fixed.
pub(crate) fn surface_role(name: Option<&str>) -> f32 {
    match name {
        Some("Engine_Port_Glow" | "Engine_Starboard_Glow") => 1.,
        Some("Energy_Core" | "Core_Spine_Inlays") => 2.,
        Some("Core_Charge_Gauge") => 3.,
        Some("Fabricator_Panels") => 4.,
        Some("Cockpit_Rudder_Pedals") => 5.,
        Some("Cockpit_Ready_Indicators") => 6.,
        _ => 0.,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_feedback_rejects_nan_infinity_and_out_of_range_inputs() {
        for v in [f32::NAN, f32::INFINITY, -2., 2.] {
            assert!(
                MachineryFeedback {
                    thrust: v,
                    ..Default::default()
                }
                .uniforms()
                .is_err()
            );
            assert!(
                MachineryFeedback {
                    phase: v,
                    ..Default::default()
                }
                .uniforms()
                .is_err()
            );
        }
        assert!(MachineryFeedback::default().uniforms().is_ok());
    }
    #[test]
    fn motion_roles_exclude_hulls_supports_and_ramp_collision_surfaces() {
        for name in [
            "Landing_Gear_Forward_Port",
            "Exit_Door",
            "Exit_Ramp_Tip",
            "Core_Cradle",
            "Fabricator_Output",
            "Engine_Port_Running_Light",
        ] {
            assert_eq!(surface_role(Some(name)), 0.);
        }
        assert_eq!(surface_role(Some("Cockpit_Rudder_Pedals")), 5.);
    }
}
