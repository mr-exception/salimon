use super::*;
use crate::{
    CELESTIAL_BODIES, CelestialBodyId,
    terrain::{
        TerrainDefinition,
        geology::{GEOLOGY_VERSION, GeologyDefinition},
    },
};
use salimon_universe::TimeScale;
fn body() -> CelestialBody {
    *CELESTIAL_BODIES
        .iter()
        .find(|b| b.id == CelestialBodyId::Moon)
        .unwrap()
}
fn epoch() -> Epoch {
    Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], TimeScale::Tt).unwrap()
}
fn climate(profile: GeologyProfile) -> ClimateDefinition {
    let terrain = TerrainDefinition::new(body(), 1, 0, vec![], vec![]).unwrap();
    ClimateDefinition::new(
        GeologyDefinition::new(terrain, GEOLOGY_VERSION, 0, profile).unwrap(),
        1,
        37,
        super::super::climate::ClimateInputs::vacuum(86400.0, 31557600.0, 1361.0),
    )
    .unwrap()
}
fn inputs(kind: ActivityKind) -> ActivityInputs {
    ActivityInputs {
        kind,
        candidate_count: 32,
        heat_flux_w_m2: 0.2,
        site_radius_meters: 5.0,
        cycle_seconds: 100.0,
        active_fraction: 0.5,
    }
}
fn definition() -> ActivityDefinition {
    ActivityDefinition::new(
        climate(GeologyProfile::DryRock),
        1,
        83,
        inputs(ActivityKind::Volcanic),
    )
    .unwrap()
}
#[test]
fn activity_stable_sources_restore_streaming_and_exact_ground() {
    let d = definition();
    assert!(!d.sites().is_empty());
    let bytes = d.to_bytes();
    let copy = ActivityDefinition::from_bytes(body(), &bytes).unwrap();
    assert_eq!(d, copy);
    let all = d
        .events_within([0.0; 3], body().radius_meters + 1.0, epoch())
        .unwrap();
    let ids: std::collections::BTreeSet<_> = all.iter().map(|e| &e.site.id).collect();
    assert_eq!(ids.len(), all.len());
    for s in d.sites() {
        let physical = d.climate().geology().terrain().sample(s.direction).unwrap();
        assert_eq!(
            physical.body_local_position_meters,
            s.body_local_position_meters
        );
        assert_eq!(physical.outward_normal, s.outward_normal);
        let near = copy
            .events_within(s.body_local_position_meters, 0.0, epoch())
            .unwrap();
        assert_eq!(near.len(), 1);
        assert_eq!(near[0], d.sample(&s.id, epoch()).unwrap());
        let _ = copy.events_within([0.0; 3], 0.0, epoch()).unwrap();
        assert_eq!(
            near,
            copy.events_within(s.body_local_position_meters, 0.0, epoch())
                .unwrap()
        );
    }
    assert_eq!(bytes, d.to_bytes());
}
#[test]
fn activity_timeline_is_partition_independent_periodic_and_bounded() {
    let d = definition();
    let id = d.sites()[0].id.clone();
    let mut saw_active = false;
    let mut saw_quiet = false;
    for n in 0..100 {
        let t = epoch().advance_nanos(n * 1_000_000_000).unwrap();
        let e = d.sample(&id, t).unwrap();
        assert!((0.0..=1.0).contains(&e.intensity));
        saw_active |= e.intensity > 0.0;
        saw_quiet |= e.intensity == 0.0;
        let repeat = d
            .sample(&id, t.advance_nanos(100_000_000_000).unwrap())
            .unwrap();
        assert!((e.intensity - repeat.intensity).abs() < 1e-12);
        assert!(e.thermal_power_watts <= 0.2 * PI * 25.0 + 1e-12);
        let mut stepped = epoch();
        for _ in 0..10 {
            stepped = stepped.advance_nanos(n * 100_000_000).unwrap();
        }
        assert_eq!(e, d.sample(&id, stepped).unwrap());
    }
    assert!(saw_active && saw_quiet);
    // Smooth onset and end, including the periodic boundary.
    for seconds in [0, 50, 100] {
        let t = epoch().advance_nanos(seconds * 1_000_000_000).unwrap();
        let a = d.sample(&id, t).unwrap();
        let b = d.sample(&id, t.advance_nanos(1000).unwrap()).unwrap();
        assert!((a.intensity - b.intensity).abs() < 1e-6);
    }
}
#[test]
fn activity_vacuum_plumes_are_ballistic_and_never_airborne_sound_or_wind() {
    let d = definition();
    let id = &d.sites()[0].id;
    for second in 0..100 {
        let e = d
            .sample(id, epoch().advance_nanos(second * 1_000_000_000).unwrap())
            .unwrap();
        assert_eq!(e.acoustic_path, AcousticPath::ContactOnly);
        assert_eq!(e.local_pressure_pascal, 0.0);
        if let Some(p) = e.plume {
            assert_eq!(p.transport, PlumeTransport::BallisticVacuum);
            for i in 0..3 {
                assert_eq!(
                    p.velocity_mps[i],
                    e.site.outward_normal[i] * e.intensity * 20.0
                );
            }
        }
    }
}
#[test]
fn activity_mechanisms_require_suitable_explicit_physical_context() {
    let dry = climate(GeologyProfile::DryRock);
    assert!(
        ActivityDefinition::new(dry.clone(), 1, 83, inputs(ActivityKind::Cryovolcanic)).is_err()
    );
    assert!(ActivityDefinition::new(dry.clone(), 1, 83, inputs(ActivityKind::Seismic)).is_err());
    let mut no_heat = inputs(ActivityKind::Volcanic);
    no_heat.heat_flux_w_m2 = 0.0;
    assert!(
        ActivityDefinition::new(dry, 1, 83, no_heat)
            .unwrap()
            .sites()
            .is_empty()
    );
    let ice = ActivityDefinition::new(
        climate(GeologyProfile::Ice),
        1,
        83,
        inputs(ActivityKind::Cryovolcanic),
    )
    .unwrap();
    assert!(!ice.sites().is_empty());
    assert!(
        ice.sites()
            .iter()
            .all(|s| s.substrate == Substrate::WaterIce)
    );
    let seismic = ActivityDefinition::new(
        climate(GeologyProfile::WeatheredRock),
        1,
        83,
        inputs(ActivityKind::Seismic),
    )
    .unwrap();
    assert!(!seismic.sites().is_empty());
    for s in seismic.sites() {
        assert!(seismic.sample(&s.id, epoch()).unwrap().plume.is_none());
    }
    let frozen = crate::canonical::START.encode().unwrap();
    let c = ClimateDefinition::new(
        TerrainDefinition::canonical_start().geology(),
        1,
        0,
        super::super::climate::ClimateInputs::vacuum(86400.0, 31557600.0, 1361.0),
    )
    .unwrap();
    assert!(ActivityDefinition::new(c.clone(), 1, 83, inputs(ActivityKind::Volcanic)).is_err());
    let dormant = ActivityDefinition::new(c, 1, 83, ActivityInputs::dormant()).unwrap();
    assert!(dormant.sites().is_empty());
    assert_eq!(frozen, crate::canonical::START.encode().unwrap());
}
#[test]
fn activity_codec_errors_do_not_create_or_rekey_sources() {
    let d = definition();
    let bytes = d.to_bytes();
    for n in 0..bytes.len() {
        assert!(ActivityDefinition::from_bytes(body(), &bytes[..n]).is_err());
    }
    let mut b = bytes.clone();
    b.push(0);
    assert!(ActivityDefinition::from_bytes(body(), &b).is_err());
    let mut b = bytes.clone();
    b[8] = 2;
    assert_eq!(
        ActivityDefinition::from_bytes(body(), &b),
        Err(Error::UnsupportedVersion)
    );
    let mut b = bytes.clone();
    b[18] = 255;
    assert!(ActivityDefinition::from_bytes(body(), &b).is_err());
    let mut b = bytes;
    b[20..28].copy_from_slice(&f64::NAN.to_le_bytes());
    assert!(ActivityDefinition::from_bytes(body(), &b).is_err());
    let next = ActivityDefinition::new(d.climate.clone(), 1, 84, d.inputs()).unwrap();
    assert!(!next.compatible_with(d.identity(), d.climate()));
    assert_eq!(
        d.sample(&next.sites()[0].id, epoch()),
        Err(Error::IncompatibleDefinition)
    );
    assert!(d.events_within([f64::NAN, 0.0, 0.0], 1.0, epoch()).is_err());
    assert!(d.events_within([0.0; 3], -1.0, epoch()).is_err());
}
#[test]
fn activity_atmospheric_sources_use_the_same_weather_and_contact_medium() {
    let c = climate(GeologyProfile::DryRock);
    let mut p = c.inputs();
    p.chemistry = Chemistry::Nitrogen;
    p.pressure_pascal = 101325.0;
    let c = ClimateDefinition::new(c.geology().clone(), 1, 37, p).unwrap();
    let d = ActivityDefinition::new(c.clone(), 1, 83, inputs(ActivityKind::Vent)).unwrap();
    for second in 0..100 {
        let t = epoch().advance_nanos(second * 1_000_000_000).unwrap();
        let e = d.sample(&d.sites()[0].id, t).unwrap();
        let w = c.sample(e.site.direction, t).unwrap();
        assert_eq!(e.local_pressure_pascal, w.pressure_pascal);
        assert_eq!(e.acoustic_path, AcousticPath::AirAndContact);
        if let Some(p) = e.plume {
            assert_eq!(p.transport, PlumeTransport::Atmospheric);
            for i in 0..3 {
                assert_eq!(
                    p.velocity_mps[i],
                    e.site.outward_normal[i] * e.intensity * 20.0 + w.wind_mps[i]
                );
            }
        }
    }
}
