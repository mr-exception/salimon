use super::super::climate::{ClimateInputs, Condensable};
use super::*;
use crate::{
    CELESTIAL_BODIES,
    terrain::{
        TerrainDefinition,
        geology::{GeologyDefinition, GeologyProfile},
    },
};
fn body() -> CelestialBody {
    *CELESTIAL_BODIES
        .iter()
        .find(|b| b.id == CelestialBodyId::Moon)
        .unwrap()
}
fn id(key: &str) -> CatalogId {
    CatalogId::from_source("fictional-fixture", key).unwrap()
}
fn climate() -> ClimateDefinition {
    let t = TerrainDefinition::new(body(), 1, 0, vec![], vec![]).unwrap();
    ClimateDefinition::new(
        GeologyDefinition::new(t, 1, 0, GeologyProfile::WeatheredRock).unwrap(),
        1,
        37,
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
        },
    )
    .unwrap()
}
fn definition(decision: BiosphereDecision, seed: u64) -> BiosphereDefinition {
    BiosphereDefinition::new(
        id("world-a"),
        BodyNature::Fictional,
        decision,
        climate(),
        1,
        seed,
    )
    .unwrap()
}
#[test]
fn ecology_potential_never_implies_actual_life_or_visible_organisms() {
    for decision in [
        BiosphereDecision::Sterile,
        BiosphereDecision::MicrobialOnly,
        BiosphereDecision::Developed,
    ] {
        let d = definition(decision, 17);
        let s = d.sample([1.0, 0.0, 0.0]).unwrap();
        assert!(s.potential > 0.05);
        if decision != BiosphereDecision::Developed {
            assert_eq!(s.macro_cover_fraction, 0.0);
            assert!(d.sites().unwrap().is_empty());
        } else {
            assert!(s.macro_cover_fraction > 0.0);
        }
        assert_eq!(
            s.family_grammar_id.is_some(),
            decision != BiosphereDecision::Sterile
        );
    }
    let observed = BiosphereDefinition::generate(
        id("observed"),
        BodyNature::ObservedNoLifeClaim,
        climate(),
        17,
    )
    .unwrap();
    assert_eq!(observed.decision(), BiosphereDecision::Unknown);
    assert_eq!(
        observed
            .sample([1.0, 0.0, 0.0])
            .unwrap()
            .macro_cover_fraction,
        0.0
    );
    for decision in [
        BiosphereDecision::Sterile,
        BiosphereDecision::MicrobialOnly,
        BiosphereDecision::Developed,
    ] {
        assert!(
            BiosphereDefinition::new(
                id("observed"),
                BodyNature::ObservedNoLifeClaim,
                decision,
                climate(),
                1,
                17
            )
            .is_err()
        );
    }
}
#[test]
fn ecology_rare_fixed_decisions_and_independent_seeds_do_not_change_climate() {
    let c = climate();
    let before = c.to_bytes();
    let mut counts = [0; 4];
    for seed in 0..1000 {
        let d = BiosphereDefinition::generate(id("batch"), BodyNature::Fictional, c.clone(), seed)
            .unwrap();
        counts[d.decision() as usize] += 1;
        assert_eq!(
            d,
            BiosphereDefinition::generate(id("batch"), BodyNature::Fictional, c.clone(), seed)
                .unwrap()
        );
    }
    assert_eq!(counts, [0, 981, 16, 3]);
    assert!(counts[1] > 950);
    assert!(counts[2] > 0 && counts[3] > 0);
    assert_eq!(counts[0], 0);
    assert_eq!(before, c.to_bytes());
    let a = definition(BiosphereDecision::Developed, 17);
    let b = definition(BiosphereDecision::Developed, 18);
    assert_ne!(a.identity(), b.identity());
    assert_ne!(a.family_grammar_id(), b.family_grammar_id());
    assert_eq!(a.climate(), b.climate());
}
#[test]
fn ecology_spherical_regions_have_smooth_transitions_and_environmental_variation() {
    let d = definition(BiosphereDecision::Developed, 17);
    for v in [
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 1.0],
        [-1.0, 0.0, 1.0],
        [0.0, 1.0, 0.0],
    ] {
        let a = d.sample(super::super::unit(v)).unwrap();
        let b = d
            .sample(super::super::unit([v[0] + 1e-9, v[1], v[2]]))
            .unwrap();
        assert!((a.potential - b.potential).abs() < 1e-6);
        assert!((a.macro_cover_fraction - b.macro_cover_fraction).abs() < 1e-6);
        assert!((a.regional_weights.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        assert!((a.community_weights.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        for (a, b) in a.regional_weights.into_iter().zip(b.regional_weights) {
            assert!((a - b).abs() < 1e-6);
        }
    }
    let equator = d.sample([1.0, 0.0, 0.0]).unwrap();
    let pole = d.sample([0.0, 1.0, 0.0]).unwrap();
    assert!(equator.potential > pole.potential);
    assert!(pole.community_weights[2] > equator.community_weights[2]);
    let mut p = climate().inputs();
    p.stellar_flux_w_m2 = 0.0;
    let dark = ClimateDefinition::new(climate().geology().clone(), 1, 37, p).unwrap();
    assert!(
        BiosphereDefinition::new(
            id("dark"),
            BodyNature::Fictional,
            BiosphereDecision::Developed,
            dark.clone(),
            1,
            17
        )
        .is_err()
    );
    assert_eq!(
        BiosphereDefinition::generate(id("dark"), BodyNature::Fictional, dark, 17)
            .unwrap()
            .decision(),
        BiosphereDecision::Sterile
    );
}
#[test]
fn ecology_site_ids_and_physical_anchors_survive_reload_streaming_and_concurrent_queries() {
    let d = definition(BiosphereDecision::Developed, 17);
    let bytes = d.to_bytes();
    let copy = BiosphereDefinition::from_bytes(body(), &bytes).unwrap();
    assert_eq!(d, copy);
    let sites = d.sites().unwrap();
    assert!(!sites.is_empty());
    let ids: std::collections::BTreeSet<_> = sites.iter().map(|s| &s.id).collect();
    assert_eq!(ids.len(), sites.len());
    for site in &sites {
        assert_eq!(
            site.body_local_position_meters,
            d.climate()
                .geology()
                .terrain()
                .sample(site.direction)
                .unwrap()
                .body_local_position_meters
        );
        let _ = copy.sample([0.0, 0.0, -1.0]);
        assert_eq!(d.sample(site.direction), copy.sample(site.direction));
    }
    assert_eq!(sites, copy.sites().unwrap());
    let clients: Vec<_> = (0..4)
        .map(|_| {
            let d = d.clone();
            std::thread::spawn(move || (d.sites().unwrap(), d.sample([1.0, 0.0, 0.0]).unwrap()))
        })
        .collect();
    for c in clients {
        let (s, v) = c.join().unwrap();
        assert_eq!(s, sites);
        assert_eq!(v, d.sample([1.0, 0.0, 0.0]).unwrap());
    }
    assert_eq!(bytes, d.to_bytes());
}
#[test]
fn ecology_catalog_save_bindings_are_exact_sorted_and_fail_closed() {
    let a = definition(BiosphereDecision::Developed, 17);
    let b = BiosphereDefinition::new(
        id("world-b"),
        BodyNature::Fictional,
        BiosphereDecision::Sterile,
        climate(),
        1,
        19,
    )
    .unwrap();
    let catalog = BiosphereCatalog::new(vec![b.clone(), a.clone()]).unwrap();
    let bytes = catalog.to_bytes();
    let copy = BiosphereCatalog::from_bytes(&bytes, |_| Some(body())).unwrap();
    assert_eq!(catalog, copy);
    assert_eq!(copy.resolve(a.catalog_body(), a.identity()).unwrap(), &a);
    assert_eq!(
        copy.resolve(a.catalog_body(), b.identity()),
        Err(Error::IncompatibleDefinition)
    );
    assert!(copy.resolve(&id("missing"), a.identity()).is_err());
    assert!(BiosphereCatalog::new(vec![a.clone(), a]).is_err());
    assert!(BiosphereCatalog::from_bytes(&bytes, |_| None).is_err());
    for n in 0..bytes.len() {
        assert!(BiosphereCatalog::from_bytes(&bytes[..n], |_| Some(body())).is_err());
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert!(BiosphereCatalog::from_bytes(&trailing, |_| Some(body())).is_err());
}
#[test]
fn ecology_rejects_corrupt_definitions_and_incompatible_revisions() {
    let a = definition(BiosphereDecision::Developed, 17);
    let bytes = a.to_bytes();
    for n in 0..bytes.len() {
        assert!(BiosphereDefinition::from_bytes(body(), &bytes[..n]).is_err());
    }
    for offset in [18, 19] {
        let mut b = bytes.clone();
        b[offset] = 255;
        assert!(BiosphereDefinition::from_bytes(body(), &b).is_err());
    }
    let mut b = bytes.clone();
    b[8] = 2;
    assert_eq!(
        BiosphereDefinition::from_bytes(body(), &b),
        Err(Error::UnsupportedVersion)
    );
    let mut b = bytes;
    b.push(0);
    assert!(BiosphereDefinition::from_bytes(body(), &b).is_err());
    assert!(!a.compatible_with(a.identity(), &id("wrong-body"), a.climate()));
    let b = definition(BiosphereDecision::Developed, 18);
    assert!(!b.compatible_with(a.identity(), a.catalog_body(), a.climate()));
    for direction in [[0.0; 3], [2.0, 0.0, 0.0], [f64::NAN, 0.0, 1.0]] {
        assert!(a.sample(direction).is_err());
    }
}
#[test]
fn ecology_vesper_is_sterile_and_publication_resources_are_unchanged() {
    let before = crate::canonical::START.encode().unwrap();
    let c = ClimateDefinition::new(
        TerrainDefinition::canonical_start().geology(),
        1,
        0,
        ClimateInputs::vacuum(86400.0, 31557600.0, 1361.0),
    )
    .unwrap();
    let id = crate::canonical::START.manifest().planet.id.clone();
    for decision in [
        BiosphereDecision::Unknown,
        BiosphereDecision::MicrobialOnly,
        BiosphereDecision::Developed,
    ] {
        assert!(
            BiosphereDefinition::new(
                id.clone(),
                BodyNature::Fictional,
                decision,
                c.clone(),
                1,
                17
            )
            .is_err()
        );
    }
    let d = BiosphereDefinition::generate(id, BodyNature::Fictional, c, 17).unwrap();
    assert_eq!(d.decision(), BiosphereDecision::Sterile);
    assert!(d.sites().unwrap().is_empty());
    let sample = d.sample([0.0, 1.0, 0.0]).unwrap();
    assert_eq!(sample.potential + sample.macro_cover_fraction, 0.0);
    assert!(sample.family_grammar_id.is_none());
    assert_eq!(before, crate::canonical::START.encode().unwrap());
}

#[test]
fn ecology_frozen_homogeneous_potential_vector() {
    let terrain = TerrainDefinition::new(body(), 1, 0, vec![], vec![]).unwrap();
    let geology = GeologyDefinition::new(terrain, 1, 0, GeologyProfile::HomogeneousRock).unwrap();
    let climate = ClimateDefinition::new(geology, 1, 37, climate().inputs()).unwrap();
    let d = BiosphereDefinition::new(
        id("homogeneous"),
        BodyNature::Fictional,
        BiosphereDecision::Developed,
        climate,
        1,
        17,
    )
    .unwrap();
    let s = d.sample([1.0, 0.0, 0.0]).unwrap();
    // Unit thermal/pressure/light/chemistry factors, 0.8 condensables and 0.3 substrate factor.
    assert!((s.potential - 0.24).abs() < 1e-12);
    let cold = (290.0 - 284.5781401165717) / 110.0;
    assert!((s.community_weights[0] - (1.0 - cold)).abs() < 1e-12);
    assert_eq!(s.community_weights[1], 0.0);
    assert!((s.community_weights[2] - cold).abs() < 1e-12);
    assert!(s.macro_cover_fraction > 0.0 && s.macro_cover_fraction <= s.potential);
}
