//! World optical policy to renderer DTO. No weather/physics mutation or inferred gas.
use salimon_world::environment::{atmosphere::OpticalProfile, climate::ClimateInputs};
pub(super) fn scene(
    center: [f64; 3],
    radius: f64,
    climate: ClimateInputs,
    sun_direction: [f32; 3],
    sun_color: [f32; 3],
    solar_visibility: f32,
) -> Result<Option<salimon_renderer::AtmosphereScene>, String> {
    Ok(OpticalProfile::from_climate(climate)
        .map_err(|e| format!("atmospheric profile: {e:?}"))?
        .map(|p| salimon_renderer::AtmosphereScene {
            center_meters: center,
            radius_meters: radius,
            scale_height_meters: p.scale_height_meters,
            ceiling_meters: p.ceiling_meters,
            rayleigh_per_meter: p.rayleigh_per_meter,
            sun_direction,
            sun_color,
            solar_visibility,
        }))
}
/// Current canonical Vesper explicitly publishes zero pressure. Never add Earth-like haze.
pub(super) fn canonical() -> Result<Option<salimon_renderer::AtmosphereScene>, String> {
    let p = &salimon_world::canonical::START.manifest().physical;
    scene(
        [0.; 3],
        p.radius_meters,
        ClimateInputs::vacuum(86400., p.period_seconds, 1361.),
        [0., 1., 0.],
        [1.; 3],
        1.,
    )
}
#[cfg(test)]
mod tests {
    #[test]
    fn airless_start_has_no_atmosphere() {
        assert_eq!(super::canonical().unwrap(), None);
    }
    #[test]
    fn chemical_input_reaches_optical_dto_without_changing_world() {
        use salimon_world::environment::climate::{Chemistry, ClimateInputs};
        let mut c = ClimateInputs::vacuum(86400., 31557600., 1361.);
        c.chemistry = Chemistry::CarbonDioxide;
        c.pressure_pascal = 10000.;
        c.scale_height_meters = 11000.;
        let a = super::scene([0.; 3], 3e6, c, [0., 1., 0.], [1., 0.8, 0.5], 0.25)
            .unwrap()
            .unwrap();
        assert_eq!(a.scale_height_meters, 11000.);
        assert_eq!(a.solar_visibility, 0.25);
        assert_eq!(a.sun_color, [1., 0.8, 0.5]);
    }
}
