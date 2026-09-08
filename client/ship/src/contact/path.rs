//! Dense physical drift and bounded ephemeris paths.
use crate::{
    Propulsion,
    atmosphere::{Aerodynamics, Air},
    surface::{error, finite},
    trajectory::State,
};
use salimon_math::{add, length, scale, sub};
use salimon_universe::{Epoch, Error, TimeScale, coordinates::GalacticPosition};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Bounds {
    pub(super) min: [f64; 3],
    pub(super) max: [f64; 3],
}
impl Bounds {
    pub(super) fn distance(self, other: Self) -> f64 {
        length(std::array::from_fn(|i| {
            (self.min[i] - other.max[i])
                .max(other.min[i] - self.max[i])
                .max(0.0)
        }))
    }
    pub(super) fn overlaps(self, other: Self, padding: f64) -> bool {
        (0..3)
            .all(|i| self.min[i] - padding <= other.max[i] && other.min[i] - padding <= self.max[i])
    }
}
pub(crate) trait MotionPath {
    fn epoch(&self) -> Epoch;
    fn duration_nanos(&self) -> i64;
    fn position(&self, nanos: i64) -> Result<GalacticPosition, Error>;
    fn speed_bound_relative_to(&self, velocity: [f64; 3]) -> Result<f64, Error>;
    fn bounds(&self, origin: GalacticPosition) -> Result<Bounds, Error>;
    fn acceleration_bound(&self) -> Result<f64, Error>;
    fn interval_bounds(
        &self,
        origin: GalacticPosition,
        start: i64,
        end: i64,
        frame: [f64; 3],
    ) -> Result<Bounds, Error> {
        let a = sub(
            self.position(start)?.relative_meters(origin)?,
            scale(frame, start as f64 * 1e-9),
        );
        let b = sub(
            self.position(end)?.relative_meters(origin)?,
            scale(frame, end as f64 * 1e-9),
        );
        let deviation = self.acceleration_bound()? * ((end - start) as f64 * 1e-9).powi(2) / 8.0;
        finite(a)?;
        finite(b)?;
        if !deviation.is_finite() {
            return Err(error("contact interval bounds overflow"));
        }
        Ok(Bounds {
            min: std::array::from_fn(|i| a[i].min(b[i]) - deviation),
            max: std::array::from_fn(|i| a[i].max(b[i]) + deviation),
        })
    }
}
/// Cubic Hermite ephemeris/dense-motion segment. Its Bezier convex hull and
/// derivative hull certify spatial/speed bounds. Input endpoints have explicit
/// TT/TDB epochs. The caller owns ephemeris-fit/terrain motion error budgets.
#[derive(Clone, Debug)]
pub struct CubicPath {
    start: Epoch,
    nanos: i64,
    origin: GalacticPosition,
    points: [[f64; 3]; 4],
}
impl CubicPath {
    pub fn between(start: State, end: State) -> Result<Self, Error> {
        finite(start.velocity_mps)?;
        finite(end.velocity_mps)?;
        let seconds = end.epoch.seconds_since(start.epoch)?;
        if start.epoch.scale() == TimeScale::Utc || seconds <= 0.0 || seconds > 60.0 {
            return Err(error("contact path requires positive TT/TDB span <=60 s"));
        }
        let nanos = (seconds * 1e9).round() as i64;
        if start.epoch.advance_nanos(nanos)? != end.epoch {
            return Err(error("contact path nanosecond grid"));
        }
        let p = end.position.relative_meters(start.position)?;
        let points = [
            [0.0; 3],
            scale(start.velocity_mps, seconds / 3.0),
            sub(p, scale(end.velocity_mps, seconds / 3.0)),
            p,
        ];
        for p in points {
            finite(p)?;
        }
        Ok(Self {
            start: start.epoch,
            nanos,
            origin: start.position,
            points,
        })
    }
    pub fn state_at(&self, nanos: i64) -> Result<State, Error> {
        let u = self.fraction(nanos)?;
        let p = bezier(self.points, u);
        let seconds = self.nanos as f64 * 1e-9;
        let d: [[f64; 3]; 3] =
            std::array::from_fn(|i| scale(sub(self.points[i + 1], self.points[i]), 3.0 / seconds));
        let velocity = add(
            add(
                scale(d[0], (1.0 - u).powi(2)),
                scale(d[1], 2.0 * u * (1.0 - u)),
            ),
            scale(d[2], u * u),
        );
        finite(velocity)?;
        Ok(State {
            position: self.origin.translated(p)?,
            velocity_mps: velocity,
            epoch: self.start.advance_nanos(nanos)?,
        })
    }
    fn fraction(&self, nanos: i64) -> Result<f64, Error> {
        if nanos < 0 || nanos > self.nanos {
            Err(error("contact path outside interval"))
        } else {
            Ok(nanos as f64 / self.nanos as f64)
        }
    }
}
fn bezier(p: [[f64; 3]; 4], u: f64) -> [f64; 3] {
    let q = std::array::from_fn::<_, 3, _>(|i| add(scale(p[i], 1.0 - u), scale(p[i + 1], u)));
    let r = std::array::from_fn::<_, 2, _>(|i| add(scale(q[i], 1.0 - u), scale(q[i + 1], u)));
    add(scale(r[0], 1.0 - u), scale(r[1], u))
}
impl MotionPath for CubicPath {
    fn acceleration_bound(&self) -> Result<f64, Error> {
        let seconds = self.nanos as f64 * 1e-9;
        let mut bound = 0.0_f64;
        for i in 0..2 {
            let a = scale(
                add(
                    sub(self.points[i + 2], scale(self.points[i + 1], 2.0)),
                    self.points[i],
                ),
                6.0 / seconds / seconds,
            );
            finite(a)?;
            bound = bound.max(length(a));
        }
        Ok(bound)
    }

    fn epoch(&self) -> Epoch {
        self.start
    }
    fn duration_nanos(&self) -> i64 {
        self.nanos
    }
    fn position(&self, nanos: i64) -> Result<GalacticPosition, Error> {
        Ok(self.state_at(nanos)?.position)
    }
    fn speed_bound_relative_to(&self, v: [f64; 3]) -> Result<f64, Error> {
        let mut bound = 0.0_f64;
        for i in 0..3 {
            let d = sub(
                scale(
                    sub(self.points[i + 1], self.points[i]),
                    3e9 / self.nanos as f64,
                ),
                v,
            );
            finite(d)?;
            bound = bound.max(length(d));
        }
        Ok(bound)
    }
    fn bounds(&self, origin: GalacticPosition) -> Result<Bounds, Error> {
        let offset = self.origin.relative_meters(origin)?;
        let p = self.points.map(|p| add(p, offset));
        Ok(Bounds {
            min: std::array::from_fn(|i| p.iter().map(|v| v[i]).fold(f64::INFINITY, f64::min)),
            max: std::array::from_fn(|i| p.iter().map(|v| v[i]).fold(f64::NEG_INFINITY, f64::max)),
        })
    }
}
/// Dense position of exactly the #417/#419 Verlet drift, at every possible
/// truncated step duration. It needs no endpoint force query inside a solid.
pub(crate) struct DriftPath {
    pub start: State,
    pub nanos: i64,
    pub gravity: [f64; 3],
    pub thrust_dv: [f64; 3],
    pub propulsion: Propulsion,
    pub aero: Aerodynamics,
    pub air: Option<Air>,
}
impl MotionPath for DriftPath {
    fn acceleration_bound(&self) -> Result<f64, Error> {
        let drag =
            self.aero
                .acceleration_mps2(self.start.velocity_mps, self.propulsion, self.air)?;
        let constant = add(self.gravity, scale(self.thrust_dv, 1e9 / self.nanos as f64));
        let bound = length(drag) + length(constant);
        if !bound.is_finite() {
            return Err(error("contact drift acceleration bound overflow"));
        }
        Ok(bound)
    }

    fn epoch(&self) -> Epoch {
        self.start.epoch
    }
    fn duration_nanos(&self) -> i64 {
        self.nanos
    }
    fn position(&self, nanos: i64) -> Result<GalacticPosition, Error> {
        if nanos < 0 || nanos > self.nanos {
            return Err(error("drift outside interval"));
        }
        let seconds = nanos as f64 * 1e-9;
        let v = self.aero.velocity_after(
            self.start.velocity_mps,
            self.propulsion,
            self.air,
            seconds * 0.5,
        )?;
        let half = add(
            add(v, scale(self.gravity, seconds * 0.5)),
            scale(self.thrust_dv, 0.5 * nanos as f64 / self.nanos as f64),
        );
        self.start.position.translated(scale(half, seconds))
    }
    fn speed_bound_relative_to(&self, frame: [f64; 3]) -> Result<f64, Error> {
        let velocity_bound = match self.air {
            None => length(sub(self.start.velocity_mps, frame)),
            Some(air) => {
                length(sub(air.velocity_mps(), frame))
                    + length(sub(self.start.velocity_mps, air.velocity_mps()))
            }
        };
        let bound = velocity_bound
            + length(add(
                scale(self.gravity, self.nanos as f64 * 1e-9),
                self.thrust_dv,
            ));
        if !bound.is_finite() {
            return Err(error("drift speed bound overflow"));
        }
        Ok(bound)
    }
    fn bounds(&self, origin: GalacticPosition) -> Result<Bounds, Error> {
        let p = self.start.position.relative_meters(origin)?;
        let radius = self.speed_bound_relative_to([0.0; 3])? * self.nanos as f64 * 1e-9;
        if !radius.is_finite() {
            return Err(error("drift bounds overflow"));
        }
        Ok(Bounds {
            min: p.map(|x| x - radius),
            max: p.map(|x| x + radius),
        })
    }
}
