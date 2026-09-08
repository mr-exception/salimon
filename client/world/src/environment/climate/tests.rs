use super::*;
use crate::{CELESTIAL_BODIES, CelestialBodyId, terrain::TerrainDefinition};
use salimon_universe::TimeScale;
pub(crate) fn body() -> CelestialBody {
    *CELESTIAL_BODIES
        .iter()
        .find(|b| b.id == CelestialBodyId::Moon)
        .unwrap()
}
pub(crate) fn inputs() -> ClimateInputs {
    ClimateInputs {
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
    }
}
pub(crate) fn definition() -> ClimateDefinition {
    ClimateDefinition::new(
        TerrainDefinition::new(body(), 1, 0, vec![], vec![])
            .unwrap()
            .geology(),
        1,
        37,
        inputs(),
    )
    .unwrap()
}
pub(crate) fn epoch() -> Epoch {
    Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], TimeScale::Tt).unwrap()
}
#[test]
fn weather_roundtrip_reconstruction_offline_and_concurrent_hosts() {
    let d = definition();
    let bytes = d.to_bytes();
    let copy = ClimateDefinition::from_bytes(body(), &bytes).unwrap();
    assert_eq!(d, copy);
    let t = epoch().advance_nanos(8_000_000_000_000_000).unwrap();
    let expected = d.sample([0.0, 0.0, 1.0], t).unwrap();
    for _ in 0..5 {
        let _ = d.sample([1.0, 0.0, 0.0], epoch()).unwrap();
    }
    assert_eq!(expected, copy.sample([0.0, 0.0, 1.0], t).unwrap());
    // Stepping partition changes neither definition nor direct shared-time evaluation.
    let mut stepped = epoch();
    for _ in 0..100 {
        stepped = stepped.advance_nanos(80_000_000_000_000).unwrap();
    }
    assert_eq!(expected, d.sample([0.0, 0.0, 1.0], stepped).unwrap());
    let clients: Vec<_> = (0..4)
        .map(|_| {
            let d = d.clone();
            std::thread::spawn(move || d.sample([0.0, 0.0, 1.0], t).unwrap())
        })
        .collect();
    for c in clients {
        assert_eq!(c.join().unwrap(), expected);
    }
    assert_eq!(bytes, d.to_bytes());
}
#[test]
fn weather_vacuum_has_no_airborne_effects_at_any_epoch_or_latitude() {
    let d = ClimateDefinition::new(
        TerrainDefinition::canonical_start().geology(),
        1,
        99,
        ClimateInputs::vacuum(86400.0, 31557600.0, 1361.0),
    )
    .unwrap();
    let frozen = crate::canonical::START.encode().unwrap();
    for dpos in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]] {
        for offset in [0, 1_000_000_000_000, 8_000_000_000_000_000] {
            let s = d
                .sample(dpos, epoch().advance_nanos(offset).unwrap())
                .unwrap();
            assert_eq!(s.wind_mps, [0.0; 3]);
            assert_eq!(s.pressure_pascal, 0.0);
            assert_eq!(
                s.cloud_fraction
                    + s.storm_fraction
                    + s.extinction_per_meter
                    + s.precipitation_kg_m2_s,
                0.0
            );
            assert_eq!(s.precipitation, Condensable::None);
        }
    }
    assert_eq!(frozen, crate::canonical::START.encode().unwrap());
}
#[test]
fn weather_spherical_seams_poles_and_temporal_fronts_are_continuous() {
    let d = definition();
    for v in [
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 1.0],
        [-1.0, 0.0, 1.0],
        [0.0, 1.0, 0.0],
    ] {
        let a = d.sample(super::super::unit(v), epoch()).unwrap();
        let b = d
            .sample(
                super::super::unit([v[0] + 1e-9, v[1], v[2]]),
                epoch().advance_nanos(1000).unwrap(),
            )
            .unwrap();
        for (x, y) in [
            a.temperature_kelvin,
            a.cloud_fraction,
            a.storm_fraction,
            a.precipitation_kg_m2_s,
        ]
        .into_iter()
        .zip([
            b.temperature_kelvin,
            b.cloud_fraction,
            b.storm_fraction,
            b.precipitation_kg_m2_s,
        ]) {
            assert!((x - y).abs() < 1e-5);
        }
        for (x, y) in a.wind_mps.into_iter().zip(b.wind_mps) {
            assert!((x - y).abs() < 1e-5);
        }
    }
    let equator = d.sample([1.0, 0.0, 0.0], epoch()).unwrap();
    let pole = d.sample([0.0, 1.0, 0.0], epoch()).unwrap();
    assert!(equator.climate_temperature_kelvin > pole.climate_temperature_kelvin);
    let mut fronts = false;
    for hour in 0..168 {
        let s = d
            .sample(
                [1.0, 0.0, 0.0],
                epoch().advance_nanos(hour * 3_600_000_000_000).unwrap(),
            )
            .unwrap();
        assert!((0.0..=1.0).contains(&s.cloud_fraction));
        assert!((0.0..=1.0).contains(&s.storm_fraction));
        assert!((s.wind_mps[0]).abs() < 1e-12); // tangent
        fronts |= s.precipitation_kg_m2_s > 0.0;
    }
    assert!(fronts);
}
#[test]
fn weather_elevation_and_geological_substrate_use_authoritative_surface() {
    let b = body();
    let terrain = TerrainDefinition::new(
        b,
        1,
        0,
        vec![],
        vec![crate::terrain::Feature {
            direction: [1.0, 0.0, 0.0],
            radius_meters: 1000.0,
            elevation_meters: 1000.0,
        }],
    )
    .unwrap();
    let d = ClimateDefinition::new(terrain.geology(), 1, 37, inputs()).unwrap();
    let high = d.sample([1.0, 0.0, 0.0], epoch()).unwrap();
    let low = definition().sample([1.0, 0.0, 0.0], epoch()).unwrap();
    assert!((low.climate_temperature_kelvin - high.climate_temperature_kelvin - 6.5).abs() < 1e-10);
    assert!((high.pressure_pascal / low.pressure_pascal - (-0.125_f64).exp()).abs() < 1e-12);
}
#[test]
fn weather_codec_migrations_and_invalid_inputs_fail_closed() {
    let d = definition();
    for length in 0..d.to_bytes().len() {
        assert!(ClimateDefinition::from_bytes(body(), &d.to_bytes()[..length]).is_err());
    }
    let mut b = d.to_bytes();
    b.push(0);
    assert!(ClimateDefinition::from_bytes(body(), &b).is_err());
    let mut b = d.to_bytes();
    b[8] = 2;
    assert_eq!(
        ClimateDefinition::from_bytes(body(), &b),
        Err(Error::UnsupportedVersion)
    );
    for offset in [18, 19] {
        let mut b = d.to_bytes();
        b[offset] = 255;
        assert!(ClimateDefinition::from_bytes(body(), &b).is_err());
    }
    let mut b = d.to_bytes();
    b[20..28].copy_from_slice(&f64::NAN.to_le_bytes());
    assert!(ClimateDefinition::from_bytes(body(), &b).is_err());
    for v in [f64::NAN, -1.0, f64::INFINITY] {
        let mut p = inputs();
        p.pressure_pascal = v;
        assert!(ClimateDefinition::new(d.geology.clone(), 1, 37, p).is_err());
    }
    let changed = ClimateDefinition::new(d.geology.clone(), 1, 38, inputs()).unwrap();
    assert!(!changed.compatible_with(d.identity(), d.geology()));
    for direction in [[0.0; 3], [f64::NAN, 0.0, 1.0], [2.0, 0.0, 0.0]] {
        assert!(d.sample(direction, epoch()).is_err());
    }
    let utc = Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], TimeScale::Utc).unwrap();
    assert_eq!(d.sample([1.0, 0.0, 0.0], utc), Err(Error::InvalidEpoch));
}
#[test]
fn weather_frozen_numerical_vector() {
    let s = definition().sample([1.0, 0.0, 0.0], epoch()).unwrap();
    // Independent Stefan–Boltzmann balance at 1361 W/m2, 0.3 albedo plus declared greenhouse.
    assert!((s.climate_temperature_kelvin - 284.5781401165717).abs() < 1e-10);
    assert_eq!(s.pressure_pascal, 101325.0);
    assert!((s.storm_fraction - 0.09885014609399624).abs() < 1e-12);
    assert!((s.cloud_fraction - 0.5668975064651461).abs() < 1e-12);
    assert!((s.wind_mps[2] - 3.7793026296919323).abs() < 1e-12);
    assert!(
        (s.temperature_kelvin - (s.climate_temperature_kelvin + 6.0 - s.storm_fraction * 4.0))
            .abs()
            < 1e-12
    );
}

#[test]
fn weather_unit_tolerance_at_poles_cannot_produce_nan() {
    let s = definition()
        .sample([0.0, 1.0 + 1e-13, 0.0], epoch())
        .unwrap();
    assert!(s.temperature_kelvin.is_finite());
    assert!(s.climate_temperature_kelvin.is_finite());
    assert!(s.wind_mps.iter().all(|v| v.is_finite()));
}
