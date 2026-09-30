use salimon_world::resource_distribution::*;
use salimon_world::resources::ResourceId;
use salimon_world::{BodyRole, CELESTIAL_BODIES, CelestialBodyId};

fn entry(resource: ResourceId) -> ResourceDistribution {
    ResourceDistribution::new(resource, 2.0, 30.0, [5.0, 25.0]).unwrap()
}
fn config(resources: Vec<ResourceDistribution>) -> BodyResourceDistribution {
    BodyResourceDistribution::new(CelestialBodyId::Moon, 42, resources, vec![]).unwrap()
}

#[test]
fn every_supported_body_has_a_valid_explicit_profile() {
    for body in CELESTIAL_BODIES {
        let profile = default_resource_distribution(body.id);
        assert_eq!(profile.body(), body.id);
        let inputs = profile.generation_inputs(100, None);
        if body.role == BodyRole::Solid {
            assert!(inputs.len() >= 2);
            for input in inputs {
                let distribution = input.distribution;
                assert!(distribution.weight() > 0.0);
                assert!(distribution.spacing_meters() > 0.0);
                assert!(distribution.mass_range_kg()[0] > 0.0);
                assert!(distribution.mass_range_kg()[1] >= distribution.mass_range_kg()[0]);
            }
        } else {
            assert!(inputs.is_empty());
        }
    }
}

#[test]
fn empty_solid_body_and_single_resource_are_supported() {
    assert!(config(vec![]).generation_inputs(7, None).is_empty());
    let item = entry(ResourceId::WaterIce);
    let inputs = config(vec![item]).generation_inputs(7, None);
    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0].distribution, item);
}

#[test]
fn deterministic_inputs_do_not_depend_on_entry_order_or_other_resources() {
    let ice = entry(ResourceId::WaterIce);
    let iron = entry(ResourceId::IronOre);
    let expected = config(vec![ice, iron]).generation_inputs(7, None);
    for _ in 0..10 {
        assert_eq!(config(vec![iron, ice]).generation_inputs(7, None), expected);
    }
    assert_eq!(expected[0].distribution.resource(), ResourceId::IronOre);
    assert_eq!(expected[0].seed, 5207797635405500880);
    assert_eq!(
        config(vec![iron]).generation_inputs(7, None)[0],
        expected[0]
    );
    assert_ne!(config(vec![iron, ice]).generation_inputs(8, None), expected);
    for (body, seed) in [(CelestialBodyId::Earth, 42), (CelestialBodyId::Moon, 43)] {
        let different = BodyResourceDistribution::new(body, seed, vec![iron, ice], vec![]).unwrap();
        assert_ne!(different.generation_inputs(7, None), expected);
    }
}

#[test]
fn biome_overrides_replace_base_and_unknown_biomes_fall_back() {
    let base = vec![entry(ResourceId::IronOre), entry(ResourceId::SilicateRock)];
    let polar =
        BiomeResourceOverride::new("polar".into(), vec![entry(ResourceId::WaterIce)]).unwrap();
    let barren = BiomeResourceOverride::new("barren".into(), vec![]).unwrap();
    let profile =
        BodyResourceDistribution::new(CelestialBodyId::Moon, 42, base, vec![polar, barren])
            .unwrap();
    assert_eq!(
        profile.generation_inputs(7, Some("unknown")),
        profile.generation_inputs(7, None)
    );
    let inputs = profile.generation_inputs(7, Some("polar"));
    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0].distribution.resource(), ResourceId::WaterIce);
    assert_eq!(inputs, profile.clone().generation_inputs(7, Some("polar")));
    assert!(profile.generation_inputs(7, Some("barren")).is_empty());
    let same_material = config(vec![entry(ResourceId::WaterIce)]).generation_inputs(7, None);
    assert_ne!(inputs[0].seed, same_material[0].seed);
}

#[test]
fn invalid_parameters_and_ambiguous_configuration_are_rejected() {
    for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            ResourceDistribution::new(ResourceId::IronOre, invalid, 10.0, [1.0, 2.0]),
            Err(DistributionError::InvalidWeight)
        );
        assert_eq!(
            ResourceDistribution::new(ResourceId::IronOre, 1.0, invalid, [1.0, 2.0]),
            Err(DistributionError::InvalidSpacing)
        );
        for range in [[invalid, 2.0], [1.0, invalid]] {
            assert_eq!(
                ResourceDistribution::new(ResourceId::IronOre, 1.0, 10.0, range),
                Err(DistributionError::InvalidMassRange)
            );
        }
    }
    assert!(ResourceDistribution::new(ResourceId::IronOre, 1.0, 10.0, [2.0, 1.0]).is_err());
    assert!(ResourceDistribution::new(ResourceId::IronOre, 1.0, 10.0, [2.0, 2.0]).is_ok());
    let duplicate = vec![entry(ResourceId::IronOre); 2];
    assert_eq!(
        BodyResourceDistribution::new(CelestialBodyId::Moon, 0, duplicate.clone(), vec![]),
        Err(DistributionError::DuplicateResource)
    );
    assert_eq!(
        BiomeResourceOverride::new("polar".into(), duplicate),
        Err(DistributionError::DuplicateResource)
    );
    for key in ["", " ", "polar ", " polar"] {
        assert_eq!(
            BiomeResourceOverride::new(key.into(), vec![]),
            Err(DistributionError::InvalidBiome)
        );
    }
    let biome = BiomeResourceOverride::new("polar".into(), vec![]).unwrap();
    assert_eq!(
        BodyResourceDistribution::new(CelestialBodyId::Moon, 0, vec![], vec![biome.clone(), biome]),
        Err(DistributionError::DuplicateBiome)
    );
}

#[test]
fn distribution_parameters_reach_generation_unchanged() {
    let original = ResourceDistribution::new(ResourceId::IronOre, 7.0, 12.5, [10.0, 90.0]).unwrap();
    let input = config(vec![original]).generation_inputs(7, None)[0];
    assert_eq!(input.distribution.weight(), 7.0);
    assert_eq!(input.distribution.spacing_meters(), 12.5);
    assert_eq!(input.distribution.mass_range_kg(), [10.0, 90.0]);
    let updated = ResourceDistribution::new(ResourceId::IronOre, 3.0, 20.0, [5.0, 30.0]).unwrap();
    let changed = config(vec![updated]).generation_inputs(7, None)[0];
    assert_ne!(input.distribution, changed.distribution);
    assert_eq!(input.seed, changed.seed);
}
