use super::*;
use crate::environment::{
    climate::ClimateDefinition,
    ecology::{BiosphereDecision, BodyNature},
    landmarks::{Geography, Scale, Signature},
    organism_placement::Exclusion,
    organisms,
};
pub(crate) fn fixture(seed: u64) -> (BiosphereDefinition, Population, Landmarks) {
    let b = organisms::tests::biosphere();
    let b = BiosphereDefinition::new(
        b.catalog_body().clone(),
        BodyNature::Fictional,
        BiosphereDecision::Developed,
        b.climate().clone(),
        1,
        seed,
    )
    .unwrap();
    let p = Population::publish(&b, &[]).unwrap();
    let g = Landmarks::publish(
        b.climate().geology().clone(),
        Geography::PublishedFictional,
        0,
        &[],
    )
    .unwrap();
    (b, p, g)
}
#[test]
fn natural_sites_have_exact_physical_sources_and_stable_streaming_binding() {
    let mut found = 0;
    let mut giants = 0;
    let mut colonies = 0;
    for seed in 0..128 {
        let (b, p, g) = fixture(seed);
        let binding = p.binding();
        let bytes = b.to_bytes();
        let a = SignatureSites::publish(&b, &p, &g, None, &[]).unwrap();
        let restored = BiosphereDefinition::from_bytes(
            *crate::CELESTIAL_BODIES
                .iter()
                .find(|b| b.id == crate::CelestialBodyId::Moon)
                .unwrap(),
            &bytes,
        )
        .unwrap();
        let p2 = Population::publish(&restored, &[]).unwrap();
        let b2 = SignatureSites::publish(&restored, &p2, &g, None, &[]).unwrap();
        assert_eq!(a, b2);
        b2.verify_binding(&a.binding()).unwrap();
        assert_eq!(p.binding(), binding);
        for s in a.sites().iter().rev() {
            found += 1;
            giants += usize::from(s.kind == Kind::GiantOrganism);
            colonies += usize::from(s.kind == Kind::DenseColony);
            assert!(a.nearby(s.position_meters, 0.).unwrap().contains(&s));
            for id in &s.organism_ids {
                let o = p.objects().iter().find(|o| o.id == *id).unwrap();
                for part in &o.parts {
                    for i in 0..3 {
                        assert!(
                            s.bounds[2 * i] <= part.bounds[2 * i]
                                && s.bounds[2 * i + 1] >= part.bounds[2 * i + 1]
                        );
                    }
                }
            }
        }
    }
    println!(
        "128 fictional developed publications: {found} sites ({giants} giant, {colonies} dense colonies)"
    );
    assert!(found > 0 && found < 128 && giants > 0 && colonies > 0);
}
#[test]
fn natural_authored_and_mixed_sites_require_real_source_geology_and_safe_population() {
    let b = organisms::tests::biosphere();
    let body = *crate::CELESTIAL_BODIES
        .iter()
        .find(|b| b.id == crate::CelestialBodyId::Moon)
        .unwrap();
    let terrain = crate::terrain::TerrainDefinition::new(
        body,
        1,
        17,
        vec![crate::terrain::Layer {
            kind: crate::terrain::LayerKind::Hills,
            amplitude_meters: 100.,
            wavelength_meters: 10000.,
        }],
        vec![],
    )
    .unwrap();
    let c = ClimateDefinition::new(terrain.geology(), 1, 37, b.climate().inputs()).unwrap();
    let mut found = false;
    for seed in 0..64 {
        let b = BiosphereDefinition::new(
            b.catalog_body().clone(),
            BodyNature::Fictional,
            BiosphereDecision::Developed,
            c.clone(),
            1,
            seed,
        )
        .unwrap();
        let p = Population::publish(&b, &[]).unwrap();
        for o in p.objects() {
            let n = length(o.position_meters);
            let d = o.position_meters.map(|v| v / n);
            let Ok(g) = Landmarks::publish(
                c.geology().clone(),
                Geography::Observed,
                0,
                &[Signature {
                    key: "physical-geology-fixture".into(),
                    direction: d,
                    scale: Scale::Regional,
                }],
            ) else {
                continue;
            };
            let signature = AuthoredSignature {
                key: "mixed-fixture".into(),
                organism_id: o.id.clone(),
            };
            let sites = SignatureSites::publish(&b, &p, &g, None, std::slice::from_ref(&signature))
                .unwrap();
            let s = sites
                .sites()
                .iter()
                .find(|s| s.authored_key.is_some())
                .unwrap();
            assert_eq!(s.kind, Kind::MixedGeological);
            assert!(s.geology_site_id.is_some());
            assert!(s.activity_site_id.is_none());
            assert!(
                SignatureSites::publish(&b, &p, &g, None, &[signature.clone(), signature]).is_err()
            );
            let bad = AuthoredSignature {
                key: "bad".into(),
                organism_id: "missing".into(),
            };
            assert!(SignatureSites::publish(&b, &p, &g, None, &[bad]).is_err());
            let excluded = Population::publish(
                &b,
                &[Exclusion {
                    id: "protected-source".into(),
                    bounds: s.bounds,
                }],
            )
            .unwrap();
            assert!(
                excluded
                    .objects()
                    .iter()
                    .all(|x| !s.organism_ids.contains(&x.id))
            );
            assert!(
                SignatureSites::publish(
                    &b,
                    &excluded,
                    &g,
                    None,
                    &[AuthoredSignature {
                        key: "excluded".into(),
                        organism_id: o.id.clone()
                    }]
                )
                .is_err()
            );
            found = true;
            break;
        }
        if found {
            break;
        }
    }
    assert!(found);
}
#[test]
fn natural_sites_reject_changed_sources_and_keep_sterile_empty_worlds_empty() {
    let (b, p, g) = fixture(17);
    let (other, _, _) = fixture(18);
    assert!(SignatureSites::publish(&other, &p, &g, None, &[]).is_err());
    let a = SignatureSites::publish(&b, &p, &g, None, &[]).unwrap();
    let mut bytes = a.binding();
    bytes.push(0);
    assert!(a.verify_binding(&bytes).is_err());
    let sterile = &crate::canonical::BIOSPHERE;
    let sites = SignatureSites::publish(
        sterile,
        &crate::canonical::ORGANISMS,
        &crate::canonical::LANDMARKS,
        None,
        &[],
    )
    .unwrap();
    assert!(sites.sites().is_empty());
}

#[test]
fn natural_geothermal_labels_require_same_actual_activity_source_and_spatial_overlap() {
    use crate::environment::activity::{ActivityDefinition, ActivityInputs, ActivityKind};
    let (b, p, g) = fixture(17);
    let inputs = ActivityInputs {
        kind: ActivityKind::Vent,
        candidate_count: 32,
        heat_flux_w_m2: 1.,
        site_radius_meters: 100.,
        cycle_seconds: 86400.,
        active_fraction: 0.1,
    };
    let a = ActivityDefinition::new(b.climate().clone(), 1, 911, inputs).unwrap();
    let sites = SignatureSites::publish(&b, &p, &g, Some(&a), &[]).unwrap();
    for site in sites.sites() {
        if let Some(id) = &site.activity_site_id {
            let actual = a.sites().iter().find(|x| x.id == *id).unwrap();
            assert!(
                distance(actual.body_local_position_meters, site.position_meters)
                    <= actual.radius_meters
            );
        } else {
            assert_ne!(site.kind, Kind::GeothermalEcology);
        }
    }
    let (_, other, _) = fixture(18);
    assert!(SignatureSites::publish(&b, &other, &g, Some(&a), &[]).is_err());
    let dormant =
        ActivityDefinition::new(b.climate().clone(), 1, 911, ActivityInputs::dormant()).unwrap();
    assert!(
        SignatureSites::publish(&b, &p, &g, Some(&dormant), &[])
            .unwrap()
            .sites()
            .iter()
            .all(|s| s.activity_site_id.is_none())
    );
}
