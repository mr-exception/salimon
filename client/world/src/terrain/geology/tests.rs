use super::*;
use crate::{CELESTIAL_BODIES, CelestialBodyId};
fn moon() -> CelestialBody {
    *CELESTIAL_BODIES
        .iter()
        .find(|b| b.id == CelestialBodyId::Moon)
        .unwrap()
}
fn unit(d: [f64; 3]) -> [f64; 3] {
    let n = query::norm(d);
    d.map(|x| x / n)
}
#[test]
fn frozen_recipe_roundtrip_and_query_order() {
    let terrain = TerrainDefinition::representative(moon()).unwrap();
    let original = terrain.to_bytes();
    let geology = terrain.geology();
    let restored = GeologyDefinition::from_bytes(moon(), &geology.to_bytes()).unwrap();
    assert_eq!(restored, geology);
    let moved = CelestialBody {
        center: moon().center.translated([1e12, -1e12, 1e12]),
        ..moon()
    };
    assert_eq!(
        GeologyDefinition::from_bytes(moved, &geology.to_bytes()).unwrap(),
        geology
    );
    let expected = geology.sample([0.0, 1.0, 0.0]).unwrap();
    let clients: Vec<_> = (0..4)
        .map(|_| {
            let copy = geology.clone();
            std::thread::spawn(move || copy.sample([0.0, 1.0, 0.0]).unwrap())
        })
        .collect();
    for client in clients {
        assert_eq!(client.join().unwrap(), expected);
    }
    assert_eq!(terrain.to_bytes(), original);
    for d in [[0.0, 0.0, 1.0], [0.0, 1.0, 0.0], unit([1.0, 2.0, 3.0])] {
        let a = geology.sample(d).unwrap();
        let _ = restored.sample(unit([-3.0, 2.0, 1.0]));
        assert_eq!(a, restored.sample(d).unwrap());
        assert_eq!(a.surface, terrain.sample(d).unwrap());
    }
    assert_eq!(
        geology.identity(),
        "feac86d05cc39f3d3f5a3e0b110a6826c3f1b6000f4d7dcce7851c4a8fa2860e"
    );
}
#[test]
fn continuous_fields_at_cube_edges_and_province_junctions() {
    let geology = TerrainDefinition::representative(moon()).unwrap().geology();
    for d in [
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 1.0],
        [-1.0, 0.0, 1.0],
        [0.0, 1.0, 0.0],
    ] {
        let a = geology.sample(unit(d)).unwrap();
        let b = geology.sample(unit([d[0] + 1e-9, d[1], d[2]])).unwrap();
        assert!((a.geology.layer_depth_meters - b.geology.layer_depth_meters).abs() < 1e-5);
        assert!((a.geology.weathering - b.geology.weathering).abs() < 1e-7);
        for (x, y) in a.geology.history.iter().zip(b.geology.history) {
            assert_eq!(x.process, y.process);
            assert!((x.relative_extent - y.relative_extent).abs() < 1e-7);
        }
    }
}
#[test]
fn revision_and_upstream_changes_require_explicit_migration() {
    let terrain = TerrainDefinition::representative(moon()).unwrap();
    let geology = terrain.geology();
    assert!(geology.compatible_with(geology.identity(), &terrain));
    let changed = TerrainDefinition::new(moon(), 1, 999, vec![], vec![]).unwrap();
    assert!(!geology.compatible_with(geology.identity(), &changed));
    let next = GeologyDefinition::new(terrain.clone(), 1, 42, GeologyProfile::Ice).unwrap();
    assert_ne!(next.identity(), geology.identity());
    assert!(!next.compatible_with(geology.identity(), &terrain));
    assert_eq!(
        GeologyDefinition::new(terrain, 2, 0, GeologyProfile::Ice),
        Err(TerrainError::UnsupportedVersion)
    );
    let mut bytes = geology.to_bytes();
    bytes[8] = 2;
    assert_eq!(
        GeologyDefinition::from_bytes(moon(), &bytes),
        Err(TerrainError::UnsupportedVersion)
    );
    let mut bytes = geology.to_bytes();
    bytes.push(0);
    assert!(GeologyDefinition::from_bytes(moon(), &bytes).is_err());
    assert!(GeologyDefinition::from_bytes(moon(), &[0; 18]).is_err());
}
#[test]
fn profiles_history_and_invalid_directions() {
    let terrain = TerrainDefinition::representative(moon()).unwrap();
    for profile in [
        GeologyProfile::DryRock,
        GeologyProfile::WeatheredRock,
        GeologyProfile::Ice,
        GeologyProfile::HomogeneousRock,
    ] {
        let definition = GeologyDefinition::new(terrain.clone(), 1, 7, profile).unwrap();
        let a = definition.sample([0.0, 1.0, 0.0]).unwrap();
        assert!(
            a.geology
                .history
                .iter()
                .all(|s| (0.0..=1.0).contains(&s.relative_extent))
        );
        assert_eq!(
            a.geology
                .history
                .iter()
                .any(|s| s.process == GeologicalProcess::TectonicDeformation),
            profile == GeologyProfile::WeatheredRock
        );
        if profile == GeologyProfile::Ice {
            assert_eq!(a.geology.substrate, Substrate::WaterIce);
        }
        if profile == GeologyProfile::HomogeneousRock {
            assert_eq!(
                a.geology.province_id,
                definition
                    .sample([1.0, 0.0, 0.0])
                    .unwrap()
                    .geology
                    .province_id
            );
        }
        for d in [[0.0; 3], [f64::NAN, 0.0, 1.0], [2.0, 0.0, 0.0]] {
            assert_eq!(definition.sample(d), Err(TerrainError::InvalidDirection));
        }
    }
}
#[test]
fn vesper_remains_the_exact_published_surface_and_resources() {
    let terrain = TerrainDefinition::canonical_start();
    let bytes = terrain.to_bytes();
    let definition = terrain.geology();
    assert_eq!(definition.profile(), GeologyProfile::HomogeneousRock);
    let frozen = crate::canonical::START.encode().unwrap();
    for d in [[0.0, 1.0, 0.0], unit([1.0, 1.0, 1.0]), [0.0, -1.0, 0.0]] {
        assert_eq!(
            definition.sample(d).unwrap().surface,
            terrain.sample(d).unwrap()
        );
    }
    assert_eq!(terrain.to_bytes(), bytes);
    assert_eq!(crate::canonical::START.encode().unwrap(), frozen);
    assert_eq!(
        crate::canonical::START.definition().sha256,
        salimon_universe::starting_system::WORLD_SHA256
    );
}

#[test]
fn golden_region_vector_is_stable_across_clients() {
    let terrain = TerrainDefinition::new(moon(), 1, 0, vec![], vec![]).unwrap();
    let geology = GeologyDefinition::new(terrain, 1, 0, GeologyProfile::DryRock).unwrap();
    let sample = geology.sample([0.0, 1.0, 0.0]).unwrap();
    assert_eq!(sample.geology.landform, Landform::Plain);
    assert_eq!(sample.geology.substrate, Substrate::Regolith);
    assert!((sample.geology.layer_depth_meters - 2.2271304538866894).abs() < 1e-12);
    assert!((sample.geology.history[0].relative_extent - 0.5391092081744686).abs() < 1e-12);
    assert_eq!(sample.surface.elevation_meters, 0.0);
}
