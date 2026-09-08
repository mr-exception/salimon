use super::*;
use crate::trajectory::{Thrust, Trajectory};
use crate::{
    Propulsion,
    atmosphere::{Aerodynamics, Air},
};
use salimon_math::{add, cross, length, scale, sub};
use salimon_universe::TimeScale;
use salimon_universe::gravity::{Config, Selector, Source};
fn epoch() -> Epoch {
    Epoch::calendar([2026, 1, 1], [0; 4], TimeScale::Tt).unwrap()
}
fn pos(v: [f64; 3]) -> GalacticPosition {
    GalacticPosition::new([1_000_000, 0, 0], v).unwrap()
}
fn id(name: &str) -> CatalogId {
    CatalogId::from_source("contact-test", name).unwrap()
}
fn state(p: [f64; 3], v: [f64; 3], n: i64) -> State {
    State {
        position: pos(p),
        velocity_mps: v,
        epoch: epoch().advance_nanos(n).unwrap(),
    }
}
fn linear(p: [f64; 3], v: [f64; 3], n: i64) -> CubicPath {
    CubicPath::between(
        state(p, v, 0),
        state(add(p, scale(v, n as f64 * 1e-9)), v, n),
    )
    .unwrap()
}
fn sphere(name: &str, p: [f64; 3], v: [f64; 3], radius: f64, n: i64) -> Collider {
    Collider::sphere(id(name), linear(p, v, n), radius, [0.0; 3]).unwrap()
}
fn flight(s: State) -> Trajectory {
    Trajectory::new(s, Selector::new(Config::default()).unwrap(), 1_000_000_000).unwrap()
}
fn coast() -> Thrust {
    Thrust::coasting(Propulsion::default())
}
fn aero() -> Aerodynamics {
    Aerodynamics::new(1.0, 100.0).unwrap()
}
#[test]
fn high_speed_moving_sphere_crossing_finds_first_time_with_clear_endpoints() {
    let ship = linear([-1000.0, 0.0, 0.0], [2000.0, 0.0, 0.0], 1_000_000_000);
    let scene = Scene::new(vec![sphere(
        "moving",
        [0.0; 3],
        [100.0, 0.0, 0.0],
        10.0,
        1_000_000_000,
    )])
    .unwrap();
    let (hit, stats) = scene.sweep(&ship, 16.0).unwrap();
    let h = hit.unwrap();
    let expected = (1000.0 - 26.0) / 1900.0;
    assert!((h.elapsed_nanos as f64 * 1e-9 - expected).abs() < 0.001 / 1900.0 + 1e-9);
    assert!(length(sub(h.surface_velocity_mps, [100.0, 0.0, 0.0])) < 1e-9);
    assert!(stats.distance_samples < 100);
}
#[test]
fn curved_ephemeris_crossing_is_not_reduced_to_endpoint_chords() {
    let body = CubicPath::between(
        state([-100.0, 50.0, 0.0], [200.0, -200.0, 0.0], 0),
        state([100.0, 50.0, 0.0], [200.0, 200.0, 0.0], 1_000_000_000),
    )
    .unwrap();
    let scene = Scene::new(vec![
        Collider::sphere(id("curve"), body, 10.0, [0.0; 3]).unwrap(),
    ])
    .unwrap();
    let (hit, _) = scene
        .sweep(&linear([0.0; 3], [0.0; 3], 1_000_000_000), 1.0)
        .unwrap();
    let hit = hit.unwrap();
    assert!(hit.elapsed_nanos > 400_000_000 && hit.elapsed_nanos < 500_000_000);
}
#[test]
fn streamed_triangle_face_edge_corner_and_moving_surface_velocity_are_resolved() {
    let n = 1_000_000_000;
    let vertices = [[-10.0, 0.0, -10.0], [10.0, 0.0, -10.0], [0.0, 0.0, 10.0]];
    let triangle = Collider::triangle(
        id("terrain"),
        42,
        vertices.map(|p| linear(p, [100.0, 0.0, 0.0], n)),
    )
    .unwrap();
    let scene = Scene::new(vec![triangle]).unwrap();
    for (x, z) in [(0.0, 0.0), (10.5, -10.0), (10.5, -10.5)] {
        let ship = linear([x, 20.0, z], [100.0, -100.0, 0.0], n);
        let (hit, _) = scene.sweep(&ship, 2.0).unwrap();
        let h = hit.unwrap();
        assert_eq!(h.primitive, 42);
        assert!(h.elapsed_nanos > 170_000_000 && h.elapsed_nanos < 210_000_000);
        assert!(length(sub(h.surface_velocity_mps, [100.0, 0.0, 0.0])) < 1e-9);
        assert!(h.normal[1] > 0.8);
    }
    assert!(
        scene
            .sweep(&linear([14.0, 20.0, -10.0], [100.0, -100.0, 0.0], n), 2.0)
            .unwrap()
            .0
            .is_none()
    );
}
#[test]
fn broadphase_rejects_far_objects_and_ties_have_canonical_identity_order() {
    let n = 1_000_000_000;
    let mut bodies = vec![
        sphere("z", [0.0; 3], [0.0; 3], 10.0, n),
        sphere("a", [0.0; 3], [0.0; 3], 10.0, n),
    ];
    for i in 0..1000 {
        bodies.push(sphere(
            &format!("far-{i}"),
            [1e9 + i as f64 * 1e6, 0.0, 0.0],
            [0.0; 3],
            10.0,
            n,
        ));
    }
    bodies.reverse();
    let scene = Scene::new(bodies).unwrap();
    let (hit, stats) = scene
        .sweep(&linear([-1000.0, 0.0, 0.0], [2000.0, 0.0, 0.0], n), 16.0)
        .unwrap();
    assert_eq!(hit.unwrap().body, id("a"));
    assert_eq!(stats.broadphase_candidates, 1002);
    assert_eq!(stats.narrowphase_candidates, 2);
}
#[test]
fn fused_production_step_intercepts_before_an_endpoint_gravity_query_enters_a_body() {
    let n = 100_000_000;
    let source = Source {
        id: id("body"),
        position: pos([0.0; 3]),
        mu_m3_per_s2: 1.0,
        radius_meters: 10.0,
    };
    let scene = Scene::new(vec![sphere("body", [0.0; 3], [0.0; 3], 10.0, n)]).unwrap();
    let mut t = flight(state([100.0, 0.0, 0.0], [-1000.0, 0.0, 0.0], 0));
    let result = t
        .step_with_contacts(
            n,
            coast(),
            aero(),
            &scene,
            16.0,
            ResponsePolicy::default(),
            |_, _| Ok(vec![source.clone()]),
            |_, _| Ok(None),
        )
        .unwrap();
    assert!(result.elapsed_nanos > 73_000_000 && result.elapsed_nanos < 75_000_000);
    assert!(
        length(
            result
                .hit
                .as_ref()
                .unwrap()
                .position
                .relative_meters(source.position)
                .unwrap()
        ) >= 26.0
    );
    assert_eq!(result.state, t.state());
    assert!(result.resolution.unwrap().disable_propulsion);
    assert!(result.remaining_nanos > 0);
}
#[test]
fn atmospheric_funded_drift_contact_settles_only_elapsed_work_and_matches_truncated_integrator() {
    let n = 1_000_000_000;
    let source = Source {
        id: id("body"),
        position: pos([0.0; 3]),
        mu_m3_per_s2: 1.0,
        radius_meters: 10.0,
    };
    let scene = Scene::new(vec![sphere("body", [0.0; 3], [0.0; 3], 10.0, n)]).unwrap();
    let initial = state([100.0, 0.0, 0.0], [-200.0, 0.0, 0.0], 0);
    let mut t = flight(initial);
    let mut reference = t.clone();
    let thrust = Thrust {
        direction: [-1.0, 0.0, 0.0],
        funded_joules: 100_000.0,
        propulsion: Propulsion::default(),
    };
    let air = |_, _| Ok(Some(Air::new(1.2, [0.0; 3])?));
    let result = t
        .step_with_contacts(
            n,
            thrust,
            aero(),
            &scene,
            16.0,
            ResponsePolicy::new(0.0, 0.0, 0.0, None).unwrap(),
            |_, _| Ok(vec![source.clone()]),
            air,
        )
        .unwrap();
    let used = thrust.funded_joules - result.unused_funded_thrust_joules;
    assert!(used > 0.0 && used < thrust.funded_joules);
    assert!((used / thrust.funded_joules - result.elapsed_nanos as f64 / n as f64).abs() < 1e-12);
    let integrated = reference
        .step_with_drag(
            result.elapsed_nanos,
            Thrust {
                funded_joules: used,
                ..thrust
            },
            aero(),
            |_, _| Ok(vec![source.clone()]),
            air,
        )
        .unwrap();
    assert!(
        length(
            integrated
                .position
                .relative_meters(result.hit.as_ref().unwrap().position)
                .unwrap()
        ) < 1e-6
    );
    assert_eq!(result.state.position, integrated.position);
    assert_eq!(
        result.resolution.unwrap().consequence,
        Consequence::GroundedAfterCrash
    );
    assert_eq!(result.state.velocity_mps, [0.0; 3]);
}
#[test]
fn moving_frame_response_bounce_grounding_and_fatal_policy_have_correct_energy() {
    let n = 1_000_000_000;
    let scene = Scene::new(vec![sphere("body", [0.0; 3], [100.0, 0.0, 0.0], 10.0, n)]).unwrap();
    let ship = linear([-1000.0, 0.0, 0.0], [2000.0, 0.0, 0.0], n);
    let h = scene.sweep(&ship, 16.0).unwrap().0.unwrap();
    let incoming = ship.state_at(h.elapsed_nanos).unwrap();
    let mut invalid = h.clone();
    invalid.normal = scale(h.normal, 2.0);
    assert!(
        invalid
            .resolve(incoming, Propulsion::default(), ResponsePolicy::default())
            .is_err(),
        "a nonunit normal must not create rebound energy"
    );
    let mut invalid = h.clone();
    invalid.spatial_uncertainty_meters = f64::NAN;
    assert!(
        invalid
            .resolve(incoming, Propulsion::default(), ResponsePolicy::default())
            .is_err()
    );
    let bounce = h
        .resolve(incoming, Propulsion::default(), ResponsePolicy::default())
        .unwrap();
    assert_eq!(bounce.consequence, Consequence::Rebounding);
    assert!(length(sub(bounce.state.velocity_mps, [92.0, 0.0, 0.0])) < 1e-8);
    assert!(bounce.dissipated_joules > 0.0);
    let fatal = h
        .resolve(
            incoming,
            Propulsion::default(),
            ResponsePolicy::new(0.35, 8.0, 0.1, Some(1000.0)).unwrap(),
        )
        .unwrap();
    assert_eq!(fatal.consequence, Consequence::Fatal);
    assert!(length(sub(fatal.state.velocity_mps, [100.0, 0.0, 0.0])) < 1e-9);
    let grounded = h
        .resolve(
            incoming,
            Propulsion::default(),
            ResponsePolicy::new(0.0, 0.0, 0.0, None).unwrap(),
        )
        .unwrap();
    assert_eq!(grounded.consequence, Consequence::GroundedAfterCrash);
    assert!((grounded.dissipated_joules - 0.5 * 100_000.0 * 1900.0_f64.powi(2)).abs() < 0.01);
    assert!(
        h.resolve(
            State {
                epoch: epoch(),
                ..incoming
            },
            Propulsion::default(),
            ResponsePolicy::default()
        )
        .is_err()
    );
}
#[test]
fn clear_space_is_exactly_atmospheric_integrator_and_failures_are_atomic() {
    let scene = Scene::new(vec![]).unwrap();
    let mut t = flight(state([100.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0));
    let mut expected = t.clone();
    let source = Source {
        id: id("body"),
        position: pos([0.0; 3]),
        mu_m3_per_s2: 1.0,
        radius_meters: 10.0,
    };
    let initial = t.state();
    let bad = Scene::new(vec![sphere(
        "bad",
        [1000.0, 0.0, 0.0],
        [0.0; 3],
        10.0,
        500_000_000,
    )])
    .unwrap();
    assert!(
        t.step_with_contacts(
            1_000_000_000,
            coast(),
            aero(),
            &bad,
            16.0,
            ResponsePolicy::default(),
            |_, _| Ok(vec![source.clone()]),
            |_, _| Ok(None)
        )
        .is_err()
    );
    assert_eq!(t.state(), initial);
    assert!(
        t.step_with_contacts(
            1_000_000_000,
            coast(),
            aero(),
            &scene,
            16.0,
            ResponsePolicy::default(),
            |at, _| if at == epoch() {
                Ok(vec![source.clone()])
            } else {
                Err(error("ephemeris missing"))
            },
            |_, _| Ok(None)
        )
        .is_err()
    );
    assert_eq!(t.state(), initial);
    let result = t
        .step_with_contacts(
            1_000_000_000,
            coast(),
            aero(),
            &scene,
            16.0,
            ResponsePolicy::default(),
            |_, _| Ok(vec![source.clone()]),
            |_, _| Ok(None),
        )
        .unwrap();
    expected
        .step_with_drag(
            1_000_000_000,
            coast(),
            aero(),
            |_, _| Ok(vec![source.clone()]),
            |_, _| Ok(None),
        )
        .unwrap();
    assert_eq!(result.state, expected.state());
    assert!(result.hit.is_none());
    assert_eq!(result.unused_funded_thrust_joules, 0.0);
}
#[test]
fn malformed_identity_geometry_and_numerics_fail_without_silent_collision_gaps() {
    let s = sphere("same", [0.0; 3], [0.0; 3], 10.0, 1_000_000_000);
    assert!(Scene::new(vec![s.clone(), s]).is_err());
    assert!(
        Collider::sphere(
            id("bad"),
            linear([0.0; 3], [0.0; 3], 1_000_000_000),
            f64::NAN,
            [0.0; 3]
        )
        .is_err()
    );
    let point = linear([0.0; 3], [0.0; 3], 1_000_000_000);
    assert!(
        Collider::triangle(id("degenerate"), 1, [point.clone(), point.clone(), point]).is_err()
    );
    assert!(ResponsePolicy::new(1.1, 8.0, 0.1, None).is_err());
    let scene = Scene::new(vec![]).unwrap();
    let ship = linear([0.0; 3], [0.0; 3], 1_000_000_000);
    assert!(scene.sweep(&ship, 0.0).is_err());
}
#[test]
fn common_orbital_velocity_and_large_galactic_anchor_preserve_event_and_relative_response() {
    let n = 1_000_000_000;
    let run = |frame: f64, cell: i64| {
        let make = |p: [f64; 3], v: [f64; 3]| {
            let start = State {
                position: GalacticPosition::new([cell, 0, 0], p).unwrap(),
                velocity_mps: v,
                epoch: epoch(),
            };
            CubicPath::between(
                start,
                State {
                    position: start.position.translated(scale(v, 1.0)).unwrap(),
                    epoch: epoch().advance_nanos(n).unwrap(),
                    ..start
                },
            )
            .unwrap()
        };
        let scene = Scene::new(vec![
            Collider::sphere(
                id("body"),
                make([0.0; 3], [frame + 100.0, 0.0, 0.0]),
                10.0,
                [0.0; 3],
            )
            .unwrap(),
        ])
        .unwrap();
        let ship = make([-1000.0, 0.0, 0.0], [frame + 2000.0, 0.0, 0.0]);
        let hit = scene.sweep(&ship, 16.0).unwrap().0.unwrap();
        let result = hit
            .resolve(
                ship.state_at(hit.elapsed_nanos).unwrap(),
                Propulsion::default(),
                ResponsePolicy::default(),
            )
            .unwrap();
        (
            hit.elapsed_nanos,
            result.relative_impact_speed_mps,
            result.dissipated_joules,
        )
    };
    let a = run(0.0, 0);
    let b = run(30000.0, 1_000_000_000);
    assert!((a.0 - b.0).abs() <= 200);
    assert!((a.1 - b.1).abs() < 1e-7);
    assert!((a.2 - b.2).abs() < 0.1);
}
#[test]
fn triangles_from_production_terrain_queries_contact_with_declared_ephemeris_motion() {
    use salimon_world::{CELESTIAL_BODIES, CelestialBodyId, terrain::TerrainMode};
    let body = *CELESTIAL_BODIES
        .iter()
        .find(|b| b.id == CelestialBodyId::Earth)
        .unwrap();
    let points = [
        [1.0, 0.0, 0.001],
        [1.0, 0.001, -0.001],
        [1.0, -0.001, -0.001],
    ]
    .map(|d| {
        TerrainMode::Representative
            .sample(body, scale(d, length(d).recip()))
            .body_local_position_meters
    });
    let centroid = scale(add(add(points[0], points[1]), points[2]), 1.0 / 3.0);
    let normal = cross(sub(points[1], points[0]), sub(points[2], points[0]));
    let normal = scale(normal, length(normal).recip());
    let orbit = [30000.0, 0.0, 0.0];
    let n = 1_000_000_000;
    let scene = Scene::new(vec![
        Collider::triangle(
            id("terrain-patch"),
            999,
            points.map(|p| linear(p, orbit, n)),
        )
        .unwrap(),
    ])
    .unwrap();
    let ship = linear(
        add(centroid, scale(normal, 100.0)),
        add(orbit, scale(normal, -200.0)),
        n,
    );
    let hit = scene.sweep(&ship, 2.0).unwrap().0.unwrap();
    assert!((hit.elapsed_nanos as f64 * 1e-9 - 0.49).abs() < 0.001 / 200.0 + 1e-9);
    assert_eq!(hit.primitive, 999);
    assert!(length(sub(hit.surface_velocity_mps, orbit)) < 1e-8);
}
#[test]
fn bounded_work_exhaustion_rejects_without_publishing_a_fake_contact_or_free_movement() {
    let n = 1_000_000_000;
    let bodies = (0..=MAX_DISTANCE_SAMPLES)
        .map(|i| sphere(&format!("overlap-{i}"), [0.0; 3], [0.0; 3], 10.0, n))
        .collect();
    let scene = Scene::new(bodies).unwrap();
    let mut t = flight(state([20.0, 0.0, 0.0], [0.0; 3], 0));
    let initial = t.state();
    let source = Source {
        id: id("source"),
        position: pos([0.0; 3]),
        mu_m3_per_s2: 1.0,
        radius_meters: 10.0,
    };
    let result = t.step_with_contacts(
        n,
        coast(),
        aero(),
        &scene,
        16.0,
        ResponsePolicy::default(),
        |_, _| Ok(vec![source.clone()]),
        |_, _| Ok(None),
    );
    assert!(result.unwrap_err().to_string().contains("work limit"));
    assert_eq!(t.state(), initial);
}
#[test]
fn planet_scale_tangent_and_near_miss_are_bounded_without_tunneling() {
    let radius = 6_371_000.0;
    let n = 1_000_000_000;
    let scene = Scene::new(vec![sphere("earth", [0.0; 3], [0.0; 3], radius, n)]).unwrap();
    let path = linear([-100000.0, radius + 16.0, 0.0], [200000.0, 0.0, 0.0], n);
    let (hit, stats) = scene.sweep(&path, 16.0).unwrap();
    assert!(hit.is_some());
    assert!(stats.distance_samples < 200, "{stats:?}");
    let miss = linear([-100000.0, radius + 16.01, 0.0], [200000.0, 0.0, 0.0], n);
    let (hit, stats) = scene.sweep(&miss, 16.0).unwrap();
    assert!(hit.is_none());
    assert!(stats.distance_samples < 200);
}
