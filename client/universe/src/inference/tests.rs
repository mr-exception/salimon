use super::*;
use crate::{Body, Dataset, Header, Star, System};

fn id(s: &str) -> CatalogId {
    CatalogId::from_source("test", s).unwrap()
}
fn provenance() -> Provenance {
    Provenance::Observed {
        dataset: id("source"),
        record_key: "fixture".into(),
    }
}
fn q(value: f64) -> Quantity {
    Quantity {
        value,
        uncertainty: Uncertainty::Unknown,
        provenance: provenance(),
        validity: None,
    }
}
fn fixture() -> (Catalog, SystemInput) {
    let header = |s| Header {
        id: id(s),
        display_name: None,
        provenance: provenance(),
    };
    let c = Catalog::new(
        "input-v1".into(),
        vec![Dataset {
            id: id("source"),
            version: "1".into(),
            reference: "synthetic test".into(),
        }],
        vec![
            Record::System(System {
                header: header("system"),
            }),
            Record::Star(Star {
                header: header("star"),
                system: id("system"),
                mass_kg: None,
                radius_meters: None,
            }),
            Record::Body(Body {
                header: header("planet"),
                system: id("system"),
                parent: id("star"),
                body_kind: BodyKind::Planet,
                mass_kg: None,
                radius_meters: None,
                definition: None,
            }),
        ],
    )
    .unwrap();
    let input = SystemInput {
        host: id("star"),
        model: StellarModel::MainSequenceV1,
        seed: 410,
        age_seconds: None,
        planets: vec![PlanetInput {
            planet: id("planet"),
            model: PlanetModel::RockyV1,
            semi_major_axis_meters: None,
            period_seconds: None,
            eccentricity: None,
            minimum_mass_kg: None,
        }],
    };
    (c, input)
}
fn run(c: &Catalog, i: &SystemInput) -> Completion {
    complete(c, i, "inferred-v1".into()).unwrap()
}
fn with_physical(
    c: &Catalog,
    star: bool,
    mass: Option<Quantity>,
    radius: Option<Quantity>,
) -> Catalog {
    let mut rs = c.records().to_vec();
    for r in &mut rs {
        match r {
            Record::Star(s) if star => {
                s.mass_kg = mass.clone();
                s.radius_meters = radius.clone();
            }
            Record::Body(b) if !star => {
                b.mass_kg = mass.clone();
                b.radius_meters = radius.clone();
            }
            _ => {}
        }
    }
    Catalog::new(c.revision().into(), c.datasets().to_vec(), rs).unwrap()
}
#[test]
fn fixed_seed_reproducibility_domain_separation_and_golden() {
    let (c, mut i) = fixture();
    let before = c.encode().unwrap();
    let a = run(&c, &i);
    assert_eq!(a, run(&c, &i));
    assert_eq!(c.encode().unwrap(), before);
    assert_ne!(
        uniform(410, 0, "test:star", "mass_kg"),
        uniform(410, 0, "test:star", "radius_meters")
    );
    // Frozen integer-hash output independent of platform DefaultHasher.
    assert_eq!(
        uniform(410, 0, "test:star", "mass_kg").to_bits(),
        4606107736618726978
    );
    i.seed += 1;
    assert_ne!(a, run(&c, &i));
}
#[test]
fn preserves_all_present_values_errors_validity_and_identity() {
    let (c, mut i) = fixture();
    let mut m = q(SOLAR_MASS_KG);
    m.uncertainty = Uncertainty::Bounds {
        minus: 1e28,
        plus: 2e28,
        confidence: Some(0.68),
    };
    m.validity = Some(crate::Validity {
        start_julian_day: 1.0,
        end_julian_day: 2.0,
        scale: crate::TimeScale::Tdb,
    });
    let c = with_physical(&c, true, Some(m), Some(q(SOLAR_RADIUS_METERS)));
    let c = with_physical(
        &c,
        false,
        Some(q(EARTH_MASS_KG)),
        Some(q(EARTH_RADIUS_METERS)),
    );
    i.age_seconds = Some(q(4.5e9 * YEAR_SECONDS));
    i.planets[0].semi_major_axis_meters = Some(q(AU_METERS));
    i.planets[0].eccentricity = Some(q(0.0));
    let a = run(&c, &i);
    assert_eq!(a.catalog.records(), c.records());
    assert_eq!(a.age_seconds, i.age_seconds.unwrap());
    assert_eq!(
        a.orbits[0].semi_major_axis_meters,
        i.planets[0].semi_major_axis_meters.clone().unwrap()
    );
    assert_eq!(a.orbits[0].eccentricity, q(0.0));
    assert_eq!(a.evidence.len(), 1);
    assert_eq!(a.evidence[0].field, "period_seconds");
    assert!((a.orbits[0].period_seconds.value / YEAR_SECONDS - 1.0).abs() < 0.0001);
}
#[test]
fn kepler_inverse_and_inconsistent_measurements() {
    let (c, mut i) = fixture();
    let c = with_physical(
        &c,
        true,
        Some(q(SOLAR_MASS_KG)),
        Some(q(SOLAR_RADIUS_METERS)),
    );
    i.planets[0].period_seconds = Some(q(YEAR_SECONDS));
    let a = run(&c, &i);
    assert!((a.orbits[0].semi_major_axis_meters.value / AU_METERS - 1.0).abs() < 0.001);
    i.planets[0].semi_major_axis_meters = Some(q(0.5 * AU_METERS));
    assert!(complete(&c, &i, "new".into()).is_err());
}
#[test]
fn constraints_over_many_fixed_seeds_and_both_planet_classes() {
    let (c, mut i) = fixture();
    for model in [PlanetModel::RockyV1, PlanetModel::GasGiantV1] {
        i.planets[0].model = model;
        for seed in 0..512 {
            i.seed = seed;
            let a = run(&c, &i);
            let Some(Record::Star(s)) = a.catalog.get(&i.host) else {
                panic!()
            };
            let Some(Record::Body(b)) = a.catalog.get(&i.planets[0].planet) else {
                panic!()
            };
            let (sm, sr, pm, pr) = (
                s.mass_kg.as_ref().unwrap().value,
                s.radius_meters.as_ref().unwrap().value,
                b.mass_kg.as_ref().unwrap().value,
                b.radius_meters.as_ref().unwrap().value,
            );
            assert!(physical(pm, pr, model));
            let o = &a.orbits[0];
            let peri = o.semi_major_axis_meters.value * (1.0 - o.eccentricity.value);
            assert!(peri > sr + pr && peri > 2.44 * pr * (sm / pm).cbrt());
            assert!(
                a.age_seconds.value
                    <= (10e9 * YEAR_SECONDS * (sm / SOLAR_MASS_KG).powf(-2.5))
                        .min(13.8e9 * YEAR_SECONDS)
            );
            for e in a.evidence {
                assert!(e.quantity.value.is_finite());
                assert!(
                    matches!(e.quantity.provenance,Provenance::Inferred {ref method,ref datasets} if method.starts_with(MODEL_VERSION) && datasets==&vec![id("source")])
                );
                if let Uncertainty::Bounds { confidence, .. } = e.quantity.uncertainty {
                    assert_eq!(confidence, None);
                }
            }
        }
    }
}
#[test]
fn conditional_mass_radius_branches_and_minimum_mass() {
    let (c, mut i) = fixture();
    for (mass, radius) in [
        (Some(q(EARTH_MASS_KG)), None),
        (None, Some(q(EARTH_RADIUS_METERS))),
    ] {
        let c = with_physical(&c, false, mass.clone(), radius.clone());
        let a = run(&c, &i);
        let Some(Record::Body(b)) = a.catalog.get(&i.planets[0].planet) else {
            panic!()
        };
        if let Some(m) = mass {
            assert_eq!(b.mass_kg.as_ref(), Some(&m));
        }
        if let Some(r) = radius {
            assert_eq!(b.radius_meters.as_ref(), Some(&r));
        }
    }
    let c = with_physical(&c, true, None, Some(q(SOLAR_RADIUS_METERS)));
    run(&c, &i);
    i.planets[0].minimum_mass_kg = Some(q(100.0 * EARTH_MASS_KG));
    assert!(complete(&c, &i, "new".into()).is_err());
}
#[test]
fn malformed_nonfinite_unsupported_and_extreme_inputs_fail_without_panics() {
    let (c, i) = fixture();
    for x in [f64::NAN, f64::INFINITY, -1.0, 0.0] {
        let mut i = i.clone();
        i.age_seconds = Some(q(x));
        assert!(complete(&c, &i, "new".into()).is_err());
    }
    for x in [f64::NAN, 1.0, -0.01] {
        let mut i = i.clone();
        i.planets[0].eccentricity = Some(q(x));
        assert!(complete(&c, &i, "new".into()).is_err());
    }
    for x in [1e-200, 1e300] {
        let c = with_physical(&c, true, None, Some(q(x)));
        assert!(complete(&c, &i, "new".into()).is_err());
    }
    let mut bad = i.clone();
    bad.planets.push(bad.planets[0].clone());
    assert!(complete(&c, &bad, "new".into()).is_err());
    bad = i.clone();
    bad.planets.clear();
    assert!(complete(&c, &bad, "new".into()).is_err());
    bad = i.clone();
    bad.age_seconds = Some(q(14e9 * YEAR_SECONDS));
    assert!(complete(&c, &bad, "new".into()).is_err());
    bad = i.clone();
    bad.planets[0].semi_major_axis_meters = Some(q(1.0));
    assert!(complete(&c, &bad, "new".into()).is_err());
    bad = i.clone();
    bad.age_seconds = Some(Quantity {
        provenance: Provenance::Observed {
            dataset: id("missing"),
            record_key: "x".into(),
        },
        ..q(1e9 * YEAR_SECONDS)
    });
    assert!(complete(&c, &bad, "new".into()).is_err());
    assert!(complete(&c, &i, c.revision().into()).is_err());
}
#[test]
fn multi_planet_order_independence_and_crossing_orbits_reject() {
    let (c, mut i) = fixture();
    let mut rs = c.records().to_vec();
    let Some(Record::Body(mut b)) = c.get(&id("planet")).cloned() else {
        panic!()
    };
    b.header.id = id("planet2");
    rs.push(Record::Body(b));
    let c = Catalog::new(c.revision().into(), c.datasets().to_vec(), rs).unwrap();
    let mut p = i.planets[0].clone();
    p.planet = id("planet2");
    i.planets.push(p);
    i.planets[0].semi_major_axis_meters = Some(q(AU_METERS));
    i.planets[1].semi_major_axis_meters = Some(q(10.0 * AU_METERS));
    let a = run(&c, &i);
    i.planets.reverse();
    assert_eq!(a, run(&c, &i));
    i.planets[0].semi_major_axis_meters = Some(q(AU_METERS));
    assert!(complete(&c, &i, "new".into()).is_err());
    // A second star is unsupported, even when no planet explicitly references it.
    let mut rs = c.records().to_vec();
    let Some(Record::Star(mut s)) = c.get(&id("star")).cloned() else {
        panic!()
    };
    s.header.id = id("star2");
    rs.push(Record::Star(s));
    let c = Catalog::new(c.revision().into(), c.datasets().to_vec(), rs).unwrap();
    assert!(complete(&c, &i, "new".into()).is_err());
}
#[test]
fn actual_u08_proxima_fixture_requires_explicit_limit_review() {
    use crate::ingestion::{exoplanets, gaia::StellarCrossMatch};
    let imported = exoplanets::import(
        &include_bytes!("../../../../catalogs/sources/fixtures/exoplanet-proxima.csv")[..],
        Dataset {
            id: id("source"),
            version: "fixture".into(),
            reference: "NASA PS frozen fixture".into(),
        },
        "observed".into(),
        None,
        &StellarCrossMatch::default(),
    )
    .unwrap();
    let o = &imported.observations[0];
    let (_, mut i) = fixture();
    i.host = o.host.clone();
    i.planets[0].planet = o.planet.clone();
    assert!(
        complete_observed(
            &imported.catalog,
            &i,
            &imported.observations,
            "completed".into()
        )
        .is_err()
    );
    // Test-only explicit review of the missing limit flag; original frozen source
    // and its U08 observation stay unchanged. No production-source approval.
    let mut reviewed = imported.observations.clone();
    reviewed[0].orbital_period_seconds.as_mut().unwrap().limit = exoplanets::Limit::Measurement;
    let a = complete_observed(&imported.catalog, &i, &reviewed, "completed".into()).unwrap();
    assert_eq!(
        a.orbits[0].period_seconds,
        o.orbital_period_seconds.as_ref().unwrap().quantity
    );
    reviewed[0].orbital_period_seconds.as_mut().unwrap().limit = exoplanets::Limit::Upper;
    assert!(complete_observed(&imported.catalog, &i, &reviewed, "completed".into()).is_err());
}
#[test]
fn u08_adapter_preserves_true_mass_errors_minimum_mass_and_rejects_conflicts() {
    use crate::ingestion::{exoplanets, gaia::StellarCrossMatch};
    let csv=b"pl_name,hostname,pl_refname,st_mass,st_masslim,st_rad,st_radlim,pl_massj,pl_massjlim,pl_radj,pl_radjlim,pl_orbper,pl_orbperlim,pl_msinij,pl_msinijlim\nHost b,Host,synthetic,1,0,1,0,,,0.08912,0,365.25,0,0.003,0\n";
    let imported = exoplanets::import(
        &csv[..],
        Dataset {
            id: id("source"),
            version: "synthetic".into(),
            reference: "synthetic".into(),
        },
        "observed".into(),
        None,
        &StellarCrossMatch::default(),
    )
    .unwrap();
    let o = &imported.observations[0];
    let (_, mut i) = fixture();
    i.host = o.host.clone();
    i.planets[0].planet = o.planet.clone();
    let a = complete_observed(
        &imported.catalog,
        &i,
        &imported.observations,
        "completed".into(),
    )
    .unwrap();
    let Some(Record::Body(b)) = a.catalog.get(&o.planet) else {
        panic!()
    };
    assert!(
        b.mass_kg.as_ref().unwrap().value >= o.minimum_mass_kg.as_ref().unwrap().quantity.value
    );
    assert_eq!(
        b.radius_meters.as_ref(),
        Some(&o.planet_radius_meters.as_ref().unwrap().quantity)
    );
    assert!(matches!(
        b.mass_kg.as_ref().unwrap().provenance,
        Provenance::Inferred { .. }
    ));
    let mut obs = imported.observations.clone();
    obs[0].host_mass_kg.as_mut().unwrap().quantity.value = 0.5 * SOLAR_MASS_KG;
    assert!(complete_observed(&a.catalog, &i, &obs, "next".into()).is_err());
}
