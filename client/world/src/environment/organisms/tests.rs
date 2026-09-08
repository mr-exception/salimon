use super::*;
use crate::{
    CELESTIAL_BODIES, CelestialBodyId,
    environment::{
        climate::{Chemistry, ClimateDefinition, ClimateInputs, Condensable},
        ecology::{BiosphereDecision, BodyNature},
    },
    terrain::TerrainDefinition,
};
pub(crate) fn biosphere() -> BiosphereDefinition {
    let b = *CELESTIAL_BODIES
        .iter()
        .find(|b| b.id == CelestialBodyId::Moon)
        .unwrap();
    let mut p = ClimateInputs::vacuum(86400., 31557600., 1361.);
    p.chemistry = Chemistry::Nitrogen;
    p.pressure_pascal = 101325.;
    p.scale_height_meters = 8000.;
    p.condensable = Condensable::Water;
    p.condensable_fraction = 0.8;
    p.greenhouse_kelvin = 30.;
    let climate = ClimateDefinition::new(
        TerrainDefinition::new(b, 1, 5, vec![], vec![])
            .unwrap()
            .geology(),
        1,
        37,
        p,
    )
    .unwrap();
    BiosphereDefinition::new(
        salimon_universe::CatalogId::from_source("fixture", "fictional-biology").unwrap(),
        BodyNature::Fictional,
        BiosphereDecision::Developed,
        climate,
        1,
        17,
    )
    .unwrap()
}
#[test]
fn organism_families_have_stable_related_bounded_forms_and_part_identities() {
    let b = biosphere();
    let before = b.to_bytes();
    let g = Grammar::from_biosphere(&b).unwrap();
    for form in [
        Form::Mat,
        Form::Column,
        Form::Frond,
        Form::Filament,
        Form::Colony,
        Form::Giant,
    ] {
        let s = g.specimen(form, 0).unwrap();
        assert_eq!(s, g.specimen(form, 0).unwrap());
        assert_ne!(s.id, g.specimen(form, 1).unwrap().id);
        assert!(!s.parts.is_empty() && s.parts.len() <= 16);
        for (i, p) in s.parts.iter().enumerate() {
            assert_eq!(p.id, i as u16);
            assert!(p.radius_start > 0. && p.radius_end > 0.);
            assert!(
                p.start
                    .iter()
                    .chain(&p.end)
                    .all(|v| v.is_finite() && v.abs() < 16.)
            );
        }
        assert!(s.color.iter().all(|v| (0.1..=0.6).contains(v)));
    }
    assert!(g.specimen(Form::Mat, 64).is_err());
    assert_eq!(before, b.to_bytes());
}
#[test]
fn cosmetic_cover_quality_is_a_stable_subset_and_never_physical_authority() {
    let b = biosphere();
    let tile = CoverTile {
        face: 0,
        u: 32768,
        v: 32768,
    };
    let small = cosmetic_tile(&b, tile, 32).unwrap();
    let large = cosmetic_tile(&b, tile, 64).unwrap();
    assert!(!large.is_empty());
    assert!(small.iter().all(|p| large.contains(p)));
    assert_eq!(large, cosmetic_tile(&b, tile, 64).unwrap());
    let c = b.climate().clone();
    for decision in [BiosphereDecision::Sterile, BiosphereDecision::MicrobialOnly] {
        let sterile = BiosphereDefinition::new(
            b.catalog_body().clone(),
            BodyNature::Fictional,
            decision,
            c.clone(),
            1,
            17,
        )
        .unwrap();
        assert!(Grammar::from_biosphere(&sterile).is_none());
        assert!(cosmetic_tile(&sterile, tile, 64).unwrap().is_empty());
    }
}
