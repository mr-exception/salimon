//! Time-addressable portable models. Accuracy labels describe methods, not
//! approved scientific error budgets. No implicit epoch-scale conversion.
use crate::{
    CatalogId, Epoch, Error, TimeScale,
    coordinates::{FrameId, FrameLink, FrameTree, Rotation, State, SystemAnchor},
    invalid,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    pub epoch: Epoch,
    pub position_meters: [f64; 3],
    pub velocity_mps: [f64; 3],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Accuracy {
    ObservedSample,
    Interpolated,
    Analytic,
    LowerAccuracyExtrapolation,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Model {
    /// Cubic Hermite between imported samples; outside validity is rejected
    /// unless an explicit, bounded linear fallback is supplied.
    Sampled {
        samples: Vec<Sample>,
        max_extrapolation_seconds: f64,
    },
    /// Explicit stellar proper/radial motion, never missing velocity -> zero.
    Linear { sample: Sample, valid_seconds: f64 },
    /// Elliptic two-body orbit in explicitly supplied right-handed parent axes.
    Elliptic {
        epoch: Epoch,
        semi_major_axis_meters: f64,
        eccentricity: f64,
        period_seconds: f64,
        mean_anomaly_radians: f64,
        axes: [[f64; 3]; 3],
    },
}
fn finite(s: Sample) -> Result<(), Error> {
    if s.epoch.scale() == TimeScale::Utc
        || !s
            .position_meters
            .into_iter()
            .chain(s.velocity_mps)
            .all(f64::is_finite)
    {
        return Err(invalid(
            "ephemeris nonfinite state/UTC needs explicit conversion",
        ));
    }
    Ok(())
}
impl Model {
    /// Load a typed model from a checksum-verified private pack sidecar.
    /// Model-specific validity is checked on evaluation, not inferred from a hash.
    pub fn from_pack(
        reader: &crate::reader::Reader,
        key: crate::partitions::CellKey,
        name: &str,
    ) -> Result<Self, Error> {
        let bytes = reader.sidecar(key, name)?;
        if bytes.len() > crate::MAX_SCHEMA_JSON_BYTES {
            return Err(invalid("ephemeris sidecar size"));
        }
        serde_json::from_slice(&bytes).map_err(|e| Error::Encoding(e.to_string()))
    }
    pub fn from_horizons(e: &crate::ingestion::horizons::Ephemeris) -> Self {
        Self::Sampled {
            samples: e
                .samples()
                .iter()
                .map(|s| Sample {
                    epoch: s.epoch,
                    position_meters: s.position_meters,
                    velocity_mps: s.velocity_mps,
                })
                .collect(),
            max_extrapolation_seconds: 0.0,
        }
    }
    pub fn evaluate(&self, epoch: Epoch) -> Result<(State, Accuracy), Error> {
        let (s, quality) = match self {
            Self::Linear {
                sample,
                valid_seconds,
            } => {
                finite(*sample)?;
                let dt = epoch.seconds_since(sample.epoch)?;
                if !valid_seconds.is_finite() || *valid_seconds < 0.0 || dt.abs() > *valid_seconds {
                    return Err(invalid("linear motion validity"));
                }
                (
                    Sample {
                        epoch,
                        position_meters: std::array::from_fn(|i| {
                            sample.position_meters[i] + sample.velocity_mps[i] * dt
                        }),
                        velocity_mps: sample.velocity_mps,
                    },
                    Accuracy::Analytic,
                )
            }
            Self::Sampled {
                samples,
                max_extrapolation_seconds,
            } => {
                if samples.is_empty()
                    || samples.len() > 65536
                    || !max_extrapolation_seconds.is_finite()
                    || *max_extrapolation_seconds < 0.0
                {
                    return Err(invalid("ephemeris sample/fallback bounds"));
                }
                for (i, s) in samples.iter().enumerate() {
                    finite(*s)?;
                    if i > 0 && s.epoch.seconds_since(samples[i - 1].epoch)? <= 0.0 {
                        return Err(invalid("ephemeris sample order"));
                    }
                }
                let t = epoch.seconds_since(samples[0].epoch)?;
                let last = samples.last().expect("nonempty samples validated");
                let end = last.epoch.seconds_since(samples[0].epoch)?;
                if t < 0.0 || t > end {
                    let edge = if t < 0.0 { samples[0] } else { *last };
                    let dt = epoch.seconds_since(edge.epoch)?;
                    if dt.abs() > *max_extrapolation_seconds {
                        return Err(invalid(
                            "ephemeris outside imported validity; fallback unavailable",
                        ));
                    }
                    (
                        Sample {
                            epoch,
                            position_meters: std::array::from_fn(|i| {
                                edge.position_meters[i] + edge.velocity_mps[i] * dt
                            }),
                            velocity_mps: edge.velocity_mps,
                        },
                        Accuracy::LowerAccuracyExtrapolation,
                    )
                } else if let Some(s) = samples.iter().find(|s| s.epoch == epoch) {
                    (*s, Accuracy::ObservedSample)
                } else {
                    let i = samples
                        .partition_point(|s| s.epoch.compare(epoch).is_ok_and(|o| o.is_lt()));
                    let a = samples[i - 1];
                    let b = samples[i];
                    let h = b.epoch.seconds_since(a.epoch)?;
                    let u = epoch.seconds_since(a.epoch)? / h;
                    let position_meters = std::array::from_fn(|j| {
                        (2.0 * u * u * u - 3.0 * u * u + 1.0) * a.position_meters[j]
                            + (u * u * u - 2.0 * u * u + u) * h * a.velocity_mps[j]
                            + (-2.0 * u * u * u + 3.0 * u * u) * b.position_meters[j]
                            + (u * u * u - u * u) * h * b.velocity_mps[j]
                    });
                    let velocity_mps = std::array::from_fn(|j| {
                        ((6.0 * u * u - 6.0 * u) * a.position_meters[j]
                            + (-6.0 * u * u + 6.0 * u) * b.position_meters[j])
                            / h
                            + (3.0 * u * u - 4.0 * u + 1.0) * a.velocity_mps[j]
                            + (3.0 * u * u - 2.0 * u) * b.velocity_mps[j]
                    });
                    (
                        Sample {
                            epoch,
                            position_meters,
                            velocity_mps,
                        },
                        Accuracy::Interpolated,
                    )
                }
            }
            Self::Elliptic {
                epoch: origin,
                semi_major_axis_meters: a,
                eccentricity: e,
                period_seconds: period,
                mean_anomaly_radians: phase,
                axes,
            } => {
                if origin.scale() == TimeScale::Utc
                    || ![a, e, period, phase].into_iter().all(|v| v.is_finite())
                    || *a <= 0.0
                    || !(0.0..1.0).contains(e)
                    || *period <= 0.0
                {
                    return Err(invalid("elliptic orbit elements"));
                }
                Rotation::new(*axes)?;
                let tau = std::f64::consts::TAU;
                let dt = epoch.seconds_since(*origin)?;
                let mean = (phase + tau * (dt.rem_euclid(*period) / period)).rem_euclid(tau);
                let (mut lo, mut hi) = (0.0, tau);
                for _ in 0..64 {
                    let mid = (lo + hi) / 2.0;
                    if mid - e * mid.sin() < mean {
                        lo = mid
                    } else {
                        hi = mid
                    }
                }
                let anomaly = (lo + hi) / 2.0;
                let (sin, cos) = anomaly.sin_cos();
                let beta = (1.0 - e * e).sqrt();
                let rate = (tau / period) / (1.0 - e * cos);
                let p = [a * (cos - e), a * beta * sin, 0.0];
                let v = [-a * sin * rate, a * beta * cos * rate, 0.0];
                let rotate =
                    |v: [f64; 3]| std::array::from_fn(|i| (0..3).map(|j| axes[j][i] * v[j]).sum());
                (
                    Sample {
                        epoch,
                        position_meters: rotate(p),
                        velocity_mps: rotate(v),
                    },
                    Accuracy::Analytic,
                )
            }
        };
        finite(s)?;
        Ok((
            State {
                meters: s.position_meters,
                meters_per_second: s.velocity_mps,
            },
            quality,
        ))
    }
}
pub struct BodyTrack {
    pub object: CatalogId,
    pub parent: FrameId,
    pub model: Model,
    /// Explicit spin about body +Z relative to parent, rad/s at spin_epoch.
    pub spin_epoch: Epoch,
    pub spin_radians: f64,
    pub spin_rps: f64,
}
pub fn frame_tree(
    anchor: SystemAnchor,
    tracks: &[BodyTrack],
    epoch: Epoch,
) -> Result<(FrameTree, Vec<(CatalogId, Accuracy)>), Error> {
    if tracks.len() > 4096 {
        return Err(invalid("ephemeris track bound"));
    }
    let mut ids = BTreeSet::new();
    let mut links = Vec::new();
    let mut quality = Vec::new();
    let position = anchor.position_at(epoch)?;
    for t in tracks {
        if !ids.insert(&t.object) || !t.spin_radians.is_finite() || !t.spin_rps.is_finite() {
            return Err(invalid("ephemeris duplicate/spin"));
        }
        let (state, q) = t.model.evaluate(epoch)?;
        let angle = t.spin_radians + t.spin_rps * epoch.seconds_since(t.spin_epoch)?;
        let (s, c) = angle.sin_cos();
        links.push(FrameLink {
            child: FrameId::Body(t.object.clone()),
            parent: t.parent.clone(),
            epoch,
            origin_meters: state.meters,
            velocity_mps: state.meters_per_second,
            rotation: Rotation::new([[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]])?,
            angular_velocity_rps: [0.0, 0.0, t.spin_rps],
        });
        quality.push((t.object.clone(), q));
    }
    let anchor = SystemAnchor {
        epoch,
        position,
        ..anchor
    };
    Ok((FrameTree::new(anchor, links)?, quality))
}
#[cfg(test)]
mod tests;
