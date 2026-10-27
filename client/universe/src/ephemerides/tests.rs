use super::*;
#[test]
fn evaluated_rotating_body_preserves_transport_and_rejects_cycles() {
    use crate::coordinates::{GalacticPosition, Rotation};
    let system = CatalogId::from_source("fixture", "system").unwrap();
    let body = CatalogId::from_source("fixture", "body").unwrap();
    let anchor = || SystemAnchor {
        system: system.clone(),
        epoch: epoch(),
        position: GalacticPosition::new([1_000_000; 3], [0.0; 3]).unwrap(),
        velocity_mps: [0.0; 3],
        rotation: Rotation::IDENTITY,
    };
    let mut track = BodyTrack {
        object: body.clone(),
        parent: FrameId::System(system.clone()),
        model: Model::Linear {
            sample: Sample {
                epoch: epoch(),
                position_meters: [100.0, 0.0, 0.0],
                velocity_mps: [0.0; 3],
            },
            valid_seconds: 10.0,
        },
        spin_epoch: epoch(),
        spin_radians: 0.0,
        spin_rps: 0.1,
    };
    let (tree, quality) = frame_tree(anchor(), &[track], epoch()).unwrap();
    assert_eq!(quality[0].1, Accuracy::Analytic);
    let s = tree
        .transform(
            &FrameId::Body(body.clone()),
            &FrameId::System(system.clone()),
            State {
                meters: [10.0, 0.0, 0.0],
                meters_per_second: [0.0; 3],
            },
            epoch(),
        )
        .unwrap();
    assert_eq!(s.meters, [110.0, 0.0, 0.0]);
    assert_eq!(s.meters_per_second, [0.0, 1.0, 0.0]);
    track = BodyTrack {
        object: body.clone(),
        parent: FrameId::Body(body),
        model: Model::Linear {
            sample: sample(0),
            valid_seconds: 10.0,
        },
        spin_epoch: epoch(),
        spin_radians: 0.0,
        spin_rps: 0.0,
    };
    assert!(frame_tree(anchor(), &[track], epoch()).is_err());
}
fn epoch() -> Epoch {
    Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], TimeScale::Tdb).unwrap()
}
fn sample(t: i64) -> Sample {
    Sample {
        epoch: epoch().advance_nanos(t * 1_000_000_000).unwrap(),
        position_meters: [t as f64, 0.0, 0.0],
        velocity_mps: [1.0, 0.0, 0.0],
    }
}
#[test]
fn hermite_exact_motion_validity_fallback_and_scales() {
    let mut m = Model::Sampled {
        samples: vec![sample(0), sample(10)],
        max_extrapolation_seconds: 0.0,
    };
    let (s, q) = m.evaluate(sample(4).epoch).unwrap();
    assert_eq!(q, Accuracy::Interpolated);
    assert!((s.meters[0] - 4.0).abs() < 1e-12);
    assert!((s.meters_per_second[0] - 1.0).abs() < 1e-12);
    assert_eq!(m.evaluate(epoch()).unwrap().1, Accuracy::ObservedSample);
    assert!(m.evaluate(sample(11).epoch).is_err());
    if let Model::Sampled {
        max_extrapolation_seconds,
        ..
    } = &mut m
    {
        *max_extrapolation_seconds = 2.0;
    }
    assert_eq!(
        m.evaluate(sample(11).epoch).unwrap().1,
        Accuracy::LowerAccuracyExtrapolation
    );
    assert!(m.evaluate(sample(13).epoch).is_err());
    assert!(
        m.evaluate(Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], TimeScale::Utc).unwrap())
            .is_err()
    );
}
#[test]
fn ellipse_kepler_quarter_period_and_periapsis_velocity() {
    let model = |e| Model::Elliptic {
        epoch: epoch(),
        semi_major_axis_meters: 1000.0,
        eccentricity: e,
        period_seconds: 100.0,
        mean_anomaly_radians: 0.0,
        axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };
    let (s, _) = model(0.0).evaluate(sample(25).epoch).unwrap();
    assert!(s.meters[0].abs() < 1e-9);
    assert!((s.meters[1] - 1000.0).abs() < 1e-9);
    let (s, _) = model(0.5).evaluate(epoch()).unwrap();
    assert!((s.meters[0] - 500.0).abs() < 1e-9);
    assert!(
        (s.meters_per_second[1] - 1000.0 * 3.0_f64.sqrt() * std::f64::consts::TAU / 100.0).abs()
            < 1e-9
    );
    assert!(model(1.0).evaluate(epoch()).is_err());
}
#[test]
fn imported_horizons_exact_samples_reproduce_without_loss() {
    let manifest = crate::sources::SourceManifest::decode(
        include_bytes!("../../../../catalogs/sources/manifest-v1.json"),
        false,
    )
    .unwrap();
    let source = manifest
        .sources
        .iter()
        .find(|s| s.id.as_str() == "jpl:horizons-earth-j2000")
        .unwrap();
    let dataset = crate::Dataset {
        id: source.id.clone(),
        version: source.release.clone(),
        reference: "fixture".into(),
    };
    let e = crate::ingestion::horizons::Ephemeris::decode(
        include_bytes!("../../../../catalogs/sources/fixtures/jpl-earth-j2000.json"),
        &dataset,
    )
    .unwrap();
    for original in e.samples() {
        let (s, q) = Model::from_horizons(&e).evaluate(original.epoch).unwrap();
        assert_eq!(s.meters, original.position_meters);
        assert_eq!(s.meters_per_second, original.velocity_mps);
        assert_eq!(q, Accuracy::ObservedSample);
    }
}
