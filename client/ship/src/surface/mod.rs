//! Real-scale contact support and physically funded maneuvers. Ephemerides and
//! gravity are supplied at explicit epochs; the compressed controller is separate.
use crate::{
    DoorState, EnergyCore, Propulsion, energy::AssistEnergyReservation, trajectory::State,
};
use salimon_math::{add, cross, dot, length, scale, sub};
use salimon_universe::{CatalogId, Epoch, Error, TimeScale, coordinates::GalacticPosition};

#[derive(Clone, Copy, Debug)]
pub struct BodyMotion {
    pub epoch: Epoch,
    pub center: GalacticPosition,
    pub velocity_mps: [f64; 3],
    pub acceleration_mps2: [f64; 3],
}
impl BodyMotion {
    pub fn validate(self, epoch: Epoch) -> Result<Self, Error> {
        if self.epoch != epoch || epoch.scale() == TimeScale::Utc {
            return Err(error("body motion epoch/scale"));
        }
        finite(self.velocity_mps)?;
        finite(self.acceleration_mps2)?;
        Ok(self)
    }
}
/// A fixed surface site, using inertial axes at the reference epoch. Rotation
/// is rigid and constant; the provider owns orbital motion and its validity.
#[derive(Clone, Debug)]
pub struct Site {
    body: CatalogId,
    epoch: Epoch,
    offset_meters: [f64; 3],
    angular_velocity_rps: [f64; 3],
}
#[derive(Clone, Copy, Debug)]
pub struct Kinematics {
    pub state: State,
    pub acceleration_mps2: [f64; 3],
    pub outward_normal: [f64; 3],
}
#[derive(Clone, Copy, Debug)]
pub struct Support {
    pub motion: Kinematics,
    pub force_newtons: [f64; 3],
}
impl Site {
    pub fn new(
        body: CatalogId,
        epoch: Epoch,
        offset_meters: [f64; 3],
        angular_velocity_rps: [f64; 3],
    ) -> Result<Self, Error> {
        finite(offset_meters)?;
        finite(angular_velocity_rps)?;
        if epoch.scale() == TimeScale::Utc || length(offset_meters) <= 0.0 {
            return Err(error("surface site radius/epoch"));
        }
        Ok(Self {
            body,
            epoch,
            offset_meters,
            angular_velocity_rps,
        })
    }
    pub fn body(&self) -> &CatalogId {
        &self.body
    }
    pub fn at(&self, epoch: Epoch, body: BodyMotion) -> Result<Kinematics, Error> {
        self.relative_at(epoch, body, [0.0; 3], [0.0; 3], [0.0; 3])
    }
    fn relative_at(
        &self,
        epoch: Epoch,
        body: BodyMotion,
        displacement: [f64; 3],
        velocity: [f64; 3],
        acceleration: [f64; 3],
    ) -> Result<Kinematics, Error> {
        let body = body.validate(epoch)?;
        let dt = epoch.seconds_since(self.epoch)?;
        let rotate = |v| rotate(v, self.angular_velocity_rps, dt);
        let r = rotate(add(self.offset_meters, displacement))?;
        let v = rotate(velocity)?;
        let a = rotate(acceleration)?;
        let spin = self.angular_velocity_rps;
        let state = State {
            position: body.center.translated(r)?,
            velocity_mps: add(body.velocity_mps, add(v, cross(spin, r))),
            epoch,
        };
        let acceleration_mps2 = add(
            body.acceleration_mps2,
            add(
                a,
                add(scale(cross(spin, v), 2.0), cross(spin, cross(spin, r))),
            ),
        );
        finite(state.velocity_mps)?;
        finite(acceleration_mps2)?;
        let n = rotate(self.offset_meters)?;
        Ok(Kinematics {
            state,
            acceleration_mps2,
            outward_normal: scale(n, 1.0 / length(n)),
        })
    }
    /// Normal contact and static friction supply constraint force, spending no
    /// propulsion energy. Tensile support or exceeded friction rejects contact.
    pub fn support(
        &self,
        epoch: Epoch,
        body: BodyMotion,
        gravity_mps2: [f64; 3],
        propulsion: Propulsion,
        friction: f64,
    ) -> Result<Support, Error> {
        finite(gravity_mps2)?;
        if !friction.is_finite() || friction < 0.0 {
            return Err(error("surface friction"));
        }
        let motion = self.at(epoch, body)?;
        let force = scale(
            sub(motion.acceleration_mps2, gravity_mps2),
            propulsion.total_mass_kg(),
        );
        finite(force)?;
        let normal = dot(force, motion.outward_normal);
        let tangent = length(sub(force, scale(motion.outward_normal, normal)));
        if normal < 0.0 || tangent > friction * normal + 1e-6 {
            return Err(error("surface cannot supply required support"));
        }
        Ok(Support {
            motion,
            force_newtons: force,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManeuverKind {
    Takeoff,
    Landing,
}
/// Trusted command context from the runtime owner, as with existing placement
/// admission. Pending assists reject a second activation; exiting the cockpit
/// after activation cannot cancel a reserved maneuver.
#[derive(Clone, Copy, Debug)]
pub struct Controls {
    pub cockpit: bool,
    pub engines_on: bool,
    pub door: DoorState,
    pub assist_active: bool,
    pub construction_ready: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct ManeuverConfig {
    pub duration_nanos: i64,
    pub lift_meters: f64,
    pub maximum_landing_range_meters: f64,
    pub maximum_landing_speed_mps: f64,
    pub static_friction: f64,
}
impl Default for ManeuverConfig {
    fn default() -> Self {
        Self {
            duration_nanos: 6_000_000_000,
            lift_meters: 50.0,
            maximum_landing_range_meters: 1000.0,
            maximum_landing_speed_mps: 20.0,
            static_friction: 1.0,
        }
    }
}
const SAMPLE_NANOS: i64 = 10_000_000;
#[derive(Debug)]
struct Knot {
    motion: Kinematics,
    spent_joules: f64,
}
/// Owns a non-clonable isolated Core reservation. Full ephemeris/gravity,
/// clearance and power admission happens before reserving anything. No setters
/// or cancellation can grant partial motion or refund already spent impulse.
#[derive(Debug)]
pub struct Maneuver {
    kind: ManeuverKind,
    knots: Vec<Knot>,
    elapsed_nanos: i64,
    duration_nanos: i64,
    reservation: AssistEnergyReservation,
    charged_joules: f64,
}
impl Maneuver {
    #[allow(clippy::too_many_arguments)]
    pub fn activate(
        kind: ManeuverKind,
        start: State,
        site: &Site,
        propulsion: Propulsion,
        controls: Controls,
        config: ManeuverConfig,
        core: &mut EnergyCore,
        mut body_at: impl FnMut(&CatalogId, Epoch) -> Result<BodyMotion, Error>,
        mut gravity_at: impl FnMut(Epoch, GalacticPosition) -> Result<[f64; 3], Error>,
    ) -> Result<Self, Error> {
        finite(start.velocity_mps)?;
        if !controls.cockpit
            || !controls.engines_on
            || controls.assist_active
            || (kind == ManeuverKind::Takeoff
                && (controls.door != DoorState::Closed || !controls.construction_ready))
        {
            return Err(error(
                "maneuver cockpit/engine/door/construction/active interlock",
            ));
        }
        if config.duration_nanos <= 0
            || config.duration_nanos > 120_000_000_000
            || config.duration_nanos % SAMPLE_NANOS != 0
            || [
                config.lift_meters,
                config.maximum_landing_range_meters,
                config.maximum_landing_speed_mps,
                config.static_friction,
            ]
            .into_iter()
            .any(|v| !v.is_finite() || v < 0.0)
            || (kind == ManeuverKind::Takeoff && config.lift_meters == 0.0)
        {
            return Err(error("maneuver duration/range/lift/speed"));
        }
        // Shift the site to the activation epoch so local polynomial derivatives
        // and rigid rotation share one frame, even for a centuries-old site.
        let mut site = site.clone();
        site.offset_meters = rotate(
            site.offset_meters,
            site.angular_velocity_rps,
            start.epoch.seconds_since(site.epoch)?,
        )?;
        site.epoch = start.epoch;
        let base = site.at(start.epoch, body_at(&site.body, start.epoch)?)?;
        let x0 = start.position.relative_meters(base.state.position)?;
        let v0 = sub(
            sub(start.velocity_mps, base.state.velocity_mps),
            cross(site.angular_velocity_rps, x0),
        );
        if kind == ManeuverKind::Takeoff && (length(x0) > 0.001 || length(v0) > 1e-6) {
            return Err(error("takeoff must start supported with surface velocity"));
        }
        if kind == ManeuverKind::Landing
            && (length(x0) > config.maximum_landing_range_meters
                || length(v0) > config.maximum_landing_speed_mps)
        {
            return Err(error("landing range/relative speed"));
        }
        let x1 = if kind == ManeuverKind::Takeoff {
            scale(base.outward_normal, config.lift_meters)
        } else {
            [0.0; 3]
        };
        let support_epoch = if kind == ManeuverKind::Takeoff {
            start.epoch
        } else {
            start.epoch.advance_nanos(config.duration_nanos)?
        };
        let support_body = body_at(&site.body, support_epoch)?;
        let support_position = site.at(support_epoch, support_body)?.state.position;
        site.support(
            support_epoch,
            support_body,
            gravity_at(support_epoch, support_position)?,
            propulsion,
            config.static_friction,
        )?;
        let duration = config.duration_nanos as f64 * 1e-9;
        let max_watts = core.config().forward_watts.min(core.config().reverse_watts);
        let dv_per_joule = propulsion.delta_velocity_meters_per_second(1.0);
        if dv_per_joule <= 0.0 {
            return Err(error("maneuver requires positive impulse conversion"));
        }
        let mut evaluate = |nanos: i64| -> Result<(Kinematics, f64), Error> {
            let t = nanos as f64 * 1e-9;
            let (x, v, a) = hermite(x0, v0, x1, [0.0; 3], duration, t);
            // A radial site supplies local clearance. Terrain adapters must select
            // an actually safe footprint before activation, as in the old assist.
            if dot(x, base.outward_normal) < -1e-7 {
                return Err(error("maneuver intersects support plane"));
            }
            let epoch = start.epoch.advance_nanos(nanos)?;
            let motion = site.relative_at(epoch, body_at(&site.body, epoch)?, x, v, a)?;
            let gravity = gravity_at(epoch, motion.state.position)?;
            finite(gravity)?;
            let watts = length(sub(motion.acceleration_mps2, gravity)) / dv_per_joule;
            if !watts.is_finite() || watts > max_watts {
                return Err(error("insufficient maneuver thrust/power"));
            }
            Ok((motion, watts))
        };
        let (first, mut previous_watts) = evaluate(0)?;
        let mut knots = vec![Knot {
            motion: first,
            spent_joules: 0.0,
        }];
        let mut total = 0.0;
        for nanos in (SAMPLE_NANOS..=config.duration_nanos).step_by(SAMPLE_NANOS as usize) {
            let (_, middle_watts) = evaluate(nanos - SAMPLE_NANOS / 2)?;
            let (motion, watts) = evaluate(nanos)?;
            // Simpson quadrature of |m(a-g)|/kappa, not kinetic energy or a
            // fixed mass-independent fee. Each 10 ms segment has one receipt.
            total +=
                (previous_watts + 4.0 * middle_watts + watts) * (SAMPLE_NANOS as f64 * 1e-9 / 6.0);
            knots.push(Knot {
                motion,
                spent_joules: total,
            });
            previous_watts = watts;
        }
        let reservation = core
            .reserve_assist(total)
            .ok_or_else(|| error("insufficient Core maneuver energy"))?;
        Ok(Self {
            kind,
            knots,
            elapsed_nanos: 0,
            duration_nanos: config.duration_nanos,
            reservation,
            charged_joules: total,
        })
    }
    pub fn kind(&self) -> ManeuverKind {
        self.kind
    }
    pub fn charged_joules(&self) -> f64 {
        self.charged_joules
    }
    pub fn finished(&self) -> bool {
        self.elapsed_nanos == self.duration_nanos
    }
    pub fn remaining_nanos(&self) -> i64 {
        self.duration_nanos - self.elapsed_nanos
    }
    /// Returns unconsumed caller time for the ordinary gravity integrator or
    /// surface support. Exactly partition independent; no hidden time discard.
    pub fn advance(&mut self, elapsed_nanos: i64) -> Result<(State, i64), Error> {
        if elapsed_nanos < 0 {
            return Err(error("negative maneuver time"));
        }
        let used = elapsed_nanos.min(self.remaining_nanos());
        let before = self.spent_at(self.elapsed_nanos);
        self.elapsed_nanos += used;
        self.reservation
            .spend(self.spent_at(self.elapsed_nanos) - before);
        Ok((self.state()?, elapsed_nanos - used))
    }
    fn spent_at(&self, nanos: i64) -> f64 {
        let index = (nanos / SAMPLE_NANOS) as usize;
        let a = &self.knots[index];
        match self.knots.get(index + 1) {
            None => a.spent_joules,
            Some(b) => {
                a.spent_joules
                    + (b.spent_joules - a.spent_joules)
                        * ((nanos % SAMPLE_NANOS) as f64 / SAMPLE_NANOS as f64)
            }
        }
    }
    pub fn state(&self) -> Result<State, Error> {
        let index = (self.elapsed_nanos / SAMPLE_NANOS) as usize;
        let a = self.knots[index].motion.state;
        let Some(b) = self.knots.get(index + 1).map(|k| k.motion.state) else {
            return Ok(a);
        };
        let dt = (self.elapsed_nanos % SAMPLE_NANOS) as f64 * 1e-9;
        let (x, v, _) = hermite(
            [0.0; 3],
            a.velocity_mps,
            b.position.relative_meters(a.position)?,
            b.velocity_mps,
            SAMPLE_NANOS as f64 * 1e-9,
            dt,
        );
        Ok(State {
            position: a.position.translated(x)?,
            velocity_mps: v,
            epoch: a.epoch.advance_nanos(self.elapsed_nanos % SAMPLE_NANOS)?,
        })
    }
}
/// Cubic Hermite motion and its exact first/second time derivatives.
pub(crate) fn hermite(
    x0: [f64; 3],
    v0: [f64; 3],
    x1: [f64; 3],
    v1: [f64; 3],
    duration: f64,
    t: f64,
) -> ([f64; 3], [f64; 3], [f64; 3]) {
    let u = t / duration;
    let c = sub(
        scale(sub(x1, x0), 3.0),
        scale(add(scale(v0, 2.0), v1), duration),
    );
    let d = add(scale(sub(x0, x1), 2.0), scale(add(v0, v1), duration));
    let x = add(
        add(x0, scale(v0, t)),
        add(scale(c, u * u), scale(d, u * u * u)),
    );
    let v = add(
        v0,
        scale(
            add(scale(c, 2.0 * u), scale(d, 3.0 * u * u)),
            duration.recip(),
        ),
    );
    let a = scale(add(scale(c, 2.0), scale(d, 6.0 * u)), duration.powi(-2));
    (x, v, a)
}
pub(crate) fn rotate(v: [f64; 3], spin: [f64; 3], seconds: f64) -> Result<[f64; 3], Error> {
    let speed = length(spin);
    if speed == 0.0 {
        return Ok(v);
    }
    let angle = speed * seconds;
    if !angle.is_finite() {
        return Err(error("surface spin overflow"));
    }
    let axis = scale(spin, speed.recip());
    let result = add(
        add(scale(v, angle.cos()), scale(cross(axis, v), angle.sin())),
        scale(axis, dot(axis, v) * (1.0 - angle.cos())),
    );
    finite(result)?;
    Ok(result)
}
pub(crate) fn error(message: &str) -> Error {
    Error::Invalid(message.into())
}
pub(crate) fn finite(v: [f64; 3]) -> Result<(), Error> {
    if v.into_iter().all(f64::is_finite) && length(v).is_finite() {
        Ok(())
    } else {
        Err(error("surface nonfinite/overflow vector"))
    }
}
#[cfg(test)]
mod tests;
