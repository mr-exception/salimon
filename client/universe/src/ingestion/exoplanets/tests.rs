use super::*;
use crate::ingestion::gaia::{self, QualityPolicy};
const FIXTURE: &[u8] =
    include_bytes!("../../../../../catalogs/sources/fixtures/exoplanet-proxima.csv");
fn dataset() -> Dataset {
    Dataset {
        id: CatalogId::from_source("dataset", "nea-ps").unwrap(),
        version: "2026-10-21".into(),
        reference: "catalogs/sources/manifest-v1.json".into(),
    }
}
fn run(bytes: &[u8]) -> Result<ImportedPlanets, Error> {
    import(
        bytes,
        dataset(),
        "fixture-v1".into(),
        None,
        &StellarCrossMatch::default(),
    )
}
#[test]
fn pinned_proxima_host_period_distance_errors_and_literature() {
    let i = run(FIXTURE).unwrap();
    assert_eq!(i.summary.planets, 1);
    assert_eq!(i.summary.hosts_using_gaia_ids, 1);
    let p = &i.observations[0];
    assert_eq!(p.host.as_str(), "gaia_dr3:5853498713190525696");
    let period = p.orbital_period_seconds.as_ref().unwrap();
    assert_eq!(period.limit, Limit::Unspecified);
    assert!((period.quantity.value - 966353.76).abs() < 1e-8);
    assert!((period.error_plus_si.unwrap() - 45.792).abs() < 1e-10);
    assert_eq!(period.error_minus_si, period.error_plus_si);
    let distance = p.host_distance_meters.as_ref().unwrap();
    assert!((distance.quantity.value / PARSEC_METERS - 1.30119).abs() < 1e-15);
    assert_eq!(distance.error_minus_si, Some(0.00035 * PARSEC_METERS));
    assert_eq!(distance.error_plus_si, Some(0.00034 * PARSEC_METERS));
    assert!(p.semi_major_axis_meters.is_none());
    assert!(p.planet_mass_kg.is_none());
    assert!(p.literature_reference.contains("2025A&A...700A..11M"));
    assert!(
        matches!(&p.provenance,Provenance::Observed { record_key,.. } if record_key=="Proxima Cen b")
    );
    let Record::Body(body) = i.catalog.get(&p.planet).unwrap() else {
        panic!("body")
    };
    assert_eq!(body.parent, p.host);
    assert!(body.mass_kg.is_none());
    assert_eq!(
        Catalog::decode(&i.catalog.encode().unwrap()).unwrap(),
        i.catalog
    );
}
#[test]
fn host_reconciles_with_gaia_without_overwriting_base_measurements_or_ids() {
    let gaia_bytes=b"source_id,ref_epoch,ra,dec,parallax,parallax_error,pmra,pmdec,radial_velocity\n5853498713190525696,2016,0,0,,,,,\n";
    let canonical = CatalogId::from_source("star", "proxima-reviewed").unwrap();
    let links = StellarCrossMatch::new([(
        CatalogId::from_source("gaia_dr3", "5853498713190525696").unwrap(),
        canonical.clone(),
    )])
    .unwrap();
    let g = gaia::import(
        &gaia_bytes[..],
        Dataset {
            id: CatalogId::from_source("dataset", "gaia").unwrap(),
            version: "synthetic".into(),
            reference: "fixture".into(),
        },
        "gaia".into(),
        QualityPolicy::default(),
        &links,
    )
    .unwrap();
    let result = import(
        FIXTURE,
        dataset(),
        "joined".into(),
        Some(&g.catalog),
        &links,
    )
    .unwrap();
    assert_eq!(result.observations[0].host, canonical);
    assert_eq!(
        result
            .catalog
            .records()
            .iter()
            .filter(|r| matches!(r, Record::Star(_)))
            .count(),
        1
    );
    assert_eq!(result.catalog.get(&canonical), g.catalog.get(&canonical));
    assert_eq!(result.catalog.datasets().len(), 2);
}
#[test]
fn source_keys_no_gaia_multiple_planets_limits_and_partial_errors_survive() {
    let header = "pl_name,hostname,pl_refname,default_flag,pl_orbper,pl_orbpererr1,pl_orbpererr2,pl_orbperlim,pl_massj,pl_msinij,pl_radj,pl_orbsmax,pl_orbeccen,pl_orbincl,pl_orblper,pl_orbtper";
    let a = "Host b,Host,ref,1,3,0.1,,1,,0.4,1,2,0.2,90,45,2459000";
    let b = "Host c,Host,ref,1,4,,,0,0.5,,1,3,0,0,,";
    let one = run(format!("{header}\n{a}\n{b}\n{a}\n").as_bytes()).unwrap();
    let two = run(format!("{header}\n{b}\n{a}\n").as_bytes()).unwrap();
    assert_eq!(one.catalog.encode().unwrap(), two.catalog.encode().unwrap());
    assert_eq!(one.observations, two.observations);
    assert_eq!(one.summary.duplicate_rows, 1);
    assert_eq!(one.summary.hosts_using_archive_ids, 1);
    let p = &one.observations[0];
    assert_eq!(
        p.orbital_period_seconds.as_ref().unwrap().limit,
        Limit::Upper
    );
    assert_eq!(
        p.orbital_period_seconds.as_ref().unwrap().error_minus_si,
        None
    );
    assert_eq!(
        p.orbital_period_seconds.as_ref().unwrap().error_plus_si,
        Some(8640.0)
    );
    assert_eq!(
        p.minimum_mass_kg.as_ref().unwrap().quantity.value,
        0.4 * JUPITER_MASS_KG
    );
    let Record::Body(body) = one.catalog.get(&p.planet).unwrap() else {
        panic!("body")
    };
    assert!(body.mass_kg.is_none());
    assert_eq!(
        p.semi_major_axis_meters.as_ref().unwrap().quantity.value,
        2.0 * AU_METERS
    );
    assert!(
        (p.inclination_radians.as_ref().unwrap().quantity.value - std::f64::consts::FRAC_PI_2)
            .abs()
            < 1e-15
    );
    assert_ne!(
        archive_id("nea_host", "A B").unwrap(),
        archive_id("nea_host", "A_B").unwrap()
    );
}
#[test]
fn malformed_errors_ambiguity_limits_and_nondefault_rows_reject() {
    let raw = std::str::from_utf8(FIXTURE).unwrap();
    for bad in [
        raw.replace("-0.00035000", "0.00035000"),
        raw.replace("11.18465000000", "NaN"),
        raw.replace(
            "Gaia DR3 5853498713190525696",
            "Gaia DR2 5853498713190525696",
        ),
        raw.replace("0.00053000000", "-0.00053000000"),
        raw.replace("11.18465000000", "-1"),
    ] {
        assert!(run(bad.as_bytes()).is_err());
    }
    for bad in [
        "pl_name,hostname,pl_refname,default_flag\np,h,r,0\n",
        "pl_name,hostname,pl_refname,pl_orbeccen\np,h,r,1\n",
        "pl_name,hostname,pl_refname,pl_orbincl\np,h,r,181\n",
        "pl_name,hostname,pl_refname,pl_orbperlim\np,h,r,1\n",
        "pl_name,hostname,pl_refname,gaia_dr3_id\na,h,r,Gaia DR3 1\nb,h,r,Gaia DR3 2\n",
        "pl_name,hostname,pl_refname,pl_orbper\np,h,r,1\np,h,r,2\n",
    ] {
        assert!(run(bad.as_bytes()).is_err(), "{bad}");
    }
}

#[test]
fn only_explicitly_unlimited_true_mass_is_promoted_and_unknown_physics_stays_missing() {
    let cases = [
        ("pl_name,hostname,pl_refname,pl_massj\np,h,ref,1\n", false),
        (
            "pl_name,hostname,pl_refname,pl_massj,pl_massjlim\np,h,ref,1,0\n",
            true,
        ),
        (
            "pl_name,hostname,pl_refname,pl_massj,pl_massjlim\np,h,ref,1,1\n",
            false,
        ),
        (
            "pl_name,hostname,pl_refname,pl_massj,pl_massjlim,pl_massjerr2\np,h,ref,1,0,-2\n",
            false,
        ),
    ];
    for (csv, promoted) in cases {
        let result = run(csv.as_bytes()).unwrap();
        let p = &result.observations[0];
        let Record::Body(body) = result.catalog.get(&p.planet).unwrap() else {
            panic!("body")
        };
        assert_eq!(body.mass_kg.is_some(), promoted);
        assert!(p.planet_mass_kg.is_some());
    }
}
