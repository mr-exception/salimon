//! U03 independent reference comparison. No ephemeris or trajectory solver.
use crate::{Epoch, Error, invalid};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_REFERENCE_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_CASES: usize = 4096;
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    LocalContact,
    SystemFlight,
    Interstellar,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    pub id: String,
    /// Exact axes/origin convention; no implicit frame conversion.
    pub frame: String,
    pub epoch: Epoch,
    pub position_meters: [f64; 3],
    pub velocity_mps: [f64; 3],
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub position_meters: f64,
    pub velocity_mps: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub expected: Sample,
    pub regime: Regime,
    pub budget: Budget,
    pub budget_basis: String,
    pub reference: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    format: String,
    schema_version: u32,
    cases: Vec<Case>,
}
/// Validated frozen references. Stored order is stable by exact case ID.
pub struct Suite {
    cases: Vec<Case>,
}
#[derive(Debug, Serialize)]
pub struct Difference {
    pub id: String,
    pub regime: Regime,
    pub position_error_meters: f64,
    pub velocity_error_mps: f64,
    pub budget: Budget,
    pub passed: bool,
}
#[derive(Debug, Serialize)]
pub struct Comparison {
    pub passed: bool,
    pub cases: Vec<Difference>,
}
fn text(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 4096
}
fn validate_sample(s: &Sample) -> Result<(), Error> {
    if !text(&s.id)
        || !text(&s.frame)
        || !s
            .position_meters
            .into_iter()
            .chain(s.velocity_mps)
            .all(f64::is_finite)
    {
        return Err(invalid("reference sample identity/frame/nonfinite state"));
    }
    Ok(())
}
impl Suite {
    pub fn new(mut cases: Vec<Case>) -> Result<Self, Error> {
        if cases.is_empty() || cases.len() > MAX_CASES {
            return Err(invalid("reference case count"));
        }
        cases.sort_by(|a, b| a.expected.id.cmp(&b.expected.id));
        let mut ids = BTreeSet::new();
        for c in &cases {
            validate_sample(&c.expected)?;
            if !ids.insert(&c.expected.id)
                || !text(&c.reference)
                || !text(&c.budget_basis)
                || [c.budget.position_meters, c.budget.velocity_mps]
                    .into_iter()
                    .any(|x| !x.is_finite() || x < 0.0)
            {
                return Err(invalid("reference duplicate/source/budget"));
            }
        }
        Ok(Self { cases })
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_REFERENCE_BYTES {
            return Err(invalid("reference input byte limit"));
        }
        let wire: Wire =
            serde_json::from_slice(bytes).map_err(|e| Error::Encoding(e.to_string()))?;
        if wire.format != "salimon.universe.references" || wire.schema_version != 1 {
            return Err(invalid("reference format/schema"));
        }
        Self::new(wire.cases)
    }
    pub fn cases(&self) -> &[Case] {
        &self.cases
    }
    /// Every expected case must appear exactly once, with identical epoch/scale
    /// and frame. Missing/extra/nonfinite data fails instead of reducing coverage.
    pub fn compare(&self, actual: &[Sample]) -> Result<Comparison, Error> {
        if actual.len() != self.cases.len() {
            return Err(invalid("reference actual coverage count"));
        }
        let mut map = BTreeMap::new();
        for s in actual {
            validate_sample(s)?;
            if map.insert(&s.id, s).is_some() {
                return Err(invalid("reference duplicate actual ID"));
            }
        }
        let mut cases = Vec::new();
        for c in &self.cases {
            let a = map
                .get(&c.expected.id)
                .ok_or_else(|| invalid("reference missing/extra ID"))?;
            if a.frame != c.expected.frame || a.epoch != c.expected.epoch {
                return Err(invalid("reference frame/epoch/scale mismatch"));
            }
            let norm =
                |x: [f64; 3], y: [f64; 3]| (x[0] - y[0]).hypot(x[1] - y[1]).hypot(x[2] - y[2]);
            let position_error_meters = norm(a.position_meters, c.expected.position_meters);
            let velocity_error_mps = norm(a.velocity_mps, c.expected.velocity_mps);
            if !position_error_meters.is_finite() || !velocity_error_mps.is_finite() {
                return Err(invalid("reference difference overflow"));
            }
            cases.push(Difference {
                id: a.id.clone(),
                regime: c.regime,
                position_error_meters,
                velocity_error_mps,
                budget: c.budget,
                passed: position_error_meters <= c.budget.position_meters
                    && velocity_error_mps <= c.budget.velocity_mps,
            });
        }
        Ok(Comparison {
            passed: cases.iter().all(|c| c.passed),
            cases,
        })
    }
}
/// Strict bounded versioned candidate envelope for offline comparator tools.
pub fn decode_samples(bytes: &[u8]) -> Result<Vec<Sample>, Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Input {
        format: String,
        schema_version: u32,
        samples: Vec<Sample>,
    }
    if bytes.len() > MAX_REFERENCE_BYTES {
        return Err(invalid("reference actual byte limit"));
    }
    let input: Input = serde_json::from_slice(bytes).map_err(|e| Error::Encoding(e.to_string()))?;
    if input.format != "salimon.universe.samples"
        || input.schema_version != 1
        || input.samples.is_empty()
        || input.samples.len() > MAX_CASES
    {
        return Err(invalid("reference actual format/schema/count"));
    }
    for sample in &input.samples {
        validate_sample(sample)?;
    }
    Ok(input.samples)
}
#[cfg(test)]
mod tests;
