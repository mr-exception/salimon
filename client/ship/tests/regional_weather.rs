//! Production climate-to-atmosphere seam; no mocked weather or drag operator.
use salimon_ship::{
    Propulsion,
    atmosphere::{Aerodynamics, Atmosphere},
    surface::BodyMotion,
};
use salimon_universe::{Epoch, TimeScale, coordinates::GalacticPosition};
use salimon_world::{
    CELESTIAL_BODIES, CelestialBodyId,
    environment::climate::{Chemistry, ClimateDefinition, ClimateInputs, Condensable},
    terrain::TerrainDefinition,
};
#[test]
fn regional_weather_feeds_existing_inertial_gas_and_dissipative_drag() {
    let body = *CELESTIAL_BODIES
        .iter()
        .find(|b| b.id == CelestialBodyId::Moon)
        .unwrap();
    let inputs = ClimateInputs {
        chemistry: Chemistry::Nitrogen,
        condensable: Condensable::Water,
        pressure_pascal: 101325.0,
        scale_height_meters: 8000.0,
        rotation_seconds: 86400.0,
        year_seconds: 31557600.0,
        axial_tilt_radians: 0.4,
        stellar_flux_w_m2: 1361.0,
        albedo: 0.3,
        greenhouse_kelvin: 30.0,
        condensable_fraction: 0.8,
    };
    let climate = ClimateDefinition::new(
        TerrainDefinition::new(body, 1, 0, vec![], vec![])
            .unwrap()
            .geology(),
        1,
        37,
        inputs,
    )
    .unwrap();
    let epoch = Epoch::calendar([2026, 1, 1], [0; 4], TimeScale::Tt).unwrap();
    let weather = climate.sample([1.0, 0.0, 0.0], epoch).unwrap();
    let position = |p| GalacticPosition::new([500_000, 0, 0], p).unwrap();
    let motion = BodyMotion {
        epoch,
        center: position([0.0; 3]),
        velocity_mps: [30000.0, 0.0, 0.0],
        acceleration_mps2: [0.0; 3],
    };
    // Body axes coincide with inertial axes in this fixture; rotate exactly once in other frames.
    let air = Atmosphere::new(
        body.radius_meters,
        1.225,
        inputs.scale_height_meters,
        100000.0,
        [0.0; 3],
    )
    .unwrap()
    .sample(
        epoch,
        position([body.radius_meters, 0.0, 0.0]),
        motion,
        weather.wind_mps,
    )
    .unwrap();
    assert_eq!(
        air.velocity_mps(),
        [30000.0, weather.wind_mps[1], weather.wind_mps[2]]
    );
    let velocity = [30000.0, weather.wind_mps[1] + 100.0, weather.wind_mps[2]];
    let after = Aerodynamics::new(1.0, 100.0)
        .unwrap()
        .velocity_after(velocity, Propulsion::default(), Some(air), 10.0)
        .unwrap();
    assert!(after[1] < velocity[1] && after[1] > air.velocity_mps()[1]);
    assert_eq!(after[0], 30000.0);
    // Same publication and epoch after unloading gives exactly the same gas velocity.
    let restored = ClimateDefinition::from_bytes(body, &climate.to_bytes()).unwrap();
    assert_eq!(restored.sample([1.0, 0.0, 0.0], epoch).unwrap(), weather);
}
