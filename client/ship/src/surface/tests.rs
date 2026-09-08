use super::*;
use crate::{
    EnergyConfig,
    trajectory::{Thrust, Trajectory},
};
use salimon_universe::gravity::{Config, Selector, Source};
fn epoch() -> Epoch {
    Epoch::calendar([2026, 1, 1], [0; 4], TimeScale::Tt).unwrap()
}
fn origin() -> GalacticPosition {
    GalacticPosition::new([1_000_000, -900, 11], [0.0; 3]).unwrap()
}
fn controls() -> Controls {
    Controls {
        cockpit: true,
        engines_on: true,
        door: DoorState::Closed,
        assist_active: false,
        construction_ready: true,
    }
}
fn motion(_: &CatalogId, at: Epoch) -> Result<BodyMotion, Error> {
    let t = at.seconds_since(epoch())?;
    Ok(BodyMotion {
        epoch: at,
        center: origin().translated([30000.0 * t, 0.0, 0.0])?,
        velocity_mps: [30000.0, 0.0, 0.0],
        acceleration_mps2: [0.0; 3],
    })
}
fn site(radius: f64, spin: f64) -> Site {
    Site::new(
        CatalogId::from_source("test", "body").unwrap(),
        epoch(),
        [radius + 16.0, 0.0, 0.0],
        [0.0, 0.0, spin],
    )
    .unwrap()
}
fn gravity(s: &Site, mu: f64, at: Epoch, p: GalacticPosition) -> Result<[f64; 3], Error> {
    let d = motion(s.body(), at)?.center.relative_meters(p)?;
    let r = length(d);
    Ok(scale(d, mu / r.powi(3)))
}
fn activate(
    s: &Site,
    mu: f64,
    kind: ManeuverKind,
    start: State,
    p: Propulsion,
    c: Controls,
    core: &mut EnergyCore,
) -> Result<Maneuver, Error> {
    Maneuver::activate(
        kind,
        start,
        s,
        p,
        c,
        ManeuverConfig::default(),
        core,
        motion,
        |at, pos| gravity(s, mu, at, pos),
    )
}
#[test]
fn rotation_orbit_and_support_force_match_independent_newtonian_result() {
    for (radius, mu, spin) in [
        (6_371_000.0, 3.986004418e14, 7.292115e-5),
        (1_737_400.0, 4.9048695e12, 2.66e-6),
        (3_389_500.0, 4.282837e13, 7.09e-5),
    ] {
        let s = site(radius, spin);
        for time in [0, 600, 86400] {
            let at = epoch().advance_nanos(time * 1_000_000_000).unwrap();
            let k = s.at(at, motion(s.body(), at).unwrap()).unwrap();
            let p = Propulsion::default();
            let support = s
                .support(
                    at,
                    motion(s.body(), at).unwrap(),
                    gravity(&s, mu, at, k.state.position).unwrap(),
                    p,
                    0.0,
                )
                .unwrap();
            let r = radius + 16.0;
            let expected = p.total_mass_kg() * (mu / r / r - spin * spin * r);
            assert!((length(support.force_newtons) - expected).abs() < 0.001);
            let angle = spin * time as f64;
            let rel = k
                .state
                .position
                .relative_meters(motion(s.body(), at).unwrap().center)
                .unwrap();
            assert!(length(sub(rel, [r * angle.cos(), r * angle.sin(), 0.0])) < 1e-5);
            assert!(
                length(sub(
                    k.state.velocity_mps,
                    [
                        30000.0 - spin * r * angle.sin(),
                        spin * r * angle.cos(),
                        0.0
                    ]
                )) < 1e-8
            );
        }
    }
    let s = site(1000.0, 1.0);
    let k = s.at(epoch(), motion(s.body(), epoch()).unwrap()).unwrap();
    assert!(
        s.support(
            epoch(),
            motion(s.body(), epoch()).unwrap(),
            [-1.0, 0.0, 0.0],
            Propulsion::default(),
            1.0
        )
        .is_err()
    );
    assert!(k.state.velocity_mps[1] > 1000.0);
}
#[test]
fn full_takeoff_coast_landing_cycle_has_continuous_inertial_handoffs_and_exact_receipts() {
    for (radius, mu, spin) in [
        (6_371_000.0, 3.986004418e14, 7.292115e-5),
        (1_737_400.0, 4.9048695e12, 2.66e-6),
    ] {
        for mass in [100_000.0, 200_000.0] {
            let s = site(radius, spin);
            let p = Propulsion::new(mass, 1.0).unwrap();
            let start = s
                .at(epoch(), motion(s.body(), epoch()).unwrap())
                .unwrap()
                .state;
            let mut core = EnergyCore::default();
            let before = core.stored_joules();
            let mut takeoff = activate(
                &s,
                mu,
                ManeuverKind::Takeoff,
                start,
                p,
                controls(),
                &mut core,
            )
            .unwrap();
            assert!(
                length(sub(
                    takeoff.state().unwrap().velocity_mps,
                    start.velocity_mps
                )) < 1e-9
            );
            assert!(
                length(
                    takeoff
                        .state()
                        .unwrap()
                        .position
                        .relative_meters(start.position)
                        .unwrap()
                ) < 1e-6
            );
            let charge = takeoff.charged_joules();
            assert!((before - core.stored_joules() - charge).abs() < 1e-5);
            for _ in 0..600 {
                takeoff.advance(10_000_000).unwrap();
            }
            assert!(takeoff.finished());
            assert!((before - core.stored_joules() - charge).abs() < 1e-5);
            let end = takeoff.state().unwrap();
            let surf = s
                .at(end.epoch, motion(s.body(), end.epoch).unwrap())
                .unwrap();
            let offset = end.position.relative_meters(surf.state.position).unwrap();
            assert!((length(offset) - 50.0).abs() < 1e-6);
            assert!(
                length(sub(
                    end.velocity_mps,
                    add(
                        surf.state.velocity_mps,
                        cross(s.angular_velocity_rps, offset)
                    )
                )) < 1e-6
            );
            // Production #417 composition: no velocity reset when free flight starts.
            let mut flight = Trajectory::new(
                end,
                Selector::new(Config::default()).unwrap(),
                1_000_000_000,
            )
            .unwrap();
            flight
                .step(1_000_000, Thrust::coasting(p), |at, _| {
                    Ok(vec![Source {
                        id: s.body.clone(),
                        position: motion(s.body(), at)?.center,
                        mu_m3_per_s2: mu,
                        radius_meters: radius,
                    }])
                })
                .unwrap();
            assert!(length(sub(flight.state().velocity_mps, end.velocity_mps)) < 0.02);
            let mut land = activate(
                &s,
                mu,
                ManeuverKind::Landing,
                flight.state(),
                p,
                controls(),
                &mut core,
            )
            .unwrap();
            let (touchdown, left) = land.advance(7_000_000_000).unwrap();
            assert_eq!(left, 1_000_000_000);
            let expected = s
                .at(touchdown.epoch, motion(s.body(), touchdown.epoch).unwrap())
                .unwrap();
            assert!(
                length(
                    touchdown
                        .position
                        .relative_meters(expected.state.position)
                        .unwrap()
                ) < 1e-6
            );
            assert!(length(sub(touchdown.velocity_mps, expected.state.velocity_mps)) < 1e-6);
            s.support(
                touchdown.epoch,
                motion(s.body(), touchdown.epoch).unwrap(),
                gravity(&s, mu, touchdown.epoch, touchdown.position).unwrap(),
                p,
                0.1,
            )
            .unwrap();
        }
    }
}
#[test]
fn mass_gravity_and_impulse_conversion_determine_cost_without_free_energy() {
    let mut costs = vec![];
    for (mass, kappa) in [(100_000.0, 1.0), (200_000.0, 1.0), (100_000.0, 2.0)] {
        let s = site(6_371_000.0, 0.0);
        let p = Propulsion::new(mass, kappa).unwrap();
        let start = s
            .at(epoch(), motion(s.body(), epoch()).unwrap())
            .unwrap()
            .state;
        let mut core = EnergyCore::default();
        let a = activate(
            &s,
            3.986004418e14,
            ManeuverKind::Takeoff,
            start,
            p,
            controls(),
            &mut core,
        )
        .unwrap();
        // Independent high-resolution midpoint force integral for the cubic lift.
        let n = 60000;
        let mut reference = 0.0;
        for i in 0..n {
            let t = (i as f64 + 0.5) * 6.0 / n as f64;
            let u = t / 6.0;
            let h = 50.0 * (3.0 * u * u - 2.0 * u * u * u);
            let accel = 50.0 * (6.0 - 12.0 * u) / 36.0;
            reference +=
                mass / kappa * (accel + 3.986004418e14 / (6_371_016.0 + h).powi(2)).abs() * 6.0
                    / n as f64;
        }
        assert!((a.charged_joules() - reference).abs() < 1.0);
        costs.push(a.charged_joules());
    }
    assert!((costs[1] - 2.0 * costs[0]).abs() < 1e-6);
    assert!((costs[2] - 0.5 * costs[0]).abs() < 1e-6);
}
#[test]
fn interlocks_insufficient_thrust_energy_bad_ephemerides_and_invalid_inputs_are_atomic() {
    let s = site(6_371_000.0, 0.0);
    let start = s
        .at(epoch(), motion(s.body(), epoch()).unwrap())
        .unwrap()
        .state;
    for c in [
        Controls {
            cockpit: false,
            ..controls()
        },
        Controls {
            engines_on: false,
            ..controls()
        },
        Controls {
            door: DoorState::Open,
            ..controls()
        },
        Controls {
            construction_ready: false,
            ..controls()
        },
        Controls {
            assist_active: true,
            ..controls()
        },
    ] {
        let mut core = EnergyCore::default();
        let before = core.stored_joules();
        assert!(
            activate(
                &s,
                3.986004418e14,
                ManeuverKind::Takeoff,
                start,
                Propulsion::default(),
                c,
                &mut core
            )
            .is_err()
        );
        assert_eq!(core.stored_joules(), before);
    }
    for (watts, charge) in [(1.0, 1e9), (1e8, 1.0)] {
        let mut core = EnergyCore::new(
            EnergyConfig {
                forward_watts: watts,
                reverse_watts: watts,
                ..EnergyConfig::default()
            },
            charge,
        )
        .unwrap();
        assert!(
            activate(
                &s,
                3.986004418e14,
                ManeuverKind::Takeoff,
                start,
                Propulsion::default(),
                controls(),
                &mut core
            )
            .is_err()
        );
        assert_eq!(core.stored_joules(), charge);
    }
    let mut core = EnergyCore::default();
    let before = core.stored_joules();
    assert!(
        Maneuver::activate(
            ManeuverKind::Takeoff,
            start,
            &s,
            Propulsion::default(),
            controls(),
            ManeuverConfig::default(),
            &mut core,
            |id, at| if at.seconds_since(epoch())? > 3.0 {
                Err(error("ephemeris unavailable"))
            } else {
                motion(id, at)
            },
            |_, _| Ok([-9.8, 0.0, 0.0])
        )
        .is_err()
    );
    assert_eq!(before, core.stored_joules());
    for bad in [f64::NAN, f64::INFINITY] {
        assert!(Site::new(s.body.clone(), epoch(), [bad, 0.0, 0.0], [0.0; 3]).is_err());
    }
    let bad = State {
        velocity_mps: [30000.0, 21.0, 0.0],
        ..start
    };
    assert!(
        activate(
            &s,
            3.986004418e14,
            ManeuverKind::Landing,
            bad,
            Propulsion::default(),
            controls(),
            &mut core
        )
        .is_err()
    );
}
#[test]
fn partitioned_and_whole_advances_match_and_velocity_is_the_path_derivative() {
    let s = site(6_371_000.0, 7.292115e-5);
    let start = s
        .at(epoch(), motion(s.body(), epoch()).unwrap())
        .unwrap()
        .state;
    let make = || {
        activate(
            &s,
            3.986004418e14,
            ManeuverKind::Takeoff,
            start,
            Propulsion::default(),
            controls(),
            &mut EnergyCore::default(),
        )
        .unwrap()
    };
    let mut whole = make();
    let mut split = make();
    for t in [1_234_567_890, 4_765_432_110] {
        let (end, _) = whole.advance(t).unwrap();
        let mut remaining = t;
        while remaining > 0 {
            let dt = remaining.min(7_123_456);
            split.advance(dt).unwrap();
            remaining -= dt;
        }
        assert_eq!(end, split.state().unwrap());
    }
    for nanos in [123_456_789, 2_000_001_234, 5_999_000_000] {
        let mut a = make();
        let mut b = make();
        let mut c = make();
        let (before, _) = a.advance(nanos - 100_000).unwrap();
        let (at, _) = b.advance(nanos).unwrap();
        let (after, _) = c.advance(nanos + 100_000).unwrap();
        let derivative = scale(
            after.position.relative_meters(before.position).unwrap(),
            5000.0,
        );
        assert!(length(sub(derivative, at.velocity_mps)) < 0.01);
    }
}
