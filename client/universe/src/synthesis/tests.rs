use super::*;

fn base() -> Catalog {
    Catalog::decode(include_bytes!("../../fixtures/schema-v1.json")).unwrap()
}
pub(super) fn request() -> Request {
    Request {
        version: VERSION,
        seed: 42,
        revision: "fiction-test-v1".into(),
        rationale: "synthetic reviewed region; not production Milky Way calibration".into(),
        regions: vec![Region {
            key: "disk".into(),
            center: GalacticPosition::new([900_000_000, 0, 0], [0.0; 3]).unwrap(),
            extent_parsecs: [2.0; 3],
            residual_systems_per_cubic_parsec: 1.0,
            age_gyr: [1.0, 12.0],
            metallicity_feh: [-2.5, 0.5],
        }],
    }
}
#[test]
fn hash_and_lane_golden() {
    assert_eq!(
        sha256(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    // Independently computed with Python hashlib and length-prefixed UTF8 parts.
    assert_eq!(lane(42, "disk.s000000", "terrain"), 10521425917048484930);
}
#[test]
fn cubic_publication_arithmetic_has_frozen_ieee_rounding() {
    // Independent Python binary64 goldens for (x*x)*x, covering body radii,
    // planetary axes and large distances. Platform libm powi is not a contract.
    for (input, bits) in [
        (2_345_678.0, 0x43e663980401b6eb),
        (71_345_600_000.0, 0x46b1e7c2e912837c),
        (1.23456789012345e16, 0x49f499949687193a),
    ] {
        assert_eq!(cube(input).to_bits(), bits);
    }
}
#[test]
fn preserves_observations_and_hashes_all_recipe_content() {
    let b = base();
    let p = build(&b, request()).unwrap();
    assert_eq!(
        request(),
        serde_json::from_str::<Request>(include_str!("../../fixtures/fiction-request-v1.json"))
            .unwrap()
    );
    assert_eq!(
        p.sha256().unwrap(),
        "97d2c46654d7afc5e4aac03fdf94358e9182f0b8ff38251990a802684ea632f1"
    );
    assert_eq!(p.sources().len(), 8);
    for record in b.records() {
        assert_eq!(p.catalog().get(record.id()), Some(record));
    }
    assert_eq!(b.datasets(), p.catalog().datasets());
    assert_eq!(
        p.encode().unwrap(),
        build(&b, request()).unwrap().encode().unwrap()
    );
    assert_ne!(p.sha256().unwrap(), p.catalog_sha256().unwrap());
    for (id, recipe) in p.bodies() {
        let Some(Record::Body(body)) = p.catalog().get(id) else {
            panic!("missing body")
        };
        assert_eq!(
            body.definition.as_ref().unwrap().sha256,
            sha256(&json(recipe).unwrap())
        );
        for domain in &recipe.domains {
            assert_eq!(
                domain.definition_sha256,
                sha256(
                    &json(&(
                        BUILDER,
                        id,
                        &domain.name,
                        domain.seed,
                        recipe.radius_meters,
                        recipe.bulk_density_kg_per_m3
                    ))
                    .unwrap()
                )
            );
        }
        assert!(matches!(body.header.provenance, Provenance::Fiction { .. }));
        assert_eq!(
            body.mass_kg.as_ref().unwrap().provenance,
            body.header.provenance
        );
    }
    for (id, recipe) in p.populations() {
        let Some(Record::Population(pop)) = p.catalog().get(id) else {
            panic!("missing population")
        };
        assert_eq!(pop.definition.sha256, sha256(&json(recipe).unwrap()));
        assert_eq!(
            recipe.estimated_count,
            if recipe.population_kind == PopulationKind::Asteroids {
                99_999
            } else {
                1_000_000
            }
        );
    }
    assert!(p.bodies().values().any(|b| b.body_kind == BodyKind::Moon));
    assert!(
        p.bodies()
            .values()
            .any(|b| b.body_kind == BodyKind::MinorBody)
    );
    assert_eq!(
        Catalog::decode(&p.catalog().encode().unwrap()).unwrap(),
        *p.catalog()
    );
}
#[test]
fn region_order_and_unrelated_region_do_not_rekey_existing_worlds() {
    let b = base();
    let original = build(&b, request()).unwrap();
    let mut r = request();
    let mut extra = r.regions[0].clone();
    extra.key = "bulge".into();
    extra.center = extra
        .center
        .translated([10.0 * PARSEC_METERS, 0.0, 0.0])
        .unwrap();
    r.regions.push(extra);
    let a = build(&b, r.clone()).unwrap();
    r.regions.reverse();
    let z = build(&b, r).unwrap();
    assert_eq!(a.encode().unwrap(), z.encode().unwrap());
    for (id, body) in original.bodies() {
        assert_eq!(a.bodies().get(id), Some(body));
    }
    for source in original.sources() {
        assert!(a.sources().contains(source));
    }
    let recipe = original.bodies().values().next().unwrap();
    assert_ne!(recipe.domains[0].seed, recipe.domains[1].seed);
    let mut r = request();
    r.seed += 1;
    let changed = build(&b, r).unwrap();
    assert_eq!(
        original
            .sources()
            .iter()
            .map(|s| &s.system)
            .collect::<Vec<_>>(),
        changed
            .sources()
            .iter()
            .map(|s| &s.system)
            .collect::<Vec<_>>()
    );
    assert_ne!(original.sha256().unwrap(), changed.sha256().unwrap());
}
#[test]
fn refuses_invalid_inputs_and_namespace_collisions_atomically() {
    let b = base();
    let bytes = b.encode().unwrap();
    let mut bad = vec![];
    let r = request();
    let mut x = r.clone();
    x.version = 2;
    bad.push(x);
    let mut x = r.clone();
    x.revision = b.revision().into();
    bad.push(x);
    let mut x = r.clone();
    x.rationale = "".into();
    bad.push(x);
    let mut x = r.clone();
    x.regions.push(x.regions[0].clone());
    bad.push(x);
    let mut x = r.clone();
    let mut overlap = x.regions[0].clone();
    overlap.key = "overlap".into();
    x.regions.push(overlap);
    bad.push(x);
    let mut x = r.clone();
    x.regions[0].key = "Bad.key".into();
    bad.push(x);
    let mut x = r.clone();
    x.regions.clear();
    bad.push(x);
    for value in [f64::NAN, f64::INFINITY, -1.0, 1001.0] {
        let mut x = r.clone();
        x.regions[0].residual_systems_per_cubic_parsec = value;
        bad.push(x);
    }
    for value in [0.0, -1.0, f64::NAN, 1001.0] {
        let mut x = r.clone();
        x.regions[0].extent_parsecs[0] = value;
        bad.push(x);
    }
    for pair in [[-1.0, 1.0], [5.0, 1.0], [1.0, 14.0], [f64::NAN, 1.0]] {
        let mut x = r.clone();
        x.regions[0].age_gyr = pair;
        bad.push(x);
    }
    let mut x = r.clone();
    x.regions[0].metallicity_feh = [-3.0, 0.0];
    bad.push(x);
    let mut x = r.clone();
    x.regions[0].residual_systems_per_cubic_parsec = 1000.0;
    bad.push(x);
    for x in bad {
        assert!(build(&b, x).is_err());
        assert_eq!(b.encode().unwrap(), bytes);
    }
    let mut datasets = b.datasets().to_vec();
    datasets.push(crate::Dataset {
        id: id("reserved").unwrap(),
        version: "1".into(),
        reference: "collision test".into(),
    });
    let occupied = Catalog::new("occupied".into(), datasets, b.records().to_vec()).unwrap();
    assert!(build(&occupied, r.clone()).is_err());
    let already = build(&b, r.clone()).unwrap();
    assert!(build(already.catalog(), r).is_err());
}
#[test]
fn density_count_zero_fraction_and_prefix_stability() {
    let b = base();
    let mut r = request();
    r.regions[0].residual_systems_per_cubic_parsec = 0.0;
    let zero = build(&b, r.clone()).unwrap();
    assert!(zero.sources().is_empty());
    assert_eq!(zero.catalog().records(), b.records());
    r.regions[0].residual_systems_per_cubic_parsec = 0.1;
    assert!(build(&b, r.clone()).unwrap().sources().len() <= 1);
    r.regions[0].residual_systems_per_cubic_parsec = 1.0;
    let eight = build(&b, r.clone()).unwrap();
    r.regions[0].residual_systems_per_cubic_parsec = 2.0;
    let sixteen = build(&b, r).unwrap();
    assert_eq!(sixteen.sources().len(), 16);
    for (id, recipe) in eight.bodies() {
        assert_eq!(sixteen.bodies().get(id), Some(recipe));
    }
}
#[test]
fn physical_and_population_sanity_over_fixed_seed_regions() {
    let b = base();
    let mut low_planets = 0;
    let mut high_planets = 0;
    let mut low_giants = 0;
    let mut high_giants = 0;
    let mut young_mass = 0.0;
    let mut old_mass = 0.0;
    for seed in 0..32 {
        for high in [false, true] {
            let mut r = request();
            r.seed = seed;
            r.regions[0].metallicity_feh = if high { [0.5, 0.5] } else { [-2.5, -2.5] };
            r.regions[0].age_gyr = if high { [12.0, 12.0] } else { [0.01, 0.01] };
            let p = build(&b, r.clone()).unwrap();
            for source in p.sources() {
                let Some(Record::Star(star)) = p.catalog().get(&source.star) else {
                    panic!("missing host")
                };
                let sm = star.mass_kg.as_ref().unwrap().value;
                let sr = star.radius_meters.as_ref().unwrap().value;
                assert!(
                    source.age_seconds / YEAR_SECONDS / 1e9
                        <= 10.0 * (sm / SOLAR_MASS_KG).powf(-2.5) + 1e-12
                );
                if high {
                    old_mass += sm;
                } else {
                    young_mass += sm;
                }
                let offset = source
                    .position
                    .relative_meters(r.regions[0].center)
                    .unwrap();
                assert!(offset.iter().all(|x| x.abs() < PARSEC_METERS));
                let mut planets = vec![];
                for body in p.bodies().values().filter(|v| v.parent == source.star) {
                    let a = body.orbit.semi_major_axis_meters;
                    assert!(a > sr + body.radius_meters);
                    let residual =
                        body.orbit.period_seconds.powi(2) / (a.powi(3)) * G * (sm + body.mass_kg)
                            / std::f64::consts::TAU.powi(2)
                            - 1.0;
                    assert!(residual.abs() < 1e-12);
                    if body.body_kind == BodyKind::Planet {
                        planets.push(body);
                        if high {
                            high_planets += 1;
                            high_giants += usize::from(body.radius_meters > 50e6);
                        } else {
                            low_planets += 1;
                            low_giants += usize::from(body.radius_meters > 50e6);
                        }
                    }
                }
                planets.sort_by(|a, b| {
                    a.orbit
                        .semi_major_axis_meters
                        .total_cmp(&b.orbit.semi_major_axis_meters)
                });
                for pair in planets.windows(2) {
                    let (a, b) = (pair[0], pair[1]);
                    let aa = a.orbit.semi_major_axis_meters;
                    let ab = b.orbit.semi_major_axis_meters;
                    let hill = ((a.mass_kg + b.mass_kg) / (3.0 * sm)).cbrt() * (aa + ab) / 2.0;
                    assert!(ab - aa > 3.5 * hill);
                }
            }
            for body in p.bodies().values() {
                let Some(parent) = p.catalog().get(&body.parent) else {
                    panic!("missing parent")
                };
                let pm = match parent {
                    Record::Star(s) => s.mass_kg.as_ref().unwrap().value,
                    Record::Body(b) => b.mass_kg.as_ref().unwrap().value,
                    _ => panic!("bad parent"),
                };
                assert!(
                    body.orbit.semi_major_axis_meters
                        > 2.44 * body.radius_meters * (pm / body.mass_kg).cbrt()
                );
                if body.body_kind == BodyKind::Moon {
                    let parent = p.bodies().get(&body.parent).unwrap();
                    let Some(Record::Star(star)) = p.catalog().get(&parent.parent) else {
                        panic!("missing grandparent")
                    };
                    let hill = parent.orbit.semi_major_axis_meters
                        * (pm / (3.0 * star.mass_kg.as_ref().unwrap().value)).cbrt();
                    assert!(body.orbit.semi_major_axis_meters < 0.2 * hill);
                }
            }
        }
    }
    assert!(high_planets > low_planets);
    assert!(high_giants > low_giants);
    assert!(old_mass < young_mass);
}
