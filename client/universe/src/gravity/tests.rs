use super::*;
fn position(x: f64) -> GalacticPosition {
    GalacticPosition::new([1_000_000_000; 3], [x, 0.0, 0.0]).unwrap()
}
fn source(id: &str, x: f64, mu: f64) -> Source {
    Source {
        id: CatalogId::from_source("fixture", id).unwrap(),
        position: position(x),
        mu_m3_per_s2: mu,
        radius_meters: 0.0,
    }
}
#[test]
fn inverse_square_reference_and_input_order_invariance() {
    let mut sources = vec![
        source("earth", 100.0, 10000.0),
        source("moon", -100.0, 1000.0),
    ];
    let mut selector = Selector::new(Config::default()).unwrap();
    let a = selector.sample(&sources, position(0.0), 0.0).unwrap();
    assert!((a.meters_per_second_squared[0] - 0.9).abs() < 1e-15);
    assert_eq!(
        relative_error(
            a.meters_per_second_squared,
            all_sources(&sources, position(0.0)).unwrap()
        )
        .unwrap(),
        0.0
    );
    sources.reverse();
    let b = Selector::new(Config::default())
        .unwrap()
        .sample(&sources, position(0.0), 0.0)
        .unwrap();
    assert_eq!(a.weights, b.weights);
    assert_eq!(a.meters_per_second_squared, b.meters_per_second_squared);
}
#[test]
fn handoff_continuity_hysteresis_and_bounded_transition() {
    let mut selector = Selector::new(Config {
        contributors: 1,
        hysteresis_fraction: 0.05,
        transition_seconds: 1.0,
    })
    .unwrap();
    let mut sources = vec![source("a", 10.0, 100.0), source("b", -10.0, 99.0)];
    let initial = selector
        .sample(&sources, position(0.0), 0.0)
        .unwrap()
        .meters_per_second_squared;
    sources[1].mu_m3_per_s2 = 104.0;
    assert_eq!(
        selector
            .sample(&sources, position(0.0), 0.0)
            .unwrap()
            .meters_per_second_squared,
        initial
    );
    sources[1].mu_m3_per_s2 = 110.0;
    let start = selector.sample(&sources, position(0.0), 0.0).unwrap();
    assert_eq!(start.meters_per_second_squared, initial);
    let mid = selector.sample(&sources, position(0.0), 0.5).unwrap();
    assert_eq!(mid.weights.len(), 2);
    assert!((mid.meters_per_second_squared[0] + 0.05).abs() < 1e-14);
    let end = selector.sample(&sources, position(0.0), 0.5).unwrap();
    assert!(!end.transitioning);
    assert_eq!(end.weights.len(), 1);
}
#[test]
fn invalid_or_missing_active_sources_preserve_selector() {
    let sources = vec![source("a", 10.0, 100.0)];
    let mut selector = Selector::new(Config::default()).unwrap();
    let before = selector.sample(&sources, position(0.0), 0.0).unwrap();
    assert!(selector.sample(&[], position(0.0), 1.0).is_err());
    let after = selector.sample(&sources, position(0.0), 0.0).unwrap();
    assert_eq!(before.weights, after.weights);
    assert!(selector.sample(&sources, position(10.0), 1.0).is_err());
    assert!(
        selector
            .sample(
                &[sources[0].clone(), sources[0].clone()],
                position(0.0),
                1.0
            )
            .is_err()
    );
}
