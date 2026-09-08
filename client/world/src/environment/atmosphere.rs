//! Pressure-scaled single-scattering coefficients. Presentation only, no gas physics mutation.
use super::{
    EnvironmentError,
    climate::{Chemistry, ClimateInputs},
};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OpticalProfile {
    pub scale_height_meters: f64,
    /// Approximate visible Rayleigh extinction at sea level, RGB wavelengths 680/550/440 nm.
    pub rayleigh_per_meter: [f64; 3],
    pub ceiling_meters: f64,
}
impl OpticalProfile {
    pub fn from_climate(inputs: ClimateInputs) -> Result<Option<Self>, EnvironmentError> {
        inputs.validate()?;
        if inputs.chemistry == Chemistry::Vacuum || inputs.pressure_pascal == 0. {
            return Ok(None);
        }
        let refractivity = match inputs.chemistry {
            Chemistry::Nitrogen => 1.,
            Chemistry::CarbonDioxide => 2.4,
            Chemistry::Hydrogen => 0.24,
            Chemistry::Vacuum => 0.,
        };
        let density = inputs.pressure_pascal / 101325.;
        let rayleigh_per_meter = [5.8e-6, 13.5e-6, 33.1e-6].map(|v| v * refractivity * density);
        Ok(Some(Self {
            scale_height_meters: inputs.scale_height_meters,
            rayleigh_per_meter,
            ceiling_meters: inputs.scale_height_meters * 12.,
        }))
    }
    pub fn transmission(
        self,
        altitude_meters: f64,
        distance_meters: f64,
    ) -> Result<[f64; 3], EnvironmentError> {
        if !altitude_meters.is_finite()
            || altitude_meters < 0.
            || !distance_meters.is_finite()
            || distance_meters < 0.
        {
            return Err(EnvironmentError::InvalidDefinition);
        }
        let column = (-altitude_meters / self.scale_height_meters).exp() * distance_meters;
        Ok(self.rayleigh_per_meter.map(|b| (-b * column).exp()))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn optical_vacuum_pressure_chemistry_and_altitude_preserve_source() {
        let mut i = ClimateInputs::vacuum(86400., 31557600., 1361.);
        let original = i;
        assert_eq!(OpticalProfile::from_climate(i).unwrap(), None);
        assert_eq!(i, original);
        i.chemistry = Chemistry::Nitrogen;
        i.pressure_pascal = 101325.;
        i.scale_height_meters = 8000.;
        let p = OpticalProfile::from_climate(i).unwrap().unwrap();
        let t = p.transmission(0., 10000.).unwrap();
        assert!(t[0] > t[1] && t[1] > t[2]);
        assert!(p.transmission(80000., 10000.).unwrap()[2] > 0.9999);
        i.chemistry = Chemistry::CarbonDioxide;
        assert!(
            OpticalProfile::from_climate(i)
                .unwrap()
                .unwrap()
                .transmission(0., 10000.)
                .unwrap()[0]
                < t[0]
        );
        assert!(p.transmission(f64::NAN, 1.).is_err());
    }
}
