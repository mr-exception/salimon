use super::*;
use crate::contact::{Collider, CubicPath};
use salimon_universe::{
    TimeScale,
    gravity::{Config as GravityConfig, Selector},
};
fn initial(position: [f64; 3], velocity_mps: [f64; 3]) -> State {
    State {
        position: GalacticPosition::new([0; 3], position).unwrap(),
        velocity_mps,
        epoch: Epoch::calendar([2026, 1, 1], [0; 4], TimeScale::Tt).unwrap(),
    }
}
fn job(state: State, seconds: u64) -> Propagation {
    Propagation::new(
        state,
        Mode::Coasting,
        Selector::new(GravityConfig::default()).unwrap(),
        Duration::from_secs(seconds),
        Config {
            step_nanos: 1_000_000_000,
            ship_radius_meters: 1.,
            propulsion: Propulsion::default(),
            aerodynamics: Aerodynamics::new(1., 100.).unwrap(),
            response: ResponsePolicy::new(0., 0., 0., None).unwrap(),
        },
    )
    .unwrap()
}
struct Space {
    solid: bool,
    fail: bool,
    gravity: bool,
}
impl Environment for Space {
    fn sources(&self, _: Epoch, _: GalacticPosition) -> Result<Vec<Source>, Error> {
        if self.fail {
            return Err(error("ephemeris outside validity"));
        }
        Ok(if self.gravity {
            vec![Source {
                id: CatalogId::from_source("offline-test", "earth").unwrap(),
                position: initial([0.; 3], [0.; 3]).position,
                mu_m3_per_s2: 3.986004418e14,
                radius_meters: 6_371_000.,
            }]
        } else {
            vec![]
        })
    }
    fn air(&self, _: Epoch, _: GalacticPosition) -> Result<Option<Air>, Error> {
        Ok(None)
    }
    fn scene(&self, start: State, nanos: i64) -> Result<Scene, Error> {
        let colliders = if self.solid {
            let a = State {
                position: initial([100., 0., 0.], [0.; 3]).position,
                velocity_mps: [0.; 3],
                epoch: start.epoch,
            };
            let b = State {
                epoch: start.epoch.advance_nanos(nanos)?,
                ..a
            };
            vec![Collider::sphere(
                CatalogId::from_source("offline-test", "rock")?,
                CubicPath::between(a, b)?,
                10.,
                [0.; 3],
            )?]
        } else {
            vec![]
        };
        Scene::new(colliders)
    }
    fn supported(&self, state: State, _: &CatalogId, nanos: i64) -> Result<State, Error> {
        Ok(State {
            epoch: state.epoch.advance_nanos(nanos)?,
            ..state
        })
    }
}
#[test]
fn bounded_polls_never_publish_an_approximate_or_truncated_resume() {
    let mut a = job(initial([0.; 3], [10., 0., 0.]), 100);
    let mut b = a.clone();
    let env = Space {
        solid: false,
        fail: false,
        gravity: false,
    };
    a.poll(&env, 10, &mut Vec::new()).unwrap();
    assert_eq!(a.progress().completed, Duration::from_secs(10));
    assert!(a.verified().is_err());
    while !a.progress().complete() {
        a.poll(&env, 7, &mut Vec::new()).unwrap();
    }
    b.poll(&env, 100, &mut Vec::new()).unwrap();
    assert_eq!(a.verified().unwrap(), b.verified().unwrap());
    assert_eq!(
        a.verified()
            .unwrap()
            .0
            .position
            .relative_meters(initial([0.; 3], [0.; 3]).position)
            .unwrap(),
        [1000., 0., 0.]
    );
    assert_eq!(a.progress().fraction(), 1.);
}
#[test]
fn invalid_provider_and_late_consumer_failure_leave_candidate_unpublished() {
    let mut a = job(initial([0.; 3], [10., 0., 0.]), 100);
    assert!(
        a.poll(
            &Space {
                solid: false,
                fail: true,
                gravity: false
            },
            10,
            &mut Vec::new()
        )
        .is_err()
    );
    assert_eq!(a.progress().completed, Duration::ZERO);
    let mut consumer = Failing { calls: 0 };
    assert!(
        a.poll(
            &Space {
                solid: false,
                fail: false,
                gravity: false
            },
            10,
            &mut consumer
        )
        .is_err()
    );
    assert_eq!(a.progress().completed, Duration::ZERO);
    assert_eq!(consumer.calls, 0);
    assert!(
        a.poll(
            &Space {
                solid: false,
                fail: false,
                gravity: false
            },
            0,
            &mut Vec::new()
        )
        .is_err()
    );
}
#[test]
fn earliest_impact_changes_mode_and_remaining_time_follows_verified_support() {
    let mut a = job(initial([0.; 3], [1000., 0., 0.]), 10);
    let mut boundaries = Vec::new();
    a.poll(
        &Space {
            solid: true,
            fail: false,
            gravity: false,
        },
        100,
        &mut boundaries,
    )
    .unwrap();
    let (state, mode) = a.verified().unwrap();
    assert!(matches!(mode, Mode::Grounded(_)));
    assert_eq!(boundaries.iter().filter(|b| b.hit.is_some()).count(), 1);
    assert!(
        (state
            .position
            .relative_meters(initial([0.; 3], [0.; 3]).position)
            .unwrap()[0]
            - 89.)
            .abs()
            < 0.002
    );
    assert_eq!(state.velocity_mps, [0.; 3]);
    assert_eq!(
        state.epoch,
        initial([0.; 3], [0.; 3])
            .epoch
            .advance_nanos(10_000_000_000)
            .unwrap()
    );
    assert!(a.progress().maximum_contact_uncertainty_meters > 0.);
}
#[test]
fn long_orbit_matches_production_reference_and_preserves_numerical_budget() {
    let radius = 7_000_000.;
    let speed = (3.986004418e14_f64 / radius).sqrt();
    let state = initial([radius, 0., 0.], [0., speed, 0.]);
    let period = std::f64::consts::TAU * radius / speed;
    let seconds = period.round() as u64;
    let mut a = job(state, seconds);
    let env = Space {
        solid: false,
        fail: false,
        gravity: true,
    };
    while !a.progress().complete() {
        a.poll(&env, 256, &mut Vec::new()).unwrap();
    }
    let (actual, _) = a.verified().unwrap();
    let angle = speed / radius * seconds as f64;
    let expected = state
        .position
        .translated([radius * angle.cos() - radius, radius * angle.sin(), 0.])
        .unwrap();
    let error = salimon_math::length(actual.position.relative_meters(expected).unwrap());
    assert!(error < 30., "orbital position error {error} m");
    let mut reference = Trajectory::new(
        state,
        Selector::new(GravityConfig::default()).unwrap(),
        500_000_000,
    )
    .unwrap();
    for _ in 0..seconds * 2 {
        reference
            .step(
                500_000_000,
                Thrust::coasting(Propulsion::default()),
                |at, p| env.sources(at, p),
            )
            .unwrap();
    }
    assert!(
        salimon_math::length(
            actual
                .position
                .relative_meters(reference.state().position)
                .unwrap()
        ) < 30.
    );
}

#[derive(Clone)]
struct Failing {
    calls: usize,
}
impl Consumer for Failing {
    fn apply(&mut self, _: Boundary) -> Result<(), Error> {
        self.calls += 1;
        if self.calls == 4 {
            Err(error("timed consumer failed"))
        } else {
            Ok(())
        }
    }
}

struct MovingBody;
impl MovingBody {
    fn center(at: Epoch) -> State {
        let mut state = initial([100., 0., 0.], [50., 0., 0.]);
        state.position = state
            .position
            .translated([50. * at.seconds_since(state.epoch).unwrap(), 0., 0.])
            .unwrap();
        state.epoch = at;
        state
    }
}
impl Environment for MovingBody {
    fn sources(&self, _: Epoch, _: GalacticPosition) -> Result<Vec<Source>, Error> {
        Ok(vec![])
    }
    fn air(&self, _: Epoch, _: GalacticPosition) -> Result<Option<Air>, Error> {
        Ok(None)
    }
    fn scene(&self, start: State, nanos: i64) -> Result<Scene, Error> {
        Scene::new(vec![Collider::sphere(
            CatalogId::from_source("offline-test", "moving")?,
            CubicPath::between(
                Self::center(start.epoch),
                Self::center(start.epoch.advance_nanos(nanos)?),
            )?,
            10.,
            [0.; 3],
        )?])
    }
    fn supported(&self, state: State, _: &CatalogId, nanos: i64) -> Result<State, Error> {
        Ok(State {
            position: state
                .position
                .translated([50. * nanos as f64 * 1e-9, 0., 0.])?,
            epoch: state.epoch.advance_nanos(nanos)?,
            ..state
        })
    }
}
#[test]
fn impact_and_supported_continuation_retain_actual_moving_body_velocity() {
    let mut a = job(initial([0.; 3], [1000., 0., 0.]), 10);
    let mut boundaries = Vec::new();
    a.poll(&MovingBody, 100, &mut boundaries).unwrap();
    let first = boundaries.iter().find(|b| b.hit.is_some()).unwrap();
    let actual_seconds = first.after.as_secs_f64();
    assert!((actual_seconds - 89. / 950.).abs() < 0.002 / 950.);
    // Cubic Hermite differentiation retains ordinary f64 rounding at 50 m/s.
    assert!(
        salimon_math::length(salimon_math::sub(first.state.velocity_mps, [50., 0., 0.])) < 1e-12
    );
    let (final_state, mode) = a.verified().unwrap();
    assert!(matches!(mode, Mode::Grounded(_)));
    assert!(
        (final_state
            .position
            .relative_meters(initial([0.; 3], [0.; 3]).position)
            .unwrap()[0]
            - 589.)
            .abs()
            < 0.002
    );
    assert!(
        salimon_math::length(salimon_math::sub(final_state.velocity_mps, [50., 0., 0.])) < 1e-12
    );
}
#[test]
fn final_nanoseconds_cannot_round_progress_to_completion() {
    let p = Progress {
        total: Duration::from_secs(1_000_000_000),
        completed: Duration::from_secs(1_000_000_000) - Duration::from_nanos(1),
        steps: 0,
        impacts: 0,
        maximum_contact_uncertainty_meters: 0.,
    };
    assert!(!p.complete());
    assert!(p.fraction() < 1.);
}
