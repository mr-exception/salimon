use super::*;

fn sphere(position: [f64; 3], radius: f64) -> ObjectState {
    ObjectState {
        position,
        radius,
        ground_support_meters: radius,
        velocity: [0.0; 3],
        angular_velocity: [0.0; 3],
        orientation: [0.0, 0.0, 0.0, 1.0],
        mass_kg: 10.0,
        side_meters: 2.0 * radius,
        hull: None,
        surface: Surface::Floor { height_meters: 0.0 },
    }
}

#[test]
fn sphere_pair_impulses_conserve_momentum_and_reduce_energy() {
    let mut a = sphere([0.0, 1.0, 0.0], 0.1);
    let mut b = sphere([0.2, 1.0, 0.0], 0.1);
    a.mass_kg = 2.0;
    b.mass_kg = 10.0;
    a.velocity = [2.0, 0.5, 0.0];
    b.velocity = [-1.0, 0.0, 0.0];
    let momentum = |a: &ObjectState, b: &ObjectState| {
        add(scale(a.velocity, a.mass_kg), scale(b.velocity, b.mass_kg))
    };
    let energy = |p: &ObjectState| {
        0.5 * p.mass_kg * dot(p.velocity, p.velocity)
            + 0.2 * p.mass_kg * p.radius.powi(2) * dot(p.angular_velocity, p.angular_velocity)
    };
    let before = momentum(&a, &b);
    let e = energy(&a) + energy(&b);
    pair_impulse(&mut a, &mut b, [1.0, 0.0, 0.0]);
    assert!(length(sub(momentum(&a, &b), before)) < 1e-12);
    assert!(energy(&a) + energy(&b) <= e);
    assert!(length(a.angular_velocity) > 0.0);
}

#[test]
fn sphere_stack_sleeps_separates_and_wakes_after_support_removal() {
    let mut bodies = [
        sphere([0.0, 0.1, 0.0], 0.1),
        sphere([0.0, 0.3, 0.0], 0.1),
        sphere([0.0, 0.5, 0.0], 0.1),
    ];
    let mut tracker = SleepTracker::default();
    for _ in 0..150 {
        advance_with_sleep(
            &mut tracker,
            &[0, 1, 2],
            &mut bodies,
            Duration::from_millis(16),
            |_, _| true,
            false,
        );
    }
    let stats = advance_with_sleep(
        &mut tracker,
        &[0, 1, 2],
        &mut bodies,
        Duration::from_millis(16),
        |_, _| true,
        true,
    );
    assert_eq!(stats.sleeping_objects, 3);
    assert!((bodies[0].position[1] - 0.1).abs() < 0.001);
    for pair in bodies.windows(2) {
        assert!(length(sub(pair[0].position, pair[1].position)) >= 0.199);
    }
    let before = bodies[1].position[1];
    let stats = advance_with_sleep(
        &mut tracker,
        &[1, 2],
        &mut bodies[1..],
        Duration::from_millis(16),
        |_, _| true,
        true,
    );
    assert!(stats.integrated_objects > 0);
    assert!(bodies[1].position[1] < before);
}

#[test]
fn five_hundred_spheres_use_grid_and_sleep_without_sat_storage() {
    let mut bodies: Vec<_> = (0..500)
        .map(|i| sphere([(i % 25) as f64 * 0.25, 0.1, (i / 25) as f64 * 0.25], 0.1))
        .collect();
    let ids: Vec<_> = (0..500).collect();
    let mut tracker = SleepTracker::default();
    let active = advance_with_sleep(
        &mut tracker,
        &ids,
        &mut bodies,
        Duration::from_millis(16),
        |_, _| true,
        true,
    );
    assert!(active.pair_visits < 500 * 499 / 2);
    assert_eq!(active.projection_builds, 0);
    for _ in 0..40 {
        advance_with_sleep(
            &mut tracker,
            &ids,
            &mut bodies,
            Duration::from_millis(16),
            |_, _| true,
            false,
        );
    }
    let before = bodies.clone();
    let asleep = advance_with_sleep(
        &mut tracker,
        &ids,
        &mut bodies,
        Duration::from_millis(16),
        |_, _| true,
        true,
    );
    assert_eq!(asleep.sleeping_objects, 500);
    assert_eq!(asleep.integrated_objects, 0);
    assert_eq!(asleep.pair_visits, 0);
    assert_eq!(asleep.projection_storage_bytes, 0);
    assert_eq!(bodies, before);
}

#[test]
fn fast_small_sphere_cannot_cross_a_stationary_neighbor_in_an_ordinary_tick() {
    let mut bodies = [
        sphere([-0.3, 1.0, 0.0], 0.02),
        sphere([0.0, 1.0, 0.0], 0.02),
    ];
    for p in &mut bodies {
        p.surface = Surface::UnpoweredFloor {
            height_meters: 0.0,
            ceiling_meters: 3.0,
        };
    }
    bodies[0].velocity = [30.0, 0.0, 0.0];
    let stats = advance_profiled(&mut bodies, Duration::from_millis(16), |_, _| true);
    assert!(stats.substeps > 2);
    assert!(stats.contacts > 0);
    assert!(bodies[0].position[0] < bodies[1].position[0]);
    assert!(length(sub(bodies[0].position, bodies[1].position)) >= 0.039);
}
