use super::*;
const FIXTURE: &[u8] =
    include_bytes!("../../../../../catalogs/sources/fixtures/jpl-earth-j2000.json");
fn dataset() -> Dataset {
    Dataset {
        id: CatalogId::from_source("dataset", "jpl").unwrap(),
        version: "frozen-api-1.2-DE441".into(),
        reference: "catalogs/sources/manifest-v1.json".into(),
    }
}
fn mutated(from: &str, to: &str) -> Vec<u8> {
    String::from_utf8(FIXTURE.to_vec())
        .unwrap()
        .replace(from, to)
        .into_bytes()
}
#[test]
fn independent_reference_states_and_physical_measurements() {
    let e = Ephemeris::decode(FIXTURE, &dataset()).unwrap();
    assert_eq!(e.target().as_str(), "jpl:399");
    assert_eq!(e.samples().len(), 2);
    assert_eq!(
        e.validity(),
        Validity {
            start_julian_day: 2451544.5,
            end_julian_day: 2451545.5,
            scale: TimeScale::Tdb
        }
    );
    assert!((e.mass_kg().unwrap().value - 5.97219e24).abs() < 1e9);
    assert_eq!(e.mean_radius_meters().unwrap().value, 6_371_010.0);
    assert_eq!(
        e.mean_radius_meters().unwrap().uncertainty,
        Uncertainty::Bounds {
            minus: 20.0,
            plus: 20.0,
            confidence: None
        }
    );
    assert!(e.mass_kg().unwrap().validity.is_none());
    let suite =
        crate::reference::Suite::decode(include_bytes!("../../../fixtures/references-v1.json"))
            .unwrap();
    let cases: Vec<_> = suite
        .cases()
        .iter()
        .filter(|c| c.expected.id.starts_with("earth-jpl-"))
        .cloned()
        .collect();
    let subset = crate::reference::Suite::new(cases).unwrap();
    let actual: Vec<_> = e
        .samples()
        .iter()
        .enumerate()
        .map(|(i, s)| crate::reference::Sample {
            id: format!("earth-jpl-{}", i + 1),
            frame: "ICRF-SSB-geometric".into(),
            epoch: s.epoch,
            position_meters: s.position_meters,
            velocity_mps: s.velocity_mps,
        })
        .collect();
    assert!(subset.compare(&actual).unwrap().passed);
    assert!(e.raw_response().contains("DE441"));
    let system = CatalogId::from_source("system", "solar").unwrap();
    let body = e.body(system.clone(), system.clone(), BodyKind::Planet);
    let catalog = crate::Catalog::new(
        "solar-fixture".into(),
        vec![dataset()],
        vec![
            crate::Record::System(crate::System {
                header: Header {
                    id: system,
                    display_name: None,
                    provenance: e.provenance().clone(),
                },
            }),
            crate::Record::Body(body),
        ],
    )
    .unwrap();
    assert!(catalog.get(e.target()).is_some());
    assert!(matches!(e.provenance(),Provenance::Observed { record_key,.. } if record_key=="399"));
}
#[test]
fn exact_epochs_and_out_of_range_policy_are_explicit() {
    let e = Ephemeris::decode(FIXTURE, &dataset()).unwrap();
    assert_eq!(e.state_at(e.samples()[0].epoch).unwrap(), &e.samples()[0]);
    for date in [[1999, 12, 31], [2000, 1, 3]] {
        assert!(
            e.state_at(Epoch::calendar(date, [0, 0, 0, 0], TimeScale::Tdb).unwrap())
                .is_err()
        );
    }
    assert!(
        e.state_at(Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], TimeScale::Tdb).unwrap())
            .is_err()
    );
    assert!(
        e.state_at(Epoch::calendar([2000, 1, 1], [0, 0, 0, 0], TimeScale::Utc).unwrap())
            .is_err()
    );
}
#[test]
fn unsupported_context_corrupt_epochs_vectors_and_headers_fail_closed() {
    for (from, to) in [
        ("KM-S", "AU-D"),
        ("Reference frame : ICRF", "Reference frame : ECLIPTIC"),
        ("Solar System Barycenter (0)", "Sun (10)"),
        ("2451544.500000000", "2451544.000000000"),
        ("2000-Jan-01", "1500-Jan-01"),
        ("X =-2.627903751048988E+07", "X =NaN"),
        ("6371.01+-0.02", "6371.01+--0.02"),
        ("5.97219+-0.0006", "5.97219+-9"),
        ("\"version\":\"1.2\"", "\"version\":\"9\""),
        ("$$EOE", "missing"),
        (
            "2000-Jan-02 00:00:00.0000 TDB ",
            "2000-Jan-01 00:00:00.0000 TDB ",
        ),
    ] {
        assert!(
            Ephemeris::decode(&mutated(from, to), &dataset()).is_err(),
            "{from}"
        );
    }
    assert!(Ephemeris::decode(&vec![b' '; MAX_BYTES + 1], &dataset()).is_err());
}

#[test]
fn selected_real_targets_preserve_physical_formats_and_missing_masses() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../catalogs/sources/observed-526-v1");
    // Independently transcribed printed header measurements, not output goldens.
    for (code, radius_km, mass) in [
        (10, 695700., Some(1.988410e30)),
        (199, 2439.4, Some(3.302e23)),
        (299, 6051.84, Some(4.8685e24)),
        (399, 6371.01, Some(5.97219e24)),
        (301, 1737.53, Some(7.349e22)),
        (499, 3389.92, Some(6.4171e23)),
        (599, 69911., Some(1.89819e27)),
        (501, 1821.49, None),
        (502, 1560.8, None),
        (503, 2631.2, None),
        (504, 2410.3, None),
        (699, 58232., Some(5.6834e26)),
        (606, 2575.5, Some(1.34553e23)),
        (799, 25362., Some(8.6813e25)),
        (899, 24624., Some(1.02409e26)),
        (801, 1352.6, None),
        (999, 1188.3, Some(1.307e22)),
        (901, 606., None),
    ] {
        let bytes = std::fs::read(root.join(format!("jpl-{code}.json"))).unwrap();
        let e = Ephemeris::decode(&bytes, &dataset()).unwrap();
        assert_eq!(e.target().as_str(), format!("jpl:{code}"));
        assert_eq!(e.samples().len(), 3);
        assert!((e.mean_radius_meters().unwrap().value - radius_km * 1000.).abs() < 1e-6);
        match mass {
            Some(expected) => assert!((e.mass_kg().unwrap().value / expected - 1.).abs() < 1e-14),
            None => assert!(e.mass_kg().is_none(), "GM must not become mass for {code}"),
        }
    }
}

#[test]
fn spaced_physical_errors_are_not_dropped_and_ambiguous_fields_reject() {
    let p = Provenance::Observed {
        dataset: dataset().id,
        record_key: "502".into(),
    };
    let q = parse_physical("= 1560.8   +- 0.3 Density = 3", 1000., &p).unwrap();
    assert_eq!(q.value, 1_560_800.);
    assert_eq!(
        q.uncertainty,
        Uncertainty::Bounds {
            minus: 300.,
            plus: 300.,
            confidence: None
        }
    );
    for bad in [
        "= 1560.8 +-",
        "= 1560.8 +- -0.3",
        "= NaN",
        "= -1",
        "= ~3 +- 1",
    ] {
        assert!(parse_physical(bad, 1000., &p).is_err(), "{bad}");
    }
    assert!(
        physical_fields(
            "Mean radius (km)=1\nMean Radius (km)=2",
            &["meanradius(km)"],
            1000.,
            &p
        )
        .is_err()
    );
    assert!(mass_field("Mass x10^23 (kg)=1\nMass (10^19 kg)=2", &p).is_err());
    assert!(mass_field("Mass (10^21 kg )=\nMass x10^23 (kg)=1", &p).is_err());
    assert!(mass_field("Mass x10^23 (kg)=garbage", &p).is_err());
}
