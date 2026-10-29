use super::*;
use crate::{
    CELESTIAL_BODIES, CelestialBodyId,
    terrain::{Feature, Layer, LayerKind, TerrainDefinition},
};
fn ground() -> GeologyDefinition {
    let body = *CELESTIAL_BODIES
        .iter()
        .find(|b| b.id == CelestialBodyId::Moon)
        .unwrap();
    TerrainDefinition::new(
        body,
        1,
        91,
        vec![Layer {
            kind: LayerKind::Ridges,
            amplitude_meters: 400.,
            wavelength_meters: 2000.,
        }],
        vec![Feature {
            direction: [0., 0., 1.],
            radius_meters: 400.,
            elevation_meters: -200.,
        }],
    )
    .unwrap()
    .geology()
}
#[test]
fn landmarks_are_existing_physical_terrain_across_reconstruction_query_order_and_quality() {
    let geology = ground();
    let original = geology.to_bytes();
    let mut found = 0;
    for seed in 0..32 {
        let a =
            Landmarks::publish(geology.clone(), Geography::PublishedFictional, seed, &[]).unwrap();
        let restored = GeologyDefinition::from_bytes(
            *CELESTIAL_BODIES
                .iter()
                .find(|b| b.id == CelestialBodyId::Moon)
                .unwrap(),
            &original,
        )
        .unwrap();
        let b = Landmarks::publish(restored, Geography::PublishedFictional, seed, &[]).unwrap();
        assert_eq!(a, b);
        b.verify_binding(&a.binding()).unwrap();
        found += a.sites().len();
        assert!(a.sites().len() < 16);
        for site in a.sites().iter().rev() {
            let actual = geology.terrain().sample(site.direction).unwrap();
            assert_eq!(site.position_meters, actual.body_local_position_meters);
            assert_eq!(site.normal, actual.outward_normal);
            assert!(a.nearby(site.position_meters, 0.).unwrap().contains(&site));
        }
    }
    assert!(found > 0 && found < 256);
    println!("rarity audit: {found} sites across 32 fully surveyed fictional publications");
    assert_eq!(geology.to_bytes(), original);
}
#[test]
fn landmark_authoring_observed_geography_empty_worlds_and_invalid_bindings() {
    let g = ground();
    let signatures = [Signature {
        key: "published-basin".into(),
        direction: [0., 0., 1.],
        scale: Scale::Regional,
    }];
    let a = Landmarks::publish(g.clone(), Geography::Observed, 0, &signatures).unwrap();
    assert_eq!(a.sites().len(), 1);
    assert_eq!(
        a.sites()[0].authored_key.as_deref(),
        Some("published-basin")
    );
    assert!(
        Landmarks::publish(g.clone(), Geography::Observed, 1, &[])
            .unwrap()
            .sites()
            .is_empty()
    );
    assert!(
        Landmarks::publish(
            g.clone(),
            Geography::Observed,
            0,
            &[signatures[0].clone(), signatures[0].clone()]
        )
        .is_err()
    );
    let mut bad = signatures[0].clone();
    bad.direction[0] = f64::NAN;
    assert!(Landmarks::publish(g.clone(), Geography::Observed, 0, &[bad]).is_err());
    let b = Landmarks::publish(g.clone(), Geography::Observed, 2, &signatures).unwrap();
    assert!(b.verify_binding(&a.binding()).is_err());
    let mut bytes = a.binding();
    bytes.push(0);
    assert!(a.verify_binding(&bytes).is_err());
    let body = *CELESTIAL_BODIES
        .iter()
        .find(|b| b.id == CelestialBodyId::Moon)
        .unwrap();
    let flat = TerrainDefinition::new(body, 1, 0, vec![], vec![])
        .unwrap()
        .geology();
    assert!(
        Landmarks::publish(flat.clone(), Geography::PublishedFictional, 0, &[])
            .unwrap()
            .sites()
            .is_empty()
    );
    assert!(Landmarks::publish(flat, Geography::Observed, 0, &signatures).is_err());
}
#[test]
fn landmark_canonical_and_source_bytes_never_change_geography() {
    let g = TerrainDefinition::canonical_start().geology();
    let bytes = g.to_bytes();
    let a = Landmarks::publish(g.clone(), Geography::PublishedFictional, 0, &[]).unwrap();
    assert_eq!(a.geology().to_bytes(), bytes);
    for seed in 0..20 {
        assert!(
            Landmarks::publish(g.clone(), Geography::PublishedFictional, seed, &[])
                .unwrap()
                .sites()
                .is_empty()
        );
    }
    // A different sidecar seed cannot be accepted under another publication binding.
    assert!(
        Landmarks::publish(g, Geography::PublishedFictional, 1, &[])
            .unwrap()
            .verify_binding(&a.binding())
            .is_err()
    );
}
