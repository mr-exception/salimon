use super::*;
use salimon_universe::{CatalogId, gravity::Config};
#[test]
fn indexed_and_complete_source_trajectories_match_exactly() {
    use salimon_universe::gravity::SourceIndex;
    let mut sources = vec![earth()];
    for i in 0..64 {
        sources.push(Source {
            id: CatalogId::from_source("fixture", &format!("far-{i}")).unwrap(),
            position: pos(1.0e12 + i as f64 * 1e8),
            mu_m3_per_s2: 1e10,
            radius_meters: 0.0,
        });
    }
    let index = std::sync::Arc::new(SourceIndex::new(sources.clone()).unwrap());
    let mut direct = trajectory(pos(7_000_000.0), [0.0, 7500.0, 0.0]);
    let mut indexed = direct.clone();
    for _ in 0..1000 {
        direct
            .step(1_000_000_000, coast(), |_, _| Ok(sources.clone()))
            .unwrap();
        indexed
            .step_indexed(1_000_000_000, coast(), |_, _| Ok(index.clone()))
            .unwrap();
    }
    assert_eq!(direct.state(), indexed.state());
}
#[test]
fn moving_body_gravity_assist_conserves_body_frame_energy_and_changes_inertial_energy() {
    let origin = pos(0.0);
    let initial = origin
        .translated([40_000_000.0, 20_000_000.0, 0.0])
        .unwrap();
    let mut t = trajectory(initial, [7000.0, 0.0, 0.0]);
    let source_at = |at: Epoch| {
        let seconds = at.seconds_since(epoch()).unwrap();
        let mut e = earth();
        e.position = origin.translated([15000.0 * seconds, 0.0, 0.0]).unwrap();
        e
    };
    let energy = |state: State| {
        let e = source_at(state.epoch);
        let r = length(state.position.relative_meters(e.position).unwrap());
        let relative = salimon_math::sub(state.velocity_mps, [15000.0, 0.0, 0.0]);
        length(relative).powi(2) / 2.0 - e.mu_m3_per_s2 / r
    };
    let before = energy(t.state());
    for _ in 0..10000 {
        t.step(2_000_000_000, coast(), |at, _| Ok(vec![source_at(at)]))
            .unwrap();
    }
    let after = energy(t.state());
    let drift = ((after - before) / before).abs();
    eprintln!(
        "moving-body flyby relative-energy drift {drift:.12}; initial inertial speed 7000; final {}",
        length(t.state().velocity_mps)
    );
    assert!(drift < 1e-5);
    assert!(length(t.state().velocity_mps) > 7000.0);
}
#[test]
fn three_body_ephemeris_field_matches_independent_rk4_reference() {
    // Reference RK4 is independent of the production Verlet/selector kernel.
    // Both consume the same declared analytic Sun/Earth/Moon field.
    use salimon_math::{add, scale, sub};
    let sun_mu = 1.32712440018e20;
    let earth_mu = 3.986004418e14;
    let moon_mu = 4.9048695e12;
    let a = 149_597_870_700.0_f64;
    let b = 384_400_000.0_f64;
    let ne = (sun_mu / a.powi(3)).sqrt();
    let nm = (earth_mu / b.powi(3)).sqrt();
    let centers = |time: f64| {
        let earth = [a * (ne * time).cos(), a * (ne * time).sin(), 0.0];
        vec![
            ([0.0; 3], sun_mu),
            (earth, earth_mu),
            (
                add(earth, [b * (nm * time).cos(), b * (nm * time).sin(), 0.0]),
                moon_mu,
            ),
        ]
    };
    let accel = |position: [f64; 3], time: f64| {
        let mut out = [0.0; 3];
        for (center, mu) in centers(time) {
            let d = sub(center, position);
            let r = d[0].hypot(d[1]).hypot(d[2]);
            out = add(out, d.map(|v| mu * (v / r) / r / r));
        }
        out
    };
    let start = [a + 7_000_000.0, 0.0, 0.0];
    let velocity = [0.0, a * ne + (earth_mu / 7_000_000.0).sqrt(), 0.0];
    let mut t = trajectory(pos(0.0).translated(start).unwrap(), velocity);
    let (mut rp, mut rv) = (start, velocity);
    for step in 0..6000 {
        let time = step as f64;
        t.step(1_000_000_000, coast(), |at, _| {
            let seconds = at.seconds_since(epoch())?;
            centers(seconds)
                .into_iter()
                .enumerate()
                .map(|(i, (center, mu))| {
                    Ok(Source {
                        id: CatalogId::from_source("fixture", &format!("body-{i}"))?,
                        position: pos(0.0).translated(center)?,
                        mu_m3_per_s2: mu,
                        radius_meters: 0.0,
                    })
                })
                .collect()
        })
        .unwrap();
        for half in 0..2 {
            let tm = time + half as f64 * 0.5;
            let h = 0.5;
            let (kp1, kv1) = (rv, accel(rp, tm));
            let (kp2, kv2) = (
                add(rv, scale(kv1, h / 2.0)),
                accel(add(rp, scale(kp1, h / 2.0)), tm + h / 2.0),
            );
            let (kp3, kv3) = (
                add(rv, scale(kv2, h / 2.0)),
                accel(add(rp, scale(kp2, h / 2.0)), tm + h / 2.0),
            );
            let (kp4, kv4) = (
                add(rv, scale(kv3, h)),
                accel(add(rp, scale(kp3, h)), tm + h),
            );
            rp = add(
                rp,
                scale(
                    add(add(kp1, scale(kp2, 2.0)), add(scale(kp3, 2.0), kp4)),
                    h / 6.0,
                ),
            );
            rv = add(
                rv,
                scale(
                    add(add(kv1, scale(kv2, 2.0)), add(scale(kv3, 2.0), kv4)),
                    h / 6.0,
                ),
            );
        }
    }
    let error = length(sub(
        t.state().position.relative_meters(pos(0.0)).unwrap(),
        rp,
    ));
    eprintln!("Sun/Earth/Moon 6000s Verlet versus RK4 position error {error:.9} m");
    assert!(error < 25.0);
}
#[test]
fn moving_ephemeris_frame_is_queried_at_both_step_epochs() {
    use salimon_universe::{
        coordinates::{FrameId, Rotation, SystemAnchor},
        ephemerides::{self, BodyTrack, Model, Sample},
    };
    let system = CatalogId::from_source("fixture", "system").unwrap();
    let body = CatalogId::from_source("fixture", "moving-body").unwrap();
    let mut trajectory = trajectory(pos(7_000_000.0), [0.0; 3]);
    let mut seen = Vec::new();
    trajectory
        .step(1_000_000_000, coast(), |at, _| {
            seen.push(at);
            let track = BodyTrack {
                object: body.clone(),
                parent: FrameId::System(system.clone()),
                model: Model::Linear {
                    sample: Sample {
                        epoch: epoch(),
                        position_meters: [0.0; 3],
                        velocity_mps: [10.0, 0.0, 0.0],
                    },
                    valid_seconds: 2.0,
                },
                spin_epoch: epoch(),
                spin_radians: 0.0,
                spin_rps: 0.0,
            };
            let anchor = SystemAnchor {
                system: system.clone(),
                epoch: epoch(),
                position: pos(0.0),
                velocity_mps: [0.0; 3],
                rotation: Rotation::IDENTITY,
            };
            let (tree, _) = ephemerides::frame_tree(anchor, &[track], at)?;
            Ok(vec![Source::from_frame(
                &tree,
                body.clone(),
                at,
                3.986004418e14,
                6_371_000.0,
            )?])
        })
        .unwrap();
    assert_eq!(
        seen,
        vec![epoch(), epoch().advance_nanos(1_000_000_000).unwrap()]
    );
    assert!(trajectory.state().velocity_mps[0] < 0.0);
}
fn epoch() -> Epoch {
    Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], TimeScale::Tdb).unwrap()
}
fn pos(x: f64) -> GalacticPosition {
    GalacticPosition::new([1_000_000_000; 3], [x, 0.0, 0.0]).unwrap()
}
fn trajectory(position: GalacticPosition, velocity: [f64; 3]) -> Trajectory {
    Trajectory::new(
        State {
            position,
            velocity_mps: velocity,
            epoch: epoch(),
        },
        Selector::new(Config::default()).unwrap(),
        2_000_000_000,
    )
    .unwrap()
}
fn earth() -> Source {
    Source {
        id: CatalogId::from_source("jpl", "399").unwrap(),
        position: pos(0.0),
        mu_m3_per_s2: 3.986004418e14,
        radius_meters: 6_371_000.0,
    }
}
fn coast() -> Thrust {
    Thrust::coasting(Propulsion::default())
}
fn circular(step: i64) -> (State, f64) {
    let radius = 7_000_000.0;
    let speed = (earth().mu_m3_per_s2 / radius).sqrt();
    let mut t = trajectory(pos(radius), [0.0, speed, 0.0]);
    for _ in 0..(6_000_000_000_000_i64 / step) {
        t.step(step, coast(), |_, _| Ok(vec![earth()])).unwrap();
    }
    let angle = speed / radius * 6000.0;
    let actual = t.state.position.relative_meters(pos(0.0)).unwrap();
    let expected = [radius * angle.cos(), radius * angle.sin(), 0.0];
    (t.state, length(salimon_math::sub(actual, expected)))
}
#[test]
fn analytic_orbit_drift_converges_at_second_order() {
    let (_, coarse) = circular(2_000_000_000);
    let (fine_state, fine) = circular(1_000_000_000);
    eprintln!("circular orbit 6000s: coarse {coarse:.9} m; fine {fine:.9} m");
    // Numerical test budgets: 6000 s circular two-body fixture at 7000 km.
    // They do not approve interpolation, source truncation or physical models.
    assert!(fine < 25.0, "fine drift {fine} m");
    assert!(
        coarse > fine * 3.8 && coarse < fine * 4.2,
        "coarse {coarse}, fine {fine}"
    );
    let radius = length(fine_state.position.relative_meters(pos(0.0)).unwrap());
    let energy = length(fine_state.velocity_mps).powi(2) / 2.0 - earth().mu_m3_per_s2 / radius;
    assert!((energy + earth().mu_m3_per_s2 / 14_000_000.0).abs() < 0.01);
}
#[test]
fn thrust_uses_actual_mass_and_funded_joules_without_speed_cap() {
    let mut t = trajectory(pos(0.0), [4.0e8, 0.0, 0.0]);
    let thrust = Thrust {
        direction: [1.0, 0.0, 0.0],
        funded_joules: 100_000.0,
        propulsion: Propulsion::new(100_000.0, 1.0).unwrap(),
    };
    t.step(1_000_000_000, thrust, |_, _| Ok(vec![])).unwrap();
    assert_eq!(t.state.velocity_mps[0], 400_000_001.0);
    let thrust = Thrust {
        propulsion: Propulsion::new(200_000.0, 1.0).unwrap(),
        ..thrust
    };
    t.step(1_000_000_000, thrust, |_, _| Ok(vec![])).unwrap();
    assert_eq!(t.state.velocity_mps[0], 400_000_001.5);
}
#[test]
fn provider_failure_does_not_commit_epoch_position_or_selector() {
    let mut t = trajectory(pos(7_000_000.0), [0.0, 7500.0, 0.0]);
    let before = t.state();
    let mut calls = 0;
    assert!(
        t.step(1_000_000_000, coast(), |_, _| {
            calls += 1;
            if calls == 2 {
                Err(error("fixture missing final ephemeris"))
            } else {
                Ok(vec![earth()])
            }
        })
        .is_err()
    );
    assert_eq!(t.state(), before);
    assert!(t.step(3_000_000_000, coast(), |_, _| Ok(vec![])).is_err());
    assert_eq!(t.state(), before);
    let mut reference = t.clone();
    assert_eq!(
        t.step(1_000_000_000, coast(), |_, _| Ok(vec![earth()]))
            .unwrap(),
        reference
            .step(1_000_000_000, coast(), |_, _| Ok(vec![earth()]))
            .unwrap()
    );
}
#[test]
fn fixed_steps_are_exactly_repeatable_and_escape_energy_remains_positive() {
    let initial = trajectory(pos(7_000_000.0), [0.0, 12_000.0, 0.0]);
    let mut a = initial.clone();
    let mut b = initial;
    for _ in 0..1000 {
        a.step(1_000_000_000, coast(), |_, _| Ok(vec![earth()]))
            .unwrap();
    }
    for _ in 0..10 {
        for _ in 0..100 {
            b.step(1_000_000_000, coast(), |_, _| Ok(vec![earth()]))
                .unwrap();
        }
    }
    assert_eq!(a.state(), b.state());
    let r = length(a.state.position.relative_meters(pos(0.0)).unwrap());
    let energy = length(a.state.velocity_mps).powi(2) / 2.0 - earth().mu_m3_per_s2 / r;
    let initial = 12_000.0_f64.powi(2) / 2.0 - earth().mu_m3_per_s2 / 7_000_000.0;
    assert!(energy > 0.0);
    assert!((energy - initial).abs() / initial < 1e-5);
}
