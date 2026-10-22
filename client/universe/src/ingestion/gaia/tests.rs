use super::*;
const FIXTURE: &[u8] =
    include_bytes!("../../../../../catalogs/sources/fixtures/gaia-synthetic.csv");
fn dataset() -> Dataset {
    Dataset {
        id: CatalogId::from_source("dataset", "gaia-dr3").unwrap(),
        version: "synthetic-v1".into(),
        reference: "catalogs/sources/manifest-v1.json".into(),
    }
}
fn run(bytes: &[u8], matches: &StellarCrossMatch) -> Result<ImportedGaia, Error> {
    import(
        bytes,
        dataset(),
        "fixture-v1".into(),
        QualityPolicy::default(),
        matches,
    )
}
#[test]
fn golden_exact_ids_missing_motion_and_negative_parallax() {
    let imported = run(FIXTURE, &StellarCrossMatch::default()).unwrap();
    assert_eq!(imported.summary.input_rows, 2);
    assert_eq!(imported.summary.canonical_stars, Some(2));
    assert_eq!(imported.summary.missing_ruwe, 2);
    assert_eq!(imported.summary.missing_or_nonpositive_parallax, 1);
    assert_eq!(imported.summary.incomplete_velocity, 2);
    let rows: Vec<_> = imported.observations.values().flatten().collect();
    assert_eq!(rows[0].source.as_str(), "gaia_dr3:1000000000000000001");
    assert_eq!(
        rows[0].parallax_mas.as_ref().unwrap().standard_error,
        Some(0.5)
    );
    assert_eq!(rows[1].parallax_mas.as_ref().unwrap().value, -0.1);
    assert!(rows[1].pmra_mas_per_year.is_none());
    assert!(rows[0].radial_velocity_km_per_second.is_none());
    assert_eq!(rows[1].source_fields["parallax"], "-0.1");
    assert!(
        matches!(&rows[1].provenance, Provenance::Observed { record_key, .. } if record_key == "1000000000000000002")
    );
    assert_eq!(
        Catalog::decode(&imported.catalog.encode().unwrap()).unwrap(),
        imported.catalog
    );
}
#[test]
fn reviewed_crossmatch_and_duplicate_resolution_are_order_independent() {
    let a = CatalogId::from_source("gaia_dr3", "1000000000000000001").unwrap();
    let b = CatalogId::from_source("gaia_dr3", "1000000000000000002").unwrap();
    let canonical = CatalogId::from_source("star", "fixture").unwrap();
    let links =
        StellarCrossMatch::new([(a.clone(), canonical.clone()), (b, canonical.clone())]).unwrap();
    let text = std::str::from_utf8(FIXTURE).unwrap();
    let lines: Vec<_> = text.lines().collect();
    let reordered = format!("{}\n{}\n{}\n{}\n", lines[0], lines[2], lines[1], lines[1]);
    let actual = run(reordered.as_bytes(), &links).unwrap();
    let expected = run(FIXTURE, &links).unwrap();
    assert_eq!(
        actual.catalog.encode().unwrap(),
        expected.catalog.encode().unwrap()
    );
    assert_eq!(actual.observations, expected.observations);
    assert_eq!(actual.summary.duplicate_rows, 1);
    assert_eq!(actual.summary.canonical_stars, Some(1));
    assert_eq!(actual.observations[&canonical].len(), 2);
    assert!(
        StellarCrossMatch::new([
            (a.clone(), canonical),
            (a, CatalogId::from_source("star", "other").unwrap())
        ])
        .is_err()
    );
    let conflict = format!("{text}{}\n", lines[1].replace("100.0", "99.0"));
    assert!(run(conflict.as_bytes(), &links).is_err());
}
#[test]
fn quality_counts_keep_uncertain_distances_but_reject_bad_positions() {
    let input = "source_id,ref_epoch,ra,dec,parallax,parallax_error,pmra,pmdec,radial_velocity,ruwe,duplicated_source\n1,2016,0,0,,,,,,1.2,false\n2,2016,0,0,-1,2,,,,1.5,false\n3,2016,0,0,1,2,,,,1.0,true\n4,2016,360,0,,,,,,,\n5,2016,,0,,,,,,,\n";
    let r = run(input.as_bytes(), &StellarCrossMatch::default()).unwrap();
    assert_eq!(r.summary.input_rows, 5);
    assert_eq!(r.summary.selected_rows, 1);
    assert_eq!(r.summary.rejected_quality, 2);
    assert_eq!(r.summary.rejected_astrometry, 2);
    assert!(
        r.observations.values().next().unwrap()[0]
            .parallax_mas
            .is_none()
    );
}
#[test]
fn malformed_data_and_sink_failure_never_succeed() {
    let text = std::str::from_utf8(FIXTURE).unwrap();
    for bad in [
        text.replace("100.0", "NaN"),
        text.replace("0.5", "-0.5"),
        text.replace("1000000000000000001", "1e18"),
        text.replace("100.0", ""),
        text.replace("1000000000000000001", "0001"),
    ] {
        assert!(run(bad.as_bytes(), &StellarCrossMatch::default()).is_err());
    }
    assert!(
        stream(FIXTURE, &dataset(), QualityPolicy::default(), |_| Err(
            invalid("sink full")
        ))
        .is_err()
    );
    assert!(
        stream(
            FIXTURE,
            &dataset(),
            QualityPolicy {
                max_ruwe: f64::NAN,
                reject_duplicated_source: true
            },
            |_| Ok(())
        )
        .is_err()
    );
}

#[test]
fn gaia_angular_values_and_source_errors_have_separate_explicit_units() {
    let csv=b"source_id,ref_epoch,ra,dec,parallax,parallax_error,pmra,pmdec,radial_velocity,ra_error,dec_error\n1,2016,120,-45,,,,,,3.6,7.2\n";
    let result = run(&csv[..], &StellarCrossMatch::default()).unwrap();
    let observation = &result.observations.values().next().unwrap()[0];
    assert_eq!(observation.ra_degrees, 120.0);
    assert_eq!(observation.dec_degrees, -45.0);
    assert_eq!(observation.ra_error_mas, Some(3.6));
    assert_eq!(observation.dec_error_mas, Some(7.2));
    assert_eq!(observation.source_fields["ra_error"], "3.6");
    assert_eq!(observation.source_fields["dec_error"], "7.2");
}
