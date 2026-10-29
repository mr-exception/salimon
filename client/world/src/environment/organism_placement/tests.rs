use super::*;
fn fixture() -> BiosphereDefinition {
    super::super::organisms::tests::biosphere()
}
#[test]
fn physical_population_reconstructs_with_identical_parts_independent_of_visual_quality() {
    let b = fixture();
    let original = b.to_bytes();
    let p = Population::publish(&b, &[]).unwrap();
    assert!(!p.objects().is_empty());
    assert_eq!(p, Population::publish(&b, &[]).unwrap());
    let binding = p.binding();
    p.verify_binding(&binding).unwrap();
    for o in p.objects() {
        assert_eq!(o.parts.len(), o.specimen.parts.len());
        for part in &o.parts {
            assert!(valid(part.bounds));
            assert!(o.specimen.parts.iter().any(|v| v.id == part.id));
        }
    }
    let _ = super::super::organisms::cosmetic_tile(
        &b,
        super::super::organisms::CoverTile {
            face: 0,
            u: 32768,
            v: 32768,
        },
        8,
    )
    .unwrap();
    assert_eq!(p, Population::publish(&b, &[]).unwrap());
    assert_eq!(original, b.to_bytes());
    let mut corrupt = binding;
    corrupt[8] ^= 1;
    assert!(p.verify_binding(&corrupt).is_err());
}
#[test]
fn physical_population_respects_opening_resource_pad_and_route_exclusions() {
    let b = fixture();
    let before = Population::publish(&b, &[]).unwrap();
    let first = &before.objects()[0];
    let bounds = first.parts.iter().map(|p| p.bounds).fold(
        [
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ],
        |a, b| {
            std::array::from_fn(|i| {
                if i % 2 == 0 {
                    a[i].min(b[i]) - 1.
                } else {
                    a[i].max(b[i]) + 1.
                }
            })
        },
    );
    let zone = Exclusion {
        id: "authored-opening-and-ramp".into(),
        bounds,
    };
    let p = Population::publish(&b, std::slice::from_ref(&zone)).unwrap();
    assert!(p.objects().iter().all(|o| {
        o.parts
            .iter()
            .all(|part| !overlap(part.bounds, zone.bounds))
    }));
    assert_ne!(p.identity(), before.identity());
    assert!(p.verify_binding(&before.binding()).is_err());
    let duplicate = vec![zone.clone(), zone];
    assert!(Population::publish(&b, &duplicate).is_err());
}
