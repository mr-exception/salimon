//! Player knowledge from declared measurements, independent of catalog truth.
use crate::{CatalogId, Epoch, Error, invalid};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub beacon_range_meters: f64,
    pub grid_meters: f64,
    pub fix_error_meters: f64,
    pub drift_per_meter: f64,
    pub drift_per_second: f64,
    pub bearing_error_radians: f64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            beacon_range_meters: 500.,
            grid_meters: 2.,
            fix_error_meters: 5.,
            drift_per_meter: 0.02,
            drift_per_second: 0.01,
            bearing_error_radians: 0.02,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), Error> {
        let positive = [
            self.beacon_range_meters,
            self.grid_meters,
            self.fix_error_meters,
            self.bearing_error_radians,
        ];
        if positive.iter().any(|v| !v.is_finite() || *v <= 0.)
            || self.bearing_error_radians > std::f64::consts::PI
            || [self.drift_per_meter, self.drift_per_second]
                .iter()
                .any(|v| !v.is_finite() || *v < 0.)
            || self.fix_error_meters < self.grid_meters * 3_f64.sqrt() / 2.
        {
            return Err(invalid("suit measurement configuration"));
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Source {
    ShipBeacon,
    DeadReckoning,
    Unavailable,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Estimate {
    pub position_meters: Option<[f64; 3]>,
    pub radius_meters: Option<f64>,
    pub source: Source,
    pub measured_at: Epoch,
}
impl Estimate {
    pub fn quality(&self) -> &'static str {
        match self.radius_meters {
            None => "UNAVAILABLE",
            Some(r) if r <= 10. => "WELL CONSTRAINED",
            Some(r) if r <= 100. => "ESTIMATED",
            Some(_) => "POORLY CONSTRAINED",
        }
    }
    fn validate(&self) -> Result<(), Error> {
        match (self.position_meters, self.radius_meters, self.source) {
            (Some(p), Some(r), Source::ShipBeacon | Source::DeadReckoning)
                if p.iter().all(|v| v.is_finite()) && r.is_finite() && r > 0. =>
            {
                Ok(())
            }
            (None, None, Source::Unavailable) => Ok(()),
            _ => Err(invalid("suit estimate position/uncertainty/source")),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mark {
    pub id: u64,
    pub epoch: Epoch,
    pub estimate: Estimate,
    /// Unit body-fixed direction only, never an inferred target coordinate.
    pub bearing: Option<[f64; 3]>,
    pub bearing_error_radians: Option<f64>,
    pub note: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Book {
    pub version: u32,
    pub frame_version: u32,
    pub body: CatalogId,
    pub definition_sha256: String,
    pub config: Config,
    pub enabled: bool,
    pub estimate: Estimate,
    next_id: u64,
    marks: Vec<Mark>,
}
impl Book {
    pub fn new(body: CatalogId, definition_sha256: String, epoch: Epoch) -> Result<Self, Error> {
        let b = Self {
            version: 1,
            frame_version: 1,
            body,
            definition_sha256,
            config: Default::default(),
            enabled: true,
            estimate: Estimate {
                position_meters: None,
                radius_meters: None,
                source: Source::Unavailable,
                measured_at: epoch,
            },
            next_id: 1,
            marks: vec![],
        };
        b.validate()?;
        Ok(b)
    }
    pub fn marks(&self) -> &[Mark] {
        &self.marks
    }
    pub fn validate(&self) -> Result<(), Error> {
        self.config.validate()?;
        self.estimate.validate()?;
        if (!self.enabled && self.estimate.source != Source::Unavailable)
            || self.version != 1
            || self.frame_version != 1
            || self.definition_sha256.len() != 64
            || !self
                .definition_sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
            || self.marks.len() > 1024
            || self.next_id == 0
        {
            return Err(invalid("surface observation version/definition/capacity"));
        }
        let mut last = 0;
        for m in &self.marks {
            m.estimate.validate()?;
            if m.id <= last
                || m.id >= self.next_id
                || m.note.chars().count() > 256
                || m.note.chars().any(char::is_control)
                || m.epoch.seconds_since(m.estimate.measured_at)? < 0.
            {
                return Err(invalid("surface observation identity/note/time"));
            }
            match (m.bearing, m.bearing_error_radians) {
                (None, None) => (),
                (Some(v), Some(e))
                    if v.iter().all(|v| v.is_finite())
                        && (norm(v) - 1.).abs() < 1e-10
                        && e.is_finite()
                        && e > 0.
                        && e <= std::f64::consts::PI => {}
                _ => return Err(invalid("surface observation bearing/error")),
            }
            last = m.id;
        }
        Ok(())
    }
    pub fn compatible(&self, body: &CatalogId, hash: &str) -> Result<(), Error> {
        self.validate()?;
        if &self.body != body || self.definition_sha256 != hash {
            return Err(invalid(
                "surface observations require explicit body/definition migration",
            ));
        }
        Ok(())
    }
    /// Measurement adapter supplies a quantizable *beacon fix*, never a mark target.
    pub fn fix(&mut self, measured_body_meters: [f64; 3], epoch: Epoch) -> Result<(), Error> {
        self.config.validate()?;
        if !self.enabled
            || !measured_body_meters.iter().all(|v| v.is_finite())
            || epoch.seconds_since(self.estimate.measured_at)? < 0.
        {
            return Err(invalid("suit fix disconnected/invalid time or position"));
        }
        let position = measured_body_meters
            .map(|v| (v / self.config.grid_meters).round() * self.config.grid_meters);
        if !position.iter().all(|v| v.is_finite()) {
            return Err(invalid("suit fix overflow"));
        }
        self.estimate = Estimate {
            position_meters: Some(position),
            radius_meters: Some(self.config.fix_error_meters),
            source: Source::ShipBeacon,
            measured_at: epoch,
        };
        Ok(())
    }
    /// Body-fixed measured displacement; frame transforms belong to coordinates.
    pub fn drift(&mut self, displacement: [f64; 3], epoch: Epoch) -> Result<(), Error> {
        self.config.validate()?;
        let dt = epoch.seconds_since(self.estimate.measured_at)?;
        if dt < 0. || !displacement.iter().all(|v| v.is_finite()) {
            return Err(invalid("suit displacement/time"));
        }
        if !self.enabled {
            self.estimate = Estimate {
                position_meters: None,
                radius_meters: None,
                source: Source::Unavailable,
                measured_at: epoch,
            };
            return Ok(());
        }
        let mut e = self.estimate.clone();
        if let (Some(p), Some(r)) = (e.position_meters, e.radius_meters) {
            // Declared 0.2% inertial scale bias, bounded by the configured path envelope.
            let gain = 1. + self.config.drift_per_meter.min(0.002);
            e.position_meters = Some(std::array::from_fn(|i| p[i] + displacement[i] * gain));
            e.radius_meters = Some(
                r + norm(displacement) * self.config.drift_per_meter
                    + dt * self.config.drift_per_second,
            );
            e.source = Source::DeadReckoning;
        }
        e.measured_at = epoch;
        e.validate()?;
        self.estimate = e;
        Ok(())
    }
    pub fn mark(
        &mut self,
        bearing: Option<[f64; 3]>,
        note: String,
        epoch: Epoch,
    ) -> Result<u64, Error> {
        let mut candidate = self.clone();
        let id = candidate.next_id;
        candidate.next_id = id
            .checked_add(1)
            .ok_or_else(|| invalid("observation IDs exhausted"))?;
        let bearing = match bearing {
            Some(v) if v.iter().all(|v| v.is_finite()) && norm(v) > 1e-12 => {
                Some(v.map(|x| x / norm(v)))
            }
            None => None,
            _ => return Err(invalid("observation sightline")),
        };
        candidate.marks.push(Mark {
            id,
            epoch,
            estimate: candidate.estimate.clone(),
            bearing,
            bearing_error_radians: bearing.map(|_| candidate.config.bearing_error_radians),
            note,
        });
        candidate.validate()?;
        *self = candidate;
        Ok(id)
    }
}
fn norm(v: [f64; 3]) -> f64 {
    v[0].hypot(v[1]).hypot(v[2])
}

impl Book {
    /// Consumers with inertial measurements must supply the evaluated rotating
    /// frame tree at the measurement epoch. No unstated frame/epoch conversion.
    pub fn fix_in_frame(
        &mut self,
        tree: &crate::coordinates::FrameTree,
        from: &crate::coordinates::FrameId,
        sample: crate::coordinates::State,
        epoch: Epoch,
    ) -> Result<(), Error> {
        let local = tree.transform(
            from,
            &crate::coordinates::FrameId::Body(self.body.clone()),
            sample,
            epoch,
        )?;
        self.fix(local.meters, epoch)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn time(s: i64) -> Epoch {
        Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], crate::TimeScale::Tt)
            .unwrap()
            .advance_nanos(s * 1_000_000_000)
            .unwrap()
    }
    fn book() -> Book {
        Book::new(
            CatalogId::from_source("test", "planet").unwrap(),
            "a".repeat(64),
            time(0),
        )
        .unwrap()
    }
    #[test]
    fn field_observations_preserve_unknown_range_provenance_and_identity_on_roundtrip() {
        let mut b = book();
        assert_eq!(b.estimate.quality(), "UNAVAILABLE");
        b.mark(Some([2., 0., 0.]), "distant ridge".into(), time(0))
            .unwrap();
        assert_eq!(b.marks()[0].estimate.position_meters, None);
        b.fix([1.4, 2.7, 3.8], time(1)).unwrap();
        assert_eq!(b.estimate.position_meters, Some([2., 2., 4.]));
        assert_eq!(b.estimate.quality(), "WELL CONSTRAINED");
        b.mark(None, "parking".into(), time(1)).unwrap();
        b.drift([5000., 0., 0.], time(21)).unwrap();
        assert_eq!(b.estimate.quality(), "POORLY CONSTRAINED");
        let decoded: Book = serde_json::from_slice(&serde_json::to_vec(&b).unwrap()).unwrap();
        assert_eq!(b, decoded);
        decoded.validate().unwrap();
        assert!(b.compatible(&b.body, &"b".repeat(64)).is_err());
        assert_eq!(b.marks()[0].bearing, Some([1., 0., 0.]));
        assert_eq!(b.marks()[1].id, 2);
    }
    #[test]
    fn field_inertial_error_bound_contains_truth_and_partitioned_measurements_agree() {
        let mut a = book();
        a.fix([0.; 3], time(0)).unwrap();
        let mut b = a.clone();
        a.drift([100., 0., 0.], time(100)).unwrap();
        for i in 1..=100 {
            b.drift([1., 0., 0.], time(i)).unwrap();
        }
        assert!(
            (a.estimate.position_meters.unwrap()[0] - b.estimate.position_meters.unwrap()[0]).abs()
                < 1e-10
        );
        assert!(
            (a.estimate.radius_meters.unwrap() - b.estimate.radius_meters.unwrap()).abs() < 1e-10
        );
        assert!(
            (a.estimate.position_meters.unwrap()[0] - 100.).abs()
                < a.estimate.radius_meters.unwrap()
        );
        let old = a.clone();
        assert!(a.mark(Some([0.; 3]), String::new(), time(100)).is_err());
        assert_eq!(a, old);
        assert!(a.fix([0.; 3], time(0)).is_err());
        assert_eq!(a, old);
        a.enabled = false;
        a.drift([10.; 3], time(101)).unwrap();
        assert_eq!(a.estimate.quality(), "UNAVAILABLE");
        assert!(a.fix([0.; 3], time(101)).is_err());
    }
    #[test]
    fn field_rotating_planet_transform_requires_exact_epoch_and_retains_body_frame() {
        use crate::coordinates::*;
        let mut b = book();
        let system = CatalogId::from_source("test", "system").unwrap();
        let tree = FrameTree::new(
            SystemAnchor {
                system: system.clone(),
                epoch: time(0),
                position: GalacticPosition::new([999; 3], [0.; 3]).unwrap(),
                velocity_mps: [0.; 3],
                rotation: Rotation::IDENTITY,
            },
            vec![FrameLink {
                child: FrameId::Body(b.body.clone()),
                parent: FrameId::System(system.clone()),
                epoch: time(0),
                origin_meters: [1000., 0., 0.],
                velocity_mps: [10., 0., 0.],
                rotation: Rotation::new([[0., 1., 0.], [-1., 0., 0.], [0., 0., 1.]]).unwrap(),
                angular_velocity_rps: [0., 0., 0.1],
            }],
        )
        .unwrap();
        b.fix_in_frame(
            &tree,
            &FrameId::System(system.clone()),
            State {
                meters: [1000., 20., 0.],
                meters_per_second: [8., 0., 0.],
            },
            time(0),
        )
        .unwrap();
        assert_eq!(b.estimate.position_meters, Some([20., 0., 0.]));
        let old = b.clone();
        assert!(
            b.fix_in_frame(
                &tree,
                &FrameId::System(system),
                State {
                    meters: [0.; 3],
                    meters_per_second: [0.; 3]
                },
                time(1)
            )
            .is_err()
        );
        assert_eq!(b, old);
    }
    #[test]
    fn field_malformed_checkpoint_and_configuration_reject_without_allocating() {
        let mut b = book();
        let old = b.clone();
        assert!(b.mark(None, "x".repeat(257), time(0)).is_err());
        assert_eq!(b, old);
        assert!(b.mark(None, "newline\n".into(), time(0)).is_err());
        assert_eq!(b, old);
        b.config.grid_meters = 100.;
        assert!(b.validate().is_err());
        let mut b = book();
        for _ in 0..1024 {
            b.mark(None, String::new(), time(0)).unwrap();
        }
        let old = b.clone();
        assert!(b.mark(None, String::new(), time(0)).is_err());
        assert_eq!(b, old);
        let mut json = serde_json::to_value(book()).unwrap();
        json["estimate"]["radius_meters"] = 0.into();
        assert!(
            serde_json::from_value::<Book>(json)
                .unwrap()
                .validate()
                .is_err()
        );
    }
}
