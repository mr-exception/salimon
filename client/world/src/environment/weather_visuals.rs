//! Deterministic presentation sampling of authoritative shared-TT weather; no simulation RNG.
use super::{
    EnvironmentError,
    climate::{Chemistry, ClimateDefinition, Condensable, WeatherSample},
    seconds,
};
use salimon_universe::Epoch;
pub const WEATHER_WIDTH: usize = 64;
pub const WEATHER_HEIGHT: usize = 32;
#[derive(Clone, Debug, PartialEq)]
pub struct WeatherFrame {
    pub definition_id: String,
    pub epoch: Epoch,
    pub time_seconds: f64,
    /// [condensed cloud fraction, storm, precipitation kg/m²/s, dust extinction/m].
    pub optical: Vec<[f32; 4]>,
    /// Body-fixed tangent wind m/s and local temperature K, same sample as optical.
    pub wind_temperature: Vec<[f32; 4]>,
    pub cloud_base_meters: f64,
    pub cloud_thickness_meters: f64,
    pub condensable: Condensable,
    pub local: WeatherSample,
}
impl WeatherFrame {
    /// Dust must be explicitly admitted by a caller with an available particulate source.
    /// This flag never asserts canonical dust or changes geology/erosion/terrain.
    pub fn sample(
        climate: &ClimateDefinition,
        epoch: Epoch,
        observer_direction: [f64; 3],
        dust_available: bool,
    ) -> Result<Option<Self>, EnvironmentError> {
        let local = climate.sample(observer_direction, epoch)?;
        let time_seconds = seconds(epoch)?;
        if climate.inputs().chemistry == Chemistry::Vacuum {
            return Ok(None);
        }
        let mut optical = Vec::with_capacity(WEATHER_WIDTH * WEATHER_HEIGHT);
        let mut wind_temperature = Vec::with_capacity(WEATHER_WIDTH * WEATHER_HEIGHT);
        for y in 0..WEATHER_HEIGHT {
            for x in 0..WEATHER_WIDTH {
                let longitude = (x as f64 + 0.5) / WEATHER_WIDTH as f64 * std::f64::consts::TAU;
                let latitude =
                    ((y as f64 + 0.5) / WEATHER_HEIGHT as f64 - 0.5) * std::f64::consts::PI;
                let direction = [
                    latitude.cos() * longitude.cos(),
                    latitude.sin(),
                    latitude.cos() * longitude.sin(),
                ];
                let w = climate.sample(direction, epoch)?;
                let dust = if dust_available {
                    w.storm_fraction * 0.0002 * (w.pressure_pascal / 101325.).min(1.)
                } else {
                    0.
                };
                optical.push([
                    w.cloud_fraction as f32,
                    w.storm_fraction as f32,
                    w.precipitation_kg_m2_s as f32,
                    dust as f32,
                ]);
                wind_temperature.push([
                    w.wind_mps[0] as f32,
                    w.wind_mps[1] as f32,
                    w.wind_mps[2] as f32,
                    w.temperature_kelvin as f32,
                ]);
            }
        }
        Ok(Some(Self {
            definition_id: climate.identity().into(),
            epoch,
            time_seconds,
            optical,
            wind_temperature,
            cloud_base_meters: climate.inputs().scale_height_meters * 0.5,
            cloud_thickness_meters: climate.inputs().scale_height_meters * 0.25,
            condensable: climate.inputs().condensable,
            local,
        }))
    }
    /// Approximate overhead cloud attenuation for presentation illumination only.
    pub fn local_light_transmission(&self) -> f32 {
        (-self.local.cloud_fraction * (2. + self.local.storm_fraction)).exp() as f32
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        environment::climate::{CLIMATE_VERSION, ClimateInputs},
        terrain::TerrainDefinition,
    };
    fn fixture() -> (ClimateDefinition, Epoch) {
        let body = crate::CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == crate::CelestialBodyId::Moon)
            .copied()
            .unwrap();
        let mut p = ClimateInputs::vacuum(86400., 31557600., 1361.);
        p.chemistry = Chemistry::Nitrogen;
        p.pressure_pascal = 101325.;
        p.scale_height_meters = 8000.;
        p.condensable = Condensable::Water;
        p.condensable_fraction = 0.8;
        (
            ClimateDefinition::new(
                TerrainDefinition::new(body, 1, 5, vec![], vec![])
                    .unwrap()
                    .geology(),
                CLIMATE_VERSION,
                37,
                p,
            )
            .unwrap(),
            Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], salimon_universe::TimeScale::Tt).unwrap(),
        )
    }
    #[test]
    fn weather_visuals_repeat_after_reconstruction_and_have_no_load_time_or_quality_input() {
        let (d, t) = fixture();
        let frame = WeatherFrame::sample(&d, t, [0., 1., 0.], false)
            .unwrap()
            .unwrap();
        let copy = ClimateDefinition::from_bytes(
            *crate::CELESTIAL_BODIES
                .iter()
                .find(|b| b.id == crate::CelestialBodyId::Moon)
                .unwrap(),
            &d.to_bytes(),
        )
        .unwrap();
        assert_eq!(
            frame,
            WeatherFrame::sample(&copy, t, [0., 1., 0.], false)
                .unwrap()
                .unwrap()
        );
        assert_eq!(frame.optical.len(), 2048);
        assert_eq!(frame.local, d.sample([0., 1., 0.], t).unwrap());
        assert!(frame.optical.iter().all(|v| v[3] == 0.));
        let dust = WeatherFrame::sample(&d, t, [0., 1., 0.], true)
            .unwrap()
            .unwrap();
        assert!(dust.optical.iter().any(|v| v[3] > 0.));
        assert!(frame.local_light_transmission() > 0. && frame.local_light_transmission() <= 1.);
        let later = WeatherFrame::sample(
            &d,
            t.advance_nanos(3_600_000_000_000).unwrap(),
            [0., 1., 0.],
            false,
        )
        .unwrap()
        .unwrap();
        assert_ne!(frame.optical, later.optical);
    }
    #[test]
    fn weather_visuals_vacuum_has_no_cloud_dust_rain_or_wind_frame() {
        let d = ClimateDefinition::new(
            TerrainDefinition::canonical_start().geology(),
            1,
            37,
            ClimateInputs::vacuum(86400., 31557600., 1361.),
        )
        .unwrap();
        let t =
            Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], salimon_universe::TimeScale::Tt).unwrap();
        assert_eq!(
            WeatherFrame::sample(&d, t, [0., 1., 0.], true).unwrap(),
            None
        );
    }
}
