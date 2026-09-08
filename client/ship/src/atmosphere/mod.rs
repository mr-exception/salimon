//! Declared exponential atmosphere and dissipative quadratic drag. No guessed
//! physical fields, heating or Core debit; online/offline use the same operator.
use crate::{
    Propulsion,
    surface::{BodyMotion, error, finite},
};
use salimon_math::{add, cross, length, scale, sub};
use salimon_universe::{Epoch, Error, coordinates::GalacticPosition};

#[derive(Clone, Copy, Debug)]
pub struct Atmosphere {
    radius_meters: f64,
    surface_density_kg_m3: f64,
    scale_height_meters: f64,
    ceiling_meters: f64,
    spin_rps: [f64; 3],
}
impl Atmosphere {
    pub fn new(
        radius_meters: f64,
        surface_density_kg_m3: f64,
        scale_height_meters: f64,
        ceiling_meters: f64,
        spin_rps: [f64; 3],
    ) -> Result<Self, Error> {
        if [radius_meters, scale_height_meters, ceiling_meters]
            .into_iter()
            .any(|v| !v.is_finite() || v <= 0.0)
            || !surface_density_kg_m3.is_finite()
            || surface_density_kg_m3 < 0.0
        {
            return Err(error("atmosphere radius/density/height/ceiling"));
        }
        finite(spin_rps)?;
        Ok(Self {
            radius_meters,
            surface_density_kg_m3,
            scale_height_meters,
            ceiling_meters,
            spin_rps,
        })
    }
    pub fn density_at_altitude(self, altitude_meters: f64) -> Result<f64, Error> {
        if !altitude_meters.is_finite() || altitude_meters < 0.0 {
            return Err(error("atmosphere altitude below solid surface/nonfinite"));
        }
        if altitude_meters >= self.ceiling_meters {
            return Ok(0.0);
        }
        // C1 taper to the declared model ceiling, avoiding a density jump.
        let u = ((altitude_meters / self.ceiling_meters - 0.9) / 0.1).clamp(0.0, 1.0);
        Ok(self.surface_density_kg_m3
            * (-altitude_meters / self.scale_height_meters).exp()
            * (1.0 - u * u * (3.0 - 2.0 * u)))
    }
    /// Wind uses inertial axes, relative to rigidly co-rotating gas. The body
    /// provider is epoch checked and supplies orbital velocity, never zero by default.
    pub fn sample(
        self,
        epoch: Epoch,
        position: GalacticPosition,
        body: BodyMotion,
        wind_mps: [f64; 3],
    ) -> Result<Air, Error> {
        let body = body.validate(epoch)?;
        finite(wind_mps)?;
        let offset = position.relative_meters(body.center)?;
        let rho = self.density_at_altitude(length(offset) - self.radius_meters)?;
        Air::new(
            rho,
            add(
                body.velocity_mps,
                add(cross(self.spin_rps, offset), wind_mps),
            ),
        )
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Air {
    density_kg_m3: f64,
    velocity_mps: [f64; 3],
}
impl Air {
    pub fn new(density_kg_m3: f64, velocity_mps: [f64; 3]) -> Result<Self, Error> {
        finite(velocity_mps)?;
        if !density_kg_m3.is_finite() || density_kg_m3 < 0.0 {
            return Err(error("air density"));
        }
        Ok(Self {
            density_kg_m3,
            velocity_mps,
        })
    }
    pub fn density_kg_m3(self) -> f64 {
        self.density_kg_m3
    }
    pub fn velocity_mps(self) -> [f64; 3] {
        self.velocity_mps
    }
}
/// Effective Cd * projected area; callers supply the actual ship configuration.
/// This is a declared continuum model, not a fitted hypersonic/reentry model.
#[derive(Clone, Copy, Debug)]
pub struct Aerodynamics {
    cd_area_m2: f64,
}
impl Aerodynamics {
    pub fn new(drag_coefficient: f64, projected_area_m2: f64) -> Result<Self, Error> {
        if [drag_coefficient, projected_area_m2]
            .into_iter()
            .any(|v| !v.is_finite() || v < 0.0)
            || !(drag_coefficient * projected_area_m2).is_finite()
        {
            return Err(error("aerodynamic coefficient/area"));
        }
        Ok(Self {
            cd_area_m2: drag_coefficient * projected_area_m2,
        })
    }
    /// Instantaneous passive acceleration for the physical assist force budget.
    /// It consumes no Core energy and is evaluated at the actual path velocity.
    pub fn acceleration_mps2(
        self,
        velocity_mps: [f64; 3],
        propulsion: Propulsion,
        air: Option<Air>,
    ) -> Result<[f64; 3], Error> {
        finite(velocity_mps)?;
        let Some(air) = air else {
            return Ok([0.0; 3]);
        };
        let relative = sub(velocity_mps, air.velocity_mps);
        finite(relative)?;
        let result = scale(
            relative,
            -(0.5 * air.density_kg_m3 * self.cd_area_m2 / propulsion.total_mass_kg())
                * length(relative),
        );
        finite(result)?;
        Ok(result)
    }
    /// Exact fixed-density/gas-velocity solution of du/dt=-k |u|u.
    /// Never reverses relative velocity or adds relative kinetic energy, even
    /// when an explicit Euler step would overshoot. Strang composition owns dt.
    pub fn velocity_after(
        self,
        velocity_mps: [f64; 3],
        propulsion: Propulsion,
        air: Option<Air>,
        seconds: f64,
    ) -> Result<[f64; 3], Error> {
        finite(velocity_mps)?;
        if !seconds.is_finite() || seconds < 0.0 {
            return Err(error("drag time"));
        }
        let Some(air) = air else {
            return Ok(velocity_mps);
        };
        if seconds == 0.0 || air.density_kg_m3 == 0.0 || self.cd_area_m2 == 0.0 {
            return Ok(velocity_mps);
        }
        let relative = sub(velocity_mps, air.velocity_mps);
        finite(relative)?;
        let ratio = (0.5 * air.density_kg_m3 * self.cd_area_m2 / propulsion.total_mass_kg())
            * length(relative)
            * seconds;
        if !ratio.is_finite() {
            return Err(error("drag stiffness overflow; reduce step/validate model"));
        }
        let result = add(air.velocity_mps, scale(relative, 1.0 / (1.0 + ratio)));
        finite(result)?;
        Ok(result)
    }
}
#[cfg(test)]
mod tests;
