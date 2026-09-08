use salimon_world::resource_distribution::{
    BodyResourceDistribution, ResourceDistribution, default_resource_distribution,
};
use salimon_world::resource_generation::{GenerationError, materialize_nearby_deposits};
use salimon_world::resources::ResourceId;
use salimon_world::{CELESTIAL_BODIES, CelestialBody, CelestialBodyId, WorldPosition};

fn earth() -> CelestialBody {
    *CELESTIAL_BODIES
        .iter()
        .find(|b| b.id == CelestialBodyId::Earth)
        .unwrap()
}
fn query(
    body: CelestialBody,
    direction: [f64; 3],
) -> Vec<salimon_world::resource_generation::SurfaceDeposit> {
    let length = direction.iter().map(|v| v * v).sum::<f64>().sqrt();
    materialize_nearby_deposits(
        body,
        &default_resource_distribution(body.id),
        123,
        None,
        body.center
            .translated(direction.map(|v| v / length * body.radius_meters)),
        150.0,
    )
    .unwrap()
}

#[test]
fn deterministic_bounded_surface_geometry_and_reload_identity() {
    for body in CELESTIAL_BODIES.iter().copied().skip(1) {
        for direction in [
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0],
            [-1.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [-1.0, -1.0, -1.0],
        ] {
            let first = query(body, direction);
            assert!(!first.is_empty());
            assert!(first.len() < 5000);
            let mut ids = std::collections::HashSet::new();
            for generated in &first {
                assert!(ids.insert(generated.deposit.id()));
                let local = generated.body_local_position_meters;
                let length = local.iter().map(|v| v * v).sum::<f64>().sqrt();
                assert!((length - body.radius_meters).abs() < 1e-8);
                assert!(
                    generated.bounds_radius_meters.is_finite()
                        && generated.bounds_radius_meters > 0.0
                );
                let volume = 4.0 / 3.0
                    * std::f64::consts::PI
                    * (generated.bounds_radius_meters
                        / salimon_world::resource_size::RESOURCE_LINEAR_SCALE)
                        .powi(3);
                assert!((volume - generated.deposit.material().volume_m3()).abs() < 1e-10);
                assert_eq!(generated.deposit.position(), body.center.translated(local));
                assert_eq!(
                    generated.deposit.remaining_mass_kg(),
                    generated.deposit.material().mass_kg()
                );
            }
            // Unload the owned active list, visit a different area, and regenerate.
            let saved = first.clone();
            drop(first);
            drop(query(body, [1.0, -1.0, 0.0]));
            assert_eq!(saved, query(body, direction));
        }
    }
}

#[test]
fn overlapping_queries_agree_and_cover_cube_seams() {
    let body = earth();
    let profile = default_resource_distribution(body.id);
    let center = body.center.translated([
        body.radius_meters / 2.0_f64.sqrt(),
        0.0,
        body.radius_meters / 2.0_f64.sqrt(),
    ]);
    let wide = materialize_nearby_deposits(body, &profile, 19, None, center, 220.0).unwrap();
    assert!(!wide.is_empty());
    for offset in [[20.0, 0.0, -20.0], [-20.0, 0.0, 20.0], [0.0, 20.0, 0.0]] {
        let player = center.translated(offset);
        let small = materialize_nearby_deposits(body, &profile, 19, None, player, 100.0).unwrap();
        let expected: Vec<_> = wide
            .iter()
            .filter(|d| d.deposit.position().distance_to(player) <= 100.0)
            .collect();
        assert_eq!(small.len(), expected.len());
        for deposit in small {
            assert!(expected.contains(&&deposit));
        }
    }
}

#[test]
fn seeds_weights_mass_ranges_order_and_large_world_translation() {
    let body = earth();
    let player = body.center.translated([0.0, 0.0, body.radius_meters]);
    let iron = ResourceDistribution::new(ResourceId::IronOre, 3.0, 30.0, [7.0, 9.0]).unwrap();
    let rock =
        ResourceDistribution::new(ResourceId::SilicateRock, 1.0, 30.0, [11.0, 11.0]).unwrap();
    let profile = BodyResourceDistribution::new(body.id, 99, vec![iron, rock], vec![]).unwrap();
    let generate = |profile: &BodyResourceDistribution, seed| {
        materialize_nearby_deposits(body, profile, seed, None, player, 300.0).unwrap()
    };
    let first = generate(&profile, 5);
    let reversed = BodyResourceDistribution::new(body.id, 99, vec![rock, iron], vec![]).unwrap();
    assert_eq!(first, generate(&reversed, 5));
    assert_ne!(first, generate(&profile, 6));
    let mut counts = [0, 0];
    for entry in &first {
        let mass = entry.deposit.material().mass_kg();
        match entry.deposit.material().resource() {
            ResourceId::IronOre => {
                counts[0] += 1;
                assert!((7.0..=9.0).contains(&mass));
            }
            ResourceId::SilicateRock => {
                counts[1] += 1;
                assert_eq!(mass, 11.0);
            }
            _ => panic!("unconfigured material"),
        }
    }
    assert!(counts[0] > counts[1] && counts[1] > 0);
    let translated = CelestialBody {
        center: WorldPosition::new(0.0, 0.0, 0.0),
        ..body
    };
    let other = materialize_nearby_deposits(
        translated,
        &profile,
        5,
        None,
        WorldPosition::new(0.0, 0.0, body.radius_meters),
        300.0,
    )
    .unwrap();
    for (a, b) in first.iter().zip(&other) {
        assert_eq!(a.deposit.id(), b.deposit.id());
        assert_eq!(a.body_local_position_meters, b.body_local_position_meters);
        assert_eq!(a.deposit.material(), b.deposit.material());
    }
    assert_eq!(first.len(), other.len());
}

#[test]
fn empty_far_invalid_and_excessive_requests_are_bounded() {
    let body = earth();
    let profile = default_resource_distribution(body.id);
    let surface = body.center.translated([0.0, 0.0, body.radius_meters]);
    let empty = BodyResourceDistribution::new(body.id, 0, vec![], vec![]).unwrap();
    assert!(
        materialize_nearby_deposits(body, &empty, 0, None, surface, 100.0)
            .unwrap()
            .is_empty()
    );
    assert!(
        materialize_nearby_deposits(
            body,
            &profile,
            0,
            None,
            surface.translated([0.0, 0.0, 5000.0]),
            100.0
        )
        .unwrap()
        .is_empty()
    );
    for radius in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            materialize_nearby_deposits(body, &profile, 0, None, surface, radius),
            Err(GenerationError::InvalidArea)
        );
    }
    assert_eq!(
        materialize_nearby_deposits(body, &profile, 0, None, surface, body.radius_meters),
        Err(GenerationError::CandidateBudgetExceeded)
    );
    assert_eq!(
        materialize_nearby_deposits(
            body,
            &default_resource_distribution(CelestialBodyId::Moon),
            0,
            None,
            surface,
            100.0
        ),
        Err(GenerationError::BodyMismatch)
    );
}

#[test]
fn nearby_sampling_matches_full_small_sphere_reference() {
    let body = CelestialBody {
        radius_meters: 100.0,
        center: WorldPosition::new(0.0, 0.0, 0.0),
        ..earth()
    };
    let profile = BodyResourceDistribution::new(
        body.id,
        1,
        vec![ResourceDistribution::new(ResourceId::IronOre, 1.0, 10.0, [5.0, 10.0]).unwrap()],
        vec![],
    )
    .unwrap();
    let full = materialize_nearby_deposits(
        body,
        &profile,
        9,
        None,
        WorldPosition::new(0.0, 0.0, 100.0),
        201.0,
    )
    .unwrap();
    assert_eq!(full.len(), 6 * 20 * 20);
    for direction in [
        [1.0, 0.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, -1.0],
        [1.0, 1.0, 1.0],
        [-1.0, 1.0, -1.0],
        [0.2, -0.7, 0.4],
    ] {
        let length = direction.iter().map(|v| v * v).sum::<f64>().sqrt();
        for altitude in [-5.0, 0.0, 5.0] {
            let local = direction.map(|v| v / length * (100.0 + altitude));
            let player = WorldPosition::new(local[0], local[1], local[2]);
            let nearby =
                materialize_nearby_deposits(body, &profile, 9, None, player, 20.0).unwrap();
            let expected: Vec<_> = full
                .iter()
                .filter(|d| d.deposit.position().distance_to(player) <= 20.0)
                .collect();
            assert_eq!(nearby.len(), expected.len());
            for deposit in nearby {
                assert!(expected.contains(&&deposit));
            }
        }
    }
}
