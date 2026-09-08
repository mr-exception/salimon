use super::*;
fn id(s: &str) -> CatalogId {
    CatalogId::from_source("fixture", s).unwrap()
}
fn observed() -> Provenance {
    Provenance::Observed {
        dataset: id("source"),
        record_key: "original-key".into(),
    }
}
fn inferred() -> Provenance {
    Provenance::Inferred {
        method: "fixture estimate, not scientific data".into(),
        datasets: vec![id("source")],
    }
}
fn fiction() -> Provenance {
    Provenance::Fiction {
        builder: "fixture-builder".into(),
        revision: "v1".into(),
        rationale: "Synthetic test world".into(),
    }
}
fn header(key: &str) -> Header {
    Header {
        id: id(key),
        display_name: Some("Synthetic fixture".into()),
        provenance: observed(),
    }
}
fn measured() -> Quantity {
    Quantity {
        value: 100.0,
        uncertainty: Uncertainty::Bounds {
            minus: 1.,
            plus: 2.,
            confidence: None,
        },
        provenance: observed(),
        validity: Some(Validity {
            start_julian_day: 1.,
            end_julian_day: 2.,
            scale: TimeScale::Tdb,
        }),
    }
}
fn definition() -> Definition {
    Definition {
        version: 1,
        sha256: "a".repeat(64),
        domains: ["terrain", "resources", "visual"]
            .into_iter()
            .enumerate()
            .map(|(i, name)| SeedDomain {
                name: name.into(),
                seed: i as u64,
                algorithm_version: 1,
                definition_sha256: "b".repeat(64),
                dependencies: vec![],
            })
            .collect(),
    }
}
fn fixture() -> Catalog {
    let mut planet = header("planet");
    planet.provenance = fiction();
    Catalog::new(
        "synthetic-v1".into(),
        vec![Dataset {
            id: id("source"),
            version: "fixture-v1".into(),
            reference: "synthetic fixture; no real observations".into(),
        }],
        vec![
            Record::System(System {
                header: header("system"),
            }),
            Record::Star(Star {
                header: header("star"),
                system: id("system"),
                mass_kg: Some(measured()),
                radius_meters: None,
            }),
            Record::Body(Body {
                header: planet,
                system: id("system"),
                parent: id("star"),
                body_kind: BodyKind::Planet,
                mass_kg: Some(Quantity {
                    provenance: inferred(),
                    ..measured()
                }),
                radius_meters: Some(Quantity {
                    provenance: fiction(),
                    ..measured()
                }),
                definition: Some(definition()),
            }),
            Record::Body(Body {
                header: header("moon"),
                system: id("system"),
                parent: id("planet"),
                body_kind: BodyKind::Moon,
                mass_kg: None,
                radius_meters: None,
                definition: None,
            }),
            Record::Body(Body {
                header: header("minor"),
                system: id("system"),
                parent: id("star"),
                body_kind: BodyKind::MinorBody,
                mass_kg: None,
                radius_meters: None,
                definition: None,
            }),
            Record::Population(Population {
                header: header("belt"),
                system: id("system"),
                parent: id("star"),
                population_kind: PopulationKind::Asteroids,
                estimated_count: 12345,
                descriptor: "Statistical fixture, no invented member IDs".into(),
                definition: definition(),
            }),
        ],
    )
    .unwrap()
}
fn rebuild(c: Catalog) -> Result<Catalog, Error> {
    Catalog::new(c.revision, c.datasets, c.records)
}
fn body(c: &mut Catalog) -> &mut Body {
    match c
        .records
        .iter_mut()
        .find(|r| r.id() == &id("planet"))
        .unwrap()
    {
        Record::Body(b) => b,
        _ => unreachable!(),
    }
}
#[test]
fn source_identity_is_exact_namespaced_and_independent_of_display_and_row_order() {
    assert_eq!(
        CatalogId::from_source("gaia-dr3", "1234567890123456789")
            .unwrap()
            .as_str(),
        "gaia-dr3:1234567890123456789"
    );
    assert_ne!(
        CatalogId::from_source("a", "B").unwrap(),
        CatalogId::from_source("a", "b").unwrap()
    );
    for bad in [
        "",
        "no-namespace",
        "A:123",
        "a:",
        ":b",
        "a:two words",
        "a:b:c",
        "a:é",
    ] {
        assert!(CatalogId::try_from(bad.to_string()).is_err(), "{bad}");
    }
    let c = fixture();
    let mut shuffled = c.clone();
    shuffled.records.reverse();
    assert_eq!(
        rebuild(shuffled).unwrap().encode().unwrap(),
        c.encode().unwrap()
    );
    let mut renamed = c.clone();
    body(&mut renamed).header.display_name = Some("Renamed world".into());
    assert_eq!(body(&mut renamed).header.id, id("planet"));
    assert!(c.get(&id("missing")).is_none());
}
#[test]
fn roundtrip_preserves_missing_values_uncertainties_provenance_and_population() {
    let c = fixture();
    let bytes = c.encode().unwrap();
    let decoded = Catalog::decode(&bytes).unwrap();
    assert_eq!(decoded, c);
    assert_eq!(decoded.encode().unwrap(), bytes);
    let Record::Star(s) = decoded.get(&id("star")).unwrap() else {
        panic!()
    };
    assert!(s.radius_meters.is_none());
    let mut d = decoded.clone();
    body(&mut d).mass_kg.as_mut().unwrap().uncertainty = Uncertainty::Unknown;
    assert_eq!(Catalog::decode(&d.encode().unwrap()).unwrap(), d);
    let Record::Population(p) = decoded.get(&id("belt")).unwrap() else {
        panic!()
    };
    assert_eq!(p.estimated_count, 12345);
}
#[test]
fn duplicate_id_wrong_system_dangling_parent_and_cycles_never_publish() {
    let c = fixture();
    let mut d = c.clone();
    d.records.push(d.records[0].clone());
    assert!(rebuild(d).is_err());
    let mut d = c.clone();
    d.datasets.push(d.datasets[0].clone());
    assert!(rebuild(d).is_err());
    let mut d = c.clone();
    body(&mut d).system = id("missing");
    assert!(rebuild(d).is_err());
    let mut d = c.clone();
    body(&mut d).parent = id("missing");
    assert!(rebuild(d).is_err());
    let mut d = c.clone();
    body(&mut d).system = id("star");
    assert!(rebuild(d).is_err());
    let mut d = c.clone();
    body(&mut d).parent = id("belt");
    assert!(rebuild(d).is_err());
    let mut d = c.clone();
    body(&mut d).body_kind = BodyKind::MinorBody;
    body(&mut d).parent = id("moon");
    assert!(rebuild(d).is_err());
    let mut d = c.clone();
    let mut h = header("other-system");
    h.display_name = None;
    d.records.push(Record::System(System { header: h }));
    body(&mut d).system = id("other-system");
    assert!(rebuild(d).is_err());
}
#[test]
fn invalid_quantity_uncertainty_validity_and_source_are_rejected() {
    for value in [0., -1., f64::NAN, f64::INFINITY] {
        let mut c = fixture();
        body(&mut c).mass_kg.as_mut().unwrap().value = value;
        assert!(rebuild(c).is_err());
    }
    for u in [
        Uncertainty::Bounds {
            minus: -1.,
            plus: 0.,
            confidence: None,
        },
        Uncertainty::Bounds {
            minus: 101.,
            plus: 0.,
            confidence: None,
        },
        Uncertainty::Bounds {
            minus: 0.,
            plus: f64::INFINITY,
            confidence: None,
        },
        Uncertainty::Bounds {
            minus: 0.,
            plus: 0.,
            confidence: Some(1.1),
        },
    ] {
        let mut c = fixture();
        body(&mut c).mass_kg.as_mut().unwrap().uncertainty = u;
        assert!(rebuild(c).is_err());
    }
    let mut c = fixture();
    body(&mut c)
        .mass_kg
        .as_mut()
        .unwrap()
        .validity
        .as_mut()
        .unwrap()
        .end_julian_day = 0.;
    assert!(rebuild(c).is_err());
    let mut c = fixture();
    body(&mut c).mass_kg.as_mut().unwrap().provenance = Provenance::Observed {
        dataset: id("missing"),
        record_key: "key".into(),
    };
    assert!(rebuild(c).is_err());
    let mut c = fixture();
    c.datasets.clear();
    assert!(rebuild(c).is_err());
}
#[test]
fn canonical_world_definition_requires_versioned_domain_separation_and_valid_dag() {
    let c = fixture();
    let mut d = c.clone();
    body(&mut d).definition = None;
    assert!(rebuild(d).is_err());
    let mut d = c.clone();
    body(&mut d).definition.as_mut().unwrap().domains.pop();
    assert!(rebuild(d).is_err());
    let mut d = c.clone();
    body(&mut d).definition.as_mut().unwrap().sha256 = "bad".into();
    assert!(rebuild(d).is_err());
    let mut d = c.clone();
    let def = body(&mut d).definition.as_mut().unwrap();
    def.domains.push(def.domains[0].clone());
    assert!(rebuild(d).is_err());
    let mut d = c.clone();
    body(&mut d).definition.as_mut().unwrap().domains[0]
        .dependencies
        .push("missing".into());
    assert!(rebuild(d).is_err());
    let mut d = c.clone();
    let def = body(&mut d).definition.as_mut().unwrap();
    def.domains[0].dependencies.push("resources".into());
    def.domains[1].dependencies.push("terrain".into());
    assert!(rebuild(d).is_err());
    let mut d = c;
    body(&mut d).definition.as_mut().unwrap().domains[1]
        .dependencies
        .push("terrain".into());
    assert!(rebuild(d).is_ok());
}
#[test]
fn schema_evolution_unknown_fields_duplicate_json_and_malformed_ids_reject() {
    let bytes = fixture().encode().unwrap();
    let original: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    for version in [0, 2, u32::MAX] {
        let mut v = original.clone();
        v["schema_version"] = version.into();
        assert_eq!(
            Catalog::decode(&serde_json::to_vec(&v).unwrap()).unwrap_err(),
            Error::UnsupportedSchema(version)
        );
    }
    let mut v = original.clone();
    v["format"] = "other".into();
    assert!(Catalog::decode(&serde_json::to_vec(&v).unwrap()).is_err());
    let mut v = original.clone();
    v["unrecognized"] = true.into();
    assert!(Catalog::decode(&serde_json::to_vec(&v).unwrap()).is_err());
    let mut v = original;
    v["records"][0]["record"]["header"]["id"] = "invalid".into();
    assert!(Catalog::decode(&serde_json::to_vec(&v).unwrap()).is_err());
    let duplicate = String::from_utf8(bytes).unwrap().replace(
        "\"schema_version\":1",
        "\"schema_version\":1,\"schema_version\":1",
    );
    assert!(Catalog::decode(duplicate.as_bytes()).is_err());
    assert!(Catalog::decode(b"{").is_err());
    assert!(Catalog::decode(&vec![b' '; MAX_SCHEMA_JSON_BYTES + 1]).is_err());
}
#[test]
fn fixture_revision_replacement_is_explicit_and_does_not_mutate_published_catalog() {
    let old = fixture();
    let bytes = old.encode().unwrap();
    let mut next = old.clone();
    next.revision = "synthetic-v2".into();
    body(&mut next).radius_meters.as_mut().unwrap().value = 200.;
    let next = rebuild(next).unwrap();
    assert_eq!(old.encode().unwrap(), bytes);
    assert_ne!(old.revision(), next.revision());
    assert_eq!(
        old.get(&id("planet")).unwrap().id(),
        next.get(&id("planet")).unwrap().id()
    );
}

#[test]
fn checked_in_schema_one_fixture_remains_compatible_and_canonical() {
    let fixture = include_str!("../fixtures/schema-v1.json").trim_end();
    let c = Catalog::decode(fixture.as_bytes()).unwrap();
    assert_eq!(c.revision(), "fixture-v1");
    assert_eq!(c.datasets().len(), 1);
    assert_eq!(c.records().len(), 2);
    assert_eq!(String::from_utf8(c.encode().unwrap()).unwrap(), fixture);
}
