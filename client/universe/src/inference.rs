//! Offline, bounded, explicitly model-conditioned completion of missing data.
//! No distance inference, orbit integration, calibrated posterior or runtime RNG.
use crate::{
    BodyKind, Catalog, CatalogId, Error, Provenance, Quantity, Record, Uncertainty, invalid,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

mod observed;
pub use observed::complete_observed;

pub const MODEL_VERSION: &str = "salimon-inference-v1";
pub const SOLAR_MASS_KG: f64 = 1.98847e30;
pub const SOLAR_RADIUS_METERS: f64 = 695_700_000.0;
pub const EARTH_MASS_KG: f64 = 5.9722e24;
pub const EARTH_RADIUS_METERS: f64 = 6_371_000.0;
pub const AU_METERS: f64 = 149_597_870_700.0;
pub const YEAR_SECONDS: f64 = 365.25 * 86400.0;
const G: f64 = 6.67430e-11;
const MAX_ATTEMPTS: u64 = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum StellarModel {
    /// Explicit caller classification; never applied to giants/remnants/binaries.
    MainSequenceV1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum PlanetModel {
    RockyV1,
    GasGiantV1,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PlanetInput {
    pub planet: CatalogId,
    pub model: PlanetModel,
    pub semi_major_axis_meters: Option<Quantity>,
    pub period_seconds: Option<Quantity>,
    pub eccentricity: Option<Quantity>,
    /// A lower bound, never substituted for a measured true mass.
    pub minimum_mass_kg: Option<Quantity>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SystemInput {
    pub host: CatalogId,
    pub model: StellarModel,
    pub seed: u64,
    pub age_seconds: Option<Quantity>,
    /// Must cover every planet of this single-star system exactly once.
    pub planets: Vec<PlanetInput>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Confidence {
    /// Qualitative model confidence, not a calibrated probability/credible interval.
    LowPrior,
    ModelConditional,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Distribution {
    Uniform {
        lower: f64,
        upper: f64,
    },
    LogUniform {
        lower: f64,
        upper: f64,
    },
    /// A deterministic relation conditional on the sampled/preserved inputs.
    Derived {
        relation: String,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Evidence {
    pub object: CatalogId,
    pub field: String,
    pub distribution: Distribution,
    pub confidence: Confidence,
    pub quantity: Quantity,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Orbit {
    pub planet: CatalogId,
    pub semi_major_axis_meters: Quantity,
    pub period_seconds: Quantity,
    pub eccentricity: Quantity,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Completion {
    pub model_version: String,
    pub seed: u64,
    pub accepted_attempt: u64,
    pub stellar_model: StellarModel,
    pub planet_models: BTreeMap<CatalogId, PlanetModel>,
    pub catalog: Catalog,
    pub age_seconds: Quantity,
    /// Partial Kepler elements only, with no invented epoch, phase or orientation.
    pub orbits: Vec<Orbit>,
    pub evidence: Vec<Evidence>,
}

struct Sampler<'a> {
    seed: u64,
    attempt: u64,
    datasets: Vec<CatalogId>,
    evidence: Vec<Evidence>,
    object: &'a CatalogId,
}
impl Sampler<'_> {
    fn inferred(
        &mut self,
        field: &str,
        value: f64,
        distribution: Distribution,
        confidence: Confidence,
    ) -> Quantity {
        let uncertainty = match distribution {
            Distribution::Uniform { lower, upper } | Distribution::LogUniform { lower, upper } => {
                Uncertainty::Bounds {
                    minus: value - lower,
                    plus: upper - value,
                    confidence: None,
                }
            }
            Distribution::Derived { .. } => Uncertainty::Unknown,
        };
        let quantity = Quantity {
            value,
            uncertainty,
            validity: None,
            provenance: Provenance::Inferred {
                method: format!("{MODEL_VERSION}/{field}"),
                datasets: self.datasets.clone(),
            },
        };
        self.evidence.push(Evidence {
            object: self.object.clone(),
            field: field.into(),
            distribution,
            confidence,
            quantity: quantity.clone(),
        });
        quantity
    }
    fn sample(
        &mut self,
        field: &str,
        lower: f64,
        upper: f64,
        log: bool,
        conditional: bool,
    ) -> Quantity {
        let u = uniform(self.seed, self.attempt, self.object.as_str(), field);
        let value = if log {
            (lower.ln() + u * (upper.ln() - lower.ln())).exp()
        } else {
            lower + u * (upper - lower)
        };
        // Clamp floating-point endpoint rounding, not invalid input or science.
        let value = value.clamp(lower, upper);
        let distribution = if log {
            Distribution::LogUniform { lower, upper }
        } else {
            Distribution::Uniform { lower, upper }
        };
        self.inferred(
            field,
            value,
            distribution,
            if conditional {
                Confidence::ModelConditional
            } else {
                Confidence::LowPrior
            },
        )
    }
    fn derived(&mut self, field: &str, value: f64, relation: &str) -> Quantity {
        self.inferred(
            field,
            value,
            Distribution::Derived {
                relation: relation.into(),
            },
            Confidence::ModelConditional,
        )
    }
}
// Frozen FNV-1a domain hash then SplitMix64 finalizer; no DefaultHasher or global RNG.
fn uniform(seed: u64, attempt: u64, object: &str, field: &str) -> f64 {
    let mut h = 0xcbf29ce484222325u64;
    for b in MODEL_VERSION
        .bytes()
        .chain([0])
        .chain(object.bytes())
        .chain([0])
        .chain(field.bytes())
    {
        h = (h ^ u64::from(b)).wrapping_mul(0x100000001b3);
    }
    let mut z = h ^ seed ^ attempt.wrapping_mul(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^= z >> 31;
    ((z >> 11) as f64 + 0.5) / ((1u64 << 53) as f64)
}
fn sources(p: &Provenance, out: &mut BTreeSet<CatalogId>) {
    match p {
        Provenance::Observed { dataset, .. } => {
            out.insert(dataset.clone());
        }
        Provenance::Inferred { datasets, .. } => out.extend(datasets.iter().cloned()),
        Provenance::Fiction { .. } => {}
    }
}
fn validate_input(catalog: &Catalog, input: &SystemInput) -> Result<Vec<CatalogId>, Error> {
    let Some(Record::Star(host)) = catalog.get(&input.host) else {
        return Err(invalid("inference host must be a catalog star"));
    };
    let registered = catalog.datasets().iter().map(|d| &d.id).collect();
    let mut datasets = BTreeSet::new();
    let mut seen = BTreeSet::new();
    if input.planets.len() > 64 {
        return Err(invalid("inference supports at most 64 planets per system"));
    }
    for p in &input.planets {
        if !seen.insert(&p.planet) {
            return Err(invalid("duplicate inference planet"));
        }
        match catalog.get(&p.planet) {
            Some(Record::Body(b)) if b.parent == input.host && b.body_kind == BodyKind::Planet => {}
            _ => return Err(invalid("inference planet must orbit the selected host")),
        }
        for q in [
            &p.semi_major_axis_meters,
            &p.period_seconds,
            &p.minimum_mass_kg,
        ]
        .into_iter()
        .flatten()
        {
            crate::validation::quantity(&Some(q.clone()), &registered)?;
            sources(&q.provenance, &mut datasets);
        }
        if let Some(q) = &p.eccentricity {
            crate::validation::provenance(&q.provenance, &registered)?;
            if !q.value.is_finite() || !(0.0..1.0).contains(&q.value) {
                return Err(invalid("eccentricity must be finite in [0,1)"));
            }
            // Reuse positive scalar metadata validation after translating e by 1.
            let mut shifted = q.clone();
            shifted.value += 1.0;
            crate::validation::quantity(&Some(shifted), &registered)?;
            if let Uncertainty::Bounds { minus, plus, .. } = q.uncertainty
                && (q.value - minus < 0.0 || q.value + plus >= 1.0)
            {
                return Err(invalid("eccentricity bounds cross [0,1)"));
            }
            sources(&q.provenance, &mut datasets);
        }
    }
    if let Some(q) = &input.age_seconds {
        crate::validation::quantity(&Some(q.clone()), &registered)?;
        sources(&q.provenance, &mut datasets);
    }
    for r in catalog
        .records()
        .iter()
        .filter(|r| r.system() == &host.system)
    {
        sources(&r.header().provenance, &mut datasets);
        let (mass, radius) = match r {
            Record::Star(s) if s.header.id == input.host => (&s.mass_kg, &s.radius_meters),
            Record::System(_) => continue,
            Record::Body(b) if seen.contains(&b.header.id) => (&b.mass_kg, &b.radius_meters),
            _ => {
                return Err(invalid(
                    "unsupported inference system: requires one star and only the listed planets",
                ));
            }
        };
        for q in [mass, radius].into_iter().flatten() {
            sources(&q.provenance, &mut datasets);
        }
    }
    if datasets.is_empty() {
        return Err(invalid(
            "inference requires registered source evidence; pure fictional population synthesis belongs to U10",
        ));
    }
    Ok(datasets.into_iter().collect())
}
fn volume(radius: f64) -> f64 {
    (4.0 / 3.0) * std::f64::consts::PI * radius.powi(3)
}
fn physical(m: f64, r: f64, model: PlanetModel) -> bool {
    let density = m / volume(r);
    match model {
        PlanetModel::RockyV1 => {
            (0.01..=15.0).contains(&(m / EARTH_MASS_KG))
                && (0.3..=1.6).contains(&(r / EARTH_RADIUS_METERS))
                && (3000.0..=8000.0).contains(&density)
        }
        PlanetModel::GasGiantV1 => {
            (0.1..=10.0).contains(&(m / 1.89813e27))
                && (0.7..=1.4).contains(&(r / 71_492_000.0))
                && (50.0..=30000.0).contains(&density)
        }
    }
}
/// Produces a new explicit catalog revision plus typed age/orbit/evidence sidecars.
/// Preserves *all* present quantities verbatim. No result is published unless the
/// complete supported system passes conservative point-estimate constraints.
/// At most 256 attempts; inconsistent observations reject rather than get repaired.
pub fn complete(
    catalog: &Catalog,
    input: &SystemInput,
    revision: String,
) -> Result<Completion, Error> {
    if revision == catalog.revision() {
        return Err(invalid(
            "inference requires an explicit new catalog revision",
        ));
    }
    let datasets = validate_input(catalog, input)?;
    let mut planets = input.planets.clone();
    planets.sort_by(|a, b| a.planet.cmp(&b.planet));
    for attempt in 0..MAX_ATTEMPTS {
        if let Some(result) = candidate(catalog, input, &planets, &revision, &datasets, attempt)? {
            return Ok(result);
        }
    }
    Err(invalid(
        "no physically admissible inference in 256 attempts; observations preserved, revise model/classification or inputs",
    ))
}
fn candidate(
    catalog: &Catalog,
    input: &SystemInput,
    planets: &[PlanetInput],
    revision: &str,
    datasets: &[CatalogId],
    attempt: u64,
) -> Result<Option<Completion>, Error> {
    let mut sampler = Sampler {
        seed: input.seed,
        attempt,
        datasets: datasets.to_vec(),
        evidence: vec![],
        object: &input.host,
    };
    let mut records = catalog.records().to_vec();
    let Some(Record::Star(host)) = records.iter_mut().find(|r| r.id() == &input.host) else {
        return Err(invalid("missing validated host"));
    };
    if host
        .mass_kg
        .as_ref()
        .is_some_and(|q| !(0.08..=2.0).contains(&(q.value / SOLAR_MASS_KG)))
        || host.radius_meters.as_ref().is_some_and(|q| {
            !(0.08_f64.powf(0.8) * 0.75..=2.0_f64.powf(0.8) * 1.25)
                .contains(&(q.value / SOLAR_RADIUS_METERS))
        })
    {
        return Ok(None);
    }
    let m = match &host.mass_kg {
        Some(q) => q.clone(),
        None => {
            if let Some(r) = &host.radius_meters {
                let center = (r.value / SOLAR_RADIUS_METERS).powf(1.0 / 0.8) * SOLAR_MASS_KG;
                sampler.sample("mass_kg", center * 0.8, center * 1.2, true, true)
            } else {
                sampler.sample(
                    "mass_kg",
                    0.08 * SOLAR_MASS_KG,
                    2.0 * SOLAR_MASS_KG,
                    true,
                    false,
                )
            }
        }
    };
    let center = SOLAR_RADIUS_METERS * (m.value / SOLAR_MASS_KG).powf(0.8);
    let r = host
        .radius_meters
        .clone()
        .unwrap_or_else(|| sampler.sample("radius_meters", center * 0.8, center * 1.2, true, true));
    if !(0.08..=2.0).contains(&(m.value / SOLAR_MASS_KG))
        || !(0.75..=1.25).contains(&(r.value / center))
    {
        return Ok(None);
    }
    host.mass_kg = Some(m.clone());
    host.radius_meters = Some(r.clone());
    let max_age =
        (10.0e9 * YEAR_SECONDS * (m.value / SOLAR_MASS_KG).powf(-2.5)).min(13.8e9 * YEAR_SECONDS);
    let age = input.age_seconds.clone().unwrap_or_else(|| {
        sampler.sample("age_seconds", 1.0e6 * YEAR_SECONDS, max_age, false, true)
    });
    if !(1.0e6 * YEAR_SECONDS..=max_age).contains(&age.value) {
        return Ok(None);
    }
    let mut orbits = vec![];
    let mut checks = vec![];
    for p in planets {
        sampler.object = &p.planet;
        let Some(Record::Body(body)) = records.iter_mut().find(|r| r.id() == &p.planet) else {
            return Err(invalid("missing validated planet"));
        };
        let (mut mass, mut radius) = (body.mass_kg.clone(), body.radius_meters.clone());
        match p.model {
            PlanetModel::RockyV1 => {
                if mass.is_none() && radius.is_none() {
                    radius = Some(sampler.sample(
                        "radius_meters",
                        0.3 * EARTH_RADIUS_METERS,
                        1.6 * EARTH_RADIUS_METERS,
                        true,
                        false,
                    ));
                }
                if mass.is_none() || radius.is_none() {
                    let density = sampler.sample("density_kg_per_m3", 3000.0, 8000.0, true, true);
                    if let Some(r) = &radius {
                        mass = Some(sampler.derived(
                            "mass_kg",
                            density.value * volume(r.value),
                            "mass = density * 4*pi*radius^3/3",
                        ));
                    } else if let Some(m) = &mass {
                        radius = Some(sampler.derived(
                            "radius_meters",
                            (m.value / (density.value * (4.0 / 3.0) * std::f64::consts::PI)).cbrt(),
                            "radius = (3*mass/(4*pi*density))^(1/3)",
                        ));
                    }
                }
            }
            PlanetModel::GasGiantV1 => {
                if mass.is_none() {
                    mass = Some(sampler.sample(
                        "mass_kg",
                        0.1 * 1.89813e27,
                        10.0 * 1.89813e27,
                        true,
                        false,
                    ));
                }
                if radius.is_none() {
                    radius = Some(sampler.sample(
                        "radius_meters",
                        0.7 * 71_492_000.0,
                        1.4 * 71_492_000.0,
                        true,
                        false,
                    ));
                }
            }
        }
        let (Some(pm), Some(pr)) = (mass, radius) else {
            return Err(invalid("incomplete model implementation"));
        };
        if !physical(pm.value, pr.value, p.model)
            || p.minimum_mass_kg
                .as_ref()
                .is_some_and(|q| pm.value < q.value)
        {
            return Ok(None);
        }
        body.mass_kg = Some(pm.clone());
        body.radius_meters = Some(pr.clone());
        let mu = G * (m.value + pm.value);
        let a = p.semi_major_axis_meters.clone().unwrap_or_else(|| {
            if let Some(t) = &p.period_seconds {
                sampler.derived(
                    "semi_major_axis_meters",
                    (mu * (t.value / std::f64::consts::TAU).powi(2)).cbrt(),
                    "Kepler a = (G*(M+m)*(period/(2*pi))^2)^(1/3)",
                )
            } else {
                sampler.sample(
                    "semi_major_axis_meters",
                    0.02 * AU_METERS,
                    100.0 * AU_METERS,
                    true,
                    false,
                )
            }
        });
        let period = std::f64::consts::TAU * (a.value.powi(3) / mu).sqrt();
        let t = p.period_seconds.clone().unwrap_or_else(|| {
            sampler.derived(
                "period_seconds",
                period,
                "Kepler period = 2*pi*sqrt(a^3/(G*(M+m)))",
            )
        });
        let e = p
            .eccentricity
            .clone()
            .unwrap_or_else(|| sampler.sample("eccentricity", 0.0, 0.3, false, false));
        let peri = a.value * (1.0 - e.value);
        let apo = a.value * (1.0 + e.value);
        let roche = 2.44 * pr.value * (m.value / pm.value).cbrt();
        if !period.is_finite()
            || period <= 0.0
            || ((t.value / period) - 1.0).abs() > 1e-3
            || !apo.is_finite()
            || peri <= (r.value + pr.value).max(roche)
        {
            return Ok(None);
        }
        checks.push((a.value, peri, apo, pm.value));
        orbits.push(Orbit {
            planet: p.planet.clone(),
            semi_major_axis_meters: a,
            period_seconds: t,
            eccentricity: e,
        });
    }
    checks.sort_by(|a, b| a.0.total_cmp(&b.0));
    for pair in checks.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let mutual_hill = ((a.3 + b.3) / (3.0 * m.value)).cbrt() * (a.0 + b.0) * 0.5;
        if b.1 - a.2 <= 3.5 * mutual_hill {
            return Ok(None);
        }
    }
    let catalog = Catalog::new(revision.into(), catalog.datasets().to_vec(), records)?;
    Ok(Some(Completion {
        model_version: MODEL_VERSION.into(),
        seed: input.seed,
        accepted_attempt: attempt,
        stellar_model: input.model,
        planet_models: planets
            .iter()
            .map(|p| (p.planet.clone(), p.model))
            .collect(),
        catalog,
        age_seconds: age,
        orbits,
        evidence: sampler.evidence,
    }))
}
#[cfg(test)]
mod tests;
