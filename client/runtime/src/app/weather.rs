//! Immutable world-weather frame to rendering/lighting, with the same local audio source.
use salimon_world::environment::{climate::ClimateDefinition, weather_visuals::WeatherFrame};
pub(super) fn scene(
    frame: &WeatherFrame,
    samples: u32,
    body_axes: [[f32; 3]; 3],
) -> salimon_renderer::WeatherScene {
    salimon_renderer::WeatherScene {
        optical: frame.optical.clone(),
        wind_temperature: frame.wind_temperature.clone(),
        cloud_base_meters: frame.cloud_base_meters,
        precipitation_freeze_kelvin: match frame.condensable {
            salimon_world::environment::climate::Condensable::None => 0.,
            salimon_world::environment::climate::Condensable::Water => 273.15,
            salimon_world::environment::climate::Condensable::CarbonDioxide => 194.65,
            salimon_world::environment::climate::Condensable::Methane => 90.7,
        },
        cloud_thickness_meters: frame.cloud_thickness_meters,
        time_seconds: frame.time_seconds,
        body_axes,
        view_samples: samples,
    }
}
pub(super) fn canonical() -> Result<Option<WeatherFrame>, String> {
    use std::sync::LazyLock;
    static CLIMATE: LazyLock<ClimateDefinition> = LazyLock::new(|| {
        ClimateDefinition::new(
            salimon_world::terrain::TerrainDefinition::canonical_start().geology(),
            1,
            0,
            salimon_world::environment::climate::ClimateInputs::vacuum(86400., 31557600., 1361.),
        )
        .expect("valid explicit Vesper vacuum")
    });
    let epoch = salimon_universe::Epoch::calendar(
        [2000, 1, 1],
        [12, 0, 0, 0],
        salimon_universe::TimeScale::Tt,
    )
    .map_err(|e| e.to_string())?;
    WeatherFrame::sample(&CLIMATE, epoch, [0., 1., 0.], false)
        .map_err(|e| format!("canonical weather: {e:?}"))
}
#[cfg(test)]
mod tests {
    #[test]
    fn canonical_weather_does_not_add_airborne_effects() {
        assert_eq!(super::canonical().unwrap(), None);
    }
}

#[cfg(test)]
mod evidence {
    #[test]
    #[ignore = "exports actual world/runtime weather DTO; set SALIMON_WEATHER_FRAME_FILE"]
    fn weather_integrated_frame_export() {
        use salimon_world::{
            environment::climate::{Chemistry, ClimateDefinition, ClimateInputs, Condensable},
            terrain::TerrainDefinition,
        };
        let body = *salimon_world::CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == salimon_world::CelestialBodyId::Moon)
            .unwrap();
        let mut inputs = ClimateInputs::vacuum(86400., 31557600., 1361.);
        inputs.chemistry = Chemistry::Nitrogen;
        inputs.pressure_pascal = 101325.;
        inputs.scale_height_meters = 8000.;
        inputs.condensable = Condensable::Water;
        inputs.condensable_fraction = 0.8;
        let climate = ClimateDefinition::new(
            TerrainDefinition::new(body, 1, 5, vec![], vec![])
                .unwrap()
                .geology(),
            1,
            37,
            inputs,
        )
        .unwrap();
        let epoch = salimon_universe::Epoch::calendar(
            [2000, 1, 1],
            [12, 0, 0, 0],
            salimon_universe::TimeScale::Tt,
        )
        .unwrap();
        let start = std::time::Instant::now();
        let frame = salimon_world::environment::weather_visuals::WeatherFrame::sample(
            &climate,
            epoch,
            [0., 1., 0.],
            false,
        )
        .unwrap()
        .unwrap();
        println!(
            "2048 actual climate samples: {:.3}ms",
            start.elapsed().as_secs_f64() * 1000.
        );
        let scene = super::scene(&frame, 16, [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]);
        let index = scene
            .optical
            .iter()
            .enumerate()
            .max_by(|a, b| a.1[0].total_cmp(&b.1[0]))
            .unwrap()
            .0;
        let longitude = ((index % 64) as f64 + 0.5) / 64. * std::f64::consts::TAU;
        let latitude =
            ((index / 64) as f64 + 0.5) / 32. * std::f64::consts::PI - std::f64::consts::FRAC_PI_2;
        let observer_direction = [
            latitude.cos() * longitude.cos(),
            latitude.sin(),
            latitude.cos() * longitude.sin(),
        ];
        let data = serde_json::json!({"definition":frame.definition_id,"observer_direction":observer_direction,"epoch":epoch,"optical":scene.optical,"wind_temperature":scene.wind_temperature,"time_seconds":scene.time_seconds,"cloud_base_meters":scene.cloud_base_meters,"cloud_thickness_meters":scene.cloud_thickness_meters,"precipitation_freeze_kelvin":scene.precipitation_freeze_kelvin});
        let path = std::env::var("SALIMON_WEATHER_FRAME_FILE").unwrap();
        std::fs::write(path, serde_json::to_vec_pretty(&data).unwrap()).unwrap();
    }
}
