use super::*;
use crate::trajectory::{State, Thrust, Trajectory};
use salimon_universe::{
    CatalogId, TimeScale,
    gravity::{Config, Selector, Source},
};
fn epoch() -> Epoch {
    Epoch::calendar([2026, 1, 1], [0; 4], TimeScale::Tt).unwrap()
}
fn pos(v: [f64; 3]) -> GalacticPosition {
    GalacticPosition::new([500_000, 0, 0], v).unwrap()
}
fn flight(p: [f64; 3], v: [f64; 3]) -> Trajectory {
    Trajectory::new(
        State {
            position: pos(p),
            velocity_mps: v,
            epoch: epoch(),
        },
        Selector::new(Config::default()).unwrap(),
        1_000_000_000,
    )
    .unwrap()
}
fn coast() -> Thrust {
    Thrust::coasting(Propulsion::default())
}
fn aero() -> Aerodynamics {
    Aerodynamics::new(1.0, 100.0).unwrap()
}
fn earth() -> Source {
    Source {
        id: CatalogId::from_source("fixture", "earth").unwrap(),
        position: pos([0.0; 3]),
        mu_m3_per_s2: 3.986004418e14,
        radius_meters: 6_371_000.0,
    }
}
#[test]
fn planet_specific_density_rotation_wind_and_model_bounds_are_explicit() {
    let earth = Atmosphere::new(
        6_371_000.0,
        1.225,
        8500.0,
        100_000.0,
        [0.0, 0.0, 7.292115e-5],
    )
    .unwrap();
    let thin =
        Atmosphere::new(3_389_500.0, 0.020, 11100.0, 100_000.0, [0.0, 0.0, 7.09e-5]).unwrap();
    assert!((earth.density_at_altitude(8500.0).unwrap() - 1.225 / 1.0_f64.exp()).abs() < 1e-12);
    assert!((thin.density_at_altitude(11100.0).unwrap() - 0.020 / 1.0_f64.exp()).abs() < 1e-12);
    assert_eq!(earth.density_at_altitude(100_000.0).unwrap(), 0.0);
    assert!(earth.density_at_altitude(99999.0).unwrap() < 1e-12);
    assert!(earth.density_at_altitude(-0.1).is_err());
    let body = BodyMotion {
        epoch: epoch(),
        center: pos([0.0; 3]),
        velocity_mps: [30000.0, 0.0, 0.0],
        acceleration_mps2: [0.0; 3],
    };
    let air = earth
        .sample(epoch(), pos([6_371_100.0, 0.0, 0.0]), body, [0.0, 5.0, 0.0])
        .unwrap();
    assert_eq!(
        air.velocity_mps(),
        [30000.0, 6_371_100.0 * 7.292115e-5 + 5.0, 0.0]
    );
    assert!(
        earth
            .sample(
                epoch().advance_nanos(1).unwrap(),
                pos([6_371_100.0, 0.0, 0.0]),
                body,
                [0.0; 3]
            )
            .is_err()
    );
    for bad in [f64::NAN, f64::INFINITY, -1.0] {
        assert!(Air::new(bad, [0.0; 3]).is_err());
        assert!(Aerodynamics::new(bad, 100.0).is_err());
    }
}
#[test]
fn quadratic_operator_matches_closed_form_and_never_reverses_relative_velocity() {
    let air = Air::new(1.225, [30000.0, 100.0, -10.0]).unwrap();
    for dt in [0.0, 0.001, 1.0, 60.0] {
        for mass in [100_000.0, 200_000.0] {
            let p = Propulsion::new(mass, 1.0).unwrap();
            let v = add(air.velocity_mps(), [0.0, 10000.0, 0.0]);
            let result = aero().velocity_after(v, p, Some(air), dt).unwrap();
            let expected = 10000.0 / (1.0 + 0.5 * 1.225 * 100.0 / mass * 10000.0 * dt);
            let relative = sub(result, air.velocity_mps());
            assert!((relative[1] - expected).abs() < 1e-9);
            assert!(relative[1] >= 0.0 && relative[1] <= 10000.0);
            assert_eq!(relative[0], 0.0);
            assert_eq!(relative[2], 0.0);
        }
    }
    let a = aero()
        .velocity_after(
            [10000.0, 0.0, 0.0],
            Propulsion::default(),
            Some(Air::new(1.0, [0.0; 3]).unwrap()),
            2.0,
        )
        .unwrap();
    let b = aero()
        .velocity_after(
            [10000.0, 0.0, 0.0],
            Propulsion::default(),
            Some(Air::new(1.0, [0.0; 3]).unwrap()),
            1.0,
        )
        .unwrap();
    let b = aero()
        .velocity_after(
            b,
            Propulsion::default(),
            Some(Air::new(1.0, [0.0; 3]).unwrap()),
            1.0,
        )
        .unwrap();
    assert!(length(sub(a, b)) < 1e-9);
}
#[test]
fn vacuum_is_exactly_the_existing_gravity_thrust_path() {
    let mut base = flight([7e6, 0.0, 0.0], [0.0, 7500.0, 0.0]);
    let mut actual = base.clone();
    for _ in 0..1000 {
        let thrust = Thrust {
            direction: [0.0, 1.0, 0.0],
            funded_joules: 100.0,
            propulsion: Propulsion::default(),
        };
        base.step(1_000_000_000, thrust, |_, _| Ok(vec![earth()]))
            .unwrap();
        actual
            .step_with_drag(
                1_000_000_000,
                thrust,
                aero(),
                |_, _| Ok(vec![earth()]),
                |_, _| Ok(None),
            )
            .unwrap();
    }
    assert_eq!(actual.state(), base.state());
}
#[test]
fn gravity_plus_drag_converges_to_terminal_speed_and_refines_at_second_order() {
    // Large-radius local fixture keeps g and density nearly constant while
    // still exercising the real #417 gravity selector/integration machinery.
    let source = Source {
        id: CatalogId::from_source("fixture", "large").unwrap(),
        position: pos([0.0; 3]),
        mu_m3_per_s2: 1e19,
        radius_meters: 1e9,
    };
    let drag = Aerodynamics::new(1.0, 10000.0).unwrap();
    let run = |step: i64| {
        let mut t = Trajectory::new(
            State {
                position: pos([1e9 + 10000.0, 0.0, 0.0]),
                velocity_mps: [0.0; 3],
                epoch: epoch(),
            },
            Selector::new(Config::default()).unwrap(),
            1_000_000_000,
        )
        .unwrap();
        for _ in 0..(10_000_000_000 / step) {
            t.step_with_drag(
                step,
                coast(),
                drag,
                |_, _| Ok(vec![source.clone()]),
                |_, _| Ok(Some(Air::new(1.2, [0.0; 3])?)),
            )
            .unwrap();
        }
        t.state()
    };
    let a = run(100_000_000);
    let b = run(50_000_000);
    let reference = run(1_000_000);
    let ea = length(sub(a.velocity_mps, reference.velocity_mps));
    let eb = length(sub(b.velocity_mps, reference.velocity_mps));
    assert!(ea > 3.8 * eb && ea < 4.2 * eb, "coarse {ea} fine {eb}");
    let g = 1e19 / (1e9 + 10000.0_f64).powi(2);
    let terminal = (2.0 * Propulsion::default().total_mass_kg() * g / (1.2 * 10000.0)).sqrt();
    assert!((length(reference.velocity_mps) - terminal).abs() / terminal < 1e-4);
}
#[test]
fn atmospheric_orbit_loses_specific_energy_and_offline_matches_online() {
    let e = earth();
    let radius = 6_371_000.0 + 120_000.0;
    let speed = (e.mu_m3_per_s2 / radius).sqrt();
    let profile = Atmosphere::new(e.radius_meters, 1.225, 8500.0, 200_000.0, [0.0; 3]).unwrap();
    let mut online = flight([radius, 0.0, 0.0], [0.0, speed, 0.0]);
    let mut offline = online.clone();
    let mut vacuum = online.clone();
    let energy = |s: State| {
        0.5 * length(s.velocity_mps).powi(2)
            - e.mu_m3_per_s2 / length(s.position.relative_meters(e.position).unwrap())
    };
    let initial = energy(online.state());
    for _ in 0..1000 {
        let air = |at, p| {
            Ok(Some(profile.sample(
                at,
                p,
                BodyMotion {
                    epoch: at,
                    center: e.position,
                    velocity_mps: [0.0; 3],
                    acceleration_mps2: [0.0; 3],
                },
                [0.0; 3],
            )?))
        };
        online
            .step_with_drag(
                100_000_000,
                coast(),
                aero(),
                |_, _| Ok(vec![e.clone()]),
                air,
            )
            .unwrap();
        offline
            .step_with_drag(
                100_000_000,
                coast(),
                aero(),
                |_, _| Ok(vec![e.clone()]),
                air,
            )
            .unwrap();
        vacuum
            .step(100_000_000, coast(), |_, _| Ok(vec![e.clone()]))
            .unwrap();
    }
    assert_eq!(online.state(), offline.state());
    assert!(energy(online.state()) < initial - 50.0);
    assert!((energy(vacuum.state()) - initial).abs() < 1.0);
    assert!(
        length(online.state().position.relative_meters(e.position).unwrap())
            < length(vacuum.state().position.relative_meters(e.position).unwrap())
    );
}
#[test]
fn provider_or_numerical_failure_preserves_trajectory_and_selector() {
    let mut t = flight([7e6, 0.0, 0.0], [0.0, 7500.0, 0.0]);
    let initial = t.state();
    assert!(
        t.step_with_drag(
            1_000_000_000,
            coast(),
            aero(),
            |_, _| Ok(vec![earth()]),
            |at, _| if at == epoch() {
                Ok(Some(Air::new(1.0, [0.0; 3])?))
            } else {
                Err(error("missing ephemeris"))
            }
        )
        .is_err()
    );
    assert_eq!(t.state(), initial);
    let mut reference = t.clone();
    t.step(1_000_000_000, coast(), |_, _| Ok(vec![earth()]))
        .unwrap();
    reference
        .step(1_000_000_000, coast(), |_, _| Ok(vec![earth()]))
        .unwrap();
    assert_eq!(t.state(), reference.state());
}
#[test]
fn atmospheric_assists_fund_drag_and_reject_unaffordable_wind_without_debit() {
    use crate::{
        DoorState, EnergyCore,
        surface::{Controls, Maneuver, ManeuverConfig, ManeuverKind, Site},
    };
    let e = earth();
    let site = Site::new(
        e.id.clone(),
        epoch(),
        [e.radius_meters + 16.0, 0.0, 0.0],
        [0.0; 3],
    )
    .unwrap();
    let body = |_: &CatalogId, at| {
        Ok(BodyMotion {
            epoch: at,
            center: e.position,
            velocity_mps: [0.0; 3],
            acceleration_mps2: [0.0; 3],
        })
    };
    let start = site
        .at(epoch(), body(&e.id, epoch()).unwrap())
        .unwrap()
        .state;
    let g = |_: Epoch, p: GalacticPosition| {
        let d = e.position.relative_meters(p)?;
        Ok(scale(d, e.mu_m3_per_s2 / length(d).powi(3)))
    };
    let controls = Controls {
        cockpit: true,
        engines_on: true,
        door: DoorState::Closed,
        assist_active: false,
        construction_ready: true,
    };
    let mut core = EnergyCore::default();
    let vacuum = Maneuver::activate(
        ManeuverKind::Takeoff,
        start,
        &site,
        Propulsion::default(),
        controls,
        ManeuverConfig::default(),
        &mut core,
        body,
        g,
    )
    .unwrap();
    let dense = Aerodynamics::new(1.0, 10000.0).unwrap();
    let mut core = EnergyCore::default();
    let before = core.stored_joules();
    let mut atmosphere = Maneuver::activate_with_drag(
        ManeuverKind::Takeoff,
        start,
        &site,
        Propulsion::default(),
        controls,
        ManeuverConfig::default(),
        &mut core,
        dense,
        body,
        g,
        |_, _| Ok(Some(Air::new(1.225, [0.0; 3])?)),
    )
    .unwrap();
    assert!(atmosphere.charged_joules() > vacuum.charged_joules() + 1e6);
    assert!((before - core.stored_joules() - atmosphere.charged_joules()).abs() < 1e-5);
    let (end, _) = atmosphere.advance(6_000_000_000).unwrap();
    assert!(length(end.velocity_mps) < 1e-8);
    let mut core = EnergyCore::default();
    let before = core.stored_joules();
    assert!(
        Maneuver::activate_with_drag(
            ManeuverKind::Takeoff,
            start,
            &site,
            Propulsion::default(),
            controls,
            ManeuverConfig::default(),
            &mut core,
            dense,
            body,
            g,
            |_, _| Ok(Some(Air::new(1.225, [1e6, 0.0, 0.0])?))
        )
        .is_err()
    );
    assert_eq!(before, core.stored_joules());
}
