use super::*;
use crate::CELESTIAL_BODIES;

fn body(id: CelestialBodyId) -> CelestialBody {
    *CELESTIAL_BODIES.iter().find(|b| b.id == id).unwrap()
}
fn direction(v: [f64; 3]) -> [f64; 3] {
    let n = query::norm(v);
    v.map(|x| x / n)
}
fn earth() -> TerrainDefinition {
    TerrainDefinition::representative(body(CelestialBodyId::Earth)).unwrap()
}

#[test]
fn roundtrip_profiles_order_and_world_translation() {
    for id in [
        CelestialBodyId::Earth,
        CelestialBodyId::Moon,
        CelestialBodyId::Mars,
    ] {
        let b = body(id);
        let original = TerrainDefinition::representative(b).unwrap();
        let bytes = original.to_bytes();
        assert_eq!(bytes.len(), 106);
        let mut translated = b;
        translated.center = b.center.translated([1e12, -1e12, 1e12]);
        let loaded = TerrainDefinition::from_bytes(translated, &bytes).unwrap();
        assert_eq!(original, loaded);
        let directions = [
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
            direction([-0.3, 0.8, 1.0]),
        ];
        let samples: Vec<_> = directions
            .iter()
            .map(|d| original.sample(*d).unwrap())
            .collect();
        for i in (0..directions.len()).rev() {
            assert_eq!(samples[i], loaded.sample(directions[i]).unwrap());
        }
        let expected = original.sample(directions[3]).unwrap();
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let copy = loaded.clone();
                std::thread::spawn(move || copy.sample(directions[3]).unwrap())
            })
            .collect();
        for handle in handles {
            assert_eq!(expected, handle.join().unwrap());
        }
    }
}
#[test]
fn rejects_invalid_inputs_and_unbounded_definitions() {
    let b = body(CelestialBodyId::Earth);
    for amplitude in [f64::NAN, f64::INFINITY, -1.0, b.radius_meters] {
        assert!(
            TerrainDefinition::new(
                b,
                1,
                0,
                vec![Layer {
                    kind: LayerKind::Hills,
                    amplitude_meters: amplitude,
                    wavelength_meters: 10.0
                }],
                vec![]
            )
            .is_err()
        );
    }
    assert_eq!(
        TerrainDefinition::new(b, 2, 0, vec![], vec![]),
        Err(TerrainError::UnsupportedVersion)
    );
    assert!(TerrainDefinition::new(body(CelestialBodyId::Sun), 1, 0, vec![], vec![]).is_err());
    for radius in [0.0, 0.5, 1e10, f64::MAX, f64::NAN] {
        let mut invalid = b;
        invalid.radius_meters = radius;
        assert!(TerrainDefinition::new(invalid, 1, 0, vec![], vec![]).is_err());
    }
    let layer = Layer {
        kind: LayerKind::Hills,
        amplitude_meters: 1.0,
        wavelength_meters: 10.0,
    };
    assert!(TerrainDefinition::new(b, 1, 0, vec![layer; MAX_LAYERS + 1], vec![]).is_err());
    let mut big = layer;
    big.amplitude_meters = b.radius_meters * 0.01;
    assert!(TerrainDefinition::new(b, 1, 0, vec![big; 2], vec![]).is_err());
    for d in [[0.0; 3], [2.0, 0.0, 0.0], [f64::NAN; 3], [f64::INFINITY; 3]] {
        assert_eq!(earth().height(d), Err(TerrainError::InvalidDirection));
    }
}
#[test]
fn decoding_rejects_truncation_corruption_and_wrong_body() {
    let b = body(CelestialBodyId::Earth);
    let bytes = earth().to_bytes();
    for length in 0..bytes.len() {
        assert!(TerrainDefinition::from_bytes(b, &bytes[..length]).is_err());
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(TerrainDefinition::from_bytes(b, &trailing).is_err());
    for offset in [0, 8, 10, 27, 28, 29, 32] {
        let mut corrupt = bytes.clone();
        corrupt[offset] = 255;
        assert!(TerrainDefinition::from_bytes(b, &corrupt).is_err());
    }
    assert!(TerrainDefinition::from_bytes(body(CelestialBodyId::Moon), &bytes).is_err());
    let mut bad = bytes.clone();
    bad[33..41].copy_from_slice(&f64::NAN.to_le_bytes());
    assert!(TerrainDefinition::from_bytes(b, &bad).is_err());
}
#[test]
fn crater_depth_support_and_normal_match_radial_surface() {
    let b = body(CelestialBodyId::Moon);
    let terrain = TerrainDefinition::new(
        b,
        1,
        42,
        vec![],
        vec![Feature {
            direction: [0.0, 0.0, 1.0],
            radius_meters: 100.0,
            elevation_meters: -20.0,
        }],
    )
    .unwrap();
    let center = terrain.sample([0.0, 0.0, 1.0]).unwrap();
    assert_eq!(center.elevation_meters, -20.0);
    assert_eq!(center.outward_normal, [0.0, 0.0, 1.0]);
    assert_eq!(center.slope, 0.0);
    let wall = terrain
        .sample(direction([50.0 / b.radius_meters, 0.0, 1.0]))
        .unwrap();
    assert!((wall.elevation_meters + 10.0).abs() < 1e-6);
    assert!((wall.slope - 0.3).abs() < 1e-5);
    assert!(wall.outward_normal[0] < 0.0);
    assert_eq!(terrain.height([1.0, 0.0, 0.0]).unwrap(), 0.0);
}
#[test]
fn poles_seams_bounds_and_normals_are_continuous() {
    let terrain = earth();
    for v in [
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
    ] {
        let left = terrain
            .sample(direction([v[0] - 1e-10, v[1], v[2]]))
            .unwrap();
        let right = terrain
            .sample(direction([v[0] + 1e-10, v[1], v[2]]))
            .unwrap();
        assert!((left.elevation_meters - right.elevation_meters).abs() < 0.001);
        for axis in 0..3 {
            assert!((left.outward_normal[axis] - right.outward_normal[axis]).abs() < 0.001);
        }
    }
    let bounds = terrain.elevation_bounds_meters();
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for x in -8..=8 {
        for y in -8..=8 {
            let d = direction([x as f64 / 8.0, y as f64 / 8.0, 1.0]);
            let sample = terrain.sample(d).unwrap();
            assert!(sample.elevation_meters >= bounds[0] && sample.elevation_meters <= bounds[1]);
            assert!((query::norm(sample.outward_normal) - 1.0).abs() < 1e-12);
            let dot: f64 = d
                .iter()
                .zip(sample.outward_normal)
                .map(|(a, b)| a * b)
                .sum();
            assert!(dot > 0.0);
            min = min.min(sample.elevation_meters);
            max = max.max(sample.elevation_meters);
        }
    }
    assert!(max - min > 50.0);
}
#[test]
fn full_capacity_wire_budget_and_definition_identity() {
    let b = body(CelestialBodyId::Earth);
    let layer = Layer {
        kind: LayerKind::Hills,
        amplitude_meters: 1.0,
        wavelength_meters: 100.0,
    };
    let feature = Feature {
        direction: [0.0, 1.0, 0.0],
        radius_meters: 100.0,
        elevation_meters: -1.0,
    };
    let terrain = TerrainDefinition::new(
        b,
        1,
        1,
        vec![layer; MAX_LAYERS],
        vec![feature; MAX_FEATURES],
    )
    .unwrap();
    assert_eq!(terrain.to_bytes().len(), MAX_DEFINITION_BYTES);
    assert_eq!(
        TerrainDefinition::from_bytes(b, &terrain.to_bytes()).unwrap(),
        terrain
    );
    let other =
        TerrainDefinition::new(b, 1, 2, terrain.layers.clone(), terrain.features.clone()).unwrap();
    assert_ne!(terrain.to_bytes(), other.to_bytes());
    assert_ne!(
        terrain.height([0.0, 0.0, 1.0]),
        other.height([0.0, 0.0, 1.0])
    );
}

#[test]
fn version_one_golden_samples() {
    for (id, expected) in [
        (
            CelestialBodyId::Earth,
            [66.25735912605658, 37.23989240497296, -98.36985516039361],
        ),
        (
            CelestialBodyId::Moon,
            [1.5998591182153077, 9.492837395038059, -119.22593903123789],
        ),
        (
            CelestialBodyId::Mars,
            [200.3973919174702, -164.48482663840207, 57.77640721994979],
        ),
    ] {
        let terrain = TerrainDefinition::representative(body(id)).unwrap();
        for (d, expected) in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
            .into_iter()
            .zip(expected)
        {
            assert!((terrain.height(d).unwrap() - expected).abs() < 1e-7);
        }
        let (height, normal) = match id {
            CelestialBodyId::Earth => (
                -124.89158729741037,
                [0.5685442354805067, 0.5737341760859231, 0.5895647101827698],
            ),
            CelestialBodyId::Moon => (
                10.302420701324818,
                [0.5811207762606919, 0.5729950676313148, 0.5779059576335546],
            ),
            CelestialBodyId::Mars => (
                -36.840271658545056,
                [0.45231320059201435, 0.6243727833407394, 0.6368448759262699],
            ),
            _ => unreachable!(),
        };
        let sample = terrain.sample(direction([1.0, 1.0, 1.0])).unwrap();
        assert!((sample.elevation_meters - height).abs() < 1e-7);
        for (actual, expected) in sample.outward_normal.into_iter().zip(normal) {
            assert!((actual - expected).abs() < 1e-6);
        }
    }
}
