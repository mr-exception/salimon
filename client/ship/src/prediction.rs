//! Bounded, observational forecasts. Coasting gravity forecasts use the same
//! split-coordinate integrator as authoritative flight. They neither spend
//! energy nor promise contact, docking or arrival between sample endpoints.
use crate::trajectory::{State, Thrust, Trajectory};
use salimon_math::{dot, length, sub};
use salimon_universe::{Epoch, Error, coordinates::GalacticPosition, gravity::Source};

#[derive(Clone, Copy, Debug)]
pub struct Request {
    /// Even nanoseconds, <=60 s; at most 128 steps (also <=1 hour total).
    pub step_nanos: i64,
    pub steps: u16,
    pub target_radius_meters: f64,
    pub heading: [f64; 3],
}
#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub position: GalacticPosition,
    pub velocity_mps: [f64; 3],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Forecast {
    pub endpoint: State,
    pub horizon_seconds: f64,
    /// Initial range to the target sphere (not a path length).
    pub range_meters: f64,
    /// Initial ship velocity minus target inertial velocity.
    pub relative_velocity_mps: [f64; 3],
    pub radial_velocity_mps: f64,
    pub heading_error_degrees: Option<f64>,
    /// Average acceleration over the first requested interval.
    pub first_interval_acceleration_mps2: [f64; 3],
    pub closest_sample_range_meters: f64,
    pub closest_sample_seconds: f64,
    /// First sampled entry into the target sphere. No continuous-contact claim.
    pub sampled_entry_window_seconds: Option<[f64; 2]>,
    /// Maximum coarse/half-step separation. A convergence indicator, NOT a
    /// certified error bound; ignores source/model uncertainty and contact.
    pub convergence_meters: f64,
}
fn invalid() -> Error {
    Error::Invalid("invalid bounded navigation forecast".into())
}
fn finite(v: [f64; 3]) -> bool {
    v.into_iter().all(f64::is_finite)
}
fn observation(s: State, t: Target, r: f64) -> Result<(f64, [f64; 3], f64, [f64; 3]), Error> {
    if !finite(t.velocity_mps) {
        return Err(invalid());
    }
    let delta = t.position.relative_meters(s.position)?;
    let distance = length(delta);
    let relative = sub(s.velocity_mps, t.velocity_mps);
    if !distance.is_finite() || !finite(relative) {
        return Err(invalid());
    }
    let radial = if distance > 0. {
        -dot(relative, delta.map(|v| v / distance))
    } else {
        0.
    };
    if !radial.is_finite() {
        return Err(invalid());
    }
    Ok(((distance - r).max(0.), relative, radial, delta))
}
/// Pure epoch-aware source and target providers are required: both resolutions
/// query the same model, including at half-step epochs. Thrust, drag, rotating
/// terrain/contact and control changes invalidate this *coasting gravity* model.
pub fn coasting(
    source: &Trajectory,
    request: Request,
    mut sources: impl FnMut(Epoch, GalacticPosition) -> Result<Vec<Source>, Error>,
    mut target: impl FnMut(Epoch) -> Result<Target, Error>,
) -> Result<Forecast, Error> {
    let r = request;
    if !(2..=60_000_000_000).contains(&r.step_nanos)
        || r.step_nanos % 2 != 0
        || !(1..=128).contains(&r.steps)
        || r.step_nanos * i64::from(r.steps) > 3_600_000_000_000
        || !r.target_radius_meters.is_finite()
        || r.target_radius_meters < 0.
        || !finite(r.heading)
        || (length(r.heading) - 1.).abs() > 1e-12
    {
        return Err(invalid());
    }
    let start = source.state();
    let (range, relative, radial, delta) =
        observation(start, target(start.epoch)?, r.target_radius_meters)?;
    let heading = (length(delta) > 0.).then(|| {
        (dot(r.heading, delta) / length(delta))
            .clamp(-1., 1.)
            .acos()
            .to_degrees()
    });
    let mut coarse = source.clone();
    let mut fine = source.clone();
    let dt = r.step_nanos as f64 / 1e9;
    let mut out = Forecast {
        endpoint: start,
        horizon_seconds: dt * f64::from(r.steps),
        range_meters: range,
        relative_velocity_mps: relative,
        radial_velocity_mps: radial,
        heading_error_degrees: heading,
        first_interval_acceleration_mps2: [0.; 3],
        closest_sample_range_meters: range,
        closest_sample_seconds: 0.,
        sampled_entry_window_seconds: (range == 0.).then_some([0., 0.]),
        convergence_meters: 0.,
    };
    for i in 1..=r.steps {
        let c = coarse.step(
            r.step_nanos,
            Thrust::coasting(crate::Propulsion::default()),
            &mut sources,
        )?;
        for _ in 0..2 {
            fine.step(
                r.step_nanos / 2,
                Thrust::coasting(crate::Propulsion::default()),
                &mut sources,
            )?;
        }
        let state = fine.state();
        if i == 1 {
            out.first_interval_acceleration_mps2 =
                sub(state.velocity_mps, start.velocity_mps).map(|v| v / dt);
        }
        out.convergence_meters = out
            .convergence_meters
            .max(length(c.position.relative_meters(state.position)?));
        let (range, _, _, _) = observation(state, target(state.epoch)?, r.target_radius_meters)?;
        let t = f64::from(i) * dt;
        if range < out.closest_sample_range_meters {
            out.closest_sample_range_meters = range;
            out.closest_sample_seconds = t;
        }
        if range == 0. && out.sampled_entry_window_seconds.is_none() {
            out.sampled_entry_window_seconds = Some([t - dt, t]);
        }
        if !finite(out.first_interval_acceleration_mps2) || !out.convergence_meters.is_finite() {
            return Err(invalid());
        }
        out.endpoint = state;
    }
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    use salimon_universe::{
        CatalogId, TimeScale,
        gravity::{Config, Selector},
    };
    fn epoch() -> Epoch {
        Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], TimeScale::Tdb).unwrap()
    }
    fn origin() -> GalacticPosition {
        GalacticPosition::new([1_000_000_000; 3], [0.; 3]).unwrap()
    }
    fn trajectory(p: [f64; 3], v: [f64; 3]) -> Trajectory {
        Trajectory::new(
            State {
                position: origin().translated(p).unwrap(),
                velocity_mps: v,
                epoch: epoch(),
            },
            Selector::new(Config::default()).unwrap(),
            2_000_000_000,
        )
        .unwrap()
    }
    fn request() -> Request {
        Request {
            step_nanos: 1_000_000_000,
            steps: 100,
            target_radius_meters: 0.,
            heading: [1., 0., 0.],
        }
    }
    #[test]
    fn navigation_coasting_matches_analytic_moving_target_across_cells_without_mutation() {
        let s = trajectory([499_999_990., 0., 0.], [100., 0., 0.]);
        let before = s.state();
        let f = coasting(
            &s,
            request(),
            |_, _| Ok(vec![]),
            |e| {
                Ok(Target {
                    position: origin().translated([
                        500_000_990. + 10. * e.seconds_since(epoch())?,
                        0.,
                        0.,
                    ])?,
                    velocity_mps: [10., 0., 0.],
                })
            },
        )
        .unwrap();
        assert_eq!(s.state(), before);
        assert!((f.range_meters - 1000.).abs() < 1e-8);
        assert_eq!(f.relative_velocity_mps, [90., 0., 0.]);
        assert_eq!(f.radial_velocity_mps, -90.);
        assert_eq!(f.heading_error_degrees, Some(0.));
        assert_eq!(f.convergence_meters, 0.);
        assert!(
            (length(
                f.endpoint
                    .position
                    .relative_meters(before.position)
                    .unwrap()
            ) - 10000.)
                .abs()
                < 1e-7
        );
        assert!(f.sampled_entry_window_seconds.is_none()); // passes through BETWEEN samples; no fake hit
    }
    #[test]
    fn navigation_gravity_forecast_equals_authoritative_half_steps_and_refines() {
        let s = trajectory([7_000_000., 0., 0.], [0., 7500., 0.]);
        let earth = Source {
            id: CatalogId::from_source("fixture", "earth").unwrap(),
            position: origin(),
            mu_m3_per_s2: 3.986004418e14,
            radius_meters: 6_371_000.,
        };
        let target = |_| {
            Ok(Target {
                position: origin(),
                velocity_mps: [0.; 3],
            })
        };
        let f = coasting(&s, request(), |_, _| Ok(vec![earth.clone()]), target).unwrap();
        let mut authoritative = s.clone();
        for _ in 0..200 {
            authoritative
                .step(
                    500_000_000,
                    Thrust::coasting(crate::Propulsion::default()),
                    |_, _| Ok(vec![earth.clone()]),
                )
                .unwrap();
        }
        assert_eq!(f.endpoint, authoritative.state());
        assert!(f.convergence_meters > 0.);
        assert!(f.first_interval_acceleration_mps2[0] < -8.);
        let mut r = request();
        r.step_nanos /= 2; // same horizon below
        r.steps = 128; // 64 seconds; compare equal 64-second requests
        let fine = coasting(&s, r, |_, _| Ok(vec![earth.clone()]), target).unwrap();
        r.step_nanos *= 2;
        r.steps = 64;
        let coarse = coasting(&s, r, |_, _| Ok(vec![earth.clone()]), target).unwrap();
        assert!(fine.convergence_meters < coarse.convergence_meters);
    }
    #[test]
    fn navigation_rejects_invalid_and_provider_failures_atomically_and_reports_sampled_window() {
        let s = trajectory([100., 0., 0.], [-10., 0., 0.]);
        let before = s.state();
        let mut r = request();
        r.target_radius_meters = 10.;
        let target = |_| {
            Ok(Target {
                position: origin(),
                velocity_mps: [0.; 3],
            })
        };
        let f = coasting(&s, r, |_, _| Ok(vec![]), target).unwrap();
        assert_eq!(f.sampled_entry_window_seconds, Some([8., 9.]));
        r.steps = 129;
        assert!(coasting(&s, r, |_, _| Ok(vec![]), target).is_err());
        r.steps = 10;
        r.heading = [f64::NAN, 0., 0.];
        assert!(coasting(&s, r, |_, _| Ok(vec![]), target).is_err());
        assert!(
            coasting(
                &s,
                request(),
                |_, _| Err(Error::Invalid("missing ephemeris".into())),
                target
            )
            .is_err()
        );
        assert!(
            coasting(
                &s,
                request(),
                |_, _| Ok(vec![]),
                |_| Ok(Target {
                    position: origin(),
                    velocity_mps: [f64::NAN; 3]
                })
            )
            .is_err()
        );
        assert_eq!(s.state(), before);
    }
}
