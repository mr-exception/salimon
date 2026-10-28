//! Analytical five-second pulse intersection with certified linear path pieces.
use super::*;
pub const PULSE_MILLIS: u64 = 5000;
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub range_meters: f64,
    pub idle_watts: f64,
    pub pulse_joules: f64,
}
impl Parameters {
    pub fn validate(self) -> Result<(), Error> {
        if !self.range_meters.is_finite()
            || self.range_meters <= 0.
            || self.range_meters > 1e20
            || !self.idle_watts.is_finite()
            || self.idle_watts <= 0.
            || self.idle_watts > 1e12
            || !self.pulse_joules.is_finite()
            || self.pulse_joules <= 0.
            || self.pulse_joules > 1e15
        {
            return Err(invalid("scanner parameters"));
        }
        Ok(())
    }
    /// Reserved future noncelestial contract; no ship/station generation here.
    pub fn noncelestial_range_meters(self) -> f64 {
        self.range_meters * 0.4
    }
    pub fn cost(self, start_ms: u64, end_ms: u64) -> Result<f64, Error> {
        self.validate()?;
        let elapsed = end_ms
            .checked_sub(start_ms)
            .ok_or_else(|| invalid("scanner interval regressed"))?;
        let pulses = end_ms / PULSE_MILLIS - start_ms / PULSE_MILLIS;
        let cost = elapsed as f64 * 0.001 * self.idle_watts + pulses as f64 * self.pulse_joules;
        if !cost.is_finite() {
            return Err(invalid("scanner power overflow"));
        }
        Ok(cost)
    }
}
/// A host-certified affine trajectory piece in the same inertial galactic frame.
/// Nonlinear orbital paths must be split/certified by U31/U32, never silently
/// flattened into one chord. Both tracks are sampled at start_ms.
#[derive(Clone, Debug)]
pub struct LinearTrack {
    pub position: Position,
    pub velocity_mps: [f64; 3],
}
/// Latest successful pulse on (start_ms,end_ms], inclusive range. Constant work
/// per object/piece regardless of elapsed pulse count; all directions, no occlusion.
pub fn last_detection(
    start_ms: u64,
    end_ms: u64,
    observer: &LinearTrack,
    target: &Observation,
    range: f64,
) -> Result<Option<(u64, Observation)>, Error> {
    let duration = end_ms
        .checked_sub(start_ms)
        .ok_or_else(|| invalid("scanner interval regressed"))? as f64
        * 0.001;
    if !range.is_finite()
        || range <= 0.
        || range > 1e20
        || !observer
            .velocity_mps
            .into_iter()
            .chain(target.velocity_mps)
            .all(f64::is_finite)
    {
        return Err(invalid("scanner trajectory/range"));
    }
    let r = target.position.relative_meters(&observer.position)?;
    let v: [f64; 3] = std::array::from_fn(|i| target.velocity_mps[i] - observer.velocity_mps[i]);
    let speed = v[0].hypot(v[1]).hypot(v[2]);
    if !speed.is_finite() {
        return Err(invalid("scanner relative speed overflow"));
    }
    let (entry, exit) = if speed == 0. {
        if r[0].hypot(r[1]).hypot(r[2]) > range {
            return Ok(None);
        }
        (0., duration)
    } else {
        let unit = v.map(|x| x / speed);
        let projection: f64 = r.iter().zip(unit).map(|(a, b)| a * b).sum();
        let perpendicular: [f64; 3] = std::array::from_fn(|i| r[i] - projection * unit[i]);
        let distance = perpendicular[0]
            .hypot(perpendicular[1])
            .hypot(perpendicular[2]);
        if !projection.is_finite() || !distance.is_finite() {
            return Err(invalid("scanner trajectory overflow"));
        }
        if distance > range {
            return Ok(None);
        }
        let half = ((range - distance) * (range + distance)).sqrt() / speed;
        let middle = -projection / speed;
        ((middle - half).max(0.), (middle + half).min(duration))
    };
    if entry > exit || exit < 0. {
        return Ok(None);
    }
    // Integer lattice avoids accumulated phase error and per-pulse iteration.
    let offset_ms = (exit * 1000.).floor().min((end_ms - start_ms) as f64) as u64;
    let last = start_ms.saturating_add(offset_ms) / PULSE_MILLIS * PULSE_MILLIS;
    if last <= start_ms || last > end_ms || (last - start_ms) as f64 * 0.001 < entry {
        return Ok(None);
    }
    let seconds = (last - start_ms) as f64 * 0.001;
    let at = Observation {
        position: target
            .position
            .translated(target.velocity_mps.map(|v| v * seconds))?,
        ..target.clone()
    };
    Ok(Some((last, at)))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scanner_pulses_crossing_edge_miss_and_long_interval() {
        let (_, mut o) = super::super::tests::fixture();
        o.position = Position::Galactic {
            position: GalacticPosition::new([0; 3], [100., 0., 0.]).unwrap(),
        };
        o.velocity_mps = [-10., 0., 0.];
        let observer = LinearTrack {
            position: Position::Galactic {
                position: GalacticPosition::new([0; 3], [0.; 3]).unwrap(),
            },
            velocity_mps: [0.; 3],
        };
        let (at, hit) = last_detection(0, 20_000, &observer, &o, 50.)
            .unwrap()
            .unwrap();
        assert_eq!(at, 15_000);
        assert_eq!(
            hit.position.relative_meters(&observer.position).unwrap(),
            [-50., 0., 0.]
        );
        assert!(
            last_detection(0, 4999, &observer, &o, 50.)
                .unwrap()
                .is_none()
        );
        assert!(
            last_detection(0, 20_000, &observer, &o, 1.)
                .unwrap()
                .is_some()
        );
        o.position = Position::Galactic {
            position: GalacticPosition::new([0; 3], [125., 0., 0.]).unwrap(),
        };
        assert!(
            last_detection(0, 20_000, &observer, &o, 1.)
                .unwrap()
                .is_none()
        );
        o.position = observer.position.clone();
        o.velocity_mps = [0.; 3];
        assert_eq!(
            last_detection(0, 1_000_000_000_000, &observer, &o, 1.)
                .unwrap()
                .unwrap()
                .0,
            1_000_000_000_000
        );
    }
    #[test]
    fn scanner_parameters_power_and_phase() {
        let p = Parameters {
            range_meters: 100.,
            idle_watts: 10.,
            pulse_joules: 50.,
        };
        assert!((p.cost(0, 4999).unwrap() - 49.99).abs() < 1e-12);
        assert_eq!(p.cost(4999, 5000).unwrap(), 50.01);
        assert_eq!(p.cost(5000, 5001).unwrap(), 0.01);
        assert_eq!(p.noncelestial_range_meters(), 40.);
        assert!(p.cost(5, 4).is_err());
    }
}

/// One funded, host-certified linear piece. Split at every power, trajectory,
/// catalog or range change. Powered=false advances expiry without discoveries.
pub struct Sweep<'a> {
    pub start_ms: u64,
    pub end_ms: u64,
    pub observer: &'a LinearTrack,
    pub targets: &'a [Observation],
    pub parameters: Parameters,
    pub powered: bool,
}
impl Contacts {
    pub fn scan_linear(&mut self, catalog: &Catalog, sweep: Sweep<'_>) -> Result<(), Error> {
        sweep.parameters.cost(sweep.start_ms, sweep.end_ms)?;
        if sweep.start_ms != self.watermark_utc_millis
            || sweep.targets.len() > MAX_CONTACTS
            || catalog.revision() != self.catalog_revision
        {
            return Err(invalid("scanner sweep continuity/catalog/size"));
        }
        let mut hits = Vec::new();
        let mut ids = std::collections::BTreeSet::new();
        for target in sweep.targets {
            validate_observation(catalog, target)?;
            if !ids.insert(&target.id) {
                return Err(invalid("duplicate scanner sweep target"));
            }
            if let Some(hit) = last_detection(
                sweep.start_ms,
                sweep.end_ms,
                sweep.observer,
                target,
                sweep.parameters.range_meters,
            )? {
                hits.push(hit);
            }
        }
        hits.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.id.cmp(&b.1.id)));
        let mut candidate = self.clone();
        if sweep.powered {
            for (at, hit) in hits {
                candidate.refresh(catalog, at, true, &[hit])?;
            }
        }
        candidate.advance(sweep.end_ms)?;
        *self = candidate;
        Ok(())
    }
}

#[cfg(test)]
mod sweep_tests {
    use super::*;
    #[test]
    fn scanner_offline_crossing_refreshes_latest_funded_pulse_and_stale_history_survives() {
        let (c, mut target) = super::super::tests::fixture();
        let origin = Position::Galactic {
            position: GalacticPosition::new([0; 3], [0.; 3]).unwrap(),
        };
        target.position = origin.translated([100., 0., 0.]).unwrap();
        target.velocity_mps = [-10., 0., 0.];
        let observer = LinearTrack {
            position: origin,
            velocity_mps: [0.; 3],
        };
        let parameters = Parameters {
            range_meters: 50.,
            idle_watts: 1.,
            pulse_joules: 1.,
        };
        let mut s = Contacts::new(c.revision()).unwrap();
        s.scan_linear(
            &c,
            Sweep {
                start_ms: 0,
                end_ms: 20_000,
                observer: &observer,
                targets: std::slice::from_ref(&target),
                parameters,
                powered: true,
            },
        )
        .unwrap();
        assert_eq!(s.history().next().unwrap().last_seen_utc_millis(), 15_000);
        let end = 20_000 + CONTACT_TTL_MILLIS;
        s.scan_linear(
            &c,
            Sweep {
                start_ms: 20_000,
                end_ms: end,
                observer: &observer,
                targets: std::slice::from_ref(&target),
                parameters,
                powered: false,
            },
        )
        .unwrap();
        assert_eq!(s.active(end).count(), 0);
        assert_eq!(s.history().count(), 1);
        let before = s.clone();
        let mut bad = target.clone();
        bad.id = CatalogId::from_source("missing", "star").unwrap();
        assert!(
            s.scan_linear(
                &c,
                Sweep {
                    start_ms: end,
                    end_ms: end + 5000,
                    observer: &observer,
                    targets: &[target, bad],
                    parameters,
                    powered: true
                }
            )
            .is_err()
        );
        assert_eq!(s, before);
    }
    #[test]
    fn scanner_mismatched_body_frames_and_nonfinite_coordinates_reject() {
        let (_, mut o) = super::super::tests::fixture();
        let c = crate::starting_system::candidate_publication()
            .unwrap()
            .catalog()
            .clone();
        let body = c
            .records()
            .iter()
            .find(|r| matches!(r, Record::Body(_)))
            .unwrap()
            .id()
            .clone();
        o.id = body.clone();
        o.position = Position::BodyLocal {
            body: body.clone(),
            meters: [f64::NAN, 0., 0.],
        };
        let mut s = Contacts::new(c.revision()).unwrap();
        assert!(s.refresh(&c, 10, true, std::slice::from_ref(&o)).is_err());
        let local = Position::BodyLocal {
            body,
            meters: [0.; 3],
        };
        let galaxy = Position::Galactic {
            position: GalacticPosition::new([0; 3], [0.; 3]).unwrap(),
        };
        assert!(local.relative_meters(&galaxy).is_err());
    }
}
